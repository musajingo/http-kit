//! Integration tests for `#[derive(ProjectionField)]`.

use http_kit::projection::{ProjectionError, ProjectionField};
use serde::Serialize;

// ── Test fixtures ─────────────────────────────────────────────────────────────

/// A flat, leaf-only struct.
#[derive(ProjectionField)]
#[allow(dead_code)]
struct Country {
    pub id: i32,
    pub iso2: String,
    pub name: String,
    pub active: bool,
    pub created_at: String,
}

/// A flat struct with an explicit resource name override.
#[derive(ProjectionField)]
#[projection_field(name = "geo_area")]
#[allow(dead_code)]
struct GeoArea {
    pub id: i32,
    pub label: String,
}

/// A struct with a nested embedded resource.
#[derive(ProjectionField)]
#[allow(dead_code)]
struct Planet {
    pub id: i32,
    pub name: String,
    #[projection_field(CountryField)]
    pub country: Country,
}

/// A struct with a skipped field.
#[derive(ProjectionField)]
#[allow(dead_code)]
struct Config {
    pub key: String,
    pub value: String,
    #[projection_skip]
    pub secret: String,
}

/// A struct with a field whose response key is overridden via serde.
#[derive(ProjectionField, Serialize)]
#[allow(dead_code)]
struct Invoice {
    pub id: i32,
    #[serde(rename = "type")]
    pub invoice_type: String,
}

/// A struct with a raw-identifier field.
#[derive(ProjectionField)]
#[allow(dead_code)]
struct Ticket {
    pub id: i32,
    pub r#type: String,
}

// ── Flat field tests ──────────────────────────────────────────────────────────

#[test]
fn flat_resource_name_defaults_to_lowercase_struct_name() {
    assert_eq!(CountryField::resource_name(), "country");
}

#[test]
fn flat_resource_name_can_be_overridden() {
    assert_eq!(GeoAreaField::resource_name(), "geo_area");
}

#[test]
fn flat_all_field_names_returns_camel_case() {
    let names = CountryField::all_field_names();
    assert_eq!(names, &["id", "iso2", "name", "active", "createdAt"]);
}

#[test]
fn field_level_serde_rename_is_the_wire_name() {
    assert_eq!(InvoiceField::all_field_names(), &["id", "type"]);
    assert_eq!(
        InvoiceField::parse("type").unwrap(),
        InvoiceField::InvoiceType
    );
    assert_eq!(InvoiceField::InvoiceType.as_ref(), "type");
    assert!(InvoiceField::parse("invoiceType").is_err());
}

#[test]
fn raw_identifier_field_derives_without_raw_prefix() {
    assert_eq!(TicketField::all_field_names(), &["id", "type"]);
    assert_eq!(TicketField::parse("type").unwrap(), TicketField::Type);
    assert_eq!(TicketField::Type.as_ref(), "type");
}

#[test]
fn flat_all_returns_all_variants() {
    let all = CountryField::all();
    assert_eq!(all.len(), 5);
    assert!(all.contains(&CountryField::Id));
    assert!(all.contains(&CountryField::Iso2));
    assert!(all.contains(&CountryField::Name));
    assert!(all.contains(&CountryField::Active));
    assert!(all.contains(&CountryField::CreatedAt));
}

#[test]
fn flat_parse_valid_camel_case() {
    assert_eq!(CountryField::parse("id").unwrap(), CountryField::Id);
    assert_eq!(CountryField::parse("iso2").unwrap(), CountryField::Iso2);
    assert_eq!(
        CountryField::parse("createdAt").unwrap(),
        CountryField::CreatedAt
    );
}

#[test]
fn flat_parse_unknown_field_returns_invalid_field_error() {
    let err = CountryField::parse("unknown").unwrap_err();
    match err {
        ProjectionError::InvalidField {
            field,
            valid_fields,
        } => {
            assert!(field.contains("unknown"), "field: {field}");
            assert!(!valid_fields.is_empty());
        }
        other => panic!("expected InvalidField, got {other:?}"),
    }
}

