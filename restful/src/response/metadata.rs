use crate::pagination::{cursor::CursorPaginationMetadata, offset::OffsetPaginationMetadata};

/// General-purpose metadata container for API responses.
#[derive(Default, Debug, Clone, serde::Serialize, bon::Builder)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ResponseMetadata {
    /// Pagination metadata, if the response is paginated.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pagination: Option<PaginationMetadata>,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_metadata_default_is_empty() {
        let meta = ResponseMetadata::default();
        assert!(meta.pagination.is_none());
    }

    #[test]
    fn test_response_metadata_with_offset_pagination() {
        let offset = OffsetPaginationMetadata::new(100, 2, 20);
        let meta = ResponseMetadata::builder().pagination(offset).build();
        assert!(matches!(
            meta.pagination,
            Some(PaginationMetadata::Offset(_))
        ));
    }

    #[test]
    fn test_response_metadata_with_cursor_pagination() {
        let cursor = CursorPaginationMetadata {
            has_more: true,
            next_cursor: Some("abc".to_string()),
            limit: 20,
        };
        let meta = ResponseMetadata::builder().pagination(cursor).build();
        assert!(matches!(
            meta.pagination,
            Some(PaginationMetadata::Cursor(_))
        ));
    }

    #[test]
    fn test_response_metadata_serializes_nested_pagination() {
        let cursor = CursorPaginationMetadata {
            has_more: true,
            next_cursor: Some("xyz".to_string()),
            limit: 10,
        };
        let meta = ResponseMetadata::builder().pagination(cursor).build();
        let json = serde_json::to_value(&meta).unwrap();
        assert_eq!(json["pagination"]["hasMore"], true);
        assert_eq!(json["pagination"]["nextCursor"], "xyz");
        assert_eq!(json["pagination"]["limit"], 10);
    }

    #[test]
    fn test_response_metadata_empty_skips_pagination() {
        let meta = ResponseMetadata::builder().build();
        let json = serde_json::to_value(&meta).unwrap();
        assert!(json.get("pagination").is_none());
    }
}
