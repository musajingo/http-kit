use chrono::{DateTime, Utc};

use crate::pagination::{cursor::CursorPaginationMetadata, offset::OffsetPaginationMetadata};

/// General-purpose metadata container for API responses.
#[derive(Default, Debug, Clone, serde::Serialize, bon::Builder)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ResponseMetadata {
    /// Pagination metadata, if the response is paginated.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pagination: Option<PaginationMetadata>,

    /// Display-currency metadata, present only when the response's money fields
    /// were converted into a client-requested display currency.
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<CurrencyMetadata>,
}

/// Display-currency metadata: the currency the response's money fields were
/// restated in, the base ledger currency, the USD→display rate applied, and when
/// that rate was last refreshed (`null` for the USD identity or a seeded rate
/// never fetched from the provider).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct CurrencyMetadata {
    /// ISO 4217 code the money fields are expressed in (e.g. `"CNY"`).
    pub display: String,
    /// The base/ledger currency the amounts are stored in — always `"USD"`.
    pub base: String,
    /// The applied USD→display rate, as a decimal string (e.g. `"7.2"`).
    pub rate: String,
    /// When the applied rate was last fetched from the provider; `null` for the
    /// USD identity or a seeded rate never fetched.
    pub as_of: Option<DateTime<Utc>>,
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

    fn currency() -> CurrencyMetadata {
        CurrencyMetadata {
            display: "CNY".to_string(),
            base: "USD".to_string(),
            rate: "7.2".to_string(),
            as_of: None,
        }
    }

    #[test]
    fn test_response_metadata_currency_serializes_block() {
        let meta = ResponseMetadata::builder().currency(currency()).build();
        let json = serde_json::to_value(&meta).unwrap();
        assert_eq!(json["currency"]["display"], "CNY");
        assert_eq!(json["currency"]["base"], "USD");
        assert_eq!(json["currency"]["rate"], "7.2");
        assert!(json["currency"]["asOf"].is_null());
        assert!(json.get("pagination").is_none());
    }

    #[test]
    fn test_response_metadata_currency_coexists_with_pagination() {
        let cursor = CursorPaginationMetadata {
            has_more: true,
            next_cursor: Some("xyz".to_string()),
            limit: 10,
        };
        let meta = ResponseMetadata::builder()
            .pagination(cursor)
            .currency(currency())
            .build();
        let json = serde_json::to_value(&meta).unwrap();
        assert_eq!(json["pagination"]["hasMore"], true);
        assert_eq!(json["currency"]["display"], "CNY");
    }

    #[test]
    fn test_response_metadata_maybe_currency_absent_skips_block() {
        let meta = ResponseMetadata::builder()
            .maybe_currency(None::<CurrencyMetadata>)
            .build();
        let json = serde_json::to_value(&meta).unwrap();
        assert!(json.get("currency").is_none());
    }
}
