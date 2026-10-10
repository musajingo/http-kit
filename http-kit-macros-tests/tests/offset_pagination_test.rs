use serde::Deserialize;

const DEFAULT_PAGE_LITERAL: u32 = 1;
const DEFAULT_PAGE_SIZE_LITERAL: u32 = 20;
const MAX_PAGE_SIZE_LITERAL: u32 = 100;

// ── Default (all attrs emitted) ────────────────────────────────────────────

#[http_kit_macros::offset_pagination]
#[derive(Debug, Deserialize, PartialEq, valid8r::Validate, utoipa::IntoParams)]
struct TestPaginationStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_fields_are_added() {
    let instance = TestPaginationStruct {
        existing_field: Some("test".to_string()),
        page: 1,
        page_size: 10,
    };

    assert_eq!(instance.page, 1);
    assert_eq!(instance.page_size, 10);
    assert_eq!(instance.existing_field, Some("test".to_string()));
}

#[test]
fn test_generated_helper_functions_exist_and_correct() {
    let default_page = default_page_TestPaginationStruct();
    assert_eq!(default_page, DEFAULT_PAGE_LITERAL);

    let default_page_size = default_page_size_TestPaginationStruct();
    assert_eq!(default_page_size, DEFAULT_PAGE_SIZE_LITERAL);
}

#[test]
fn test_serde_defaults_apply_when_fields_missing() {
    let instance: TestPaginationStruct =
        serde_json::from_str("{}").expect("deserialize empty JSON");

    assert_eq!(instance.page, DEFAULT_PAGE_LITERAL);
    assert_eq!(instance.page_size, DEFAULT_PAGE_SIZE_LITERAL);
}

#[test]
fn test_validator_rejects_out_of_range() {
    use valid8r::Validate;

    let zero_page = TestPaginationStruct {
        existing_field: None,
        page: 0,
        page_size: DEFAULT_PAGE_SIZE_LITERAL,
    };
    assert!(zero_page.validate().is_err(), "page = 0 must fail");

    let zero_page_size = TestPaginationStruct {
        existing_field: None,
        page: DEFAULT_PAGE_LITERAL,
        page_size: 0,
    };
    assert!(
        zero_page_size.validate().is_err(),
        "page_size = 0 must fail"
    );

    let too_big = TestPaginationStruct {
        existing_field: None,
        page: DEFAULT_PAGE_LITERAL,
        page_size: MAX_PAGE_SIZE_LITERAL + 1,
    };
    assert!(
        too_big.validate().is_err(),
        "page_size = {} must fail",
        MAX_PAGE_SIZE_LITERAL + 1
    );

    let ok = TestPaginationStruct {
        existing_field: None,
        page: 1,
        page_size: MAX_PAGE_SIZE_LITERAL,
    };
    assert!(
        ok.validate().is_ok(),
        "page = 1 / page_size = max must pass"
    );
}

// ── skip(validate) ─────────────────────────────────────────────────────────
//
// No #[validate(range(...))] emitted — any page/page_size value passes Validate.

#[http_kit_macros::offset_pagination(skip(validate))]
#[derive(Debug, Deserialize, PartialEq, valid8r::Validate, utoipa::IntoParams)]
struct SkipValidatePaginationStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_skip_validate_serde_defaults_still_work() {
    let instance: SkipValidatePaginationStruct =
        serde_json::from_str("{}").expect("deserialize empty JSON");
    assert_eq!(instance.page, DEFAULT_PAGE_LITERAL);
    assert_eq!(instance.page_size, DEFAULT_PAGE_SIZE_LITERAL);
}

#[test]
fn test_skip_validate_out_of_range_passes_validation() {
    use valid8r::Validate;

    let bad = SkipValidatePaginationStruct {
        existing_field: None,
        page: 0,
        page_size: 0,
    };
    assert!(
        bad.validate().is_ok(),
        "skip(validate): page = 0, page_size = 0 should pass"
    );

    let huge = SkipValidatePaginationStruct {
        existing_field: None,
        page: 1,
        page_size: MAX_PAGE_SIZE_LITERAL + 999,
    };
    assert!(
        huge.validate().is_ok(),
        "skip(validate): page_size > max should pass"
    );
}

// ── skip(utoipa) ───────────────────────────────────────────────────────────
//
// No #[param(...)] attrs emitted. Struct still fully functional.

#[http_kit_macros::offset_pagination(skip(utoipa))]
#[derive(Debug, Deserialize, PartialEq, valid8r::Validate, utoipa::IntoParams)]
struct SkipUtoipaPaginationStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_skip_utoipa_serde_and_validate_still_work() {
    use valid8r::Validate;

    let instance: SkipUtoipaPaginationStruct =
        serde_json::from_str("{}").expect("deserialize empty JSON");
    assert_eq!(instance.page, DEFAULT_PAGE_LITERAL);
    assert_eq!(instance.page_size, DEFAULT_PAGE_SIZE_LITERAL);

    let bad = SkipUtoipaPaginationStruct {
        existing_field: None,
        page: 0,
        page_size: 1,
    };
    assert!(
        bad.validate().is_err(),
        "skip(utoipa): validate still rejects page = 0"
    );
}

// ── skip(serde) ────────────────────────────────────────────────────────────
//
// No #[serde(...)] attrs and no helper fns emitted.
// Validate and utoipa attrs are still present.

#[http_kit_macros::offset_pagination(skip(serde))]
#[derive(Debug, PartialEq, valid8r::Validate, utoipa::IntoParams)]
struct SkipSerdePaginationStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_skip_serde_fields_exist_and_validate_works() {
    use valid8r::Validate;