#[test]
fn flat_as_ref_returns_camel_case_wire_name() {
    assert_eq!(CountryField::Id.as_ref(), "id");
    assert_eq!(CountryField::Iso2.as_ref(), "iso2");
    assert_eq!(CountryField::CreatedAt.as_ref(), "createdAt");
}

#[test]
fn flat_try_from_string_succeeds() {
    let f: CountryField = "name".to_string().try_into().unwrap();
    assert_eq!(f, CountryField::Name);
}

#[test]
fn flat_try_from_str_succeeds() {
    let f: CountryField = CountryField::try_from("active").unwrap();
    assert_eq!(f, CountryField::Active);
}

#[test]
fn flat_try_from_unknown_string_returns_error() {
    let result: Result<CountryField, _> = "nope".try_into();
    assert!(result.is_err());
}

#[test]
fn skipped_field_absent_from_all() {
    // `Config` has `secret` marked `#[projection_skip]`.
    let all = ConfigField::all();
    let names = ConfigField::all_field_names();
    assert_eq!(all.len(), 2, "only key and value");
    assert!(
        !names.contains(&"secret"),
        "skipped field should not appear"
    );
    assert!(names.contains(&"key") && names.contains(&"value"));
}

// ── Nested field tests ────────────────────────────────────────────────────────

#[test]
fn nested_resource_name() {
    assert_eq!(PlanetField::resource_name(), "planet");
}

#[test]
fn nested_all_field_names_includes_prefixed_leaf_names() {
    let names = PlanetField::all_field_names();
    // Top-level leaf names.
    assert!(names.contains(&"id"), "missing 'id'");
    assert!(names.contains(&"name"), "missing 'name'");
    // Nested names with 'country.' prefix.
    assert!(names.contains(&"country.id"), "missing 'country.id'");
    assert!(names.contains(&"country.iso2"), "missing 'country.iso2'");
    assert!(
        names.contains(&"country.createdAt"),
        "missing 'country.createdAt'"
    );
}

#[test]
fn nested_all_includes_nested_variants() {
    let all = PlanetField::all();
    // Should contain top-level variants.
    assert!(all.contains(&PlanetField::Id));
    assert!(all.contains(&PlanetField::Name));
    // Should contain nested variants for each CountryField.
    assert!(all.contains(&PlanetField::Country(CountryField::Id)));
    assert!(all.contains(&PlanetField::Country(CountryField::Iso2)));
    assert!(all.contains(&PlanetField::Country(CountryField::CreatedAt)));
}

#[test]
fn nested_parse_top_level_leaf() {
    let f = PlanetField::parse("id").unwrap();
    assert_eq!(f, PlanetField::Id);
}

#[test]
fn nested_parse_dotted_field() {
    let f = PlanetField::parse("country.iso2").unwrap();
    assert_eq!(f, PlanetField::Country(CountryField::Iso2));
}

#[test]
fn nested_parse_unknown_namespace_returns_error() {
    let err = PlanetField::parse("region.id").unwrap_err();
    assert!(matches!(err, ProjectionError::InvalidField { .. }));
}

#[test]
fn nested_parse_unknown_leaf_in_known_namespace_returns_error() {
    let err = PlanetField::parse("country.doesNotExist").unwrap_err();
    assert!(matches!(err, ProjectionError::InvalidField { .. }));
}

#[test]
fn nested_as_ref_leaf_returns_wire_name() {
    assert_eq!(PlanetField::Id.as_ref(), "id");
    assert_eq!(PlanetField::Name.as_ref(), "name");
}

#[test]
fn nested_as_ref_nested_returns_prefixed_wire_name() {
    assert_eq!(
        PlanetField::Country(CountryField::Id).as_ref(),
        "country.id"
    );
    assert_eq!(
        PlanetField::Country(CountryField::Iso2).as_ref(),
        "country.iso2"
    );
    assert_eq!(
        PlanetField::Country(CountryField::CreatedAt).as_ref(),
        "country.createdAt"
    );
}

