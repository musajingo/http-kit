//! Integration tests for `#[derive(PartialResponse)]`.

use http_kit_macros::PartialResponse;
use serde::Serialize;
use serde_json::json;

// ── Test fixtures ─────────────────────────────────────────────────────────────

/// Flat struct — all leaf fields.
#[derive(Serialize, PartialResponse)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct Product {
    pub id: u32,
    pub name: String,
    pub price: f64,
    pub description: Option<String>,
    pub active: bool,
}

/// Struct with a nested embedded resource.
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Tag {
    pub id: u32,
    pub label: String,
}

/// Manually defined partial for `Tag` (required so the embed can reference it).
#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PartialTag {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Serialize, PartialResponse)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct Article {
    pub id: u32,
    pub title: String,
    #[partial_include(PartialTag)]
    pub tag: Tag,
    #[partial_include(PartialTag)]
    pub featured_tag: Option<Tag>,
    pub view_count: u32,
}

/// A struct whose generated partial uses a custom name via `#[partial_response(name = "...")]`.
#[derive(Serialize, PartialResponse)]
#[partial_response(name = "SlimOrder")]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct Order {
    pub id: u64,
    pub total: f64,
    pub status: String,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// The generated `PartialProduct` should exist and be constructable with
/// all fields set to `None` via `Default`.
#[test]
fn default_gives_all_none() {
    let p = PartialProduct::default();
    assert!(p.id.is_none());
    assert!(p.name.is_none());
    assert!(p.price.is_none());
    assert!(p.description.is_none());
    assert!(p.active.is_none());
}

/// A `Product` field with type `Option<String>` becomes `Option<Option<String>>`
/// in the partial response.
#[test]
fn option_field_becomes_double_option() {
    #![allow(clippy::field_reassign_with_default)]
    let mut p = PartialProduct::default();
    // `description` was `Option<String>`; partial wraps it in another Option.
    p.description = Some(None); // Some(None) means "field present, value null"
    p.description = Some(Some("hello".to_string())); // field present with value
    let _ = p; // must compile
}

/// Setting individual fields and serialising should only emit set fields.
#[test]
fn serialise_partial_omits_none_fields() {
    #![allow(clippy::field_reassign_with_default)]
    let mut p = PartialProduct::default();
    p.id = Some(42);
    p.name = Some("Widget".to_string());

    let json = serde_json::to_value(&p).unwrap();
    assert_eq!(json["id"], json!(42));
    assert_eq!(json["name"], json!("Widget"));
    // Unset fields must not appear.
    assert!(json.get("price").is_none());
    assert!(json.get("description").is_none());
    assert!(json.get("active").is_none());
}

/// `serde(rename_all = "camelCase")` is inherited from the original struct.
#[test]
fn camel_case_keys_in_output() {
    #![allow(clippy::field_reassign_with_default)]
    let mut p = PartialProduct::default();
    p.active = Some(true);
    p.price = Some(9.99);

    let json = serde_json::to_value(&p).unwrap();
    // snake_case fields should appear as camelCase.
    assert!(json.get("active").is_some()); // "active" stays "active"
    assert!(json.get("price").is_some());
    // No snake_case artefacts.
    assert!(json.get("view_count").is_none());
}

/// `#[partial_include]` should generate `Option<PartialTag>` for the field.
#[test]
fn partial_include_wraps_in_partial_type() {
    #![allow(clippy::field_reassign_with_default)]
    let mut a = PartialArticle::default();
    // `tag` field should be `Option<PartialTag>`.
    a.tag = Some(PartialTag {
        id: Some(1),
        label: None,
    });
    let json = serde_json::to_value(&a).unwrap();
    assert_eq!(json["tag"]["id"], json!(1));
    assert!(json["tag"].get("label").is_none());
}

/// A fully-populated partial should serialise all its fields.
#[test]
fn fully_populated_partial_serialises_all_fields() {
    let p = PartialProduct {
        id: Some(1),
        name: Some("Gadget".to_string()),
        price: Some(19.95),
        description: Some(Some("A fine gadget".to_string())),
        active: Some(true),
    };
    let json = serde_json::to_value(&p).unwrap();
    assert_eq!(json.as_object().unwrap().len(), 5);
}

/// A partial with `description = Some(None)` (null) should serialise the key
/// with a JSON `null` value.
#[test]
fn double_option_some_none_serialises_as_null() {
    let p = PartialProduct {
        description: Some(None),
        ..Default::default()
    };
    let json = serde_json::to_value(&p).unwrap();
    assert!(json.get("description").is_some());
    assert!(json["description"].is_null());
}

/// `camelCase` rename applies to multi-word field names from the partial struct.
#[test]
fn multi_word_field_renamed_to_camel_case() {
    #![allow(clippy::field_reassign_with_default)]
    // PartialArticle has `view_count: Option<u32>`.
    let mut a = PartialArticle::default();
    a.view_count = Some(100);
    let json = serde_json::to_value(&a).unwrap();
    assert!(json.get("viewCount").is_some(), "expected 'viewCount' key");
    assert!(
        json.get("view_count").is_none(),
        "unexpected 'view_count' key"
    );
}

// ── field-level #[serde(rename = "...")] ─────────────────────────────────────

#[derive(Serialize, PartialResponse)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct Renamed {
    pub id: u32,
    #[serde(rename = "type")]
    pub kind: String,
}

