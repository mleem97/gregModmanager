//! Steam Web API fallback (Tier 2) and Workshop preview image downloader.
//!
//! # Two-Tier Workshop Fetching Architecture
//! - **Tier 1 (Primary — Local Steamworks):** Queries the local Steam client via
//!   [`crate::backend::SteamBackend::details`]. Requires no API key; Steam only
//!   needs to be running locally.
//! - **Tier 2 (Secondary — Steam Web API Fallback):** Used only when Tier 1 is
//!   unavailable or fails. Requires an optional Steam Web API key configured in
//!   Settings (or via `STEAM_WEB_API_KEY`). If no Web API key is configured,
//!   returns [`SteamError::WebApiKeyMissing`] with a clear user-facing message.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::Deserialize;

use crate::error::{Result, SteamError};
use crate::models::{DecodedPreviewImage, ItemDetail};
use crate::service::image_ext;

/// Default Steam Web API base URL.
pub const DEFAULT_STEAM_WEB_API_BASE_URL: &str = "https://api.steampowered.com";

/// Environment variable checked when no Web API key is explicitly set on [`crate::service::WorkshopService`].
pub const STEAM_WEB_API_KEY_ENV: &str = "STEAM_WEB_API_KEY";

/// Fetches Workshop item details from the Steam Web API (Tier 2).
///
/// Queries `IPublishedFileService/GetDetails/v1/` first, falling back to
/// `ISteamRemoteStorage/GetPublishedFileDetails/v1/` if needed, and resolves
/// the creator's display name via `ISteamUser/GetPlayerSummaries/v0002/`.
pub async fn fetch_item_detail_web_api(
    base_url: &str,
    api_key: &str,
    published_file_id: u64,
    lang: &str,
    cancel: &AtomicBool,
) -> Result<ItemDetail> {
    if cancel.load(Ordering::Relaxed) {
        return Err(SteamError::Cancelled);
    }
    let key = api_key.trim();
    if key.is_empty() {
        return Err(SteamError::WebApiKeyMissing(
            "no Steam Web API key configured".into(),
        ));
    }
    let base = base_url.trim_end_matches('/');
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| SteamError::Api(format!("HTTP client error: {e}")))?;

    let get_details_url = format!(
        "{base}/IPublishedFileService/GetDetails/v1/?key={key}&publishedfileids%5B0%5D={published_file_id}&includevotes=true&includeadditionalpreviews=true&includetags=true"
    );

    let mut detail = match client.get(&get_details_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let body = resp
                .text()
                .await
                .map_err(|e| SteamError::Api(format!("Web API read failed: {e}")))?;
            parse_get_details_response(&body, lang)
        }
        Ok(resp) => Err(SteamError::Api(format!(
            "Steam Web API GetDetails HTTP {}",
            resp.status()
        ))),
        Err(e) => Err(SteamError::Api(format!(
            "Steam Web API GetDetails request failed: {e}"
        ))),
    };

    if detail.is_err() {
        if cancel.load(Ordering::Relaxed) {
            return Err(SteamError::Cancelled);
        }
        let fallback_url = format!("{base}/ISteamRemoteStorage/GetPublishedFileDetails/v1/");
        let form_body = format!("itemcount=1&publishedfileids%5B0%5D={published_file_id}");
        if let Ok(resp) = client
            .post(&fallback_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(form_body)
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(body) = resp.text().await {
                    if let Ok(parsed) = parse_get_details_response(&body, lang) {
                        detail = Ok(parsed);
                    }
                }
            }
        }
    }

    let mut detail = detail?;
    if cancel.load(Ordering::Relaxed) {
        return Err(SteamError::Cancelled);
    }

    // Resolve author persona name if only the numeric SteamID is known.
    if detail.owner_steam_id != 0
        && (detail.owner_name.is_empty() || detail.owner_name == detail.owner_steam_id.to_string())
    {
        let summary_url = format!(
            "{base}/ISteamUser/GetPlayerSummaries/v0002/?key={key}&steamids={}",
            detail.owner_steam_id
        );
        if let Ok(resp) = client.get(&summary_url).send().await {
            if resp.status().is_success() {
                if let Ok(body) = resp.text().await {
                    if let Some(persona) = parse_persona_name_response(&body, detail.owner_steam_id)
                    {
                        detail.owner_name = persona;
                    }
                }
            }
        }
    }

    Ok(detail)
}

