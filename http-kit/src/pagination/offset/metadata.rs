/// Offset-based pagination metadata for list endpoints.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct OffsetPaginationMetadata {
    /// Total number of items across all pages.
    pub total: u32,
    /// Current page number (1-indexed).
    pub page: u32,
    /// Maximum items per page.
    pub page_size: u32,
    /// Total number of pages (`ceil(total / page_size)`).
    pub total_pages: u32,
    /// Next page number, or `None` if on the last page.
    pub next_page: Option<u32>,
    /// Previous page number, or `None` if on the first page.
    pub prev_page: Option<u32>,
}

impl OffsetPaginationMetadata {
    /// Creates pagination metadata with auto-calculated navigation. `page` is
    /// clamped into `1..=total_pages` so an out-of-range page cannot produce
    /// phantom navigation links.
    pub fn new(total: u32, page: u32, page_size: u32) -> Self {
        let page_size = page_size.max(1);
        let total_pages = (total as u64).div_ceil(page_size as u64) as u32;
        let page = page.clamp(1, total_pages.max(1));
        let next_page = if page < total_pages {
            Some(page + 1)
        } else {
            None
        };
        let prev_page = if page > 1 { Some(page - 1) } else { None };

        Self {
            total,
            page,
            page_size,
            total_pages,
            next_page,
            prev_page,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_metadata_new() {
        let meta = OffsetPaginationMetadata::new(100, 1, 20);
        assert_eq!(meta.total_pages, 5);
        assert_eq!(meta.next_page, Some(2));
        assert_eq!(meta.prev_page, None);

        let meta = OffsetPaginationMetadata::new(100, 3, 20);
        assert_eq!(meta.next_page, Some(4));
        assert_eq!(meta.prev_page, Some(2));

        let meta = OffsetPaginationMetadata::new(100, 5, 20);
        assert_eq!(meta.next_page, None);
        assert_eq!(meta.prev_page, Some(4));

        let meta = OffsetPaginationMetadata::new(0, 1, 20);
        assert_eq!(meta.total_pages, 0);
        assert_eq!(meta.next_page, None);
        assert_eq!(meta.prev_page, None);
    }

    #[test]
    fn out_of_range_page_is_clamped() {
        let meta = OffsetPaginationMetadata::new(100, 0, 20);
        assert_eq!(meta.page, 1);
        assert_eq!(meta.next_page, Some(2));
        assert_eq!(meta.prev_page, None);

        let meta = OffsetPaginationMetadata::new(100, 999, 20);
        assert_eq!(meta.page, 5);
        assert_eq!(meta.next_page, None);
        assert_eq!(meta.prev_page, Some(4));

        let meta = OffsetPaginationMetadata::new(0, 999, 20);
        assert_eq!(meta.page, 1);
        assert_eq!(meta.next_page, None);
        assert_eq!(meta.prev_page, None);
    }

    #[test]
    fn zero_page_size_clamps_to_one() {
        let meta = OffsetPaginationMetadata::new(100, 1, 0);
        assert_eq!(meta.page_size, 1);
        assert_eq!(meta.total_pages, 100);
    }

    #[test]
    fn large_totals_no_overflow() {
        let meta = OffsetPaginationMetadata::new(20_000_000, 1, 100);
        assert_eq!(meta.total_pages, 200_000);
    }
}
