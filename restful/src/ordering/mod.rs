//! Sort order for list endpoints.
//!
//! With the `macros` feature, `#[order_by(Fields)]` adds `order_by` and
//! `order_dir` query fields to a struct.

mod direction;

pub use direction::OrderByDirection;

#[cfg(feature = "macros")]
pub use restful_macros::order_by;
