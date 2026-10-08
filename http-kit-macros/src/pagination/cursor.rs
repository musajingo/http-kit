//! Expansion logic for the `#[cursor_pagination]` attribute macro.
//!
//! Injects `cursor` and `limit` fields onto the target struct.
//!
//! ## Configurable arguments
//!
//! | Argument | Type | Default | Description |
//! |---|---|---|---|
//! | `default_limit = N` | `u32` | `20` | Default `limit` when omitted |
//! | `max_limit = N` | `u32` | `100` | Maximum allowed `limit`; enforced by the `validate` attr |
//! | `skip(serde)` | — | off | Omit `#[serde(...)]` attrs and the default-limit helper fn |
//! | `skip(validate)` | — | off | Omit `#[validate(range(...))]` on `limit` |
//! | `skip(utoipa)` | — | off | Omit `#[param(...)]` on both fields |
//!
//! All arguments are optional and can be combined freely:
//!
//! ```text
//! #[cursor_pagination]
//! #[cursor_pagination(default_limit = 10)]
//! #[cursor_pagination(max_limit = 50)]
//! #[cursor_pagination(default_limit = 10, max_limit = 50)]
//! #[cursor_pagination(skip(validate, utoipa))]
//! #[cursor_pagination(skip(validate), default_limit = 10, max_limit = 50)]
//! ```

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemStruct, parse_macro_input};

use crate::args::{SkipArgs, parse_named_field};

const DEFAULT_LIMIT: u32 = 20;
const MAX_LIMIT: u32 = 100;

struct CursorArgs {
    skip: SkipArgs,
    default_limit: u32,
    max_limit: u32,
}

impl CursorArgs {
    fn parse(attr: TokenStream) -> syn::Result<Self> {
        let mut skip = SkipArgs::default();
        let mut default_limit = DEFAULT_LIMIT;
        let mut max_limit = MAX_LIMIT;

        if !attr.is_empty() {
            let parser = syn::meta::parser(|meta| {
                if meta.path.is_ident("skip") {
                    return meta.parse_nested_meta(|inner| {
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
                        Err(inner.error(
                            "unknown skip target; expected `serde`, `validate`, or `utoipa`",
                        ))
                    });
                }
                if meta.path.is_ident("default_limit") {
                    let val: syn::LitInt = meta.value()?.parse()?;
                    default_limit = val.base10_parse()?;
                    return Ok(());
                }
                if meta.path.is_ident("max_limit") {
                    let val: syn::LitInt = meta.value()?.parse()?;
                    max_limit = val.base10_parse()?;
                    return Ok(());
                }
                Err(meta.error(
                    "unknown argument; expected `skip(...)`, `default_limit = N`, or `max_limit = N`",
                ))
            });
            syn::parse::Parser::parse(parser, attr)?;
        }

        if default_limit > max_limit {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "`default_limit` ({default_limit}) must not exceed `max_limit` ({max_limit})"
                ),
            ));
        }

        Ok(CursorArgs {
            skip,
            default_limit,
            max_limit,
        })
    }
}

pub(crate) fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let CursorArgs {
        skip,
        default_limit,
        max_limit,
    } = match CursorArgs::parse(attr) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut ast = parse_macro_input!(item as ItemStruct);

    let struct_name = &ast.ident;
    let default_limit_fn = format_ident!("default_limit_{}", struct_name);
    let default_limit_fn_str = default_limit_fn.to_string();

    let validate_msg = format!("Limit must be between 1 and {max_limit}");
    let limit_doc = format!(
        "Maximum number of items to return (defaults to {default_limit}, max {max_limit})."
    );

    // Build conditional attribute token fragments.
    let serde_cursor_attr = (!skip.serde).then(|| quote! { #[serde(default)] });
    let param_cursor_attr = (!skip.utoipa).then(|| {
        quote! { #[param(example = "eyJjb2wiOiJjcmVhdGVkX2F0Ii4uLn0")] }
    });

    let serde_limit_attr = (!skip.serde).then(|| {
        quote! { #[serde(default = #default_limit_fn_str)] }
    });
    let validate_limit_attr = (!skip.validate).then(|| {
        quote! {
            #[validate(range(min = 1, max = #max_limit, message = #validate_msg))]
        }
    });
    let param_limit_attr = (!skip.utoipa).then(|| {
        quote! { #[param(minimum = 1, maximum = #max_limit, default = #default_limit)] }
    });

    if let syn::Fields::Named(ref mut fields) = ast.fields {
        fields.named.push(parse_named_field(quote! {
            #[doc = "Opaque cursor from previous response for keyset pagination."]
            #serde_cursor_attr
            #param_cursor_attr
            pub cursor: Option<String>
        }));

        fields.named.push(parse_named_field(quote! {
            #[doc = #limit_doc]
            #serde_limit_attr
            #validate_limit_attr
            #param_limit_attr
            pub limit: u32
        }));
    } else {
        return TokenStream::from(quote! {
            compile_error!("cursor_pagination macro only supports structs with named fields.");
        });
    }

    let helper_fn = (!skip.serde).then(|| {
        quote! {
            #[allow(non_snake_case)]
            fn #default_limit_fn() -> u32 {
                #default_limit
            }
        }
    });

    TokenStream::from(quote! {
        #helper_fn
        #ast
    })
}
