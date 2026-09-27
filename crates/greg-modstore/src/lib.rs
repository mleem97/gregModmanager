//! `greg-modstore`: Modstore REST client, installs and auth.

pub mod auth_client;
pub mod client;
pub mod error;
pub mod install;
pub mod intent;
pub mod models;

pub use error::{ModStoreError, Result};
