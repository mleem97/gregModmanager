//! `greg-platform`: filesystem, OS integration and workspace management.

pub mod error;
pub mod filelog;
pub mod instance;
pub mod paths;
pub mod prefs;
pub mod process;
pub mod protocol;
pub mod repro;
pub mod telemetry;
pub mod workspace;

pub use error::{PlatformError, Result};
