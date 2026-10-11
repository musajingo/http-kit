mod metadata;

pub use metadata::CursorPaginationMetadata;

#[cfg(feature = "macros")]
pub use restful_macros::cursor_pagination;
