# restful-macros

Attributes add query fields; derives generate field enums and partial response
types. All macros operate on structs with named fields.

Usually enable `restful`'s `macros` feature and use its exports. Projection
exports also need `projection`. Generated ordering and field-selection code
also needs the runtime `restful` crate.

## Macros

| Macro                | Output                                                         |
| -------------------- | -------------------------------------------------------------- |
| `offset_pagination`  | `page: u32`, `page_size: u32`                                  |
| `cursor_pagination`  | `cursor: Option<String>`, `limit: u32`                         |
| `order_by(Fields)`   | `order_by: Option<Fields>`, `order_dir: OrderByDirection`      |
| `projection(Fields)` | `fields: Option<Projection<Fields>>`                           |
| `ProjectionField`    | A `{Struct}Field` enum and projection trait implementations    |
| `PartialResponse`    | A `Partial{Struct}` with one extra `Option` per included field |

## Query example

This example uses the macros directly. Add these dependencies; `restful`
supplies the generated ordering type:

```toml
[dependencies]
restful-macros = "0.2.0"
restful = { version = "0.2.0", default-features = false }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
valid8r = "0.2.2"
```

```rust,ignore
use serde::Deserialize;
use valid8r::Validate;
use restful_macros::{cursor_pagination, order_by};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SortField { Name }

#[cursor_pagination(skip(utoipa))]
#[order_by(SortField)]
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct ListQuery {}

let query: ListQuery = serde_json::from_str("{}").unwrap();
query.validate().unwrap();
assert_eq!(query.limit, 20);
assert!(query.order_by.is_none());
```

Put query attributes before `#[derive(...)]`. Serde fills defaults; calling
`validate()` or using a validated extractor enforces the generated rules.
Pagination defaults to 20 items with a maximum of 100. Defaults and maxima
must be positive, with default ≤ maximum; invalid settings fail compilation,
even when validation attributes are skipped.

Pagination supports `skip(serde, validate, utoipa)`. Ordering supports
`skip(serde, utoipa)`; projection supports `skip(validate, utoipa)`.
Skipping an attribute group leaves its behavior to your application.

Pagination emits Utoipa attributes unless skipped; derive `utoipa::IntoParams`
and add Utoipa directly to use them. Ordering and projection check the final
derive name for `IntoParams`; renamed imports are not recognized.
`ProjectionField` emits Utoipa schema implementations unless
`#[projection_field(skip(utoipa))]` is set, independently of `restful`'s
`utoipa` feature.

## Response derives

`ProjectionField` uses serialize-side Serde field names. `PartialResponse`
omits fields whose outer `Option` is `None`; `Some(None)` represents a selected
nullable field and serializes as null. The application fills the selected fields.

`PartialResponse` requires a struct without lifetime, type, or const parameters.
It rejects field-level `serde(serialize_with)` and `serde(with)`; implement `Serialize`
on the field type or skip that field instead. Its optional `row = ...`
conversion requires a row with matching field shapes.
