//! Project publish flow shared by the CLI and the desktop UI.
//!
//! ```text
//! zip `content/` (or a single file) → POST /api/upload-url
//!   → PUT bytes → POST /api/mods/submit → moderation + security pipeline
//! ```
//!
//! [`PublishPlan::from_project`] is pure (zip + request shaping) and
//! unit-tested; only [`PublishPlan::execute`] touches the network.

use greg_core::models::{ProjectContentKind, WorkshopMetadata};
use std::path::{Path, PathBuf};

use crate::error::{ModStoreError, Result};
use crate::upload_client::{
    guess_content_type, infer_method, ModStoreUploadClient, SubmitModRequest, SubmitModResponse,
    UploadUrlRequest,
};

/// Hard limit mirrors `POST /api/upload-url` (MAX_MOD_BYTES = 500 MiB).
/// The readiness gate (`greg_core::upload::check`) enforces the same bound.
pub const MAX_UPLOAD_BYTES: u64 = 500 * 1024 * 1024;

/// What to publish: a zipped project `content/` dir or one raw file.
#[derive(Debug, Clone)]
pub enum PublishPayload {
    /// Zip of a directory (project `content/`).
    Directory {
        /// Directory to zip (files land at the archive root).
        dir: PathBuf,
        /// Archive file name (`<project>.zip`).
        file_name: String,
    },
    /// One raw file (`dll`, `lua`, `py`, `go`, `zip`, ...).
    File {
        /// File to upload as-is.
        path: PathBuf,
    },
}

/// Metadata for the submit call (mirrors the web UploadWizard fields).
#[derive(Debug, Clone, Default)]
pub struct PublishMetadata {
    /// Explicit shared type. None supports legacy callers; metadata.json-based
    /// publishers always set this and reject unsupported values.
    pub mod_type: Option<String>,
    /// Existing mod id for updates; `None` creates a new mod.
    pub mod_id: Option<String>,
    pub title: String,
    pub version: String,
    /// Native Markdown (the Modstore keeps it, unlike Steam BBCode).
    pub description: String,
    /// Best-effort: sent when the API accepts it, ignored otherwise.
    pub changelog: String,
    pub category: Option<String>,
    pub license: Option<String>,
    pub tags: Vec<String>,
    pub installation: Option<String>,
    pub support_url: Option<String>,
    pub min_game_version: Option<String>,
    pub recommended_game_version: Option<String>,
}

impl PublishMetadata {
    /// One metadata.json feeds both providers. Only provider IDs remain separate.
    pub fn from_workshop(
        meta: &WorkshopMetadata,
        description: impl Into<String>,
        changelog: impl Into<String>,
    ) -> Self {
        Self {
            mod_type: Some(meta.mod_type.clone()),
            mod_id: (!meta.modstore_id.trim().is_empty()).then(|| meta.modstore_id.clone()),
            title: meta.title.clone(),
            version: meta.version.clone(),
            description: description.into(),
            changelog: changelog.into(),
            category: meta.category.clone(),
            license: meta.license.clone(),
            tags: meta.publication_tags(),
            installation: meta.installation.clone(),
            support_url: meta.support_url.clone(),
            min_game_version: meta.min_game_version.clone(),
            recommended_game_version: meta.recommended_game_version.clone(),
        }
    }
}

/// A ready-to-send publish (bytes + shaped requests).
#[derive(Debug, Clone)]
pub struct PublishPlan {
    /// Archive/file name as sent to the upload URL.
    pub file_name: String,
    /// Raw bytes to PUT.
    pub bytes: Vec<u8>,
    /// Submit `method` (`zip` | `dll` | `lua` | `py` | `go`).
    pub method: String,
    /// Content type for the upload-URL request and the PUT.
    pub content_type: String,
    /// Shaped `POST /api/mods/submit` body (`file_key` filled on execute).
    pub submit: SubmitModRequest,
}

