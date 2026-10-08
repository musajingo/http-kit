/// Cursor pagination metadata included in list responses.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CursorPaginationMetadata {
    /// Whether there are more items after this page.
    #[cfg_attr(feature = "utoipa", schema(example = true))]
    pub has_more: bool,
    /// Opaque cursor to pass as `?cursor=` for the next page.
    /// `None` when `hasMore` is `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// The limit that was applied to this request.
    #[cfg_attr(feature = "utoipa", schema(example = 20))]
    pub limit: u32,
}

impl CursorPaginationMetadata {
    /// Creates cursor pagination metadata.
    pub fn new(limit: u32, next_cursor: Option<String>) -> Self {
        Self {
            has_more: next_cursor.is_some(),
            next_cursor,
            limit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_cursor_means_no_more() {
        let meta = CursorPaginationMetadata::new(20, None);
        assert!(!meta.has_more);
        assert!(meta.next_cursor.is_none());
        assert_eq!(meta.limit, 20);
    }

    #[test]
    fn some_cursor_means_has_more() {
        let meta = CursorPaginationMetadata::new(20, Some("opaque-token".to_string()));
        assert!(meta.has_more);
        assert_eq!(meta.next_cursor.as_deref(), Some("opaque-token"));
        assert_eq!(meta.limit, 20);
    }
}
