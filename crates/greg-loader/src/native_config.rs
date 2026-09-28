//! Native `config.json` / `modconfig.json` store (`NativeModConfigStore` port).

use std::path::{Path, PathBuf};

use crate::content::{ModOptionsConfigFile, NativeModConfig};
use crate::error::{io_err, json_err, Result};

/// `<root>/content/config.json` (or `<root>/config.json` for plain roots).
pub fn config_json_path(root: &Path) -> PathBuf {
    let nested = root.join("content").join("config.json");
    if nested.is_file() || !root.join("config.json").is_file() {
        nested
    } else {
        root.join("config.json")
    }
}

/// `<root>/content/modconfig.json` (optional runtime options).
pub fn mod_options_json_path(root: &Path) -> PathBuf {
    root.join("content").join("modconfig.json")
}

/// Loads the native config (default when missing).
pub fn load_config(root: &Path) -> Result<NativeModConfig> {
    let path = config_json_path(root);
    if !path.is_file() {
        return Ok(NativeModConfig::default());
    }
    let text = std::fs::read_to_string(&path).map_err(io_err)?;
    serde_json::from_str(&text).map_err(json_err)
}

/// Saves the native config.
pub fn save_config(root: &Path, config: &NativeModConfig) -> Result<()> {
    let path = config_json_path(root);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(io_err)?;
    }
    let text = serde_json::to_string_pretty(config).map_err(json_err)?;
    std::fs::write(path, text).map_err(io_err)?;
    Ok(())
}

/// Loads the optional runtime options (default when missing).
pub fn load_mod_options(root: &Path) -> Result<ModOptionsConfigFile> {
    let path = mod_options_json_path(root);
    if !path.is_file() {
        return Ok(ModOptionsConfigFile::default());
    }
    let text = std::fs::read_to_string(&path).map_err(io_err)?;
    serde_json::from_str(&text).map_err(json_err)
}

/// Saves the optional runtime options.
pub fn save_mod_options(root: &Path, options: &ModOptionsConfigFile) -> Result<()> {
    let path = mod_options_json_path(root);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(io_err)?;
    }
    let text = serde_json::to_string_pretty(options).map_err(json_err)?;
    std::fs::write(path, text).map_err(io_err)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "greg-nativecfg-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let cfg = NativeModConfig {
            mod_name: "My Workshop Mod".into(),
            ..Default::default()
        };
        save_config(&dir, &cfg).unwrap();
        assert_eq!(load_config(&dir).unwrap(), cfg);
        let opts = ModOptionsConfigFile {
            schema_version: 1,
            mod_kind: "greg".into(),
            ..Default::default()
        };
        save_mod_options(&dir, &opts).unwrap();
        assert_eq!(load_mod_options(&dir).unwrap(), opts);
        std::fs::remove_dir_all(&dir).ok();
    }
}
