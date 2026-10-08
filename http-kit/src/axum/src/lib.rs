//! # Validation - Axum Extractors and Service-Layer Errors
//!
//! This crate provides:
//!
//! - Axum extractors that automatically validate incoming requests using the
//!   [`validator`] framework — payloads are validated before handler code runs.
//! - Service-layer validation error types ([`ValidationError`],
//!   [`ValidationErrorKind`]) for business-logic constraint violations.
//!
//! ## Overview
//!
//! This crate sits on top of the workspace validation packages:
//!
//! | Crate | Purpose |
//! |-------|---------|
//! | `validator` | Core traits, built-in validators, and the `Validate` trait |
//! | `validator_derive` | `#[derive(Validate)]` macro |
//! | **`validation`** (this crate) | Axum request extractors, service-layer errors |
//!
//! Consumers using `#[derive(Validate)]` must depend on `validator` directly
//! (the derive macro expands to `::validator::...` paths), so all `Validate`
//! / `validator::*` imports continue to come from the `validator` crate.
//! This crate intentionally does not re-export those items.
//!
//! ## Available Extractors
//!
//! | Extractor | Use Case |
//! |-----------|----------|
//! | [`ValidatedJsonPayload<T>`] | JSON request bodies |
//! | [`ValidatedFormPayload<T>`] | Form-encoded request bodies |
//! | [`ValidatedQuery<T>`] | URL query string parameters |
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use axum::{Router, routing::post};
//! use serde::Deserialize;
//! use validator::Validate;
//! use validation::ValidatedJsonPayload;
//!
//! #[derive(Deserialize, Validate)]
//! struct CreateUser {
//!     #[validate(email)]
//!     email: String,
//!
//!     #[validate(length(min = 8, max = 100))]
//!     password: String,
//!
//!     #[validate(range(min = 18))]
//!     age: u8,
//! }
//!
//! async fn create_user(
//!     ValidatedJsonPayload(user): ValidatedJsonPayload<CreateUser>
//! ) -> &'static str {
//!     // `user` is guaranteed to be valid here
//!     "User created successfully!"
//! }
//!
//! let app = Router::new().route("/users", post(create_user));
//! ```
//!
//! ## Service-layer vs. per-field validation errors
//!
//! This crate's [`ValidationError`] is a *service-layer* error and is
//! distinct from `validator::ValidationError` (the per-field error returned
//! from `#[validate(custom = ...)]` callbacks). Use this crate's type to
//! express domain constraint violations raised by services; use the
//! validator's type inside custom-validator callbacks.
//!
//! ## Error Responses
//!
//! When extractor validation fails, the extractor returns a structured JSON
//! error response with HTTP status 422 (Unprocessable Entity):
//!
//! ```json
//! {
//!   "error": {
//!     "type": "validation_error",
//!     "message": "Validation failed",
//!     "details": {
//!       "email": [
//!         {
//!           "code": "email",
//!           "message": "Invalid email format",
//!           "params": { "value": "not-an-email" }
//!         }
//!       ]
//!     }
//!   }
//! }
//! ```

mod error;
mod error_kind;
mod extractors;

pub use error::ValidationError;
pub use error_kind::ValidationErrorKind;
pub use extractors::{ValidatedFormPayload, ValidatedJsonPayload, ValidatedQuery};
