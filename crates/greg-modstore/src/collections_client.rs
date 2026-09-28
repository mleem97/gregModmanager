//! Online Collections client (P0: Modpacks/Collections sync with the Modstore).
//!
//! Endpoints (all camelCase, Bearer session token where auth is required):
//! - `GET {base}/api/collections` (public; `?owner=me` for own)
//! - `POST {base}/api/collections` (auth)
//! - `GET|PATCH|DELETE {base}/api/collections/{id}` (GET public w/ private guard)
//! - `POST|DELETE {base}/api/collections/{id}/mods` (auth, owner)
//! - `GET {base}/api/collections/share?code=...` (public share link)
//! - `POST {base}/api/collections/share` (auth, owner)
//!
//! Failover across base URLs; 401/403 are returned immediately (no host failover).

use serde::{Deserialize, Serialize};

use crate::error::{ModStoreError, Result};

/// Collection summary as returned by list endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct OnlineCollection {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub image_url: Option<String>,
    pub creator: Option<String>,
    pub creator_id: Option<String>,
    pub mod_count: i64,
    pub total: Option<i64>,
}

/// `GET /api/collections` response.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct CollectionsListResponse {
    pub collections: Vec<OnlineCollection>,
    pub total: i64,
}

/// `POST /api/collections` request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateCollectionRequest {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mod_ids: Option<Vec<String>>,
}

/// `POST /api/collections` response.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct CreateCollectionResponse {
    pub success: bool,
    pub collection: OnlineCollection,
}

/// Detail mod entry inside `GET /api/collections/{id}`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct CollectionModEntry {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub author: Option<String>,
    pub latest_version: Option<String>,
}

/// `GET /api/collections/{id}` response.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct CollectionDetailResponse {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub image_url: Option<String>,
    pub creator: Option<String>,
    pub creator_id: Option<String>,
    pub mod_count: i64,
    pub mods: Vec<CollectionModEntry>,
}

/// `PATCH /api/collections/{id}` request.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateCollectionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

/// Share-link detail (`GET /api/collections/share?code=...`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ShareDetailResponse {
    pub collection: OnlineCollection,
    pub mods: Vec<CollectionModEntry>,
}

/// `POST /api/collections/share` response.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct CreateShareResponse {
    pub share_code: String,
    pub short_code: String,
    pub share_url: String,
    pub collection_id: String,
}

/// Online collections client.
#[derive(Debug, Clone)]
pub struct CollectionsClient {
    http: reqwest::Client,
    base_urls: Vec<String>,
    access_token: Option<String>,
}

impl CollectionsClient {
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

    /// Attaches the session token (Bearer).
    pub fn with_token(mut self, access_token: impl Into<String>) -> Self {
        let token = access_token.into();
        self.access_token = if token.trim().is_empty() {
            None
        } else {
            Some(token)
        };
        self
    }

    fn authed(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self.access_token.as_deref() {
            Some(t) => req.bearer_auth(t),
            None => req,
        }
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, path: &str, auth: bool) -> Result<T> {
        let mut last: Option<ModStoreError> = None;
        for base in &self.base_urls {
            let url = format!("{}{path}", base.trim_end_matches('/'));
            let req = self.http.get(&url);
            let req = if auth { self.authed(req) } else { req };
            match req.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if auth
                        && (status == reqwest::StatusCode::UNAUTHORIZED
                            || status == reqwest::StatusCode::FORBIDDEN)
                    {
                        return Err(ModStoreError::Request(format!("collections GET {status}")));
                    }
                    match resp.error_for_status() {
                        Ok(ok) => match ok.json::<T>().await {
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

    async fn post_json<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let mut last: Option<ModStoreError> = None;
        for base in &self.base_urls {
            let url = format!("{}{path}", base.trim_end_matches('/'));
            let resp = self.authed(self.http.post(&url).json(body)).send().await;
            match resp {
                Ok(r) => {
                    let status = r.status();
                    if status == reqwest::StatusCode::UNAUTHORIZED
                        || status == reqwest::StatusCode::FORBIDDEN
                    {
                        return Err(ModStoreError::Request(format!("collections POST {status}")));
                    }
                    match r.error_for_status() {
                        Ok(ok) => match ok.json::<T>().await {
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

    /// Lists public collections (or own with `owner=me` + token).
    pub async fn list(&self, owner_me: bool) -> Result<CollectionsListResponse> {
        let path = if owner_me {
            "/api/collections?owner=me"
        } else {
            "/api/collections"
        };
        self.get_json(path, owner_me).await
    }

    /// Creates a collection (auth required).
    pub async fn create(&self, req: &CreateCollectionRequest) -> Result<CreateCollectionResponse> {
        self.post_json("/api/collections", req).await
    }

    /// Fetches collection detail incl. mods.
    pub async fn detail(&self, id: &str) -> Result<CollectionDetailResponse> {
        self.get_json(&format!("/api/collections/{id}"), false)
            .await
    }

    /// Adds a mod to a collection (auth + owner).
    pub async fn add_mod(&self, id: &str, mod_id: &str) -> Result<serde_json::Value> {
        let body = serde_json::json!({ "modId": mod_id });
        self.post_json(&format!("/api/collections/{id}/mods"), &body)
            .await
    }

    /// Resolves a share link (`GET /api/collections/share?code=...`).
    pub async fn share_detail(&self, code: &str) -> Result<ShareDetailResponse> {
        let encoded: String = code
            .bytes()
            .map(|b| {
                if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
                    (b as char).to_string()
                } else {
                    format!("%{b:02X}")
                }
            })
            .collect();
        self.get_json(&format!("/api/collections/share?code={encoded}"), false)
            .await
    }

    /// Creates a share link for an owned collection.
    pub async fn create_share(
        &self,
        collection_id: &str,
        expires_in_days: Option<i64>,
    ) -> Result<CreateShareResponse> {
        let body = serde_json::json!({
            "collectionId": collection_id,
            "expiresInDays": expires_in_days,
        });
        self.post_json("/api/collections/share", &body).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_endpoints() {
        assert!(CollectionsClient::new(vec![]).is_err());
        assert!(CollectionsClient::new(vec!["https://example.com".into()]).is_ok());
    }

    #[test]
    fn create_shape_is_camel_case() {
        let req = CreateCollectionRequest {
            title: "Pack".to_string(),
            description: None,
            is_public: Some(true),
            image_url: None,
            mod_ids: Some(vec!["m1".to_string()]),
        };
        let v = serde_json::to_value(&req).expect("json");
        assert_eq!(v["title"], "Pack");
        assert_eq!(v["isPublic"], true);
        assert_eq!(v["modIds"][0], "m1");
    }
}
