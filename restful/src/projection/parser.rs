//! Parses the `?fields=` query parameter. See [`Projection`] for syntax and validation.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use valid8r::{Validate, ValidationError, ValidationErrors};

#[cfg(feature = "pgkit")]
use super::column_selections::IntoColumnSelections;
use super::error::{ProjectionError, ProjectionErrors};
use super::field::ProjectionField;
use super::utilities::{self, ParsingState};

/// Caps how many unknown-field tokens are reported (attaching the valid-fields
/// list only to the first) so a hostile `?fields=` value cannot amplify into an
/// unbounded 422 body.
const MAX_REPORTED_INVALID_FIELDS: usize = 10;

fn push_invalid_field(
    errors: &mut Vec<ProjectionError>,
    seen: &mut usize,
    mut err: ProjectionError,
) {
    *seen += 1;
    if *seen > MAX_REPORTED_INVALID_FIELDS {
        return;
    }
    if *seen > 1
        && let ProjectionError::InvalidField { valid_fields, .. } = &mut err
    {
        valid_fields.clear();
    }
    errors.push(err);
}

/// Parses requested response fields. The application builds the selected response.
///
/// # Selection syntax
///
/// Input is a comma-separated list; whitespace around tokens is trimmed.
/// Field names follow the implementation of [`ProjectionField`].
///
/// | Token | Meaning |
/// | --- | --- |
/// | `id,name` | Include these top-level fields |
/// | `all` | Include every top-level field |
/// | `-name` or `all,-name` | Include every top-level field except `name` |
/// | `id,child.name` | Include `id` and the embedded field `child.name` |
/// | `id,child.all` | Include `id` and every field in `child` |
/// | `id,child.grand.all` | Include `id` and every field in `child.grand` |
///
/// Each namespace is selected separately. `all` selects only that namespace's
/// fields, not descendants. Exclusions imply `all` in their namespace.
///
/// # Validation
///
/// - Input must be nonempty and field names must be known. Empty tokens,
///   a lone `-`, and negating `all` (including `-child.all`) are invalid.
/// - The top-level namespace must be referenced; embedded fields alone are invalid.
/// - Within a namespace, inclusions cannot mix with exclusions or `all`.
///   `all` can mix with exclusions.
/// - Every referenced namespace must resolve to at least one field.
///
/// `From<String>` and deserialization store parsing errors rather than rejecting
/// the input. Check [`Self::is_valid`] or call [`Validate::validate`] before using
/// the fields.
#[derive(Debug, Deserialize, Serialize)]
#[serde(from = "String", bound = "T: ProjectionField + Serialize")]
pub struct Projection<T: ProjectionField> {
    /// The resolved fields to include in the response.
    fields: Vec<T>,

    /// Errors encountered during parsing.
    errors: Vec<ProjectionError>,
}

