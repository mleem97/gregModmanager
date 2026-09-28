//! Modstore upload client (P0 for the desktop Modmanager).
//!
//! Flow (mirrors `UploadWizard` + `POST /api/upload-url` + `POST /api/mods/submit`):
//! 1. `POST {base}/api/upload-url` with Bearer session token → presigned PUT URL.
//! 2. `PUT <uploadUrl>` with file bytes.
//! 3. `POST {base}/api/mods/submit` with `fileKey` (+ metadata) → moderation + security pipeline.
//!
//! All calls try each configured base URL in order (failover). 401/403 never
//! trigger failover to another host — they are returned immediately.

use serde::{Deserialize, Serialize};

use crate::error::{ModStoreError, Result};

/// Upload kind (`mod` artifact vs. cover `image`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UploadKind {
    #[default]
    #[serde(rename = "mod")]
    Mod,
    #[serde(rename = "image")]
    Image,
}

/// `POST /api/upload-url` request (camelCase, matches `UploadUrlSchema`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UploadUrlRequest {
    #[serde(default = "default_mod_kind", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub mod_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,
    pub file_name: String,
    pub content_type: String,
    pub file_size: i64,
}

fn default_mod_kind() -> Option<String> {
    Some("mod".to_string())
}

/// `POST /api/upload-url` response.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct UploadUrlResponse {
    pub upload_url: String,
    pub public_url: String,
    pub key: String,
    pub expires_in: i64,
}

/// `POST /api/mods/submit` request (subset of `SubmitModSchema`, camelCase).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SubmitModRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mod_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_id: Option<String>,
    /// `zip` | `dll` | `lua` | `py` | `go` | `git` (+ single-file asset types).
    pub method: String,
    pub title: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compatibility_notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security_notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_url: Option<String>,
    /// Quarantine object key from step 1 (`fileKey`) or direct URL (`fileUrl`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_game_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_game_version: Option<String>,
}

/// `POST /api/mods/submit` response (success shape).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct SubmitModResponse {
    pub success: bool,
    pub mod_id: String,
    pub release_id: String,
    pub scan_status: String,
    pub manual_review_required: bool,
}

/// Upload client with Bearer session token.
#[derive(Debug, Clone)]
pub struct ModStoreUploadClient {
    http: reqwest::Client,
    base_urls: Vec<String>,
    access_token: Option<String>,
}

impl ModStoreUploadClient {
    /// Creates the client. At least one base URL is required.
    pub fn new(base_urls: Vec<String>) -> Result<Self> {
        if base_urls.is_empty() {
            return Err(ModStoreError::NotAvailable(
                "at least one Modstore endpoint is required".into(),
            ));
        }
        Ok(Self {
            http: reqwest::Client::new(),
            base_urls,
            access_token: None,
        })
    }

    /// Attaches the session token (sent as `Authorization: Bearer`).
    pub fn with_token(mut self, access_token: impl Into<String>) -> Self {
        let token = access_token.into();
        self.access_token = if token.trim().is_empty() {
            None
        } else {
            Some(token)
        };
        self
    }

    fn bearer(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self.access_token.as_deref() {
            Some(t) => req.bearer_auth(t),
            None => req,
        }
    }

    /// Step 1: requests a presigned upload URL.
    pub async fn request_upload_url(&self, req: &UploadUrlRequest) -> Result<UploadUrlResponse> {
        let mut last: Option<ModStoreError> = None;
        for base in &self.base_urls {
            let url = format!("{}/api/upload-url", base.trim_end_matches('/'));
            let resp = self.bearer(self.http.post(&url).json(req)).send().await;
            match resp {
                Ok(r) => {
                    let status = r.status();
                    if status == reqwest::StatusCode::UNAUTHORIZED
                        || status == reqwest::StatusCode::FORBIDDEN
                    {
                        let msg = format!("upload-url rejected: {status}");
                        return Err(ModStoreError::Request(msg));
                    }
                    match r.error_for_status() {
                        Ok(ok) => match ok.json::<UploadUrlResponse>().await {
                            Ok(body) if !body.upload_url.is_empty() => return Ok(body),
                            Ok(_) => {
                                last = Some(ModStoreError::Request("empty uploadUrl".into()));
                            }
                            Err(e) => last = Some(ModStoreError::Request(e.to_string())),
                        },
                        Err(e) => last = Some(ModStoreError::Request(e.to_string())),
                    }
                }
                Err(e) => last = Some(ModStoreError::Request(e.to_string())),
            }
        }
        Err(last.unwrap_or_else(|| {
            ModStoreError::NotAvailable("all Modstore endpoints are unavailable".into())
        }))
    }

