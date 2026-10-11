use crate::pagination::PaginationMetadata;

/// General-purpose metadata container for API responses.
#[derive(Default, Debug, Clone, serde::Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ResponseMetadata {
    /// Pagination metadata, if the response is paginated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pagination: Option<PaginationMetadata>,
}

impl ResponseMetadata {
    /// Metadata for a paginated response.
    pub fn new(pagination: impl Into<PaginationMetadata>) -> Self {
        Self {
            pagination: Some(pagination.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pagination::{cursor::CursorPaginationMetadata, offset::OffsetPaginationMetadata};

    #[test]
    fn test_response_metadata_default_is_empty() {
        let meta = ResponseMetadata::default();
        assert!(meta.pagination.is_none());
    }

    #[test]
    fn test_response_metadata_with_offset_pagination() {
        let offset = OffsetPaginationMetadata::new(100, 2, 20);
        let meta = ResponseMetadata::new(offset);
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
        let meta = ResponseMetadata::new(cursor);
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
        let meta = ResponseMetadata::new(cursor);
        let json = serde_json::to_value(&meta).unwrap();
        assert_eq!(json["pagination"]["hasMore"], true);
        assert_eq!(json["pagination"]["nextCursor"], "xyz");
        assert_eq!(json["pagination"]["limit"], 10);
    }

    #[test]
    fn test_response_metadata_empty_skips_pagination() {
        let meta = ResponseMetadata::default();
        let json = serde_json::to_value(&meta).unwrap();
        assert!(json.get("pagination").is_none());
    }
}
