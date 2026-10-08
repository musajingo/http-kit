//! Argument parsing for all macros in this crate.
//!
//! Each macro accepts an optional `skip(...)` list that suppresses specific
//! groups of generated attributes at the call-site:
//!
//! - `skip(serde)`    — omit `#[serde(...)]` attrs and the default-value helper fn
//! - `skip(validate)` — omit `#[validate(...)]` attrs
//! - `skip(utoipa)`   — omit `#[param(...)]` attrs
//!
//! Multiple targets can be combined: `skip(serde, validate)`.
//! An empty attribute list (no args) leaves all attributes enabled.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

#[derive(Default)]
pub(crate) struct SkipArgs {
    pub serde: bool,
    pub validate: bool,
    pub utoipa: bool,
}

impl SkipArgs {
    /// Parse the raw attribute token stream into a `SkipArgs`.
    ///
    /// Accepts either an empty stream (no skips) or `skip(ident, ...)`.
    /// Returns a `syn::Error` on unrecognized input.
    pub fn parse(attr: TokenStream) -> syn::Result<Self> {
        let mut skip = SkipArgs::default();

        if attr.is_empty() {
            return Ok(skip);
        }

        let parser = syn::meta::parser(|meta| {
            if meta.path.is_ident("skip") {
                meta.parse_nested_meta(|inner| {
                    if inner.path.is_ident("serde") {
                        skip.serde = true;
                        return Ok(());
                    }
                    if inner.path.is_ident("validate") {
                        skip.validate = true;
                        return Ok(());
                    }
                    if inner.path.is_ident("utoipa") {
                        skip.utoipa = true;
                        return Ok(());
                    }
                    Err(inner
                        .error("unknown skip target; expected `serde`, `validate`, or `utoipa`"))
                })
            } else {
                Err(meta.error("unknown argument; expected `skip(...)`"))
            }
        });

        syn::parse::Parser::parse(parser, attr)?;
        Ok(skip)
    }

    /// Parse from a `proc_macro2::TokenStream` (used when composing inside another parser).
    pub fn parse2(attr: proc_macro2::TokenStream) -> syn::Result<Self> {
        SkipArgs::parse(attr.into())
    }
}

/// Parse a named field from a token stream by wrapping it in a temporary struct.
///
/// `syn::Field` does not implement `Parse` directly, so this is the standard
/// workaround: emit `struct __Tmp { <tokens> }` and extract the single field.
pub(crate) fn parse_named_field(tokens: TokenStream2) -> syn::Field {
    let item: syn::ItemStruct = syn::parse2(quote! { struct __HttpKitTmp { #tokens } })
        .expect("field token stream is always a valid named field");
    item.fields
        .into_iter()
        .next()
        .expect("struct has exactly one field")
}

/// Whether the struct's `#[derive(...)]` list mentions `IntoParams`.
///
/// Used to gate emission of `#[param(...)]` attrs, which utoipa only accepts
/// on structs that derive `IntoParams`.
pub(crate) fn derives_into_params(ast: &syn::ItemStruct) -> bool {
    ast.attrs
        .iter()
        .filter(|a| a.path().is_ident("derive"))
        .any(|a| {
            let mut found = false;
            let _ = a.parse_nested_meta(|meta| {
                if meta.path.segments.last().map(|s| s.ident == "IntoParams") == Some(true) {
                    found = true;
                }
                Ok(())
            });
            found
        })
}
