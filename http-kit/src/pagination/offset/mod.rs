mod metadata;

pub use metadata::OffsetPaginationMetadata;

#[cfg(feature = "macros")]
pub use http_kit_macros::offset_pagination;
