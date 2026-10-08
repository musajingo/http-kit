//! Validation error types for service-layer validation.
//!
//! This module provides structured validation errors that occur during business logic
//! validation in services. These are distinct from input validation errors (handled by
//! the `validator` crate) and represent domain-level constraint violations.
//!
//! # Usage
//!
//! ```rust
//! use validation::{ValidationError, ValidationErrorKind};
//!
//! // Business logic violation
//! let error = ValidationError {
//!     field: "email".to_string(),
//!     message: "Email must be verified before requesting shipment".to_string(),
//!     kind: ValidationErrorKind::BusinessLogic,
//!     params: None,
//! };
//!
//! // Data integrity violation with context
//! let error = ValidationError {
//!     field: "country_id".to_string(),
//!     message: "Country does not exist".to_string(),
//!     kind: ValidationErrorKind::DataIntegrity,
//!     params: Some(serde_json::json!({
//!         "provided_id": "invalid-uuid"
//!     })),
//! };
//! ```

use super::error_kind::ValidationErrorKind;

/// A structured validation error with field context and categorization.
///
/// This error type provides detailed information about validation failures that
/// occur during service-layer business logic checks. It captures which field failed,
/// why it failed, and optional contextual parameters for debugging or logging.
///
/// # Fields
///
/// | Field     | Description |
/// |-----------|-------------|
/// | `field`   | The name of the field or parameter that failed validation |
/// | `message` | A human-readable description of the validation failure |
/// | `kind`    | The category of validation error (see [`ValidationErrorKind`]) |
/// | `params`  | Optional JSON object with additional context (e.g., provided values, limits) |
///
/// # Error Display
///
/// The error formats as: `"<field>" failed validation with message <message> (validation kind: <kind>)`
///
/// # Examples
///
/// Creating a simple validation error:
///
/// ```rust
/// use validation::{ValidationError, ValidationErrorKind};
///
/// let error = ValidationError {
///     field: "status".to_string(),
///     message: "Cannot cancel a completed order".to_string(),
///     kind: ValidationErrorKind::InvalidState,
///     params: None,
/// };
/// ```
///
/// Creating an error with additional context:
///
/// ```rust
/// use validation::{ValidationError, ValidationErrorKind};
///
/// let error = ValidationError {
///     field: "weight".to_string(),
///     message: "Package weight exceeds courier maximum".to_string(),
///     kind: ValidationErrorKind::BusinessLogic,
///     params: Some(serde_json::json!({
///         "provided_weight_kg": 35.5,
///         "max_weight_kg": 30.0,
///         "courier": "FedEx"
///     })),
/// };
/// ```
#[derive(Debug, thiserror::Error)]
#[error("\"{field}\" failed validation with message {message} (validation kind: {kind})")]
pub struct ValidationError {
    /// The name of the field or parameter that failed validation.
    ///
    /// Use dot notation for nested fields (e.g., `"address.postal_code"`).
    pub field: String,

    /// A human-readable description of the validation failure.
    ///
    /// This message should be suitable for display to end users and clearly
    /// explain why the validation failed.
    pub message: String,

    /// The category of validation error.
    ///
    /// See [`ValidationErrorKind`] for the available categories and when to use each.
    pub kind: ValidationErrorKind,

    /// Optional additional context about the validation failure.
    ///
    /// Use this to include relevant values, limits, or other debugging information.
    /// Common uses include:
    ///
    /// - The value that was provided (`"provided_id": "..."`)
    /// - Constraint limits (`"max_weight_kg": 30.0`)
    /// - Related entity information (`"courier": "FedEx"`)
    pub params: Option<serde_json::Value>,
}
