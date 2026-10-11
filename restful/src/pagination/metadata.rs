use crate::pagination::{cursor::CursorPaginationMetadata, offset::OffsetPaginationMetadata};

/// Pagination metadata that can be either offset-based or cursor-based.
///
/// Uses `#[serde(untagged)]` so the JSON output contains the variant's fields
/// directly, without a discriminator tag.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(untagged)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum PaginationMetadata {
    /// Traditional offset-based pagination.
    Offset(OffsetPaginationMetadata),

    /// Cursor-based keyset pagination.
    Cursor(CursorPaginationMetadata),
}

impl From<OffsetPaginationMetadata> for PaginationMetadata {
    fn from(offset: OffsetPaginationMetadata) -> Self {
        PaginationMetadata::Offset(offset)
    }
}

impl From<CursorPaginationMetadata> for PaginationMetadata {
    fn from(cursor: CursorPaginationMetadata) -> Self {
        PaginationMetadata::Cursor(cursor)
    }
}
