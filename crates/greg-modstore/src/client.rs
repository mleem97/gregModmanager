//! Modstore HTTP client (`ModStoreUpdateService` query part).
//!
//! Tries each configured base URL in order (failover), mirroring the C#
//! `GetJsonWithFallbackAsync` / `PostJsonWithFallbackAsync` behavior.

use crate::error::{ModStoreError, Result};
use crate::models::{
    ModStoreCatalogResponse, ModStoreInstalledVersion, ModStoreUpdateCheckRequest,
    ModStoreUpdateResponse,
};

/// Catalog + update-check client.
#[derive(Debug, Clone)]
pub struct ModStoreClient {
    http: reqwest::Client,
    base_urls: Vec<String>,
}

impl ModStoreClient {
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
        })
    }

    /// Base URLs (for diagnostics).
    pub fn base_urls(&self) -> &[String] {
        &self.base_urls
    }

    /// Fetches the catalog (`GET /api/v1/mods`).
    pub async fn catalog(&self) -> Result<ModStoreCatalogResponse> {
        self.get_json("/api/v1/mods").await
    }

    /// Checks installed versions for updates
    /// (`POST /api/v1/mods/updates/check`).
    pub async fn check_updates(
        &self,
        installed: Vec<ModStoreInstalledVersion>,
    ) -> Result<ModStoreUpdateResponse> {
        let request = ModStoreUpdateCheckRequest { installed };
        self.post_json("/api/v1/mods/updates/check", &request).await
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let mut last: Option<ModStoreError> = None;
        for base in &self.base_urls {
            let url = format!("{}{path}", base.trim_end_matches('/'));
            match self.http.get(&url).send().await {
                Ok(resp) => match resp.error_for_status() {
                    Ok(ok) => match ok.json::<T>().await {
                        Ok(body) => return Ok(body),
                        Err(e) => last = Some(ModStoreError::Request(e.to_string())),
                    },
                    Err(e) => last = Some(ModStoreError::Request(e.to_string())),
                },
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
            match self.http.post(&url).json(body).send().await {
                Ok(resp) => match resp.error_for_status() {
                    Ok(ok) => match ok.json::<T>().await {
                        Ok(parsed) => return Ok(parsed),
                        Err(e) => last = Some(ModStoreError::Request(e.to_string())),
                    },
                    Err(e) => last = Some(ModStoreError::Request(e.to_string())),
                },
                Err(e) => last = Some(ModStoreError::Request(e.to_string())),
            }
        }
        Err(last.unwrap_or_else(|| {
            ModStoreError::NotAvailable("all Modstore endpoints are unavailable".into())
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_endpoints() {
        assert!(ModStoreClient::new(vec![]).is_err());
        assert!(ModStoreClient::new(vec!["https://example.com".into()]).is_ok());
    }
}
