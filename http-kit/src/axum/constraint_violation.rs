use super::constraint_violation_kind::ConstraintViolationKind;

/// A service-layer constraint violation, distinct from `validator::ValidationError`.
#[derive(Debug, thiserror::Error)]
#[error("\"{field}\" failed validation with message {message} (validation kind: {kind})")]
pub struct ConstraintViolation {
    /// Field path, such as `address.postal_code`.
    pub field: String,
    pub message: String,
    pub kind: ConstraintViolationKind,
    /// Optional error details.
    pub params: Option<serde_json::Value>,
}
