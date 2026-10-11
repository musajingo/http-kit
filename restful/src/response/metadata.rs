use crate::pagination::PaginationMetadata;
use crate::response::ReservedKeyError;

/// General-purpose metadata container for API responses.
///
/// Custom fields serialize alongside pagination. Use [`Self::default`] when
/// adding custom fields to a response without pagination.
///
/// ```
/// use restful::pagination::offset::OffsetPaginationMetadata;
/// use restful::response::ResponseMetadata;
///
/// let mut meta = ResponseMetadata::paginated(OffsetPaginationMetadata::new(40, 1, 20));
/// meta.set("requestId", serde_json::json!("req_123")).unwrap();
///
/// let json = serde_json::to_value(&meta).unwrap();
/// assert_eq!(json["pagination"]["nextPage"], 2);
/// assert_eq!(json["requestId"], "req_123");
/// ```
#[derive(Default, Debug, Clone, serde::Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ResponseMetadata {
    /// Pagination metadata, if the response is paginated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pagination: Option<PaginationMetadata>,

    /// Consumer-defined metadata fields.
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}

impl ResponseMetadata {
    /// Create new empty metadata.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create metadata for a paginated response.
    ///
    /// ```
    /// use restful::pagination::cursor::CursorPaginationMetadata;
    /// use restful::response::ResponseMetadata;
    ///
    /// let meta = ResponseMetadata::paginated(CursorPaginationMetadata {
    ///     has_more: true,
    ///     next_cursor: Some("abc".to_owned()),
    ///     limit: 20,
    /// });
    /// let json = serde_json::to_value(&meta).unwrap();
    /// assert_eq!(json["pagination"]["nextCursor"], "abc");
    /// ```
    pub fn paginated(pagination: impl Into<PaginationMetadata>) -> Self {
        Self::new().with_pagination(pagination)
    }

    /// Set or replace the pagination metadata.
    pub fn with_pagination(mut self, pagination: impl Into<PaginationMetadata>) -> Self {
        self.pagination = Some(pagination.into());
        self
    }

    /// Set a custom metadata field, replacing any previous value for its key.
    ///
    /// Use [`serde_json::to_value`] to convert a typed value before adding it.
    /// Returns an error for the reserved `"pagination"` key; set it with
    /// [`Self::paginated`] or [`Self::with_pagination`].
    ///
    /// ```
    /// use restful::response::{ReservedKeyError, ResponseMetadata};
    ///
    /// let mut meta = ResponseMetadata::new();
    /// meta.set("requestId", serde_json::json!("req_123")).unwrap();
    /// assert_eq!(
    ///     meta.set("pagination", serde_json::json!({})),
    ///     Err(ReservedKeyError)
    /// );
    /// ```
    pub fn set(
        &mut self,
        key: impl Into<String>,
        value: serde_json::Value,
    ) -> Result<(), ReservedKeyError> {
        let key = key.into();
        if key == "pagination" {
            return Err(ReservedKeyError);
        }
        self.extra.insert(key, value);
        Ok(())
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
        let meta = ResponseMetadata::paginated(offset);
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
        let meta = ResponseMetadata::paginated(cursor);
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
        let meta = ResponseMetadata::paginated(cursor);
        let json = serde_json::to_value(&meta).unwrap();
        assert_eq!(json["pagination"]["hasMore"], true);
        assert_eq!(json["pagination"]["nextCursor"], "xyz");
        assert_eq!(json["pagination"]["limit"], 10);
    }

    #[test]
    fn test_response_metadata_empty_skips_pagination() {
        let meta = ResponseMetadata::default();
        let json = serde_json::to_value(&meta).unwrap();
        assert_eq!(json, serde_json::json!({}));
    }

    #[test]
    fn custom_metadata_without_pagination_serializes_flat() {
        let mut meta = ResponseMetadata::default();
        meta.set("requestId", serde_json::json!("req_123")).unwrap();
        meta.set("details", serde_json::json!({"tags": ["a", "b"]}))
            .unwrap();
        meta.set("asOf", serde_json::Value::Null).unwrap();
        assert_eq!(
            serde_json::to_value(meta).unwrap(),
            serde_json::json!({
                "requestId": "req_123",
                "details": {"tags": ["a", "b"]},
                "asOf": null
            })
        );
    }

    #[test]
    fn repeated_custom_key_replaces_previous_value() {
        let mut meta = ResponseMetadata::default();
        meta.set("count", serde_json::json!(1)).unwrap();
        meta.set("count", serde_json::json!(2)).unwrap();
        assert_eq!(serde_json::to_string(&meta).unwrap(), r#"{"count":2}"#);
    }

    #[test]
    fn pagination_key_is_reserved_without_pagination() {
        let mut meta = ResponseMetadata::default();
        assert_eq!(
            meta.set("pagination", serde_json::Value::Null),
            Err(ReservedKeyError)
        );
    }

    #[test]
    fn custom_field_cannot_overwrite_pagination() {
        let mut meta = ResponseMetadata::paginated(OffsetPaginationMetadata::new(10, 1, 5));
        assert_eq!(
            meta.set("pagination", serde_json::Value::Null),
            Err(ReservedKeyError)
        );
        let json = serde_json::to_value(&meta).unwrap();
        assert!(json["pagination"].is_object());
    }

    #[cfg(feature = "utoipa")]
    #[test]
    fn schema_allows_custom_metadata_fields() {
        let schema =
            serde_json::to_value(<ResponseMetadata as utoipa::PartialSchema>::schema()).unwrap();
        assert!(schema["properties"].get("pagination").is_some());
        assert!(schema["properties"].get("extra").is_none());
        assert!(schema["additionalProperties"].is_object());
        assert!(schema["additionalProperties"].get("type").is_none());
    }
}