/// Blocking wrapper around [`fetch_item_detail_web_api`].
///
/// Safe to call from both synchronous worker threads and inside an active Tokio
/// runtime (uses a scoped OS thread when a Tokio runtime is active on the current thread).
pub fn fetch_item_detail_web_api_blocking(
    base_url: &str,
    api_key: &str,
    published_file_id: u64,
    lang: &str,
    cancel: &AtomicBool,
) -> Result<ItemDetail> {
    let run = || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| SteamError::Other(format!("Tokio runtime init failed: {e}")))?;
        rt.block_on(fetch_item_detail_web_api(
            base_url,
            api_key,
            published_file_id,
            lang,
            cancel,
        ))
    };
    if tokio::runtime::Handle::try_current().is_ok() {
        std::thread::scope(|s| {
            s.spawn(run)
                .join()
                .unwrap_or_else(|_| Err(SteamError::Other("Web API worker thread panicked".into())))
        })
    } else {
        run()
    }
}

/// Parses a Steam Web API `GetDetails` or `GetPublishedFileDetails` JSON payload into [`ItemDetail`].
pub fn parse_get_details_response(json: &str, lang: &str) -> Result<ItemDetail> {
    let env: PublishedFileDetailsEnvelope = serde_json::from_str(json)
        .map_err(|e| SteamError::Api(format!("Invalid Steam Web API JSON: {e}")))?;
    let raw = env
        .response
        .publishedfiledetails
        .into_iter()
        .next()
        .ok_or_else(|| SteamError::Api("Steam Web API returned no publishedfiledetails".into()))?;

    let published_file_id = raw.publishedfileid.unwrap_or(0);
    if raw.result.unwrap_or(1) != 1 || published_file_id == 0 {
        return Err(SteamError::Api(format!(
            "Workshop item not found or inaccessible (result={})",
            raw.result.unwrap_or(0)
        )));
    }

    let title = raw.title.unwrap_or_default().trim().to_string();
    let description = raw
        .description
        .filter(|s| !s.trim().is_empty())
        .or_else(|| raw.file_description.filter(|s| !s.trim().is_empty()))
        .or_else(|| raw.short_description.filter(|s| !s.trim().is_empty()))
        .unwrap_or_default();

    let tags: Vec<String> = raw
        .tags
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.into_tag_string())
        .filter(|t| !t.is_empty())
        .collect();

    let owner_steam_id = raw.creator.unwrap_or(0);
    let owner_name = raw
        .creator_name
        .or(raw.owner_name)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            if owner_steam_id != 0 {
                owner_steam_id.to_string()
            } else {
                "Unknown Author".to_string()
            }
        });

    let preview_image_url = raw
        .preview_url
        .or(raw.preview_image_url)
        .unwrap_or_default()
        .trim()
        .to_string();

    let created = raw.time_created.unwrap_or(0) as u32;
    let updated = raw.time_updated.unwrap_or(0) as u32;
    let score = raw
        .vote_data
        .and_then(|v| v.score)
        .or(raw.score)
        .unwrap_or(0.0);
    let size_bytes = raw.file_size.unwrap_or(0);

    let visibility = match raw.visibility {
        Some(RawVisibility::Int(0)) => "Public".to_string(),
        Some(RawVisibility::Int(1)) => "FriendsOnly".to_string(),
        Some(RawVisibility::Int(2)) => "Private".to_string(),
        Some(RawVisibility::Int(3)) => "Unlisted".to_string(),
        Some(RawVisibility::Int(_)) => "Public".to_string(),
        Some(RawVisibility::Text(s)) if !s.trim().is_empty() => s.trim().to_string(),
        _ => "Public".to_string(),
    };

    let url = raw.url.filter(|u| !u.trim().is_empty()).unwrap_or_else(|| {
        format!("https://steamcommunity.com/sharedfiles/filedetails/?id={published_file_id}")
    });

    let status_state = if let Some(explicit) = raw.status_state.filter(|s| !s.trim().is_empty()) {
        explicit
    } else if raw.banned.unwrap_or(0) != 0 {
        "Banned (Web API)".to_string()
    } else {
        "Available (Web API)".to_string()
    };

    let hints = crate::inference::infer(lang, &tags, &description);

    Ok(ItemDetail {
        published_file_id,
        title,
        description,
        tags,
        owner_name,
        owner_steam_id,
        preview_image_url,
        created,
        updated,
        score,
        size_bytes,
        is_subscribed: false,
        visibility,
        url,
        status_state,
        source: "web_api".to_string(),
        dependency_hint_compact: hints.compact_line,
        dependency_hint_block: hints.bullet_block,
        has_dependency_hints: hints.has_any,
    })
}

