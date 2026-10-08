use http_kit::ordering::OrderByDirection;
use http_kit_macros::order_by;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Default, Deserialize, PartialEq, Clone, Copy, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TestSortableFields {
    #[default]
    Name,
    Value,
    CreatedAt,
}

impl std::fmt::Display for TestSortableFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestSortableFields::Name => write!(f, "name"),
            TestSortableFields::CreatedAt => write!(f, "created_at"),
            TestSortableFields::Value => write!(f, "value_column"),
        }
    }
}

// ── Default (serde attrs emitted) ──────────────────────────────────────────

#[order_by(TestSortableFields)]
#[derive(Debug, Deserialize, PartialEq, IntoParams)]
struct TestSortingParams {
    #[allow(dead_code)]
    existing_param: Option<String>,
}

#[test]
fn test_fields_are_added_by_macro() {
    let instance = TestSortingParams {
        existing_param: Some("test".to_string()),
        order_by: Some(TestSortableFields::Name),
        order_dir: OrderByDirection::Asc,
    };

    assert_eq!(instance.order_by, Some(TestSortableFields::Name));
    assert_eq!(instance.order_dir, OrderByDirection::Asc);
    assert_eq!(instance.existing_param, Some("test".to_string()));
}

#[test]
fn test_fields_defaults() {
    // Absent `orderBy` deserializes to `None` — no hardcoded default. The
    // service layer is expected to fall back to the table primary key.
    let instance: TestSortingParams = serde_json::from_str("{}").expect("deserialize empty JSON");
    assert!(instance.order_by.is_none());
    assert_eq!(instance.order_dir, OrderByDirection::Asc);
    assert!(instance.existing_param.is_none());
}

#[test]
fn test_fields_deserialize_explicit_value() {
    let instance: TestSortingParams = serde_json::from_str(r#"{"order_by":"created_at"}"#)
        .expect("deserialize explicit order_by");
    assert_eq!(instance.order_by, Some(TestSortableFields::CreatedAt));
}

// ── skip(serde) ────────────────────────────────────────────────────────────
//
// No #[serde(default)] on order_by or order_dir. The caller is responsible for
// providing values or using their own deserialization defaults.

#[order_by(TestSortableFields, skip(serde))]
#[derive(Debug, PartialEq, IntoParams)]
struct SkipSerdeSortingParams {
    #[allow(dead_code)]
    existing_param: Option<String>,
}

#[test]
fn test_skip_serde_sorting_fields_exist() {
    let instance = SkipSerdeSortingParams {
        existing_param: None,
        order_by: Some(TestSortableFields::CreatedAt),
        order_dir: OrderByDirection::Desc,
    };
    assert_eq!(instance.order_by, Some(TestSortableFields::CreatedAt));
    assert_eq!(instance.order_dir, OrderByDirection::Desc);
}