impl PublishPlan {
    /// Zips or reads `payload` and shapes the submit body from `meta`.
    pub fn from_project(payload: &PublishPayload, meta: &PublishMetadata) -> Result<Self> {
        let kind = match meta.mod_type.as_deref() {
            Some(name) => Some(ProjectContentKind::parse(name).ok_or_else(|| {
                ModStoreError::InvalidArtefact(format!("Unknown project content type: {name}"))
            })?),
            None => ProjectContentKind::from_publication_tags(&meta.tags)
                .map_err(ModStoreError::InvalidArtefact)?,
        };
        let mut tags = meta.tags.clone();
        if let Some(kind) = kind {
            tags.retain(|tag| {
                !tag.trim()
                    .split_once(':')
                    .is_some_and(|(prefix, _)| prefix.eq_ignore_ascii_case("greg-mod-type"))
            });
            tags.push(format!("greg-mod-type:{}", kind.name()));
        }
        let (file_name, bytes) = match payload {
            PublishPayload::Directory { dir, file_name } => (
                file_name.clone(),
                zip_dir_bytes_with_kind(dir, file_name, kind)?,
            ),
            PublishPayload::File { path } => {
                if std::fs::metadata(path)
                    .map_err(|e| ModStoreError::Other(e.to_string()))?
                    .len()
                    > MAX_UPLOAD_BYTES
                {
                    return Err(ModStoreError::InvalidArtefact(
                        "upload exceeds the 500 MiB limit".into(),
                    ));
                }
                let bytes = std::fs::read(path).map_err(|e| {
                    ModStoreError::Other(format!("cannot read {}: {e}", path.display()))
                })?;
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("upload.bin")
                    .to_string();
                (file_name, bytes)
            }
        };
        if bytes.is_empty() {
            return Err(ModStoreError::InvalidArtefact("nothing to upload".into()));
        }
        if bytes.len() as u64 > MAX_UPLOAD_BYTES {
            return Err(ModStoreError::InvalidArtefact(format!(
                "upload is {} — exceeds the 500 MiB limit",
                format_bytes(bytes.len() as u64),
            )));
        }
        let method = infer_method(&file_name).to_string();
        let content_type = guess_content_type(&file_name).to_string();
        let submit = SubmitModRequest {
            mod_id: meta.mod_id.clone(),
            release_id: None,
            method: method.clone(),
            title: meta.title.clone(),
            version: meta.version.clone(),
            category: meta.category.clone(),
            description: (!meta.description.trim().is_empty()).then(|| meta.description.clone()),
            license: meta.license.clone(),
            dependencies: None,
            compatibility_notes: None,
            security_notes: None,
            github_url: None,
            file_key: None,
            file_url: None,
            tags: (!tags.is_empty()).then_some(tags),
            installation: meta.installation.clone(),
            support_url: meta.support_url.clone(),
            min_game_version: meta.min_game_version.clone(),
            recommended_game_version: meta.recommended_game_version.clone(),
            changelog: (!meta.changelog.trim().is_empty()).then(|| meta.changelog.clone()),
        };
        Ok(Self {
            file_name,
            bytes,
            method,
            content_type,
            submit,
        })
    }

    /// Runs upload-URL → PUT → submit. `progress` gets 0–100.
    pub async fn execute(
        &self,
        client: &ModStoreUploadClient,
        progress: &dyn Fn(u8),
    ) -> Result<SubmitModResponse> {
        progress(5);
        let url_req = UploadUrlRequest {
            kind: Some("mod".to_string()),
            mod_id: self
                .submit
                .mod_id
                .clone()
                .unwrap_or_else(|| "new".to_string()),
            release_id: None,
            file_name: self.file_name.clone(),
            content_type: self.content_type.clone(),
            file_size: self.bytes.len() as i64,
        };
        let granted = client.request_upload_url(&url_req).await?;
        progress(25);
        client
            .put_bytes(&granted.upload_url, &self.content_type, &self.bytes)
            .await?;
        progress(75);
        let mut submit = self.submit.clone();
        submit.file_key = Some(granted.key.clone());
        let response = client.submit_mod(&submit).await?;
        progress(100);
        Ok(response)
    }
}

