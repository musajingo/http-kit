Generates `Partial{Struct}`, with one extra `Option` around each included
field. It derives `Debug`, `Default`, and `Serialize`. This example needs
direct Serde (with `derive`) and Serde JSON dependencies.

```rust,ignore
#[derive(serde::Serialize, restful_macros::PartialResponse)]
struct Item {
    id: u32,
    note: Option<String>,
}

let partial = PartialItem { id: Some(1), note: Some(None) };
assert_eq!(serde_json::to_value(partial).unwrap(), serde_json::json!({
    "id": 1,
    "note": null
}));
```

An outer `None` omits the field. For nullable fields, `Some(None)` includes
JSON null. Your application decides which fields to fill.

| Attribute | Effect |
| --- | --- |
| `#[partial_response(name = "Summary")]` on the struct | Change the generated type name |
| `#[partial_response(row = PartialRow)]` on the struct | Generate `From<PartialRow>` |
| `#[partial_response(row_field = source)]` on a field | Read a different row field |
| `#[partial_response(skip)]` on a field | Omit it from the partial and conversion |
| `#[partial_include(PartialChild)]` on a field | Use an embedded partial type |
| `#[partial_include(PartialChild, with = convert)]` | Use a row-to-child conversion function |

`row = ...` requires matching row field shapes: leaves move directly;
embedded fields convert using `Into` (or `with`) across their `Option` layers.
Without `row`, no conversion is generated.

Container and field Serde attributes are carried over, except field
`skip_serializing_if`, which is replaced by the outer `Option::is_none` check.
Field `serialize_with` and `with` are rejected because their signatures would
not match the added `Option` layer. Implement `Serialize` on the field type
or skip the field instead.

Only named-field structs without lifetime, type, or const parameters are
supported. `skip` cannot be combined with `row_field` or `partial_include`.
This derive does not generate `Deserialize`, `Clone`, or Utoipa schemas.