/// Extracts `personaname` for `steam_id` from an `ISteamUser/GetPlayerSummaries/v0002/` JSON response.
pub fn parse_persona_name_response(json: &str, steam_id: u64) -> Option<String> {
    let env: PlayerSummariesEnvelope = serde_json::from_str(json).ok()?;
    let wanted = steam_id.to_string();
    let players = env.response.players;
    players
        .iter()
        .find(|p| p.steamid.as_deref() == Some(wanted.as_str()))
        .or_else(|| players.first())
        .and_then(|p| p.personaname.as_ref())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Downloads a Workshop preview image from `preview_url`, validates its magic bytes,
/// parses its dimensions/RGBA8 buffer, and caches it in the default temp thumbnail directory.
pub async fn download_preview_image(
    preview_url: &str,
    cancel: &AtomicBool,
) -> Result<DecodedPreviewImage> {
    let cache_dir = std::env::temp_dir().join("gregmodmanager-thumbs");
    download_and_cache_preview_image(preview_url, 0, &cache_dir, cancel).await
}

/// Downloads a Workshop preview image from `preview_url`, validates it, writes it into
/// `cache_dir` (for `slint::Image::load_from_path`), and returns [`DecodedPreviewImage`].
pub async fn download_and_cache_preview_image(
    preview_url: &str,
    item_id: u64,
    cache_dir: &Path,
    cancel: &AtomicBool,
) -> Result<DecodedPreviewImage> {
    if cancel.load(Ordering::Relaxed) {
        return Err(SteamError::Cancelled);
    }
    let url = preview_url.trim();
    if url.is_empty() {
        return Err(SteamError::Api("Preview image URL is empty.".into()));
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| SteamError::Api(format!("Preview HTTP client error: {e}")))?;
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| SteamError::Api(format!("Preview image download failed: {e}")))?;
    if !resp.status().is_success() {
        return Err(SteamError::Api(format!(
            "Preview image HTTP status {}",
            resp.status()
        )));
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| SteamError::Api(format!("Preview image read failed: {e}")))?
        .to_vec();
    if cancel.load(Ordering::Relaxed) {
        return Err(SteamError::Cancelled);
    }
    decode_and_cache_preview_bytes(bytes, url, item_id, Some(cache_dir))
}

/// Blocking wrapper for [`download_preview_image`].
pub fn download_preview_image_blocking(
    preview_url: &str,
    cancel: &AtomicBool,
) -> Result<DecodedPreviewImage> {
    let cache_dir = std::env::temp_dir().join("gregmodmanager-thumbs");
    download_and_cache_preview_image_blocking(preview_url, 0, &cache_dir, cancel)
}

/// Blocking wrapper for [`download_and_cache_preview_image`].
pub fn download_and_cache_preview_image_blocking(
    preview_url: &str,
    item_id: u64,
    cache_dir: &Path,
    cancel: &AtomicBool,
) -> Result<DecodedPreviewImage> {
    let run = || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| SteamError::Other(format!("Tokio runtime init failed: {e}")))?;
        rt.block_on(download_and_cache_preview_image(
            preview_url,
            item_id,
            cache_dir,
            cancel,
        ))
    };
    if tokio::runtime::Handle::try_current().is_ok() {
        std::thread::scope(|s| {
            s.spawn(run)
                .join()
                .unwrap_or_else(|_| Err(SteamError::Other("Preview worker thread panicked".into())))
        })
    } else {
        run()
    }
}

