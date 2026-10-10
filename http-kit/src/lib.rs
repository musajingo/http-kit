pub mod ordering;

/// PATCH-body field semantics: unchanged / clear / set.
#[cfg(feature = "field-delta")]
pub use field_delta::Delta;

pub mod pagination;

#[cfg(feature = "response")]
pub mod response;

#[cfg(feature = "projection")]
pub mod projection;

/// Validated Axum request extractors.
#[cfg(feature = "axum")]
pub mod axum;
