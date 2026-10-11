# restful

Request and response helpers for Rust. Ordering and pagination metadata are
always available; the other modules are feature-gated.

## Features

| Feature              | What it enables                                                          |
| -------------------- | ------------------------------------------------------------------------ |
| `response` (default) | JSON API envelopes, metadata, errors, and Axum `IntoResponse`            |
| `axum`               | Validated JSON, form, and query extractors; also enables `response`      |
| `projection`         | Field-selection parsing and validation                                   |
| `macros`             | Macro exports in ordering, pagination, and, when enabled, projection     |
| `field-delta`        | Re-exports `field_delta::Delta` as `restful::Delta`                      |
| `pgkit`              | Sort-direction conversion; column selections with `projection`           |
| `utoipa`             | Schema support for runtime types and an enabled `field-delta` dependency |
| `full`               | All features above                                                       |

Use `default-features = false` for ordering and pagination metadata alone.
The `response` feature includes Axum even without the extractor feature.
Macro-generated code may need direct Serde, Valid8r, Utoipa, or Pgkit
dependencies; enabling a feature does not add imports to your application.

## Responses and pagination

Add `serde_json = "1"` to run this example. It uses the default features:

```rust
use restful::pagination::offset::OffsetPaginationMetadata;
use restful::response::{ApiResponse, HttpErrorResponse, ResponseMetadata};

let pagination = OffsetPaginationMetadata::new(40, 1, 20);
let meta = ResponseMetadata::paginated(pagination);
let response = ApiResponse::ok(vec!["example"], Some(meta)).unwrap();
assert_eq!(response.code().as_u16(), 200);
assert_eq!(serde_json::to_value(response).unwrap()["meta"]["pagination"]["nextPage"], 2);

let error = ApiResponse::from(HttpErrorResponse::not_found("Item not found"));
assert_eq!(error.code().as_u16(), 404);
```

Success envelopes contain `data` and optional `meta`; failures contain
`error` with `type`, `message`, and optional `details`. `created` uses 201,
`accepted` uses 202, and `no_content` produces an empty 204 response.

Offset metadata clamps page size to at least one and the page to the available
range. Cursor metadata sets `has_more` from whether `next_cursor` is present.

Add custom metadata with `.set(key, serde_json::Value)`. Custom fields appear
directly inside `meta`, alongside pagination. Start with `ResponseMetadata::default()`
for a response without pagination:

```rust
use restful::response::{ApiResponse, ResponseMetadata};
use serde_json::json;

let mut meta = ResponseMetadata::default();
meta.set("requestId", json!("req_123")).unwrap();
meta.set("currency", json!({"display": "AED", "rate": "3.67"})).unwrap();
let response = ApiResponse::ok(vec!["example"], Some(meta)).unwrap();
let json = serde_json::to_value(response).unwrap();
assert_eq!(json["meta"]["requestId"], "req_123");
assert_eq!(json["meta"]["currency"]["display"], "AED");
assert!(json["meta"].get("pagination").is_none());
```

Repeated custom keys replace their previous values. The `"pagination"` key is
reserved; `.set()` returns `ReservedKeyError` for it. Convert typed metadata with
`serde_json::to_value(value)?` before adding it. With `utoipa`, custom fields are
represented by an open object schema rather than individual typed properties.

## Validated requests

Enable `axum` and add direct `serde` (with `derive`) and `valid8r = "0.2.2"`
dependencies. This handler accepts only JSON values that pass validation:

```rust,ignore
use restful::axum::ValidatedJson;
use restful::response::ApiResponse;
use serde::{Deserialize, Serialize};
use valid8r::Validate;

#[derive(Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
struct NewItem {
    #[validate(length(min = 1))]
    display_name: String,
}

async fn create(ValidatedJson(item): ValidatedJson<NewItem>) -> Result<ApiResponse, ApiResponse> {
    ApiResponse::created(item).map_err(|_| {
        ApiResponse::internal_server_error("Could not serialize response".into())
    })
}
```

`ValidatedForm` validates form data; `ValidatedQuery` validates URL queries.
They all require `DeserializeOwned + valid8r::Validate`.

Validation failures use 422. JSON data errors with a field path also use 422;
root JSON data errors, malformed input, and form/query deserialization errors
use 400. Body limits use 413; rejected body content types use 415.

Validation error keys follow a camelCase policy. Use
`#[serde(rename_all = "camelCase")]` and avoid conflicting field renames when
keys must match the request. Nested list errors retain their indices as object
keys, such as `"items": {"1": {...}}`.

Built-in deserialization messages are sanitized, but error paths can include
client-sent keys. Keep input out of custom deserializer messages. Validation
errors retain Valid8r parameters; use `#[validate(sensitive)]` on sensitive
fields and keep secrets out of custom messages and parameters.

## Field selection and PATCH

With `projection`, `Projection<T>::from(String)` stores parsing errors rather
than returning a `Result`. Check `is_valid()` or call `validate()` before using
its fields. Deserializing alone does not run validation.

With `projection` and `macros`, the `ProjectionField` and `PartialResponse`
derives support parsing field names and building partial response types.
They do not apply a projection to an existing response. See `Projection`'s API
documentation for selection syntax and `ProjectionField`'s for Pgkit column mapping.

End to end, the application checks the parsed fields and fills the partial:

```rust,ignore
use restful::projection::{PartialResponse, Projection, ProjectionField};

#[derive(serde::Serialize, ProjectionField, PartialResponse)]
#[projection_field(skip(utoipa))]
struct Item {
    id: u32,
    note: Option<String>,
}

// From ?fields=id,note
let fields = Projection::<ItemField>::from("id,note".to_owned());
assert!(fields.is_valid());

let mut partial = PartialItem::default();
if fields.fields().contains(&ItemField::Id) {
    partial.id = Some(7);
}
if fields.fields().contains(&ItemField::Note) {
    partial.note = Some(None); // selected, but null
}
assert_eq!(
    serde_json::to_value(partial).unwrap(),
    serde_json::json!({ "id": 7, "note": null })
);
```

With `field-delta`, `restful::Delta<T>` represents unchanged, clear, and set
PATCH values. Use `#[serde(default)]` on a Delta field to treat omitted input
as unchanged.
