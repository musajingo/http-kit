//! Field selection from the `?fields=` query parameter.
//!
//! [`Projection`] parses and validates the selection; [`ProjectionField`]
//! lists a type's selectable fields and is usually derived (`macros` feature).
//! The application builds the selected response from the parsed fields.

#[cfg(feature = "pgkit")]
mod column_selections;
mod error;
mod field;
mod parser;
mod utilities;

#[cfg(feature = "pgkit")]
pub use column_selections::IntoColumnSelections;
pub use error::{ProjectionError, ProjectionErrors};
pub use field::ProjectionField;
pub use parser::Projection;

// –– Re-export macros ––
#[cfg(feature = "macros")]
pub use restful_macros::{PartialResponse, ProjectionField, projection};
