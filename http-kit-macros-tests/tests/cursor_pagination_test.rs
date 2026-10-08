use serde::Deserialize;
use validator::Validate;

const DEFAULT_LIMIT_LITERAL: u32 = 20;
const MAX_LIMIT_LITERAL: u32 = 100;

// ── Default (all attrs emitted) ────────────────────────────────────────────

#[http_kit_macros::cursor_pagination]
#[derive(Debug, Deserialize, PartialEq, validator::Validate, utoipa::IntoParams)]
struct TestCursorStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_fields_are_added() {
    let instance = TestCursorStruct {
        existing_field: Some("test".to_string()),
        cursor: Some("abc".to_string()),
        limit: 25,
    };

    assert_eq!(instance.existing_field, Some("test".to_string()));
    assert_eq!(instance.cursor, Some("abc".to_string()));
    assert_eq!(instance.limit, 25);
}

#[test]
fn test_generated_helper_function_exists_and_correct() {
    let default_limit = default_limit_TestCursorStruct();
    assert_eq!(default_limit, DEFAULT_LIMIT_LITERAL);
}

#[test]
fn test_serde_defaults_apply_when_fields_missing() {
    let instance: TestCursorStruct = serde_json::from_str("{}").expect("deserialize empty JSON");

    assert!(instance.cursor.is_none());
    assert_eq!(instance.limit, DEFAULT_LIMIT_LITERAL);
}

#[test]
fn test_cursor_field_round_trips() {
    let none = TestCursorStruct {
        existing_field: None,
        cursor: None,
        limit: DEFAULT_LIMIT_LITERAL,
    };
    assert!(none.cursor.is_none());

    let some = TestCursorStruct {
        existing_field: None,
        cursor: Some("opaque-cursor-token".to_string()),
        limit: DEFAULT_LIMIT_LITERAL,
    };
    assert_eq!(some.cursor.as_deref(), Some("opaque-cursor-token"));
}

#[test]
fn test_validator_rejects_out_of_range_limit() {
    let zero = TestCursorStruct {
        existing_field: None,
        cursor: None,
        limit: 0,
    };
    assert!(zero.validate().is_err(), "limit = 0 must fail");

    let too_big = TestCursorStruct {
        existing_field: None,
        cursor: None,
        limit: MAX_LIMIT_LITERAL + 1,
    };
    assert!(
        too_big.validate().is_err(),
        "limit = {} must fail",
        MAX_LIMIT_LITERAL + 1
    );

    let min_ok = TestCursorStruct {
        existing_field: None,
        cursor: None,
        limit: 1,
    };
    assert!(min_ok.validate().is_ok(), "limit = 1 must pass");

    let max_ok = TestCursorStruct {
        existing_field: None,
        cursor: None,
        limit: MAX_LIMIT_LITERAL,
    };
    assert!(
        max_ok.validate().is_ok(),
        "limit = {} must pass",
        MAX_LIMIT_LITERAL
    );
}

// ── skip(validate) ─────────────────────────────────────────────────────────
//
// No #[validate(range(...))] is emitted — any limit value passes Validate.

#[http_kit_macros::cursor_pagination(skip(validate))]
#[derive(Debug, Deserialize, PartialEq, validator::Validate, utoipa::IntoParams)]
struct SkipValidateCursorStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_skip_validate_serde_defaults_still_work() {
    let instance: SkipValidateCursorStruct =
        serde_json::from_str("{}").expect("deserialize empty JSON");
    assert_eq!(instance.limit, DEFAULT_LIMIT_LITERAL);
    assert!(instance.cursor.is_none());
}

#[test]
fn test_skip_validate_out_of_range_limit_passes_validation() {
    // Without the #[validate(range(...))] attribute, Validate imposes no constraint.
    let zero = SkipValidateCursorStruct {
        existing_field: None,
        cursor: None,
        limit: 0,
    };
    assert!(
        zero.validate().is_ok(),
        "skip(validate): limit = 0 should pass"
    );

    let huge = SkipValidateCursorStruct {
        existing_field: None,
        cursor: None,
        limit: MAX_LIMIT_LITERAL + 999,
    };
    assert!(
        huge.validate().is_ok(),
        "skip(validate): limit > max should pass"
    );
}

// ── skip(utoipa) ───────────────────────────────────────────────────────────
//
// No #[param(...)] attrs are emitted. The struct still compiles and works;
// the OpenAPI schema simply lacks the min/max hints.

#[http_kit_macros::cursor_pagination(skip(utoipa))]
#[derive(Debug, Deserialize, PartialEq, validator::Validate, utoipa::IntoParams)]
struct SkipUtopiaeCursorStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_skip_utoipa_fields_and_serde_defaults_work() {
    let instance: SkipUtopiaeCursorStruct =
        serde_json::from_str("{}").expect("deserialize empty JSON");
    assert_eq!(instance.limit, DEFAULT_LIMIT_LITERAL);
    assert!(instance.cursor.is_none());
}

#[test]
fn test_skip_utoipa_validate_still_enforced() {
    let zero = SkipUtopiaeCursorStruct {
        existing_field: None,
        cursor: None,
        limit: 0,
    };
    assert!(
        zero.validate().is_err(),
        "skip(utoipa): validate still rejects limit = 0"
    );
}

// ── skip(serde) ────────────────────────────────────────────────────────────
//
// No #[serde(...)] attrs and no default helper fn are emitted.
// Without a serde default, fields must be provided explicitly when deserializing.

