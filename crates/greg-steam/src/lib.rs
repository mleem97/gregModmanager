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
pub mod web_api;

pub use backend::{connect, SteamBackend, UnavailableBackend};
pub use error::{Result, SteamError};
pub use models::{DecodedPreviewImage, ItemDetail, WorkshopItemMetadata, EXAMPLE_WORKSHOP_ITEM_ID};
pub use web_api::{
    download_and_cache_preview_image, download_and_cache_preview_image_blocking,
    download_preview_image, download_preview_image_blocking, parse_get_details_response,
    parse_persona_name_response,
};
