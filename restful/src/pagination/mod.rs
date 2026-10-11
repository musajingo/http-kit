//! Offset and cursor pagination.
//!
//! Each submodule pairs a response-metadata type with a query-field macro
//! (`macros` feature). [`PaginationMetadata`] wraps either kind for a
//! response's metadata.

pub mod offset;

pub mod cursor;

mod metadata;
pub use metadata::PaginationMetadata;
