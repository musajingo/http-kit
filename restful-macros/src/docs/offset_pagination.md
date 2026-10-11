Adds `page: u32` and `page_size: u32` query fields.

Serde defaults are page 1 and page size 20. Generated Valid8r rules require
page ≥ 1 and page size between 1 and 100. Validation must be called explicitly
or through a validated extractor.

| Argument | Effect |
| --- | --- |
| `default_page_size = N` | Change the omitted page size |
| `max_page_size = N` | Change its validation maximum |
| `skip(serde)` | Omit defaults and their helpers |
| `skip(validate)` | Omit range rules |
| `skip(utoipa)` | Omit Utoipa parameter attributes |

Defaults and maxima must be positive, with default ≤ maximum. Invalid settings
fail compilation even with `skip(validate)`. Put this attribute before derives.
This example needs direct Serde, Valid8r, and Serde JSON dependencies:

```rust,ignore
#[restful_macros::offset_pagination(default_page_size = 25, max_page_size = 200, skip(utoipa))]
#[derive(serde::Deserialize, valid8r::Validate)]
struct ListQuery {}

let query: ListQuery = serde_json::from_str("{}").unwrap();
valid8r::Validate::validate(&query).unwrap();
assert_eq!((query.page, query.page_size), (1, 25));
```

Without `skip(utoipa)`, add Utoipa and derive `utoipa::IntoParams`.
