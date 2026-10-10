//! Validated Axum extractors and service-layer validation errors.
//!
//! Enable the `axum` feature to use this module. It also enables `response`.
//! Payload types must implement `serde::Deserialize` and `valid8r::Validate`.
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
