//! `greg-steam`: Steam Workshop integration.
//!
//! All Steamworks calls go through [`backend::SteamBackend`]. Without a
//! running Steam client (or without the `steam` feature) every call fails
//! with a clear hint instead of crashing.

pub mod backend;
pub mod error;
pub mod inference;
pub mod models;
pub mod poller;
pub mod rate_limit;
pub mod service;
pub mod vdf;

pub use backend::{connect, SteamBackend, UnavailableBackend};
pub use error::{Result, SteamError};
