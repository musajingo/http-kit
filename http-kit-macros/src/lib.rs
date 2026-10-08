//! Proc-macro helpers for HTTP query-parameter DTOs.
//!
//! Each macro injects fields onto a struct so that repetitive pagination and
//! ordering boilerplate never has to be written by hand. All macros are
//! **attribute macros** applied to structs with named fields.
//!
//! ## Quick start
//!
//! ```ignore
//! use serde::Deserialize;
//! use validator::Validate;
//! use utoipa::IntoParams;
//!
//! #[http_kit_macros::cursor_pagination]
//! #[http_kit_macros::ordering(MyOrderByFields)]
//! #[derive(Debug, Deserialize, Validate, IntoParams)]
//! #[into_params(parameter_in = Query)]
//! pub struct ListItemsQuery {
//!     pub search: Option<String>,
//!     // ↑ your fields  ↓ injected automatically
//!     // cursor: Option<String>
//!     // limit:  u32
//!     // order_by:  MyOrderByFields
//!     // order_dir: OrderByDirection
//! }
//! ```
//!
//! ## Configuring generated attributes with `skip(...)`
//!
//! By default every macro emits `#[serde(...)]`, `#[validate(...)]`, and
//! `#[param(...)]` (utoipa) attributes on each generated field. If your crate
//! does not use one of those libraries, pass a `skip(...)` list to suppress the
//! corresponding attributes at the call-site:
//!
//! | Argument | Effect |
//! |---|---|
//! | `skip(serde)` | Omit `#[serde(...)]` attrs and the per-struct default-value helper fns |
//! | `skip(validate)` | Omit `#[validate(range(...))]` attrs |
//! | `skip(utoipa)` | Omit `#[param(...)]` attrs |
//!
//! Multiple targets may be combined: `skip(validate, utoipa)`.
//!
//! ```ignore
//! // Serde only — no validator, no utoipa
//! #[http_kit_macros::cursor_pagination(skip(validate, utoipa))]
//! #[derive(Debug, serde::Deserialize)]
//! pub struct MinimalCursorQuery { /* your fields */ }
//!
//! // Offset pagination — skip utoipa only
//! #[http_kit_macros::pagination(skip(utoipa))]
//! #[derive(Debug, serde::Deserialize, validator::Validate)]
//! pub struct ListQuery { /* your fields */ }
//!
//! // Ordering — skip serde (manage deserialization yourself)
//! #[http_kit_macros::ordering(MyOrderByFields, skip(serde))]
//! #[derive(Debug)]
//! pub struct OrderOnlyQuery { /* your fields */ }
//! ```
//!
//! ## Default values
//!
//! When `serde` is **not** skipped the macros also emit private helper
//! functions (e.g. `default_limit_<StructName>`) that serde calls to fill in
//! missing query-parameter values. These functions are suppressed when
//! `skip(serde)` is used.
//!
//! ## Macro order
//!
//! Apply pagination / ordering macros **before** `#[derive(...)]`. Each macro
//! modifies the struct token stream in place; `#[derive]` then sees the
//! fully-expanded struct including all injected fields.

use proc_macro::TokenStream;

mod args;
mod ordering;
mod pagination;
mod partial_response;
mod projection;
mod projection_field;

#[doc = include_str!("docs/pagination.md")]
#[proc_macro_attribute]
pub fn pagination(attr: TokenStream, item: TokenStream) -> TokenStream {
    pagination::offset::expand(attr, item)
}

#[doc = include_str!("docs/cursor_pagination.md")]
#[proc_macro_attribute]
pub fn cursor_pagination(attr: TokenStream, item: TokenStream) -> TokenStream {
    pagination::cursor::expand(attr, item)
}

#[doc = include_str!("docs/order_by.md")]
#[proc_macro_attribute]
pub fn order_by(attr: TokenStream, item: TokenStream) -> TokenStream {
    ordering::attr::expand(attr, item)
}

#[doc = include_str!("docs/partial_response.md")]
#[proc_macro_derive(PartialResponse, attributes(partial_response, partial_include))]
pub fn derive_partial_response(input: TokenStream) -> TokenStream {
    partial_response::expand(input)
}

#[doc = include_str!("docs/projection_field.md")]
#[proc_macro_derive(ProjectionField, attributes(projection_field, projection_skip, column))]
pub fn derive_projection_field(input: TokenStream) -> TokenStream {
    projection_field::expand(input)
}

#[doc = include_str!("docs/projection.md")]
#[proc_macro_attribute]
pub fn projection(attr: TokenStream, item: TokenStream) -> TokenStream {
    projection::attr::expand(attr, item)
}
