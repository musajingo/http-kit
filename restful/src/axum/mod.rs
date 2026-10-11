//! Validated Axum request extractors.
//!
//! Enable the `axum` feature to use this module. It also enables `response`.
//! Payload types need `serde::Deserialize` and `Validate` from the
//! [`valid8r`](https://crates.io/crates/valid8r) crate.
//! Invalid payloads return structured API errors before the handler runs.
//! Error details key fields in camelCase: payload structs must use
//! `#[serde(rename_all = "camelCase")]` for error keys to match the wire.
//!
//! ```rust
//! use restful::axum::{ExtractorRejection, ValidatedForm, ValidatedJson, ValidatedQuery};
//! ```

mod form;
mod json;
mod query;
mod rejection;
mod utils;

pub use form::ValidatedForm;
pub use json::ValidatedJson;
pub use query::ValidatedQuery;
pub use rejection::ExtractorRejection;
