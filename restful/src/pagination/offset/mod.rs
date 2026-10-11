mod metadata;

pub use metadata::OffsetPaginationMetadata;

#[cfg(feature = "macros")]
pub use restful_macros::offset_pagination;
