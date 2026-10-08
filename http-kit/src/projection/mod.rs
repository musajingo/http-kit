#[cfg(feature = "with-pgkit")]
mod column_selections;
mod error;
mod field;
mod parser;
mod utilities;

#[cfg(feature = "with-pgkit")]
pub use column_selections::IntoColumnSelections;
pub use error::{ProjectionError, ProjectionErrors};
pub use field::ProjectionField;
pub use parser::Projection;

// –– Re-export macros ––
pub use http_kit_macros::{PartialResponse, ProjectionField, projection};