/// Zips `dir` (files at the archive root) into memory.
pub fn zip_dir_bytes(dir: &Path, arc_name: &str) -> Result<Vec<u8>> {
    zip_dir_bytes_with_kind(dir, arc_name, None)
}

fn zip_dir_bytes_with_kind(
    dir: &Path,
    arc_name: &str,
    kind: Option<ProjectContentKind>,
) -> Result<Vec<u8>> {
    if !dir.is_dir() {
        return Err(ModStoreError::InvalidArtefact(format!(
            "not a directory: {}",
            dir.display()
        )));
    }
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        let mut stack = vec![dir.to_path_buf()];
        let mut empty = true;
        while let Some(current) = stack.pop() {
            let entries = std::fs::read_dir(&current).map_err(|e| {
                ModStoreError::Other(format!("cannot scan {}: {e}", current.display()))
            })?;
            for entry in entries {
                let entry = entry.map_err(|e| ModStoreError::Other(e.to_string()))?;
                let path = entry.path();
                if entry
                    .file_type()
                    .map_err(|e| ModStoreError::Other(e.to_string()))?
                    .is_symlink()
                {
                    continue;
                }
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                let rel = path.strip_prefix(dir).map_err(|e| {
                    ModStoreError::Other(format!("cannot relativize {}: {e}", path.display()))
                })?;
                let name = rel.to_string_lossy().replace('\\', "/");
                if name.is_empty() || (kind.is_some() && name == "greg-modmanager.meta.json") {
                    continue;
                }
                empty = false;
                zip.start_file(name, options)
                    .map_err(|e| ModStoreError::Other(format!("cannot zip {arc_name}: {e}")))?;
                let mut file = std::fs::File::open(&path).map_err(|e| {
                    ModStoreError::Other(format!("cannot read {}: {e}", path.display()))
                })?;
                std::io::copy(&mut file, &mut zip)
                    .map_err(|e| ModStoreError::Other(format!("cannot zip {arc_name}: {e}")))?;
            }
        }
        if empty {
            return Err(ModStoreError::InvalidArtefact(format!(
                "nothing to upload in {}",
                dir.display()
            )));
        }
        if let Some(kind) = kind {
            zip.start_file("greg-modmanager.meta.json", options)
                .map_err(|e| ModStoreError::Other(e.to_string()))?;
            use std::io::Write as _;
            let marker = serde_json::json!({"modType": kind.name()});
            zip.write_all(marker.to_string().as_bytes())
                .map_err(|e| ModStoreError::Other(e.to_string()))?;
        }
        zip.finish()
            .map_err(|e| ModStoreError::Other(format!("cannot zip {arc_name}: {e}")))?;
    }
    Ok(buf)
}

