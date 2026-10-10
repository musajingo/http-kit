Generates a `{Struct}Field` enum — the typed, validated set of fields a client
may request via the `?fields=` query parameter — plus every trait impl the
projection parser needs to turn that query string into the enum.

# Why this exists

This is the **request-parsing** half of sparse fieldsets. A client sends
`?fields=id,name,country.iso2`; this enum is what lets the backend turn that
string into a typed, validated `Projection<{Struct}Field>`, rejecting unknown
fields with a helpful error listing the valid ones.

It pairs with the `PartialResponse` derive (the
**serialization** half): `ProjectionField` says _which_ fields were requested,
`PartialResponse` carries _only those_ in the response body. The two are almost
always derived together on the same response struct.

# What it generates

For a struct `Foo`, it emits an enum **`FooField`** with:

- one **unit variant per leaf field**, named in PascalCase: `id` → `Id`,
  `created_at` → `CreatedAt`;
- one **tuple variant per embedded resource**, wrapping that resource's own
  field enum: `#[projection_field(BarField)] bar: Bar` → `Bar(BarField)`.

…and these impls on the enum:

- `ProjectionField` — `parse`, `all`, `all_field_names`, `resource_name`;
- `AsRef<str>` — the camelCase wire name (`CreatedAt` → `"createdAt"`,
  `Bar(..)` → `"bar.<inner>"`);