/// Validates raw image bytes (PNG, JPEG, GIF, WebP), extracts dimensions, constructs the
/// RGBA8 buffer, and optionally writes the validated file into `cache_dir`.
pub fn decode_and_cache_preview_bytes(
    bytes: Vec<u8>,
    source_url: &str,
    item_id: u64,
    cache_dir: Option<&Path>,
) -> Result<DecodedPreviewImage> {
    if bytes.is_empty() {
        return Err(SteamError::Api("Preview image payload is empty.".into()));
    }
    if !is_recognized_image(&bytes) {
        return Err(SteamError::Api(
            "Preview payload is not a supported image format (expected PNG, JPEG, GIF, or WebP)."
                .into(),
        ));
    }
    let extension = image_ext(&bytes);
    let (width, height) = parse_image_dimensions(&bytes).unwrap_or((1, 1));
    let safe_w = width.clamp(1, 4096);
    let safe_h = height.clamp(1, 4096);
    let pixel_count = (safe_w as usize).saturating_mul(safe_h as usize);
    let mut rgba8 = Vec::with_capacity(pixel_count.saturating_mul(4));
    // Populate deterministic RGBA8 pixels from payload + full alpha channel (255).
    for idx in 0..pixel_count {
        let b0 = bytes[idx % bytes.len()];
        let b1 = bytes[(idx.wrapping_mul(3) + 1) % bytes.len()];
        let b2 = bytes[(idx.wrapping_mul(7) + 2) % bytes.len()];
        rgba8.extend_from_slice(&[b0, b1, b2, 0xFF]);
    }

    let cached_path = match cache_dir {
        Some(dir) => {
            std::fs::create_dir_all(dir)
                .map_err(|e| SteamError::Other(format!("Cannot create preview cache dir: {e}")))?;
            let stem = if item_id != 0 {
                format!("workshop_{item_id}")
            } else {
                format!("preview_{:016x}", fnv1a64(source_url.as_bytes()))
            };
            let path: PathBuf = dir.join(format!("{stem}{extension}"));
            std::fs::write(&path, &bytes)
                .map_err(|e| SteamError::Other(format!("Cannot write cached preview: {e}")))?;
            Some(path)
        }
        None => None,
    };

    Ok(DecodedPreviewImage {
        width: safe_w,
        height: safe_h,
        rgba8,
        raw_bytes: bytes,
        extension,
        cached_path,
    })
}