/// A field-level `#[serde(rename = "...")]` must carry onto the partial so the
/// projected response shape matches the full response (and the OpenAPI schema),
/// rather than falling back to the struct-level `rename_all`.
#[test]
fn field_level_rename_carries_to_partial() {
    let p = PartialRenamed {
        id: Some(1),
        kind: Some("shipping".to_string()),
    };
    let json = serde_json::to_value(&p).unwrap();
    assert_eq!(json["type"], json!("shipping"));
    assert!(json.get("kind").is_none(), "unexpected 'kind' key");
}

// ── #[partial_response(name = "...")] ────────────────────────────────────────

/// `#[partial_response(name = "SlimOrder")]` makes the generated struct use that
/// name instead of the default `PartialOrder`.
#[test]
fn custom_name_generates_named_struct() {
    // `SlimOrder` must exist (not `OrderPartialResponse`).
    let s = SlimOrder::default();
    assert!(s.id.is_none());
    assert!(s.total.is_none());
    assert!(s.status.is_none());
}

#[test]
fn custom_named_struct_serialises_set_fields() {
    let s = SlimOrder {
        id: Some(7),
        total: Some(49.99),
        status: None,
    };
    let json = serde_json::to_value(&s).unwrap();
    assert_eq!(json["id"], json!(7));
    assert_eq!(json["total"], json!(49.99));
    assert!(json.get("status").is_none());
}

// ── Option<T> field with #[partial_include] ───────────────────────────────────

/// An `Option<Tag>` field annotated with `#[partial_include(PartialTag)]`
/// must become `Option<Option<PartialTag>>` in the partial struct.
#[test]
fn optional_partial_include_field_is_double_option() {
    #![allow(clippy::field_reassign_with_default)]
    let mut a = PartialArticle::default();
    // `featured_tag` was `Option<Tag>`; partial wraps it in another Option.
    a.featured_tag = Some(None); // field present, value null
    let json = serde_json::to_value(&a).unwrap();
    assert!(json.get("featuredTag").is_some());
    assert!(json["featuredTag"].is_null());
}

#[test]
fn optional_partial_include_field_with_value_serialises_inner_partial() {
    #![allow(clippy::field_reassign_with_default)]
    let mut a = PartialArticle::default();
    a.featured_tag = Some(Some(PartialTag {
        id: Some(99),
        label: None,
    }));
    let json = serde_json::to_value(&a).unwrap();
    assert_eq!(json["featuredTag"]["id"], json!(99));
    assert!(json["featuredTag"].get("label").is_none());
}

#[test]
fn optional_partial_include_field_absent_when_none() {
    let a = PartialArticle::default();
    let json = serde_json::to_value(&a).unwrap();
    assert!(
        json.get("featuredTag").is_none(),
        "unset field should be omitted"
    );
}

// ── `row = …` → From<PartialRow> for the partial ────────────────────────────────

// Stand-in for a pgkit partial row: every column `Option`-wrapped (nullable
// columns doubly so).
#[allow(dead_code)]
struct WidgetRow {
    pub id: Option<u32>,
    pub label: Option<Option<String>>,
    pub full_name: Option<String>,
}

#[derive(Serialize, PartialResponse)]
#[partial_response(row = WidgetRow)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct Widget {
    pub id: u32,
    pub label: Option<String>,
    #[partial_response(row_field = full_name)]
    pub name: String,
}

#[test]
fn row_generates_from_with_leaf_move_and_override() {
    let row = WidgetRow {
        id: Some(7),
        label: Some(Some("hi".to_string())),
        full_name: Some("Widget Seven".to_string()),
    };
    let p = PartialWidget::from(row);
    assert_eq!(p.id, Some(7));
    assert_eq!(p.label, Some(Some("hi".to_string())));
    assert_eq!(p.name, Some("Widget Seven".to_string()));
}

// Embedded conversion: a row whose embed field's inner type is `Into` the
// embedded partial response.
struct GadgetTagRow {
    pub id: Option<u32>,
}

impl From<GadgetTagRow> for PartialTag {
    fn from(r: GadgetTagRow) -> Self {
        PartialTag {
            id: r.id,
            label: None,
        }
    }
}

#[allow(dead_code)]
struct GadgetRow {
    pub id: Option<u32>,
    pub tag: Option<GadgetTagRow>,
    pub featured_tag: Option<Option<GadgetTagRow>>,
}

#[derive(Serialize, PartialResponse)]
#[partial_response(row = GadgetRow)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct Gadget {
    pub id: u32,
    #[partial_include(PartialTag)]
    pub tag: Tag,
    #[partial_include(PartialTag)]
    pub featured_tag: Option<Tag>,
}

#[test]
fn row_generates_from_with_embedded_conversion() {
    let row = GadgetRow {
        id: Some(1),
        tag: Some(GadgetTagRow { id: Some(10) }),
        featured_tag: Some(Some(GadgetTagRow { id: Some(20) })),
    };
    let p = PartialGadget::from(row);
    assert_eq!(p.id, Some(1));
    assert_eq!(p.tag.as_ref().unwrap().id, Some(10));
    assert_eq!(p.featured_tag.unwrap().unwrap().id, Some(20));
}

// ── `#[partial_response(skip)]` ─────────────────────────────────────────────────

#[derive(Serialize, PartialResponse)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct SkipDemo {
    pub id: u32,
    #[partial_response(skip)]
    pub internal: String,
}

#[test]
fn skip_excludes_field_from_partial() {
    // This literal compiles only because `internal` is absent from the partial.
    let p = PartialSkipDemo { id: Some(1) };
    let json = serde_json::to_value(&p).unwrap();
    assert_eq!(json.get("id").and_then(|v| v.as_u64()), Some(1));
    assert!(
        json.get("internal").is_none(),
        "skipped field must be absent"
    );
}