- `TryFrom<String>` and `TryFrom<&str>` — parse a single field token;
- `utoipa` `ToSchema` / `ComposeSchema` stubs (so a `Projection<FooField>`
  field satisfies utoipa's bounds).

The enum derives `Debug`, `Clone`, `Hash`, `PartialEq`, `Eq`, `Serialize`,
`Deserialize`.

# Field → variant and wire name

| Struct field                                               | Enum variant                        | Wire name (in `?fields=`) |
| ---------------------------------------------------------- | ----------------------------------- | ------------------------- |
| `id: i32`                                                  | `Id`                                | `id`                      |
| `created_at: DateTime`                                     | `CreatedAt`                         | `createdAt`               |
| `#[projection_field(CountryHttpResponseField)] country: …` | `Country(CountryHttpResponseField)` | `country.<field>`         |

Wire names are camelCase to match the JSON API. Embedded resources are addressed
with a dotted namespace taken from the **field name** (`country.iso2`).

# The `?fields=` grammar

A comma-separated list of tokens (whitespace around each is trimmed):

| Token       | Meaning                                                |
| ----------- | ------------------------------------------------------ |
| `field`     | include the top-level `field`                          |
| `-field`    | exclude `field` (implies "all other top-level fields") |
| `all`       | include every top-level field                          |
| `ns.field`  | include `field` of embedded resource `ns`              |
| `-ns.field` | exclude `field` of embedded resource `ns`              |
| `ns.all`    | include every field of embedded resource `ns`          |

Key validation rules (a violation makes parsing fail with a typed error):

- the input must be non-empty and every token must name a known field or `all`;
- the **top-level** resource must be referenced — you cannot select _only_
  embedded fields;
- within one namespace, `all` may be combined only with exclusions, never with
  explicit inclusions; inclusions and exclusions cannot be mixed;
- a namespace cannot resolve to zero fields.

# Arguments

## Struct attribute — `#[projection_field(...)]`

| Key                        | Effect                                                                                   | Default                |
| -------------------------- | ---------------------------------------------------------------------------------------- | ---------------------- |
| `name = "country"`         | The resource name surfaced in validation error messages                                  | lowercased struct name |
| `column = CountriesColumn` | Generate the `IntoColumnSelections` impl mapping fields to `CountriesColumn` (see below) | none — not generated   |

## Field attributes

| Attribute                                    | Effect                                                                                                                     |
| -------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `#[projection_field(InnerField)]`            | Mark the field as an **embedded resource**; the variant wraps `InnerField` and its fields are addressed as `field.<inner>` |
| `#[projection_field(column = Col::Variant)]` | Override the field→column mapping (ProjectionField-specific) — leaf fields only                                            |
| `#[column(Col::Variant)]`                    | Override the field→column mapping, **shared** with other derives — leaf fields only                                        |
| `#[projection_skip]`                         | The field is **not projectable** — it gets no variant and is rejected in `?fields=`                                        |

`#[projection_skip]` and `#[projection_field(...)]` on the same field are
mutually exclusive. A column override on a **nested** field is an error — an
embedded resource maps to no column.

# Generated `IntoColumnSelections`

When the struct sets `column = SomeColumn`, the derive implements
`IntoColumnSelections`, which `Projection::into_column_selections()` calls to turn
a parsed projection into pgkit column selection(s). The shape depends on whether
the resource embeds others:

**Leaf resource** (no embedded fields) → one `ColumnSelection`, plus a total
`From<{Struct}Field> for SomeColumn`:

```rust,ignore
// CountryHttpResponse { id, iso2, … }, column = CountriesColumn
impl IntoColumnSelections for CountryHttpResponseField {
    type Output = ColumnSelection<CountriesColumn>;
    // …maps each field via From, name-matched (Iso2 → CountriesColumn::Iso2)…
}
```

**Embedding resource** (has `#[projection_field(InnerField)]` fields) → a tuple:
the resource's own selection plus one `Option` selection per embed, in
field-declaration order (`None` when that embed wasn't projected, so it won't
trigger a join):

```rust,ignore
// AreaHttpResponse { id, name, country: CountryHttpResponse }, column = AreasColumn
impl IntoColumnSelections for AreaHttpResponseField {
    type Output = (
        ColumnSelection<AreasColumn>,             // own
        Option<ColumnSelection<CountriesColumn>>, // country
    );
    // …partitions fields by variant; own → AreasColumn, country.* → the embed…
}
// let (area, country) = projection.into_column_selections();
```

Each own field maps to the **same-named** column by default (`CreatedAt` →
`AreasColumn::CreatedAt`; a divergent name is a compile error). Override with
`#[projection_field(column = AreasColumn::Variant)]` or `#[column(AreasColumn::Variant)]`.
Each embed slot is the child's own `IntoColumnSelections::Output`, so nesting
composes to any depth. Same-type multi-embed works — each embedded field is its
own tuple element, partitioned by its own variant.

**Requires `pgkit`.** The generated impl references `ColumnSelection`, so a crate
using `column = …` must have `pgkit` available as a dependency.

# Example

```rust,ignore
use http_kit_macros::ProjectionField;

#[derive(ProjectionField)]
#[projection_field(name = "country")]
pub struct CountryHttpResponse {
    pub id: i32,
    pub iso2: String,
    pub created_at: DateTime<Utc>,
}
```

generates (abridged):

```rust,ignore
pub enum CountryHttpResponseField {
    Id,
    Iso2,
    CreatedAt,
}

impl ProjectionField for CountryHttpResponseField {
    fn parse(s: &str) -> Result<Self, ProjectionError> { /* "id" => Id, … */ }
    fn all() -> &'static [Self] { /* [Id, Iso2, CreatedAt] */ }
    fn all_field_names() -> &'static [&'static str] { /* ["id","iso2","createdAt"] */ }
    fn resource_name() -> &'static str { "country" }
}
// + AsRef<str>, TryFrom<String>, TryFrom<&str>, utoipa stubs
```

With an embedded resource:

```rust,ignore
#[derive(ProjectionField)]
#[projection_field(name = "area")]
pub struct AreaHttpResponse {
    pub id: i32,
    pub name: String,

    #[projection_field(CountryHttpResponseField)]
    pub country: CountryHttpResponse,
}
// → enum AreaHttpResponseField { Id, Name, Country(CountryHttpResponseField) }
// valid tokens: id, name, country.iso2, country.all, all, -name, …
```

# Using it end-to-end

`ProjectionField` only produces the enum; you consume it through
`http_kit::projection::Projection<T>`:

```rust,ignore
// 1. Inject `fields: Option<Projection<AreaHttpResponseField>>` into the query DTO:
#[http_kit_macros::projection(AreaHttpResponseField)]
#[derive(serde::Deserialize, valid8r::Validate)]
pub struct ListAreasQuery { /* other params */ }

// 2. In the service, turn the projection into per-table pgkit column selections:
let (area, country) = query.fields.expect("validated present").into_column_selections();
// area:    ColumnSelection<AreasColumn>
// country: Option<ColumnSelection<CountriesColumn>>  (None ⇒ don't join)

// 3. Project those columns from the DB, build the PartialResponse, return it.
```

# Constraints

- Only **structs with named fields** are supported.

# See also

- The `PartialResponse` derive — the serialization half; derive it on the same
  struct.
- The `projection` attribute macro — injects the
  `fields: Option<Projection<{Struct}Field>>` query parameter.