#[http_kit_macros::cursor_pagination(skip(serde))]
#[derive(Debug, PartialEq, validator::Validate, utoipa::IntoParams)]
struct SkipSerdeCursorStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_skip_serde_fields_exist_and_validate_works() {
    let instance = SkipSerdeCursorStruct {
        existing_field: None,
        cursor: Some("tok".to_string()),
        limit: 10,
    };
    assert_eq!(instance.limit, 10);
    assert!(instance.validate().is_ok());

    let bad = SkipSerdeCursorStruct {
        existing_field: None,
        cursor: None,
        limit: 0,
    };
    assert!(
        bad.validate().is_err(),
        "skip(serde): validate still rejects limit = 0"
    );
}

// ── skip(validate, utoipa) — serde only ───────────────────────────────────

#[http_kit_macros::cursor_pagination(skip(validate, utoipa))]
#[derive(Debug, Deserialize, PartialEq)]
struct SerdeOnlyCursorStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_serde_only_cursor_defaults_and_fields() {
    let instance: SerdeOnlyCursorStruct =
        serde_json::from_str("{}").expect("deserialize empty JSON");
    assert_eq!(instance.limit, DEFAULT_LIMIT_LITERAL);
    assert!(instance.cursor.is_none());
}

// ── skip(serde, validate, utoipa) — bare fields only ──────────────────────

#[http_kit_macros::cursor_pagination(skip(serde, validate, utoipa))]
#[derive(Debug, PartialEq)]
struct BareCursorStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_bare_cursor_fields_exist() {
    let instance = BareCursorStruct {
        existing_field: None,
        cursor: None,
        limit: 5,
    };
    assert_eq!(instance.limit, 5);
    assert!(instance.cursor.is_none());
}

// ── default = N ────────────────────────────────────────────────────────────
//
// Custom default limit; max stays at 100.

#[http_kit_macros::cursor_pagination(default_limit = 10)]
#[derive(Debug, Deserialize, PartialEq, validator::Validate, utoipa::IntoParams)]
struct CustomDefaultCursorStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_custom_default_limit_applied_by_serde() {
    let instance: CustomDefaultCursorStruct =
        serde_json::from_str("{}").expect("deserialize empty JSON");
    assert_eq!(instance.limit, 10, "custom default should be 10");
    assert!(instance.cursor.is_none());
}

#[test]
fn test_custom_default_helper_function_returns_custom_value() {
    assert_eq!(default_limit_CustomDefaultCursorStruct(), 10);
}

// ── max = N ────────────────────────────────────────────────────────────────
//
// Custom max limit; validate should reject values above it.

#[http_kit_macros::cursor_pagination(max_limit = 50)]
#[derive(Debug, Deserialize, PartialEq, validator::Validate, utoipa::IntoParams)]
struct CustomMaxCursorStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_custom_max_validate_rejects_above_custom_max() {
    let too_big = CustomMaxCursorStruct {
        existing_field: None,
        cursor: None,
        limit: 51,
    };
    assert!(
        too_big.validate().is_err(),
        "limit = 51 should fail when max = 50"
    );

    let ok = CustomMaxCursorStruct {
        existing_field: None,
        cursor: None,
        limit: 50,
    };
    assert!(
        ok.validate().is_ok(),
        "limit = 50 should pass when max = 50"
    );

    // Old hard-coded limit of 100 is now allowed (since max = 50 raised above its old value means
    // values above 50 fail; 100 is above 50 so it should fail).
    let old_max = CustomMaxCursorStruct {
        existing_field: None,
        cursor: None,
        limit: 100,
    };
    assert!(
        old_max.validate().is_err(),
        "limit = 100 should fail when max = 50"
    );
}

// ── default = N, max = N ───────────────────────────────────────────────────

#[http_kit_macros::cursor_pagination(default_limit = 5, max_limit = 25)]
#[derive(Debug, Deserialize, PartialEq, validator::Validate, utoipa::IntoParams)]
struct CustomBothCursorStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_custom_both_default_and_max() {
    let from_empty: CustomBothCursorStruct =
        serde_json::from_str("{}").expect("deserialize empty JSON");
    assert_eq!(from_empty.limit, 5);

    let too_big = CustomBothCursorStruct {
        existing_field: None,
        cursor: None,
        limit: 26,
    };
    assert!(
        too_big.validate().is_err(),
        "limit = 26 should fail when max = 25"
    );

    let at_max = CustomBothCursorStruct {
        existing_field: None,
        cursor: None,
        limit: 25,
    };
    assert!(
        at_max.validate().is_ok(),
        "limit = 25 should pass when max = 25"
    );
}

// ── skip + custom values ───────────────────────────────────────────────────

#[http_kit_macros::cursor_pagination(skip(utoipa), default_limit = 15, max_limit = 60)]
#[derive(Debug, Deserialize, PartialEq, validator::Validate, utoipa::IntoParams)]
struct SkipUtoipaCustomCursorStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_skip_utoipa_with_custom_values() {
    let from_empty: SkipUtoipaCustomCursorStruct =
        serde_json::from_str("{}").expect("deserialize empty JSON");
    assert_eq!(from_empty.limit, 15);

    let too_big = SkipUtoipaCustomCursorStruct {
        existing_field: None,
        cursor: None,
        limit: 61,
    };
    assert!(
        too_big.validate().is_err(),
        "limit = 61 should fail when max = 60"
    );
}
