//! Central defaults: endpoints, feature flags, local-build overrides.

/// Data Center Steam AppID.
pub const DATA_CENTER_APP_ID: u32 = 4_170_200;

/// Set `GREG_LOCAL_BUILD=1` to target local test services.
pub fn is_local_build() -> bool {
    std::env::var("GREG_LOCAL_BUILD")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Set `GREG_LOCAL_TEST_BUILD=1` for the `.home` test domains.
pub fn is_local_test_build() -> bool {
    std::env::var("GREG_LOCAL_TEST_BUILD")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Modstore web base URL.
pub fn modstore_web_url() -> String {
    std::env::var("GREG_MODSTORE_WEB_URL").unwrap_or_else(|_| {
        if is_local_test_build() {
            "https://datacentermods.home".to_string()
        } else {
            "https://datacentermods.com".to_string()
        }
    })
}

/// Modstore API base URL.
pub fn modstore_api_url() -> String {
    std::env::var("GREG_MODSTORE_API_URL").unwrap_or_else(|_| {
        if is_local_test_build() {
            "https://api.datacentermods.home".to_string()
        } else {
            "https://datacentermods.com".to_string()
        }
    })
}

/// Auth endpoint candidates (first reachable wins).
pub fn auth_endpoint_candidates() -> Vec<String> {
    if let Ok(single) = std::env::var("GREG_AUTH_URL") {
        return vec![single];
    }
    if is_local_build() {
        return vec!["http://localhost:5001/auth".to_string()];
    }
    vec![
        "https://datacentermods.com/auth".to_string(),
        "https://datacentermods.home/auth".to_string(),
    ]
}

/// Upstream MelonLoader release feed.
pub fn melon_loader_feed_url() -> String {
    std::env::var("GREG_MELONLOADER_FEED_URL").unwrap_or_else(|_| {
        if is_local_build() {
            "http://localhost:5000/releases".to_string()
        } else {
            "https://github.com/LavaGang/MelonLoader/releases".to_string()
        }
    })
}

/// GitHub API URL for the latest MelonLoader release.
pub const MELON_LOADER_LATEST_API_URL: &str =
    "https://api.github.com/repos/LavaGang/MelonLoader/releases/latest";

/// SteamModfix release feed.
pub const STEAM_MODFIX_LATEST_API_URL: &str =
    "https://api.github.com/repos/mleem97/gregPlugin.SteamModfix/releases/latest";

/// Whether the Modstore integration is enabled.
pub fn is_modstore_enabled() -> bool {
    std::env::var("GREG_MODSTORE_DISABLED")
        .map(|v| !(v == "1" || v.eq_ignore_ascii_case("true")))
        .unwrap_or(true)
}

/// Whether telemetry may be sent (opt-in).
pub fn is_telemetry_enabled() -> bool {
    std::env::var("GREG_TELEMETRY_ENABLED")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}
