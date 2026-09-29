//! `greg-loader`: game, MelonLoader and mod installation logic.

pub mod channels;
pub mod content;
pub mod discovery;
pub mod error;
pub mod game;
pub mod health;
pub mod installers;
pub mod launch;
pub mod local_content;
pub mod native_config;
pub mod packs;
pub mod sync;
pub mod templates;

pub use error::{LoaderError, Result};

/// Today's date as `YYYY-MM-DD` (single source: `greg_core::util`).
pub fn date_today() -> String {
    greg_core::util::today_ymd()
}
