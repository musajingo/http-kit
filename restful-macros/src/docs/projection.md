Adds `fields: Option<restful::projection::Projection<Fields>>` to a query.
Enable `restful`'s `projection` feature and add direct Serde (with `derive`),
Valid8r, and Serde JSON dependencies for this example.

```rust,ignore
#[derive(restful_macros::ProjectionField)]
#[projection_field(skip(utoipa))]
struct Article { id: u32, title: String }

#[restful_macros::projection(ArticleField)]
#[derive(serde::Deserialize, valid8r::Validate)]
struct ListQuery {}

let query: ListQuery = serde_json::from_str(r#"{"fields":"id"}"#).unwrap();
valid8r::Validate::validate(&query).unwrap();
assert_eq!(query.fields.unwrap().fields(), &[ArticleField::Id]);
```

Default validation requires `fields` to be present and the projection to be
valid. Deserialization alone stores invalid projections without rejecting
them. `skip(validate)` omits both rules, leaving validation to your application.

When a derive path ends in `IntoParams`, the macro adds a schema helper and
parameter attribute; renamed imports are not recognized. `skip(utoipa)`
suppresses both. The OpenAPI parameter is described as optional because its
type is `Option`, even when validation requires it.
