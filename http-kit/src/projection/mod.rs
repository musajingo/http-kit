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
pub use http_kit_macros::{PartialResponse, ProjectionField, projection};