impl<T: ProjectionField> Projection<T> {
    fn parse(s: String) -> Result<Self, ProjectionErrors> {
        let raw = s.trim();
        if raw.is_empty() {
            return Err(ProjectionErrors(vec![ProjectionError::Empty]));
        }

        let mut errors: Vec<ProjectionError> = Vec::new();
        let mut invalid_seen = 0usize;
        let mut state: HashMap<String, ParsingState<T>> = HashMap::new();

        // ––– PARSING –––

        for raw_token in raw.split(',').map(str::trim) {
            let (negated, name) = match raw_token.strip_prefix('-') {
                Some(rest) => (true, rest),
                None => (false, raw_token),
            };

            if name.is_empty() {
                push_invalid_field(
                    &mut errors,
                    &mut invalid_seen,
                    ProjectionError::InvalidField {
                        field: utilities::qualify(T::resource_name(), raw_token),
                        valid_fields: T::all_field_names().to_vec(),
                    },
                );
                continue;
            }

            let (ns, leaf) = utilities::split_ns(name);

            // `all` / `<ns>.all` is a meta-token, not a real field.
            if leaf == "all" {
                if negated {
                    push_invalid_field(
                        &mut errors,
                        &mut invalid_seen,
                        ProjectionError::InvalidField {
                            field: utilities::qualify(T::resource_name(), name),
                            valid_fields: T::all_field_names().to_vec(),
                        },
                    );
                    continue;
                }
                state.entry(ns.to_string()).or_default().has_all = true;
                continue;
            }

            match T::parse(name) {
                Ok(field) => {
                    let entry = state.entry(ns.to_string()).or_default();
                    if negated {
                        entry.exclude.insert(name.to_string());
                    } else {
                        entry.include.insert(field);
                    }
                }
                Err(e) => push_invalid_field(&mut errors, &mut invalid_seen, e),
            }
        }

        // Terminate early if there were any parsing errors
        if !errors.is_empty() {
            return Err(ProjectionErrors(errors));
        }

        // ––– VALIDATION –––

        // The top-level resource/namespace must be referenced (positives, negatives, or `all`).
        if !state.contains_key("") {
            errors.push(ProjectionError::TopLevelMissing);
        }

        // `all` cannot be combined with explicit inclusions (but can be combined with exclusions).
        // Inclusion and exclusion cannot be used at the same time.
        for (ns, state) in &state {
            let has_inc = !state.include.is_empty();
            let has_exc = !state.exclude.is_empty();
            if has_inc && state.has_all {
                errors.push(ProjectionError::AllWithInclusions {
                    namespace: ns.clone(),
                });
            } else if has_inc && has_exc {
                errors.push(ProjectionError::AmbiguousSelection {
                    namespace: ns.clone(),
                });
            }
        }

        // Terminate early if there were any validation errors
        if !errors.is_empty() {
            return Err(ProjectionErrors(errors));
        }

        // ––– RESOLUTION –––

        // Resolve the final set of fields to be included in the projection.
        let mut result: Vec<T> = Vec::new();

        for f in T::all() {
            let name = f.as_ref();
            let (ns, _) = utilities::split_ns(name);
            let Some(state) = state.get(ns) else { continue };

            let include = if state.has_all || state.include.is_empty() {
                // `all` mode (explicit) or implicit-all with exclusions only.
                !state.exclude.contains(name)
            } else {
                // Includes-only mode: only fields explicitly listed.
                state.include.contains(f)
            };

            if include {
                result.push(f.clone());
            }
        }

        // Any referenced namespace that resolved to zero fields is an error.
        for ns in state.keys() {
            let count = result
                .iter()
                .filter(|f| utilities::split_ns(f.as_ref()).0 == ns)
                .count();
            if count == 0 {
                let total = T::all_field_names()
                    .iter()
                    .filter(|f| utilities::split_ns(f).0 == ns)
                    .count();
                errors.push(ProjectionError::AllExcluded {
                    namespace: ns.clone(),
                    count: total,
                });
            }
        }

        if !errors.is_empty() {
            return Err(ProjectionErrors(errors));
        }

        Ok(Self {
            fields: result,
            errors: Vec::new(),
        })
    }

    /// Get the resolved fields to include in the response.
    pub fn fields(&self) -> &[T] {
        &self.fields
    }

    /// Consume the projection and return its resolved fields.
    pub fn into_fields(self) -> Vec<T> {
        self.fields
    }

    /// Create a projection directly from already-resolved fields.
    pub fn from_fields(fields: Vec<T>) -> Self {
        Self {
            fields,
            errors: Vec::new(),
        }
    }

    /// Get any errors encountered during parsing.
    pub fn errors(&self) -> &[ProjectionError] {
        &self.errors
    }

    /// Whether the projection is valid (no parsing errors).
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Consume the projection, iterating its resolved fields by value.
impl<T: ProjectionField> IntoIterator for Projection<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}

/// Borrow the projection, iterating its resolved fields by reference.
impl<'a, T: ProjectionField> IntoIterator for &'a Projection<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.iter()
    }
}

