Injects cursor-pagination fields (`cursor`, `limit`) into a struct.

## Generated fields

| Field | Type | Default | Range |
|---|---|---|---|
| `cursor` | `Option<String>` | `None` | — |
| `limit` | `u32` | `20` (or `default`) | `1 – 100` (or `max`) |

## Arguments

| Argument | Type | Default | Description |
|---|---|---|---|
| `default_limit = N` | `u32` | `20` | Default limit when the query parameter is omitted |
| `max_limit = N` | `u32` | `100` | Maximum allowed limit; enforced by `#[validate(range(...))]` |
| `skip(serde)` | — | off | Omit `#[serde(...)]` attrs and the default-limit helper fn |
| `skip(validate)` | — | off | Omit `#[validate(range(...))]` on `limit` |
| `skip(utoipa)` | — | off | Omit `#[param(...)]` on both fields |

## Syntax

```text
#[cursor_pagination]
#[cursor_pagination(default_limit = 10)]
#[cursor_pagination(max_limit = 50)]
#[cursor_pagination(default_limit = 10, max_limit = 50)]
#[cursor_pagination(skip(validate, utoipa))]
#[cursor_pagination(skip(validate), default_limit = 10, max_limit = 50)]
#[cursor_pagination(skip(serde, validate, utoipa))]
```

## Example

```ignore
#[http_kit_macros::cursor_pagination(default_limit = 10, max_limit = 50)]
#[derive(Debug, serde::Deserialize, valid8r::Validate, utoipa::IntoParams)]
pub struct ListItemsQuery {
    pub search: Option<String>,
}

// Injected: cursor (default None), limit (default 10, max 50)
let q: ListItemsQuery = serde_json::from_str("{}").unwrap();
assert!(q.cursor.is_none());
assert_eq!(q.limit, 10);
```