#[test]
fn nested_try_from_dotted_string_succeeds() {
    let f: PlanetField = "country.name".to_string().try_into().unwrap();
    assert_eq!(f, PlanetField::Country(CountryField::Name));
}

/// `all_field_names` count for `Planet` should be 2 (top-level) + 5 (country.*).
#[test]
fn nested_all_field_names_count() {
    let count = PlanetField::all_field_names().len();
    let expected = 2 + CountryField::all_field_names().len();
    assert_eq!(count, expected);
}

/// `all()` count for `Planet` should be 2 (top-level) + 5 (country).
#[test]
fn nested_all_count() {
    let count = PlanetField::all().len();
    let expected = 2 + CountryField::all().len();
    assert_eq!(count, expected);
}

// ── `column = …` → `IntoColumnSelections` ───────────────────────────────────────

use http_kit::projection::IntoColumnSelections;

// Declares a minimal pgkit column enum (table + column + the trait impls
// `ColumnSelection` requires), so `column = …` fixtures have real columns.
macro_rules! pgkit_column {
    ($table:ident, $col:ident { $($variant:ident => $name:literal),+ $(,)? }, pk = $pk:ident) => {
        struct $table;

        #[derive(Debug, Clone, Copy, PartialEq, Eq, strum::VariantNames, strum::EnumIter)]
        enum $col { $($variant),+ }

        impl From<$col> for &'static str {
            fn from(c: $col) -> Self {
                match c { $($col::$variant => $name),+ }
            }
        }
        impl pgkit::types::DatabaseTableColumn for $col {
            type Table = $table;
            fn table_primary_key() -> Self { $col::$pk }
        }
        impl pgkit::types::DatabaseTable for $table {
            type Columns = $col;
            const NAME: &'static str = stringify!($table);
            fn primary_key() -> Self::Columns { $col::$pk }
        }
    };
}

pgkit_column!(WidgetTable, WidgetColumn {
    Id => "id", Label => "label", CreatedAt => "created_at", FancyName => "fancy_name"
}, pk = Id);

// --- leaf: name-matched mapping + overrides ---

#[derive(ProjectionField)]
#[projection_field(name = "widget", column = WidgetColumn)]
#[allow(dead_code)]
struct Widget {
    pub id: i32,
    pub label: String,
    pub created_at: String,
}

#[derive(ProjectionField)]
#[projection_field(name = "widget_pf", column = WidgetColumn)]
#[allow(dead_code)]
struct WidgetPf {
    pub id: i32,
    #[projection_field(column = WidgetColumn::FancyName)]
    pub name: String,
}

#[derive(ProjectionField)]
#[projection_field(name = "widget_shared", column = WidgetColumn)]
#[allow(dead_code)]
struct WidgetShared {
    pub id: i32,
    #[column(WidgetColumn::FancyName)]
    pub name: String,
}

#[test]
fn leaf_output_is_a_single_selection_with_name_matched_columns() {
    let sel = WidgetField::into_column_selections(vec![WidgetField::Label, WidgetField::CreatedAt]);
    assert!(sel.contains(&WidgetColumn::Label));
    assert!(sel.contains(&WidgetColumn::CreatedAt));
    assert!(sel.contains(&WidgetColumn::Id)); // PK auto-injected
}

#[test]
fn projection_field_column_overrides_name_match() {
    let sel = WidgetPfField::into_column_selections(vec![WidgetPfField::Name]);
    assert!(sel.contains(&WidgetColumn::FancyName));
}

#[test]
fn shared_column_attr_overrides_name_match() {
    let sel = WidgetSharedField::into_column_selections(vec![WidgetSharedField::Name]);
    assert!(sel.contains(&WidgetColumn::FancyName));
}

// --- embedding: own selection + per-embed `Option` ---

pgkit_column!(NestTable, NestColumn { Id => "id", A => "a" }, pk = Id);
pgkit_column!(OuterTable, OuterColumn { Id => "id" }, pk = Id);
pgkit_column!(PairTable, PairColumn { Id => "id" }, pk = Id);

