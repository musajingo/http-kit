use std::hash::Hash;

use super::error::ProjectionError;

/// A single selectable field in a projection. Typically implemented for
/// an enum whose variants correspond to the fields in the projection.
pub trait ProjectionField: Sized + Clone + Hash + Eq + 'static + AsRef<str> {
    /// Convert a string field name (e.g. `"firstName"`) into a typed field of this projection.
    fn parse(s: &str) -> Result<Self, ProjectionError>;

    /// The exhaustive list of valid field names for this projection.
    fn all_field_names() -> &'static [&'static str];

    /// All typed fields of this projection.
    fn all() -> &'static [Self];

    /// The resource name for this projection.
    fn resource_name() -> &'static str;
}
