//! # Validation Error Kinds
//!
//! - [`ValidationErrorKind::BusinessLogic`] - Policy violations (e.g., unverified email)
//! - [`ValidationErrorKind::DataIntegrity`] - Reference violations (e.g., invalid foreign key)
//! - [`ValidationErrorKind::InvalidState`] - State conflicts (e.g., modifying locked resource)
//!

/// Categorizes validation errors by their underlying cause.
///
/// This enum helps API consumers understand *why* a validation failed, enabling
/// appropriate error handling and user messaging. Each variant maps to a distinct
/// category of domain constraint.
///
/// # Serialization
///
/// Variants serialize to PascalCase (e.g., `"BusinessLogic"`, `"DataIntegrity"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, strum::Display)]
#[serde(rename_all = "PascalCase")]
#[strum(serialize_all = "PascalCase")]
pub enum ValidationErrorKind {
    /// A violation of business rules or policies.
    ///
    /// Use this for domain-specific constraints that aren't related to data integrity
    /// or resource state. These represent policy-level rules defined by the business
    /// that must be satisfied before an operation can proceed.
    ///
    /// # Examples
    ///
    /// - Customer hasn't verified their email before requesting shipment
    BusinessLogic,

    /// A database or cross-module reference constraint violation.
    ///
    /// Use this when a referenced entity doesn't exist or violates a uniqueness
    /// constraint. This includes both database-level constraints (foreign keys,
    /// unique indexes) and cross-module reference validation.
    ///
    /// # Examples
    ///
    /// - Referencing a non-existent country ID from the geo module
    /// - Attempting to create a duplicate entry that violates a unique constraint
    DataIntegrity,

    /// The resource exists but its current state prevents the operation.
    ///
    /// Use this when an entity is in a lifecycle state that conflicts with the
    /// requested operation. The entity exists and references are valid, but the
    /// operation is not permitted given the entity's current status.
    ///
    /// # Examples
    ///
    /// - Modifying a shipment that has been locked for processing
    /// - Activating a user account that is already active
    /// - Cancelling an order that has already been completed or shipped
    /// - Deleting a warehouse that has pending shipments
    InvalidState,
}
