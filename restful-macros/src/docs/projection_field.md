Generates `{Struct}Field`: a typed set of selectable fields implementing
`restful::projection::ProjectionField`. Enable `restful`'s `projection`
feature and add Serde (with `derive`) directly.

```rust,ignore
use restful::projection::{Projection, ProjectionField};

#[derive(serde::Serialize, restful_macros::ProjectionField)]
#[serde(rename_all = "camelCase")]
#[projection_field(skip(utoipa))]
struct Item { id: u32, display_name: String }

assert_eq!(ItemField::all_field_names(), &["id", "displayName"]);
let fields = Projection::<ItemField>::from("id,displayName".to_owned());
assert!(fields.is_valid());
assert_eq!(fields.fields(), &[ItemField::Id, ItemField::DisplayName]);
assert_eq!(ItemField::DisplayName.as_ref(), "displayName");
assert_eq!(ItemField::try_from("displayName").unwrap(), ItemField::DisplayName);
assert_eq!(ItemField::try_from("id".to_owned()).unwrap(), ItemField::Id);
```

Wire names use serialize-side Serde field renames, then `rename_all`, then
the raw Rust field name. Enum variants use PascalCase Rust names. The enum
also derives `Debug`, `Clone`, `Hash`, `PartialEq`, `Eq`, `Serialize`, and
`Deserialize`; its Serde representation is separate from projection tokens.
`AsRef<str>` returns the wire name; `TryFrom<&str>` and `TryFrom<String>` parse
a single field name, returning `ProjectionError` for an unknown field.

| Attribute | Effect |
| --- | --- |
| `#[projection_field(name = "item")]` on the struct | Set the resource name in errors; defaults to the lowercased struct name |
| `#[projection_field(skip(utoipa))]` on the struct | Omit generated Utoipa implementations |
| `#[projection_field(column = ItemColumn)]` on the struct | Generate Pgkit column selections |
| `#[projection_field(ChildField)]` on a field | Embed a child's field enum under this field's wire name |
| `#[projection_field(column = ItemColumn::Other)]` or `#[column(ItemColumn::Other)]` | Override a leaf's column mapping |
| `#[projection_skip]` on a field | Exclude it from selectable fields |

Without `skip(utoipa)`, add Utoipa directly for the generated schema
implementations. `column = ...` also needs a direct Pgkit dependency and
`restful`'s `pgkit` feature. Leaves map to same-named column variants unless
overridden. An embedding resource returns its own selection plus one optional
child selection per embed, in declaration order; child outputs can be nested
tuples. This generates `IntoColumnSelections`, not a `From` conversion.

See `restful::projection::Projection` for selection syntax and validation rules.

Only named-field structs are supported. Nested fields cannot have column
overrides. `projection_skip` cannot combine with nested or column attributes.
