//! `greg-core`: domain models and pure logic for gregModmanager.
//!
//! No UI dependencies, no Steam client, no network. Everything here is
//! unit-testable without a game, Steam, or server.

pub mod docs;
pub mod error;
pub mod l10n;
pub mod limits;
pub mod log;
pub mod models;
pub mod prefs;
pub mod upload;
pub mod util;

pub use error::{CoreError, Result};
