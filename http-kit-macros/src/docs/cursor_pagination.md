Adds `cursor: Option<String>` and `limit: u32` query fields.

Serde defaults are no cursor and a limit of 20. The generated Valid8r rule
requires a limit between 1 and 100. The application creates and interprets
the cursor; the macro adds only query fields and attributes.

| Argument | Effect |
| --- | --- |
| `default_limit = N` | Change the omitted limit |
| `max_limit = N` | Change its validation maximum |
| `skip(serde)` | Omit defaults and their helper |
| `skip(validate)` | Omit the range rule |
| `skip(utoipa)` | Omit Utoipa parameter attributes |

Defaults and maxima must be positive, with default ≤ maximum. Invalid settings
fail compilation even with `skip(validate)`. Put this attribute before derives.
This example needs direct Serde, Valid8r, and Serde JSON dependencies:

```rust,ignore
#[http_kit_macros::cursor_pagination(default_limit = 10, max_limit = 50, skip(utoipa))]
#[derive(serde::Deserialize, valid8r::Validate)]
struct ListQuery {}

let query: ListQuery = serde_json::from_str("{}").unwrap();
valid8r::Validate::validate(&query).unwrap();
assert!(query.cursor.is_none());
assert_eq!(query.limit, 10);
```

Call `validate()` or use a validated extractor to enforce the range. Without
`skip(utoipa)`, add Utoipa and derive `utoipa::IntoParams`.