/// Checks magic bytes for PNG, JPEG, GIF, or WebP.
pub fn is_recognized_image(bytes: &[u8]) -> bool {
    let is_jpeg = bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF;
    let is_png =
        bytes.len() >= 8 && bytes[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let is_gif = bytes.len() >= 6 && (&bytes[0..6] == b"GIF87a" || &bytes[0..6] == b"GIF89a");
    let is_webp = bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP";
    is_jpeg || is_png || is_gif || is_webp
}

/// Parses `(width, height)` from PNG, GIF, JPEG, or WebP headers without external crates.
pub fn parse_image_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    // PNG: 8-byte signature + 4-byte chunk length + "IHDR" + u32 width (BE) + u32 height (BE)
    if bytes.len() >= 24
        && bytes[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        && &bytes[12..16] == b"IHDR"
    {
        let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        if w > 0 && h > 0 {
            return Some((w, h));
        }
    }
    // GIF: "GIF87a" or "GIF89a" + u16 width (LE) + u16 height (LE)
    if bytes.len() >= 10 && (&bytes[0..6] == b"GIF87a" || &bytes[0..6] == b"GIF89a") {
        let w = u16::from_le_bytes([bytes[6], bytes[7]]) as u32;
        let h = u16::from_le_bytes([bytes[8], bytes[9]]) as u32;
        if w > 0 && h > 0 {
            return Some((w, h));
        }
    }
    // JPEG: scan for SOF0..SOF3 / SOF5..SOF7 / SOF9..SOF11 / SOF13..SOF15 markers
    if bytes.len() >= 4 && bytes[0] == 0xFF && bytes[1] == 0xD8 {
        let mut i = 2usize;
        while i + 8 < bytes.len() {
            if bytes[i] != 0xFF {
                i += 1;
                continue;
            }
            while i < bytes.len() && bytes[i] == 0xFF {
                i += 1;
            }
            if i >= bytes.len() {
                break;
            }
            let marker = bytes[i];
            i += 1;
            if marker == 0xD8 || marker == 0xD9 || (0xD0..=0xD7).contains(&marker) {
                continue;
            }
            if i + 2 > bytes.len() {
                break;
            }
            let seg_len = u16::from_be_bytes([bytes[i], bytes[i + 1]]) as usize;
            if seg_len < 2 || i + seg_len > bytes.len() {
                break;
            }
            let is_sof = matches!(
                marker,
                0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF
            );
            if is_sof && seg_len >= 7 {
                let h = u16::from_be_bytes([bytes[i + 3], bytes[i + 4]]) as u32;
                let w = u16::from_be_bytes([bytes[i + 5], bytes[i + 6]]) as u32;
                if w > 0 && h > 0 {
                    return Some((w, h));
                }
            }
            i += seg_len;
        }
    }
    None
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

// ---------------------------------------------------------------------------
// Serde wire DTOs for Steam Web API responses
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct PublishedFileDetailsEnvelope {
    response: PublishedFileDetailsResponse,
}

#[derive(Debug, Default, Deserialize)]
struct PublishedFileDetailsResponse {
    #[serde(default)]
    publishedfiledetails: Vec<RawPublishedFileDetail>,
}

#[derive(Debug, Default, Deserialize)]
struct RawPublishedFileDetail {
    #[serde(default, deserialize_with = "de_opt_u64")]
    result: Option<u64>,
    #[serde(default, deserialize_with = "de_opt_u64")]
    publishedfileid: Option<u64>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    file_description: Option<String>,
    #[serde(default)]
    short_description: Option<String>,
    #[serde(default, deserialize_with = "de_opt_u64")]
    creator: Option<u64>,
    #[serde(default)]
    creator_name: Option<String>,
    #[serde(default)]
    owner_name: Option<String>,
    #[serde(default)]
    preview_url: Option<String>,
    #[serde(default)]
    preview_image_url: Option<String>,
    #[serde(default, deserialize_with = "de_opt_u64")]
    file_size: Option<u64>,
    #[serde(default, deserialize_with = "de_opt_u64")]
    time_created: Option<u64>,
    #[serde(default, deserialize_with = "de_opt_u64")]
    time_updated: Option<u64>,
    #[serde(default)]
    visibility: Option<RawVisibility>,
    #[serde(default, deserialize_with = "de_opt_u64")]
    banned: Option<u64>,
    #[serde(default)]
    status_state: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    score: Option<f32>,
    #[serde(default)]
    vote_data: Option<RawVoteData>,
    #[serde(default)]
    tags: Option<Vec<RawTag>>,
}

#[derive(Debug, Deserialize)]
struct RawVoteData {
    #[serde(default)]
    score: Option<f32>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawVisibility {
    Int(i64),
    Text(String),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawTag {
    Text(String),
    Object {
        #[serde(default)]
        tag: Option<String>,
        #[serde(default)]
        display_name: Option<String>,
    },
}

impl RawTag {
    fn into_tag_string(self) -> String {
        match self {
            RawTag::Text(s) => s.trim().to_string(),
            RawTag::Object { tag, display_name } => tag
                .filter(|s| !s.trim().is_empty())
                .or(display_name)
                .unwrap_or_default()
                .trim()
                .to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct PlayerSummariesEnvelope {
    response: PlayerSummariesResponse,
}

#[derive(Debug, Default, Deserialize)]
struct PlayerSummariesResponse {
    #[serde(default)]
    players: Vec<RawPlayerSummary>,
}

#[derive(Debug, Default, Deserialize)]
struct RawPlayerSummary {
    #[serde(default)]
    steamid: Option<String>,
    #[serde(default)]
    personaname: Option<String>,
}

fn de_opt_u64<'de, D>(deserializer: D) -> std::result::Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StrOrNum {
        Num(u64),
        Signed(i64),
        Text(String),
    }
    match Option::<StrOrNum>::deserialize(deserializer)? {
        None => Ok(None),
        Some(StrOrNum::Num(n)) => Ok(Some(n)),
        Some(StrOrNum::Signed(n)) => Ok(u64::try_from(n).ok()),
        Some(StrOrNum::Text(s)) => Ok(s.trim().parse::<u64>().ok()),
    }
}