#[cfg(feature = "pgkit")]
impl<T: IntoColumnSelections> Projection<T> {
    /// Partition the resolved fields into column selection(s).
    /// See [`IntoColumnSelections`]
    pub fn into_column_selections(self) -> <T as IntoColumnSelections>::Output {
        <T as IntoColumnSelections>::into_column_selections(self.into_fields())
    }
}

impl<T: ProjectionField> From<String> for Projection<T> {
    fn from(s: String) -> Self {
        match Projection::parse(s) {
            Ok(projection) => projection,
            Err(ProjectionErrors(errors)) => Self {
                fields: Vec::new(),
                errors,
            },
        }
    }
}

#[cfg(feature = "utoipa")]
impl<T: ProjectionField> utoipa::__dev::ComposeSchema for Projection<T> {
    fn compose(
        _: Vec<utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>>,
    ) -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::Type::String)
            .build()
            .into()
    }
}

#[cfg(feature = "utoipa")]
impl<T: ProjectionField> utoipa::ToSchema for Projection<T> {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Projection")
    }
}

impl<T: ProjectionField> Validate for Projection<T> {
    fn validate(&self) -> Result<(), ValidationErrors> {
        if self.is_valid() {
            return Ok(());
        }
        let mut validation_errors = ValidationErrors::new();
        for err in &self.errors {
            let code = match err {
                ProjectionError::InvalidField { .. } => "ProjectionInvalidField",
                ProjectionError::AllExcluded { .. } => "ProjectionAllExcluded",
                ProjectionError::AmbiguousSelection { .. } => "ProjectionAmbiguousSelection",
                ProjectionError::AllWithInclusions { .. } => "ProjectionAllWithInclusions",
                ProjectionError::TopLevelMissing => "ProjectionTopLevelMissing",
                ProjectionError::Empty => "ProjectionEmpty",
            };
            let mut validation_error = ValidationError::new(code);
            validation_error.message = Some(err.to_string().into());
            if let ProjectionError::InvalidField { valid_fields, .. } = err
                && !valid_fields.is_empty()
            {
                validation_error.add_param("valid_fields".into(), valid_fields);
            }
            validation_errors.add("projection", validation_error);
        }
        Err(validation_errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Test fixture ──────────────────────────────────────────────────────

    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    enum SubField {
        Id,
        Label,
    }

    impl ProjectionField for SubField {
        fn parse(s: &str) -> Result<Self, ProjectionError> {
            match s {
                "id" => Ok(SubField::Id),
                "label" => Ok(SubField::Label),
                _ => Err(ProjectionError::InvalidField {
                    field: format!("{}.{s}", Self::resource_name()),
                    valid_fields: Self::all_field_names().to_vec(),
                }),
            }
        }
        fn all_field_names() -> &'static [&'static str] {
            &["id", "label"]
        }
        fn all() -> &'static [Self] {
            &[SubField::Id, SubField::Label]
        }
        fn resource_name() -> &'static str {
            "sub"
        }
    }

    impl AsRef<str> for SubField {
        fn as_ref(&self) -> &str {
            match self {
                SubField::Id => "id",
                SubField::Label => "label",
            }
        }
    }

    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    enum OtherField {
        Id,
        Active,
    }

    impl ProjectionField for OtherField {
        fn parse(s: &str) -> Result<Self, ProjectionError> {
            match s {
                "id" => Ok(OtherField::Id),
                "active" => Ok(OtherField::Active),
                _ => Err(ProjectionError::InvalidField {
                    field: format!("{}.{s}", Self::resource_name()),
                    valid_fields: Self::all_field_names().to_vec(),
                }),
            }
        }
        fn all_field_names() -> &'static [&'static str] {
            &["id", "active"]
        }
        fn all() -> &'static [Self] {
            &[OtherField::Id, OtherField::Active]
        }
        fn resource_name() -> &'static str {
            "other"
        }
    }

    impl AsRef<str> for OtherField {
        fn as_ref(&self) -> &str {
            match self {
                OtherField::Id => "id",
                OtherField::Active => "active",
            }
        }
    }

    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    enum TestField {
        Id,
        Name,
        CreatedAt,
        Sub(SubField),
        Other(OtherField),
    }

    impl ProjectionField for TestField {
        fn parse(s: &str) -> Result<Self, ProjectionError> {
            if let Some((ns, leaf)) = s.split_once('.') {
                return match ns {
                    "sub" => SubField::parse(leaf).map(TestField::Sub).map_err(|_| {
                        ProjectionError::InvalidField {
                            field: s.to_string(),
                            valid_fields: SubField::all_field_names().to_vec(),
                        }
                    }),
                    "other" => OtherField::parse(leaf).map(TestField::Other).map_err(|_| {
                        ProjectionError::InvalidField {
                            field: s.to_string(),
                            valid_fields: OtherField::all_field_names().to_vec(),
                        }
                    }),
                    _ => Err(ProjectionError::InvalidField {
                        field: s.to_string(),
                        valid_fields: Self::all_field_names().to_vec(),
                    }),
                };
            }
            match s {
                "id" => Ok(TestField::Id),
                "name" => Ok(TestField::Name),
                "createdAt" => Ok(TestField::CreatedAt),
                _ => Err(ProjectionError::InvalidField {
                    field: format!("{}.{s}", Self::resource_name()),
                    valid_fields: Self::all_field_names().to_vec(),
                }),
            }
        }

        fn all_field_names() -> &'static [&'static str] {
            &[
                "id",
                "name",
                "createdAt",
                "sub.id",
                "sub.label",
                "other.id",
                "other.active",
            ]
        }

        fn all() -> &'static [Self] {
            &[
                TestField::Id,
                TestField::Name,
                TestField::CreatedAt,
                TestField::Sub(SubField::Id),
                TestField::Sub(SubField::Label),
                TestField::Other(OtherField::Id),
                TestField::Other(OtherField::Active),
            ]
        }

        fn resource_name() -> &'static str {
            "test"
        }
    }

    impl AsRef<str> for TestField {
        fn as_ref(&self) -> &str {
            match self {
                TestField::Id => "id",
                TestField::Name => "name",
                TestField::CreatedAt => "createdAt",
                TestField::Sub(SubField::Id) => "sub.id",
                TestField::Sub(SubField::Label) => "sub.label",
                TestField::Other(OtherField::Id) => "other.id",
                TestField::Other(OtherField::Active) => "other.active",
            }
        }
    }

    fn projection(input: &str) -> Projection<TestField> {
        Projection::from(input.to_string())
    }

    fn names(p: &Projection<TestField>) -> Vec<&str> {
        p.fields.iter().map(|f| f.as_ref()).collect()
    }

    // ─── Empty / whitespace ────────────────────────────────────────────────

    #[test]
    fn empty_input_yields_empty_error() {
        let p = projection("");
        assert!(matches!(p.errors[..], [ProjectionError::Empty]));
        assert!(p.fields.is_empty());
    }

    #[test]
    fn whitespace_only_yields_empty_error() {
        let p = projection("   \t  ");
        assert!(matches!(p.errors[..], [ProjectionError::Empty]));
    }

    #[test]
    fn whitespace_around_tokens_is_trimmed() {
        let p = projection("  id , name  ");
        assert!(p.errors.is_empty());
        assert_eq!(names(&p), vec!["id", "name"]);
    }

    // ─── Inclusion-only at top-level ───────────────────────────────────────

    #[test]
    fn single_top_level_field() {
        let p = projection("id");
        assert!(p.errors.is_empty());
        assert_eq!(names(&p), vec!["id"]);
    }

    #[test]
    fn multiple_top_level_fields_emitted_in_declaration_order() {
        let p = projection("name,id");
        assert_eq!(names(&p), vec!["id", "name"]);
    }

    #[test]
    fn duplicate_inclusions_collapse() {
        let p = projection("id,id,name");
        assert_eq!(names(&p), vec!["id", "name"]);
    }

    // ─── Explicit `all` (top-level) ────────────────────────────────────────

    #[test]
    fn explicit_all_alone_includes_all_top_level() {
        let p = projection("all");
        assert_eq!(names(&p), vec!["id", "name", "createdAt"]);
    }

    #[test]
    fn explicit_all_minus_field() {
        let p = projection("all,-createdAt");
        assert_eq!(names(&p), vec!["id", "name"]);
    }

    #[test]
    fn explicit_all_with_inclusion_is_all_with_inclusions() {
        let p = projection("all,id");
        assert!(matches!(
            p.errors[..],
            [ProjectionError::AllWithInclusions { ref namespace }] if namespace.is_empty()
        ));
        assert!(p.fields.is_empty());
    }

    #[test]
    fn explicit_all_with_inclusion_and_exclusion_still_all_with_inclusions() {
        let p = projection("all,id,-createdAt");
        assert!(matches!(
            p.errors[..],
            [ProjectionError::AllWithInclusions { ref namespace }] if namespace.is_empty()
        ));
    }

    // ─── Implicit all (negatives only) ─────────────────────────────────────

    #[test]
    fn implicit_all_top_level_minus_one() {
        let p = projection("-createdAt");
        assert_eq!(names(&p), vec!["id", "name"]);
    }

    #[test]
    fn implicit_all_top_level_minus_many() {
        let p = projection("-name,-createdAt");
        assert_eq!(names(&p), vec!["id"]);
    }

    #[test]
    fn negating_all_top_level_fields_is_all_excluded() {
        let p = projection("-id,-name,-createdAt");
        assert!(matches!(
            p.errors[..],
            [ProjectionError::AllExcluded { ref namespace, count: 3 }] if namespace.is_empty()
        ));
    }

    // ─── Nested: explicit positives ────────────────────────────────────────

    #[test]
    fn top_level_with_one_nested_inclusion() {
        let p = projection("id,sub.id");
        assert_eq!(names(&p), vec!["id", "sub.id"]);
    }

    #[test]
    fn top_level_with_multiple_nested_inclusions_across_namespaces() {
        let p = projection("id,sub.id,sub.label,other.active");
        assert_eq!(names(&p), vec!["id", "sub.id", "sub.label", "other.active"]);
    }

    // ─── Nested: explicit `all` ────────────────────────────────────────────

    #[test]
    fn nested_all_includes_all_namespace_fields() {
        let p = projection("id,sub.all");
        assert_eq!(names(&p), vec!["id", "sub.id", "sub.label"]);
    }

    #[test]
    fn nested_all_minus_one() {
        let p = projection("id,sub.all,-sub.label");
        assert_eq!(names(&p), vec!["id", "sub.id"]);
    }

    #[test]
    fn nested_all_with_inclusion_is_all_with_inclusions() {
        let p = projection("id,sub.all,sub.id");
        assert!(matches!(
            p.errors[..],
            [ProjectionError::AllWithInclusions { ref namespace }] if namespace == "sub"
        ));
    }

    // ─── Nested: implicit all ──────────────────────────────────────────────

    #[test]
    fn nested_implicit_all_minus_one() {
        let p = projection("id,-sub.id");
        assert_eq!(names(&p), vec!["id", "sub.label"]);
    }

    #[test]
    fn nested_negating_every_field_is_all_excluded() {
        let p = projection("id,-sub.id,-sub.label");
        assert!(matches!(
            p.errors[..],
            [ProjectionError::AllExcluded { ref namespace, count: 2 }] if namespace == "sub"
        ));
    }

    // ─── Combinations ──────────────────────────────────────────────────────

    #[test]
    fn explicit_all_everywhere() {
        let p = projection("all,sub.all,other.all");
        assert_eq!(
            names(&p),
            vec![
                "id",
                "name",
                "createdAt",
                "sub.id",
                "sub.label",
                "other.id",
                "other.active"
            ]
        );
    }

    #[test]
    fn complex_per_namespace_combination() {
        // top-level: all - createdAt; sub: just sub.id; other: implicit-all - other.active.
        let p = projection("all,-createdAt,sub.id,-other.active");
        assert_eq!(names(&p), vec!["id", "name", "sub.id", "other.id"]);
    }

    #[test]
    fn implicit_top_level_with_explicit_nested_all() {
        let p = projection("-name,sub.all");
        assert_eq!(names(&p), vec!["id", "createdAt", "sub.id", "sub.label"]);
    }

    // ─── TopLevelMissing ───────────────────────────────────────────────────

    #[test]
    fn nested_inclusion_only_misses_top_level() {
        let p = projection("sub.id");
        assert!(matches!(p.errors[..], [ProjectionError::TopLevelMissing]));
    }

    #[test]
    fn nested_negation_only_misses_top_level() {
        let p = projection("-sub.id");
        assert!(matches!(p.errors[..], [ProjectionError::TopLevelMissing]));
    }

    #[test]
    fn nested_all_alone_misses_top_level() {
        let p = projection("sub.all");
        assert!(matches!(p.errors[..], [ProjectionError::TopLevelMissing]));
    }

    // ─── AmbiguousSelection ────────────────────────────────────────────────

    #[test]
    fn top_level_inclusion_and_exclusion_is_ambiguous() {
        let p = projection("id,-createdAt");
        assert!(matches!(
            p.errors[..],
            [ProjectionError::AmbiguousSelection { ref namespace }] if namespace.is_empty()
        ));
    }

    #[test]
    fn nested_inclusion_and_exclusion_is_ambiguous() {
        let p = projection("id,sub.id,-sub.label");
        assert!(matches!(
            p.errors[..],
            [ProjectionError::AmbiguousSelection { ref namespace }] if namespace == "sub"
        ));
    }

    #[test]
    fn ambiguity_is_per_namespace() {
        // top-level fine; sub is mixed → only sub flagged.
        let p = projection("id,sub.id,-sub.label");
        let codes: Vec<_> = p
            .errors
            .iter()
            .map(|e| match e {
                ProjectionError::AmbiguousSelection { namespace } => namespace.as_str(),
                _ => "other",
            })
            .collect();
        assert_eq!(codes, vec!["sub"]);
    }

    // ─── InvalidField ──────────────────────────────────────────────────────

    #[test]
    fn unknown_top_level_field_is_qualified_with_resource() {
        let p = projection("id,unknown");
        assert!(matches!(
            p.errors[..],
            [ProjectionError::InvalidField { ref field, .. }] if field == "test.unknown"
        ));
    }

    #[test]
    fn unknown_nested_leaf_field_keeps_namespace() {
        let p = projection("id,sub.foo");
        assert!(matches!(
            p.errors[..],
            [ProjectionError::InvalidField { ref field, .. }] if field == "sub.foo"
        ));
    }

    #[test]
    fn unknown_namespace_is_invalid_field() {
        let p = projection("id,unknown.field");
        assert!(matches!(
            p.errors[..],
            [ProjectionError::InvalidField { ref field, .. }] if field == "unknown.field"
        ));
    }

    #[test]
    fn negated_top_level_meta_is_invalid_field() {
        let p = projection("id,-all");
        assert!(matches!(
            p.errors[..],
            [ProjectionError::InvalidField { ref field, .. }] if field == "test.all"
        ));
    }

    #[test]
    fn negated_nested_meta_is_invalid_field() {
        let p = projection("id,-sub.all");
        assert!(matches!(
            p.errors[..],
            [ProjectionError::InvalidField { ref field, .. }] if field == "sub.all"
        ));
    }

    #[test]
    fn empty_token_in_list_is_invalid_field() {
        let p = projection("id,,name");
        assert!(matches!(
            p.errors[..],
            [ProjectionError::InvalidField { .. }]
        ));
    }

    #[test]
    fn dash_only_token_is_invalid_field() {
        let p = projection("id,-");
        assert!(matches!(
            p.errors[..],
            [ProjectionError::InvalidField { .. }]
        ));
    }

    #[test]
    fn multiple_invalid_fields_all_collected() {
        let p = projection("foo,bar,baz");
        assert_eq!(p.errors.len(), 3);
        assert!(
            p.errors
                .iter()
                .all(|e| matches!(e, ProjectionError::InvalidField { .. }))
        );
    }

    #[test]
    fn invalid_field_reporting_is_capped() {
        let bad: String = (0..100)
            .map(|i| format!("bogus{i}"))
            .collect::<Vec<_>>()
            .join(",");
        let p = projection(&bad);
        assert_eq!(p.errors.len(), MAX_REPORTED_INVALID_FIELDS);
        let with_list = p
            .errors
            .iter()
            .filter(|e| {
                matches!(e, ProjectionError::InvalidField { valid_fields, .. } if !valid_fields.is_empty())
            })
            .count();
        assert_eq!(with_list, 1);
    }

    #[test]
    fn invalid_field_carries_valid_fields_list() {
        let p = projection("id,unknown");
        match &p.errors[..] {
            [ProjectionError::InvalidField { valid_fields, .. }] => {
                assert!(valid_fields.contains(&"id"));
                assert!(valid_fields.contains(&"sub.id"));
                assert!(valid_fields.contains(&"other.active"));
            }
            other => panic!("unexpected errors: {other:?}"),
        }
    }

    // ─── Error accumulation ────────────────────────────────────────────────

    #[test]
    fn distinct_validation_errors_all_collected() {
        // top-level missing AND sub has all+inclusion.
        let p = projection("sub.all,sub.id");
        assert_eq!(p.errors.len(), 2);
        assert!(
            p.errors
                .iter()
                .any(|e| matches!(e, ProjectionError::TopLevelMissing))
        );
        assert!(p.errors.iter().any(|e| matches!(
            e,
            ProjectionError::AllWithInclusions { namespace } if namespace == "sub"
        )));
    }

    #[test]
    fn invalid_field_short_circuits_before_resolution() {
        // Even with mixed inc/exc for sub (would normally be Ambiguous),
        // the unknown field error is the only thing reported because parsing
        // failures bail out before per-namespace validation.
        let p = projection("id,sub.id,-sub.label,unknown");
        assert!(
            p.errors
                .iter()
                .any(|e| matches!(e, ProjectionError::InvalidField { .. }))
        );
        assert!(
            p.errors
                .iter()
                .all(|e| matches!(e, ProjectionError::InvalidField { .. }))
        );
    }

    // ─── Output ordering ───────────────────────────────────────────────────

    #[test]
    fn output_follows_declaration_order_regardless_of_input_order() {
        let p = projection("other.active,sub.label,id,sub.id,other.id,name");
        assert_eq!(
            names(&p),
            vec![
                "id",
                "name",
                "sub.id",
                "sub.label",
                "other.id",
                "other.active"
            ]
        );
    }

    // ─── Validate trait surfaces errors ────────────────────────────────────

    #[test]
    fn validate_returns_ok_for_valid_projection() {
        let p = projection("id,name");
        assert!(p.validate().is_ok());
    }

    #[test]
    fn validate_returns_errors_for_invalid_projection() {
        let p = projection("");
        assert!(p.validate().is_err());
    }

    #[test]
    fn validate_carries_error_codes() {
        let p = projection("id,unknown");
        let err = p.validate().unwrap_err();
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("ProjectionInvalidField"));
    }
}
