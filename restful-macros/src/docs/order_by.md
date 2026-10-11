Adds `order_by: Option<Fields>` and
`order_dir: restful::ordering::OrderByDirection` query fields.

Serde defaults are no sort field and ascending direction. The application
chooses a fallback field and applies the ordering. `Fields` must deserialize,
but does not need `Default`.

```rust,ignore
#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum SortField { Name }

#[restful_macros::order_by(SortField)]
#[derive(serde::Deserialize)]
struct ListQuery {}

let query: ListQuery = serde_json::from_str("{}").unwrap();
assert!(query.order_by.is_none());
assert_eq!(query.order_dir, restful::ordering::OrderByDirection::Asc);
```

Put the attribute before derives. Add direct `restful`, Serde (with
`derive`), and Serde JSON dependencies for this example. Ordering types
are always available in `restful`.

`skip(serde)` omits defaults. The macro adds parameter attributes when a derive
path ends in `IntoParams`; renamed imports are not recognized.
`skip(utoipa)` suppresses those attributes.
