Generates a **partial** companion of a response struct — the same struct with
every field wrapped in an extra `Option` — so an endpoint can serialize only the
fields a client actually asked for (sparse fieldsets, e.g. `?fields=id,name`).

# Why this exists

Endpoints support sparse fieldsets: a client sends `?fields=id,name` and the
response body must contain **only** `id` and `name`. The normal response struct
has non-optional fields and cannot express "this field was not requested." The
generated partial solves that:

- a field that **was** requested → `Some(value)` → serialized;
- a field that **was not** requested → `None` → omitted entirely (every field
  gets `#[serde(skip_serializing_if = "Option::is_none")]`).

`PartialResponse` is the **serialization** half of projection. Its counterpart
is the `ProjectionField` derive, which parses the `?fields=`
query parameter. A handler typically: parses the projection, asks the repository
for just those columns, builds the partial (selected fields `Some`, the rest
`None`), and returns it.

# What it generates

For a struct `Foo`, it emits a sibling struct **`PartialFoo`** (the name is the
struct name prefixed with `Partial`; override with
`#[partial_response(name = "...")]`).

The generated struct:

- derives `Debug`, `Default`, `Serialize` (it is a serialize-only output type);
- inherits the **container-level** `#[serde(...)]` attributes of the original
  (e.g. `#[serde(rename_all = "camelCase")]`);
- wraps every field in one extra `Option` and tags it
  `#[serde(skip_serializing_if = "Option::is_none")]`.

# Field transformation rules

| Original field                                    | Generated partial field           |
| ------------------------------------------------- | --------------------------------- |
| `id: i32`                                         | `id: Option<i32>`                 |
| `name: String`                                    | `name: Option<String>`            |
| `native: Option<String>` (nullable)               | `native: Option<Option<String>>`  |
| `#[partial_include(PartialBar)] bar: Bar`         | `bar: Option<PartialBar>`         |
| `#[partial_include(PartialBar)] bar: Option<Bar>` | `bar: Option<Option<PartialBar>>` |

The extra `Option` everywhere encodes _"was this projected?"_. For an
already-nullable field the result is `Option<Option<T>>`, where the **outer**
`Option` is projection and the **inner** is the column's nullability:

- `None` — not projected (omitted from JSON);
- `Some(None)` — projected, value is SQL `NULL` (serialized as `null`);
- `Some(Some(v))` — projected with a value.

# Arguments

## Struct attribute — `#[partial_response(...)]`

| Key                    | Effect                                                        | Default               |
| ---------------------- | ------------------------------------------------------------- | --------------------- |
| `name = "CustomName"`  | Name of the generated struct                                  | `Partial{StructName}` |
| `row = PartialFooRow`  | Generate `impl From<PartialFooRow> for {Partial}` (see below) | none — not generated  |

## Field attribute — `#[partial_include(PartialInner)]`

Marks a field as an **embedded resource**: instead of wrapping the original
field type, the partial uses the embedded type's own partial,
`PartialInner`. Use it when a response embeds another resource (e.g. an address
that embeds a country).

An optional `with = <path>` names the row→field conversion function used in the
generated `From` instead of `Into` — for when the row and field types have no
`From` impl between them (e.g. a raw decimal column projected into a money
type): `#[partial_include(Pesa, with = crate::money::usd)]`.

## Field attribute — `#[partial_response(row_field = name)]`

Overrides which **row field** a partial field is read from in the generated
`From` (below), for when the response field name differs from the row field
name. Applies to leaf and embedded fields alike.

## Field attribute — `#[partial_response(skip)]`

Drops the field from the partial entirely — it appears in neither the generated
`Partial{Struct}` nor the `From`. Use it for fields of the full response that
shouldn't be part of the (projectable) partial response at all. (A field you
*do* want in the partial but that can't be mapped from the row isn't a `skip`
case — omit `row =` and write the `From` by hand instead.) Cannot be combined
with `row_field` or `#[partial_include(...)]`.

# Generated `From<PartialRow>`

When the struct sets `row = PartialFooRow`, the derive also emits a conversion
from that partial row into the partial response:

```rust,ignore
impl From<PartialFooRow> for PartialFoo {
    fn from(row: PartialFooRow) -> Self { /* field-by-field */ }
}
```

This replaces the hand-written `From<PartialRow>` that maps a pgkit partial row
into the partial response. Each field is built as:

- **leaf field** → `field: row.field` — a direct move. The pgkit partial row and
  this partial response use the same shape (non-null column → `Option<T>`,
  nullable → `Option<Option<T>>`), so the move type-checks as-is.
- **embedded field** (`#[partial_include(PInner)]`) → converted via `Into` over
  its `Option` depth: `row.field.map(Into::into)` (or `.map(|x| x.map(Into::into))`
  when the field was itself `Option`). With `with = path`, `path` replaces
  `Into::into` in the same position.

The source field defaults to the response field's name; override it per field
with `#[partial_response(row_field = other_name)]`.

**You must give the macro a convertible row.** For every response field, the
`row` type must have a field of that name (or the override) whose value moves
(leaf) or whose inner type is `Into` the embedded partial (embedded). If it
doesn't, it's an ordinary compile error — provide a row the macro can convert,
or omit `row =` and write the `From` by hand. (Note: unlike `ProjectionField`,
this derive does **not** read `#[column]` — it works with struct fields, not
column variants.)

# Example

```rust,ignore
use http_kit_macros::PartialResponse;
use serde::Serialize;

#[derive(Serialize, PartialResponse)]
#[serde(rename_all = "camelCase")]
pub struct CountryHttpResponse {
    pub id: i32,
    pub name: String,
    pub native: Option<String>, // nullable column
}
```

generates:

```rust,ignore
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct PartialCountryHttpResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native: Option<Option<String>>,
}
```

so a value with only some fields set serializes to only those fields:

```rust,ignore
let partial = PartialCountryHttpResponse {
    id: Some(1),
    name: Some("Uganda".to_string()),
    native: None, // not projected
};
// serde_json::to_string(&partial) == r#"{"id":1,"name":"Uganda"}"#
// `native` is skipped because it is `None`.
```

## Embedding a related resource

```rust,ignore
#[derive(Serialize, PartialResponse)]
pub struct AddressHttpResponse {
    pub id: i32,

    #[partial_include(PartialCountryHttpResponse)]
    pub country: CountryHttpResponse,
}
```

generates:

```rust,ignore
#[derive(Debug, Default, Serialize)]
#[allow(dead_code)]
pub struct PartialAddressHttpResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<PartialCountryHttpResponse>,
}
```

## Renaming the generated struct

```rust,ignore
#[derive(Serialize, PartialResponse)]
#[partial_response(name = "CountrySummary")]
pub struct CountryHttpResponse { /* … */ }
// generates `struct CountrySummary { … }` instead of `PartialCountryHttpResponse`
```

# What it does **not** generate

- **No conversion.** It does not create any `From` impl. You convert your data
  into the partial yourself — typically `impl From<PartialFooRow> for PartialFoo`
  from a pgkit partial row.
- **No `ToSchema` / `Deserialize` / `Clone`.** The partial is a serialize-only
  output type; the original struct carries the OpenAPI schema and any other
  derives.
- **Field-level `#[serde(...)]` attributes are not propagated** — only the
  container-level ones are. The sole field attribute added is the generated
  `skip_serializing_if`.

# Constraints

- Only **structs with named fields** are supported.
- **No generics** (lifetime, type, or const parameters). Derive on a concrete
  type, or write the partial by hand. (Borrowed fields like `&'a str` must
  become owned, e.g. `String`.)
