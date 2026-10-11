//! Cursor (keyset) pagination metadata and query-field macro.
//!
//! The application creates and interprets cursors; this module only carries
//! them.

mod metadata;

pub use metadata::CursorPaginationMetadata;

#[cfg(feature = "macros")]
pub use restful_macros::cursor_pagination;
