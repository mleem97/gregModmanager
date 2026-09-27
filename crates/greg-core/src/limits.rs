//! Steam Workshop limits shared by checks, publish and hashing.

/// Maximum Workshop title length.
pub const MAX_TITLE_LENGTH: usize = 128;
/// Maximum Workshop description length.
pub const MAX_DESCRIPTION_LENGTH: usize = 8000;
/// Maximum main preview image size: 1 MiB.
pub const MAX_PREVIEW_IMAGE_BYTES: i64 = 1_048_576;
/// Maximum 32-bit Workshop file id accepted by the game backend.
pub const MAX_WORKSHOP_FILE_ID: u64 = 4_294_967_295;

/// True for usable Workshop file ids (non-zero, 32-bit).
pub fn is_usable_workshop_id(file_id: u64) -> bool {
    file_id != 0 && file_id <= MAX_WORKSHOP_FILE_ID
}
