//! greg plugin channels (`IGregPluginChannelSource`, registry, `stable` /
//! `github` / `beta` sources port).

use std::sync::Arc;

use crate::content::PluginPackageInfo;
use crate::error::{LoaderError, Result};

/// Preferences key for the beta server URL.
pub const PREF_KEY_BETA_SERVER_URL: &str = "greg_beta_server_url";

/// A plugin distribution channel.
#[async_trait::async_trait(?Send)]
pub trait PluginChannelSource: Send + Sync {
    /// Stable channel name (`stable`, `github`, `beta`).
    fn channel_name(&self) -> &'static str;
    /// Installs a plugin package into `target_dir`.
    async fn install(&self, plugin: &PluginPackageInfo, target_dir: &std::path::Path)
        -> Result<()>;
}

/// Registry of all channels.
#[derive(Default)]
pub struct PluginChannelRegistry {
    sources: Vec<Arc<dyn PluginChannelSource>>,
}

impl PluginChannelRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registry with the built-in HTTP channels.
    pub fn with_defaults(beta_server_url: Option<String>) -> Self {
        let mut registry = Self::new();
        registry.register(Arc::new(StableChannelSource));
        registry.register(Arc::new(GitHubChannelSource));
        registry.register(Arc::new(BetaChannelSource { beta_server_url }));
        registry
    }

    /// Registers a source.
    pub fn register(&mut self, source: Arc<dyn PluginChannelSource>) {
        self.sources.push(source);
    }

    /// Resolves a channel by name.
    pub fn resolve(&self, channel: &str) -> Option<Arc<dyn PluginChannelSource>> {
        self.sources
            .iter()
            .find(|s| s.channel_name() == channel)
            .cloned()
    }

    /// All channel names.
    pub fn channels(&self) -> Vec<&'static str> {
        self.sources.iter().map(|s| s.channel_name()).collect()
    }
}

fn http_client() -> reqwest::Client {
    reqwest::Client::new()
}

async fn download_to(url: &str, dest: &std::path::Path) -> Result<()> {
    let bytes = http_client()
        .get(url)
        .send()
        .await
        .map_err(|e| LoaderError::Network(e.to_string()))?
        .error_for_status()
        .map_err(|e| LoaderError::Network(e.to_string()))?
        .bytes()
        .await
        .map_err(|e| LoaderError::Network(e.to_string()))?;
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(crate::error::io_err)?;
    }
    std::fs::write(dest, &bytes).map_err(crate::error::io_err)?;
    Ok(())
}

/// Stable channel: direct download URLs.
pub struct StableChannelSource;

#[async_trait::async_trait(?Send)]
impl PluginChannelSource for StableChannelSource {
    fn channel_name(&self) -> &'static str {
        "stable"
    }

    async fn install(
        &self,
        plugin: &PluginPackageInfo,
        target_dir: &std::path::Path,
    ) -> Result<()> {
        if plugin.download_url.trim().is_empty() {
            return Err(LoaderError::Other(
                "stable plugin has no download URL".into(),
            ));
        }
        let file_name = if plugin.name.trim().is_empty() {
            "plugin.dll".to_string()
        } else if plugin.name.ends_with(".dll") {
            plugin.name.clone()
        } else {
            format!("{}.dll", plugin.name)
        };
        download_to(&plugin.download_url, &target_dir.join(file_name)).await
    }
}

/// GitHub channel: `{owner}/{repo}` releases (`{plugin}.dll` artefacts).
pub struct GitHubChannelSource;

#[async_trait::async_trait(?Send)]
impl PluginChannelSource for GitHubChannelSource {
    fn channel_name(&self) -> &'static str {
        "github"
    }

    async fn install(
        &self,
        plugin: &PluginPackageInfo,
        target_dir: &std::path::Path,
    ) -> Result<()> {
        // `download_url` carries `owner/repo` (or a full release asset URL).
        let url = plugin.download_url.trim();
        let direct = if url.starts_with("http") {
            url.to_string()
        } else {
            format!(
                "https://github.com/{url}/releases/latest/download/{}.dll",
                plugin.name
            )
        };
        let file_name = if plugin.name.ends_with(".dll") {
            plugin.name.clone()
        } else {
            format!("{}.dll", plugin.name)
        };
        download_to(&direct, &target_dir.join(file_name)).await
    }
}

/// Beta channel: custom server base URL + plugin path.
pub struct BetaChannelSource {
    /// Configured server base URL (preferences).
    pub beta_server_url: Option<String>,
}

#[async_trait::async_trait(?Send)]
impl PluginChannelSource for BetaChannelSource {
    fn channel_name(&self) -> &'static str {
        "beta"
    }

    async fn install(
        &self,
        plugin: &PluginPackageInfo,
        target_dir: &std::path::Path,
    ) -> Result<()> {
        let Some(base) = self
            .beta_server_url
            .as_deref()
            .filter(|s| !s.trim().is_empty())
        else {
            return Err(LoaderError::Other(
                "beta server URL is not configured".into(),
            ));
        };
        let url = format!("{}/{}.dll", base.trim_end_matches('/'), plugin.name);
        download_to(&url, &target_dir.join(format!("{}.dll", plugin.name))).await
    }
}
