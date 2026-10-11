#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;

mod args;
mod ordering;
mod pagination;
mod partial_response;
mod projection;
mod projection_field;

#[doc = include_str!("docs/offset_pagination.md")]
#[proc_macro_attribute]
pub fn offset_pagination(attr: TokenStream, item: TokenStream) -> TokenStream {
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
