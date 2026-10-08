//! Pagination attribute macros.
//!
//! - [`offset`] — implements `#[pagination]` (page / page_size, classic offset
//!   pagination with `LIMIT` / `OFFSET`).
//! - [`cursor`] — implements `#[cursor_pagination]` (opaque cursor + limit,
//!   keyset pagination with the N+1 hasMore-detection pattern).

pub(crate) mod cursor;
pub(crate) mod offset;
