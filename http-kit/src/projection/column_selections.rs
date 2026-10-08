//! Creates one or more [`ColumnSelection`](pgkit::projection::ColumnSelection)s
//! from a list of [`ProjectionField`]s.

use super::field::ProjectionField;

/// Converts a list of [`ProjectionField`]s into the [`ColumnSelection`](pgkit::projection::ColumnSelection)(s) they select.
///
/// # One table → a single `ColumnSelection`
///
/// Map each field to a column and collect:
///
/// ```ignore
/// impl IntoColumnSelections for CountryField {
///     type Output = ColumnSelection<CountriesColumn>;
///     fn into_column_selections(fields: Vec<Self>) -> Self::Output {
///         ColumnSelection::new(fields.into_iter().map(|f| match f {
///             CountryField::Id   => CountriesColumn::Id,
///             CountryField::Iso2 => CountriesColumn::Iso2,
///             // …
///         }))
///     }
/// }
/// ```
///
/// # Several tables → a tuple of selections
///
/// Each related selection is an `Option`, left `None` when none of its fields
/// were selected: an empty `ColumnSelection` still carries the primary key, so
/// `None` — not an empty selection — is what tells a caller to skip the join.
///
/// ```ignore
/// impl IntoColumnSelections for AreaField {
///     type Output = (ColumnSelection<AreasColumn>, Option<ColumnSelection<CountriesColumn>>);
///     fn into_column_selections(fields: Vec<Self>) -> Self::Output {
///         let mut area = Vec::new();
///         let mut country = Vec::new();   // CountryField values
///         for f in fields {
///             match f {
///                 AreaField::Id         => area.push(AreasColumn::Id),
///                 AreaField::Country(c) => country.push(c),
///                 // …
///             }
///         }
///         let country = (!country.is_empty())
///             .then(|| CountryField::into_column_selections(country));
///         (ColumnSelection::new(area), country)
///     }
/// }
/// ```
///
/// The `ProjectionField` derive generates this impl from a `column = …`
/// attribute, but it can be implemented by hand for any shape.
pub trait IntoColumnSelections: ProjectionField {
    type Output;

    fn into_column_selections(fields: Vec<Self>) -> Self::Output;
}