    /// Step 2: PUTs raw bytes to the presigned URL.
    pub async fn put_bytes(
        &self,
        upload_url: &str,
        content_type: &str,
        bytes: &[u8],
    ) -> Result<()> {
        let resp = self
            .http
            .put(upload_url)
            .header("Content-Type", content_type)
            .body(bytes.to_vec())
            .send()
            .await
            .map_err(|e| ModStoreError::Request(e.to_string()))?;
        resp.error_for_status()
            .map(|_| ())
            .map_err(|e| ModStoreError::Request(format!("upload PUT failed: {e}")))
    }

    /// Step 3: submits the mod/release for moderation + security pipeline.
    pub async fn submit_mod(&self, req: &SubmitModRequest) -> Result<SubmitModResponse> {
        let mut last: Option<ModStoreError> = None;
        for base in &self.base_urls {
            let url = format!("{}/api/mods/submit", base.trim_end_matches('/'));
            let resp = self.bearer(self.http.post(&url).json(req)).send().await;
            match resp {
                Ok(r) => {
                    let status = r.status();
                    if status == reqwest::StatusCode::UNAUTHORIZED
                        || status == reqwest::StatusCode::FORBIDDEN
                    {
                        return Err(ModStoreError::Request(format!("submit rejected: {status}")));
                    }
                    match r.error_for_status() {
                        Ok(ok) => match ok.json::<SubmitModResponse>().await {
                            Ok(body) => return Ok(body),
                            Err(e) => last = Some(ModStoreError::Request(e.to_string())),
                        },
                        Err(e) => last = Some(ModStoreError::Request(e.to_string())),
                    }
                }
                Err(e) => last = Some(ModStoreError::Request(e.to_string())),
            }
        }
        Err(last.unwrap_or_else(|| {
            ModStoreError::NotAvailable("all Modstore endpoints are unavailable".into())
        }))
    }
}

/// Infers the submit `method` from a file name (mirrors the web UploadWizard).
pub fn infer_method(file_name: &str) -> &'static str {
    let lower = file_name.to_lowercase();
    if lower.ends_with(".dll") {
        "dll"
    } else if lower.ends_with(".lua") {
        "lua"
    } else if lower.ends_with(".py") {
        "py"
    } else if lower.ends_with(".go") {
        "go"
    } else {
        "zip"
    }
}

/// Minimal content-type guess for upload-url requests.
pub fn guess_content_type(file_name: &str) -> &'static str {
    let lower = file_name.to_lowercase();
    if lower.ends_with(".zip") {
        "application/zip"
    } else if lower.ends_with(".dll") {
        "application/octet-stream"
    } else if lower.ends_with(".lua") {
        "text/plain"
    } else if lower.ends_with(".py") {
        "text/x-python"
    } else if lower.ends_with(".go") {
        "text/plain"
    } else if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else {
        "application/octet-stream"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_endpoints() {
        assert!(ModStoreUploadClient::new(vec![]).is_err());
        assert!(ModStoreUploadClient::new(vec!["https://example.com".into()]).is_ok());
    }

    #[test]
    fn method_inference() {
        assert_eq!(infer_method("mod.ZIP"), "zip");
        assert_eq!(infer_method("x.dll"), "dll");
        assert_eq!(infer_method("init.lua"), "lua");
        assert_eq!(infer_method("tool.py"), "py");
        assert_eq!(infer_method("srv.go"), "go");
    }

    #[test]
    fn upload_url_shape_is_camel_case() {
        let req = UploadUrlRequest {
            kind: Some("mod".to_string()),
            mod_id: "m".to_string(),
            release_id: None,
            file_name: "mod.zip".to_string(),
            content_type: "application/zip".to_string(),
            file_size: 42,
        };
        let v = serde_json::to_value(&req).expect("json");
        assert_eq!(v["modId"], "m");
        assert_eq!(v["fileName"], "mod.zip");
        assert_eq!(v["fileSize"], 42);
    }
}