#[derive(ProjectionField)]
#[projection_field(name = "nest", column = NestColumn)]
#[allow(dead_code)]
struct Nest {
    pub id: i32,
    pub a: i32,
}

#[derive(ProjectionField)]
#[projection_field(name = "outer", column = OuterColumn)]
#[allow(dead_code)]
struct Outer {
    pub id: i32,
    #[projection_field(NestField)]
    pub nest: Nest,
}

#[test]
fn embedding_partitions_own_and_embed() {
    let (own, nest) =
        OuterField::into_column_selections(vec![OuterField::Id, OuterField::Nest(NestField::A)]);
    assert!(own.contains(&OuterColumn::Id));
    let nest = nest.expect("nest projected");
    assert!(nest.contains(&NestColumn::A));
}

#[test]
fn embedding_embed_is_none_when_not_projected() {
    let (_own, nest) = OuterField::into_column_selections(vec![OuterField::Id]);
    assert!(nest.is_none());
}

// --- same-type multi-embed: two fields of the same child type ---

#[derive(ProjectionField)]
#[projection_field(name = "pair", column = PairColumn)]
#[allow(dead_code)]
struct Pair {
    pub id: i32,
    #[projection_field(NestField)]
    pub first: Nest,
    #[projection_field(NestField)]
    pub second: Nest,
}

#[test]
fn same_type_multi_embed_keeps_embeds_separate() {
    let (_own, first, second) = PairField::into_column_selections(vec![
        PairField::Id,
        PairField::First(NestField::A),
        PairField::Second(NestField::Id),
    ]);
    let first = first.expect("first projected");
    let second = second.expect("second projected");
    assert!(first.contains(&NestColumn::A));
    assert!(second.contains(&NestColumn::Id));
}

// --- deep nesting: an embed whose own Output is itself a tuple ---

pgkit_column!(DeepTable, DeepColumn { Id => "id" }, pk = Id);

#[derive(ProjectionField)]
#[projection_field(name = "deep", column = DeepColumn)]
#[allow(dead_code)]
struct Deep {
    pub id: i32,
    #[projection_field(OuterField)]
    pub outer: Outer,
}

#[test]
fn deep_nesting_composes_nested_tuples() {
    let (deep, outer) = DeepField::into_column_selections(vec![
        DeepField::Id,
        DeepField::Outer(OuterField::Id),
        DeepField::Outer(OuterField::Nest(NestField::A)),
    ]);
    assert!(deep.contains(&DeepColumn::Id));
    let (outer_own, nest) = outer.expect("outer projected");
    assert!(outer_own.contains(&OuterColumn::Id));
    assert!(nest.expect("nest projected").contains(&NestColumn::A));
}

// --- `#[column(…)]` shared with the PartialResponse derive ---

pgkit_column!(ComboTable, ComboColumn { Id => "id", FancyName => "fancy_name" }, pk = Id);

#[derive(http_kit_macros::PartialResponse, ProjectionField)]
#[projection_field(name = "combo", column = ComboColumn)]
#[allow(dead_code)]
struct Combo {
    pub id: i32,
    #[column(ComboColumn::FancyName)]
    pub name: String,
}

#[test]
fn shared_column_attr_coexists_with_partial_response_derive() {
    let sel = ComboField::into_column_selections(vec![ComboField::Id, ComboField::Name]);
    assert!(sel.contains(&ComboColumn::Id));
    assert!(sel.contains(&ComboColumn::FancyName));
    // The PartialResponse companion was still generated alongside.
    let _ = PartialCombo::default();
}

// ─── skip(utoipa) ──────────────────────────────────────────────────────────────

#[derive(http_kit::projection::ProjectionField)]
#[projection_field(name = "plain", skip(utoipa))]
#[allow(dead_code)]
struct PlainNoUtoipa {
    id: i32,
    display_name: String,
}

#[test]
fn skip_utoipa_struct_still_parses() {
    use http_kit::projection::ProjectionField;
    assert!(PlainNoUtoipaField::parse("displayName").is_ok());
    assert_eq!(
        PlainNoUtoipaField::all_field_names(),
        &["id", "displayName"]
    );
}
