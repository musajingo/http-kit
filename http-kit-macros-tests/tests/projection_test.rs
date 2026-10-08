//! Integration tests for `#[projection(ResponseField)]`.

use http_kit::projection::ProjectionField;
use http_kit_macros::projection;
use serde::Deserialize;
use validator::Validate;

// ── Test fixtures ─────────────────────────────────────────────────────────────

/// A simple resource to project against.
#[derive(ProjectionField)]
#[allow(dead_code)]
struct Article {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

/// Default usage — `fields` is required (validate attr present).
#[projection(ArticleField)]
#[derive(Debug, Deserialize, Validate)]
struct ListArticlesQuery {
    pub search: Option<String>,
}

/// With `skip(validate)` — `fields` is optional, no validate attr injected.
#[projection(ArticleField, skip(validate))]
#[derive(Debug, Deserialize, Validate)]
struct ListArticlesQueryNoValidate {
    #[allow(dead_code)]
    pub search: Option<String>,
}

/// Composed with other injected fields (sorting-style pattern).
#[projection(ArticleField)]
#[derive(Debug, Deserialize, Validate)]
struct ComposedQuery {
    #[allow(dead_code)]
    pub search: Option<String>,
    pub page: Option<u32>,
}

// ── Field injection ───────────────────────────────────────────────────────────

#[test]
fn fields_field_is_injected_as_option() {
    // `fields` defaults to None when absent.
    let q: ListArticlesQuery = serde_json::from_str("{}").unwrap();
    assert!(q.fields.is_none());
}

#[test]
fn fields_field_parses_valid_projection_string() {
    let q: ListArticlesQuery = serde_json::from_str(r#"{"fields":"id,title"}"#).unwrap();
    let projection = q.fields.as_ref().unwrap();
    let names: Vec<&str> = projection.fields().iter().map(|f| f.as_ref()).collect();
    assert_eq!(names, vec!["id", "title"]);
}

#[test]
fn fields_field_parses_all_keyword() {
    let q: ListArticlesQuery = serde_json::from_str(r#"{"fields":"all"}"#).unwrap();
    let projection = q.fields.as_ref().unwrap();
    assert_eq!(projection.fields().len(), ArticleField::all().len());
}

#[test]
fn existing_fields_are_preserved() {
    let q: ListArticlesQuery = serde_json::from_str(r#"{"search":"rust","fields":"id"}"#).unwrap();
    assert_eq!(q.search.as_deref(), Some("rust"));
}

// ── Validate behaviour ────────────────────────────────────────────────────────

#[test]
fn validate_fails_when_fields_is_none() {
    let q: ListArticlesQuery = serde_json::from_str("{}").unwrap();
    let err = q.validate().unwrap_err();
    let json = serde_json::to_string(&err).unwrap();
    assert!(
        json.contains("fields"),
        "error should mention the 'fields' key: {json}"
    );
}

#[test]
fn validate_passes_for_valid_projection() {
    let q: ListArticlesQuery = serde_json::from_str(r#"{"fields":"id,title"}"#).unwrap();
    assert!(q.validate().is_ok());
}

#[test]
fn validate_fails_for_invalid_projection_string() {
    // "unknown" is not a valid ArticleField name.
    let q: ListArticlesQuery = serde_json::from_str(r#"{"fields":"unknown"}"#).unwrap();
    assert!(q.validate().is_err());
}

// ── skip(validate) ────────────────────────────────────────────────────────────

#[test]
fn skip_validate_passes_when_fields_is_none() {
    let q: ListArticlesQueryNoValidate = serde_json::from_str("{}").unwrap();
    // No required constraint — should pass even without `fields`.
    assert!(q.validate().is_ok());
}

#[test]
fn skip_validate_still_parses_when_fields_provided() {
    let q: ListArticlesQueryNoValidate = serde_json::from_str(r#"{"fields":"id,body"}"#).unwrap();
    let projection = q.fields.unwrap();
    let names: Vec<&str> = projection.fields().iter().map(|f| f.as_ref()).collect();
    assert_eq!(names, vec!["id", "body"]);
}

// ── Composed structs ──────────────────────────────────────────────────────────

#[test]
fn composed_query_other_fields_unaffected() {
    let q: ComposedQuery = serde_json::from_str(r#"{"page":2,"fields":"title"}"#).unwrap();
    assert_eq!(q.page, Some(2));
    let projection = q.fields.unwrap();
    let names: Vec<&str> = projection.fields().iter().map(|f| f.as_ref()).collect();
    assert_eq!(names, vec!["title"]);
}
