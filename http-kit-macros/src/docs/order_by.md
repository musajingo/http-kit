Injects ordering fields (`order_by`, `order_dir`) into a struct.

Takes one required positional argument: the enum that represents sortable
fields. The enum does **not** need to implement `Default` — `order_by`
is wrapped in `Option<_>` so an absent value deserializes to `None`,
and the service layer picks a fallback (typically the table PK).

## Generated fields

| Field | Type | Default |
|---|---|---|
| `order_by` | `Option<FieldsEnum>` | `None` |
| `order_dir` | `::http_kit::ordering::OrderByDirection` | `OrderByDirection::default()` |

## Generated attributes (suppressible via `skip(...)`)

- `#[serde(default)]` on both fields — suppressed with `skip(serde)`

## Syntax

```text
#[order_by(MyOrderByFields)]
#[order_by(MyOrderByFields, skip(serde))]
```

## Example

```ignore
#[http_kit_macros::order_by(UserOrderByFields)]
#[derive(Debug, serde::Deserialize)]
pub struct ListUsersQuery {
    pub search: Option<String>,
}

// Injected: order_by (default None — caller did not specify),
//           order_dir (default OrderByDirection::default())
let q: ListUsersQuery = serde_json::from_str("{}").unwrap();
assert!(q.order_by.is_none());
```

## Requirements

Consumer crates must depend on `http-kit` with the `ordering` feature
enabled for `OrderByDirection` to be in scope.
