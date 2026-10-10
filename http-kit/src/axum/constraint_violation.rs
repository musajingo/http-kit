use super::constraint_violation_kind::ConstraintViolationKind;

/// A service-layer constraint violation, distinct from `valid8r::ValidationError`.
#[derive(Debug, thiserror::Error)]
#[error("\"{field}\" failed validation with message {message} (validation kind: {kind})")]
pub struct ConstraintViolation {
    /// Field path, such as `address.postal_code`.
    pub field: String,
    /// Human-readable explanation suitable for display to the user.
    pub message: String,
    /// The category of constraint that failed.
    pub kind: ConstraintViolationKind,
    /// Optional structured context, such as a limit or an invalid reference ID.
    pub params: Option<serde_json::Value>,
}