fn format_bytes(n: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut value = n as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{n} B")
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_content(tag: &str, files: &[(&str, &[u8])]) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("test-tmp")
            .join(format!(
                "greg-publish-test-{tag}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        for (name, bytes) in files {
            let path = dir.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, bytes).unwrap();
        }
        dir
    }

    fn meta() -> PublishMetadata {
        PublishMetadata {
            title: "Test Mod".to_string(),
            version: "1.2.3".to_string(),
            description: "Desc".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn zips_directory_at_archive_root() {
        let dir = tmp_content("zip", &[("a.txt", b"hello"), ("sub/b.txt", b"world")]);
        let bytes = zip_dir_bytes(&dir, "test.zip").unwrap();
        assert!(!bytes.is_empty());
        let cursor = std::io::Cursor::new(&bytes);
        let mut zip = zip::ZipArchive::new(cursor).unwrap();
        let mut names: Vec<String> = (0..zip.len())
            .map(|i| zip.by_index(i).unwrap().name().to_string())
            .collect();
        names.sort();
        assert_eq!(names, vec!["a.txt", "sub/b.txt"]);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rejects_empty_and_missing_dirs() {
        let dir = tmp_content("empty", &[]);
        assert!(zip_dir_bytes(&dir, "x.zip").is_err());
        assert!(zip_dir_bytes(&dir.join("nope"), "x.zip").is_err());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn plan_shapes_submit() {
        let dir = tmp_content("plan", &[("mod.dll", b"fake")]);
        let payload = PublishPayload::Directory {
            dir: dir.clone(),
            file_name: "testmod.zip".to_string(),
        };
        let plan = PublishPlan::from_project(&payload, &meta()).unwrap();
        assert_eq!(plan.method, "zip");
        assert_eq!(plan.content_type, "application/zip");
        assert_eq!(plan.submit.title, "Test Mod");
        assert_eq!(plan.submit.version, "1.2.3");
        assert_eq!(plan.submit.description.as_deref(), Some("Desc"));
        assert!(plan.submit.file_key.is_none());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn one_shared_config_classifies_both_request_and_archive() {
        for kind in [
            ProjectContentKind::Userlib,
            ProjectContentKind::MelonloaderPlugin,
            ProjectContentKind::DataCenterMod,
            ProjectContentKind::PlacableObject,
        ] {
            let dir = tmp_content(
                kind.name(),
                &[
                    ("mod.dll", b"fake"),
                    ("greg-modmanager.meta.json", br#"{"modType":"stale"}"#),
                ],
            );
            let shared = WorkshopMetadata {
                title: "Shared project".into(),
                version: "2.0.0".into(),
                published_file_id: 123,
                modstore_id: "store-id".into(),
                mod_type: kind.name().into(),
                category: Some("Automation".into()),
                license: Some("Apache-2.0".into()),
                ..Default::default()
            };
            let meta = PublishMetadata::from_workshop(&shared, "README markdown", "release notes");
            let plan = PublishPlan::from_project(
                &PublishPayload::Directory {
                    dir: dir.clone(),
                    file_name: "mod.zip".into(),
                },
                &meta,
            )
            .unwrap();
            assert_eq!(plan.submit.mod_id.as_deref(), Some("store-id"));
            assert_eq!(plan.submit.category.as_deref(), Some("Automation"));
            assert_eq!(plan.submit.license.as_deref(), Some("Apache-2.0"));
            assert_eq!(
                ProjectContentKind::from_publication_tags(plan.submit.tags.as_ref().unwrap())
                    .unwrap(),
                Some(kind)
            );
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&plan.bytes)).unwrap();
            assert_eq!(archive.len(), 2);
            let mut marker = archive.by_name("greg-modmanager.meta.json").unwrap();
            let value: serde_json::Value = serde_json::from_reader(&mut marker).unwrap();
            assert_eq!(value["modType"], kind.name());
            assert!(!value.as_object().unwrap().contains_key("publishedFileId"));
            assert_eq!(
                std::fs::read_to_string(dir.join("greg-modmanager.meta.json")).unwrap(),
                r#"{"modType":"stale"}"#
            );
            drop(marker);
            drop(archive);
            std::fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn unknown_explicit_type_is_rejected_before_packaging() {
        let meta = PublishMetadata {
            mod_type: Some("../../outside".into()),
            ..Default::default()
        };
        assert!(PublishPlan::from_project(
            &PublishPayload::File {
                path: PathBuf::from("not-read.dll")
            },
            &meta
        )
        .is_err());
    }

    #[test]
    fn plan_reads_single_file_with_inferred_method() {
        let dir = tmp_content("single", &[("init.lua", b"print(1)")]);
        let payload = PublishPayload::File {
            path: dir.join("init.lua"),
        };
        let plan = PublishPlan::from_project(&payload, &meta()).unwrap();
        assert_eq!(plan.file_name, "init.lua");
        assert_eq!(plan.method, "lua");
        std::fs::remove_dir_all(&dir).ok();
    }
}
