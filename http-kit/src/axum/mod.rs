//! Validated Axum extractors and service-layer validation errors.
//!
//! Enable the `axum` feature to use this module. It also enables `response`.
//! Payload types need `serde::Deserialize` and `Validate` from the
//! [`valid8r`](https://crates.io/crates/valid8r) crate.
//! Invalid payloads return structured API errors before the handler runs.
//!
//! ```rust
//! use http_kit::axum::{
//!     ConstraintViolation, ConstraintViolationKind, ExtractorRejection,
//!     ValidatedForm, ValidatedJson, ValidatedQuery,
//! };
//! ```

mod constraint_violation;
mod constraint_violation_kind;
mod extractors;

pub use constraint_violation::ConstraintViolation;
pub use constraint_violation_kind::ConstraintViolationKind;
pub use extractors::{ExtractorRejection, ValidatedForm, ValidatedJson, ValidatedQuery};