    let instance = SkipSerdePaginationStruct {
        existing_field: None,
        page: 2,
        page_size: 10,
    };
    assert_eq!(instance.page, 2);
    assert_eq!(instance.page_size, 10);
    assert!(instance.validate().is_ok());

    let bad = SkipSerdePaginationStruct {
        existing_field: None,
        page: 0,
        page_size: 1,
    };
    assert!(
        bad.validate().is_err(),
        "skip(serde): validate still rejects page = 0"
    );
}

// ── skip(validate, utoipa) — serde only ───────────────────────────────────

#[http_kit_macros::offset_pagination(skip(validate, utoipa))]
#[derive(Debug, Deserialize, PartialEq)]
struct SerdeOnlyPaginationStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_serde_only_pagination_defaults_and_fields() {
    let instance: SerdeOnlyPaginationStruct =
        serde_json::from_str("{}").expect("deserialize empty JSON");
    assert_eq!(instance.page, DEFAULT_PAGE_LITERAL);
    assert_eq!(instance.page_size, DEFAULT_PAGE_SIZE_LITERAL);
}

// ── skip(serde, validate, utoipa) — bare fields only ──────────────────────

#[http_kit_macros::offset_pagination(skip(serde, validate, utoipa))]
#[derive(Debug, PartialEq)]
struct BarePaginationStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_bare_pagination_fields_exist() {
    let instance = BarePaginationStruct {
        existing_field: None,
        page: 3,
        page_size: 15,
    };
    assert_eq!(instance.page, 3);
    assert_eq!(instance.page_size, 15);
}

// ── default_page_size = N ──────────────────────────────────────────────────
//
// Custom default page size; max stays at 100.

#[http_kit_macros::offset_pagination(default_page_size = 25)]
#[derive(Debug, Deserialize, PartialEq, valid8r::Validate, utoipa::IntoParams)]
struct CustomDefaultPageSizeStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_custom_default_page_size_applied_by_serde() {
    let instance: CustomDefaultPageSizeStruct =
        serde_json::from_str("{}").expect("deserialize empty JSON");
    assert_eq!(
        instance.page, DEFAULT_PAGE_LITERAL,
        "page default unchanged"
    );
    assert_eq!(
        instance.page_size, 25,
        "custom default page size should be 25"
    );
}

#[test]
fn test_custom_default_page_size_helper_returns_custom_value() {
    assert_eq!(default_page_size_CustomDefaultPageSizeStruct(), 25);
}

// ── max_page_size = N ──────────────────────────────────────────────────────
//
// Custom max page size; validate should reject values above it.

#[http_kit_macros::offset_pagination(max_page_size = 50)]
#[derive(Debug, Deserialize, PartialEq, valid8r::Validate, utoipa::IntoParams)]
struct CustomMaxPageSizeStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_custom_max_page_size_validate_rejects_above_it() {
    use valid8r::Validate;

    let too_big = CustomMaxPageSizeStruct {
        existing_field: None,
        page: 1,
        page_size: 51,
    };
    assert!(
        too_big.validate().is_err(),
        "page_size = 51 should fail when max = 50"
    );

    let at_max = CustomMaxPageSizeStruct {
        existing_field: None,
        page: 1,
        page_size: 50,
    };
    assert!(
        at_max.validate().is_ok(),
        "page_size = 50 should pass when max = 50"
    );

    // Old hard-coded max of 100 now fails because our max is 50.
    let old_max = CustomMaxPageSizeStruct {
        existing_field: None,
        page: 1,
        page_size: 100,
    };
    assert!(
        old_max.validate().is_err(),
        "page_size = 100 should fail when max = 50"
    );
}

// ── default_page_size = N, max_page_size = N ───────────────────────────────

#[http_kit_macros::offset_pagination(default_page_size = 10, max_page_size = 200)]
#[derive(Debug, Deserialize, PartialEq, valid8r::Validate, utoipa::IntoParams)]
struct CustomBothPageSizeStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_custom_both_page_size_and_max() {
    use valid8r::Validate;

    let from_empty: CustomBothPageSizeStruct =
        serde_json::from_str("{}").expect("deserialize empty JSON");
    assert_eq!(from_empty.page_size, 10, "custom default page_size");

    // Old max of 100 is now within the new max of 200.
    let was_max = CustomBothPageSizeStruct {
        existing_field: None,
        page: 1,
        page_size: 100,
    };
    assert!(
        was_max.validate().is_ok(),
        "page_size = 100 should pass when max = 200"
    );

    let too_big = CustomBothPageSizeStruct {
        existing_field: None,
        page: 1,
        page_size: 201,
    };
    assert!(
        too_big.validate().is_err(),
        "page_size = 201 should fail when max = 200"
    );
}

// ── skip + custom values ───────────────────────────────────────────────────

#[http_kit_macros::offset_pagination(skip(utoipa), default_page_size = 15, max_page_size = 75)]
#[derive(Debug, Deserialize, PartialEq, valid8r::Validate, utoipa::IntoParams)]
struct SkipUtoipaCustomPageSizeStruct {
    #[allow(dead_code)]
    existing_field: Option<String>,
}

#[test]
fn test_skip_utoipa_with_custom_page_size_values() {
    use valid8r::Validate;

    let from_empty: SkipUtoipaCustomPageSizeStruct =
        serde_json::from_str("{}").expect("deserialize empty JSON");
    assert_eq!(from_empty.page_size, 15);

    let too_big = SkipUtoipaCustomPageSizeStruct {
        existing_field: None,
        page: 1,
        page_size: 76,
    };
    assert!(
        too_big.validate().is_err(),
        "page_size = 76 should fail when max = 75"
    );
}
