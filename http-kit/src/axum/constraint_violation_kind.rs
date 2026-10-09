/// The cause of a service-layer validation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, strum::Display)]
#[serde(rename_all = "PascalCase")]
#[strum(serialize_all = "PascalCase")]
pub enum ConstraintViolationKind {
    /// A business rule was violated.
    BusinessLogic,
    /// A reference or uniqueness constraint failed.
    DataIntegrity,
    /// The current state does not allow the operation.
    InvalidState,
}
