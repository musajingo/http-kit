Injects offset-pagination fields (`page`, `page_size`) into a struct.

## Generated fields

| Field | Type | Default | Range |
|---|---|---|---|
| `page` | `u32` | `1` (or `default_page`) | `≥ 1` |
| `page_size` | `u32` | `20` (or `default_page_size`) | `1 – 100` (or `max_page_size`) |

## Arguments

| Argument | Type | Default | Description |
|---|---|---|---|
| `default_page_size = N` | `u32` | `20` | Default page size |
| `max_page_size = N` | `u32` | `100` | Maximum allowed page size |
| `skip(serde)` | — | off | Omit `#[serde(...)]` attrs and helper fns |
| `skip(validate)` | — | off | Omit `#[validate(range(...))]` on both fields |
| `skip(utoipa)` | — | off | Omit `#[param(...)]` on both fields |

## Syntax

```text
#[offset_pagination]
#[offset_pagination(default_page_size = 25)]
#[offset_pagination(max_page_size = 50)]
#[offset_pagination(default_page_size = 25, max_page_size = 50)]
#[offset_pagination(skip(utoipa), default_page_size = 10)]
#[offset_pagination(skip(serde, validate, utoipa))]
```

## Example

```ignore
#[http_kit_macros::offset_pagination(default_page_size = 25, max_page_size = 200)]
#[derive(Debug, serde::Deserialize, valid8r::Validate, utoipa::IntoParams)]
pub struct ListUsersQuery {
    pub search: Option<String>,
}

// Injected: page (default 1), page_size (default 25, max 200)
let q: ListUsersQuery = serde_json::from_str("{}").unwrap();
assert_eq!(q.page, 1);
assert_eq!(q.page_size, 25);
```
