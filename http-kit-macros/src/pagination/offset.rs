//! Expansion logic for the `#[pagination]` attribute macro (offset-based).
//!
//! Injects `page` and `page_size` fields onto the target struct.
//!
//! ## Configurable arguments
//!
//! | Argument | Type | Default | Description |
//! |---|---|---|---|
//! | `default_page_size = N` | `u32` | `20` | Default page size when omitted |
//! | `max_page_size = N` | `u32` | `100` | Maximum allowed page size; enforced by the `validate` attr |
//! | `skip(serde)` | — | off | Omit `#[serde(...)]` attrs and the per-struct default helper fns |
//! | `skip(validate)` | — | off | Omit `#[validate(range(...))]` on both fields |
//! | `skip(utoipa)` | — | off | Omit `#[param(...)]` on both fields |
//!
//! All arguments are optional and can be combined freely:
//!
//! ```text
//! #[pagination]
//! #[pagination(default_page_size = 25)]
//! #[pagination(max_page_size = 50)]
//! #[pagination(default_page_size = 25, max_page_size = 50)]
//! #[pagination(skip(utoipa), default_page_size = 10)]
//! #[pagination(skip(serde, validate), max_page_size = 200)]
//! ```

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemStruct, parse_macro_input};

use crate::args::{SkipArgs, parse_named_field};

const DEFAULT_PAGE: u32 = 1;
const DEFAULT_PAGE_SIZE: u32 = 20;
const MAX_PAGE_SIZE: u32 = 100;

struct OffsetArgs {
    skip: SkipArgs,
    default_page_size: u32,
    max_page_size: u32,
}

impl OffsetArgs {
    fn parse(attr: TokenStream) -> syn::Result<Self> {
        let mut skip = SkipArgs::default();
        let mut default_page_size = DEFAULT_PAGE_SIZE;
        let mut max_page_size = MAX_PAGE_SIZE;

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
                if meta.path.is_ident("default_page_size") {
                    let val: syn::LitInt = meta.value()?.parse()?;
                    default_page_size = val.base10_parse()?;
                    return Ok(());
                }
                if meta.path.is_ident("max_page_size") {
                    let val: syn::LitInt = meta.value()?.parse()?;
                    max_page_size = val.base10_parse()?;
                    return Ok(());
                }
                Err(meta.error(
                    "unknown argument; expected `skip(...)`, `default_page_size = N`, \
                     or `max_page_size = N`",
                ))
            });
            syn::parse::Parser::parse(parser, attr)?;
        }

        if default_page_size > max_page_size {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "`default_page_size` ({default_page_size}) must not exceed \
                     `max_page_size` ({max_page_size})"
                ),
            ));
        }

        Ok(OffsetArgs {
            skip,
            default_page_size,
            max_page_size,
        })
    }
}

pub(crate) fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let OffsetArgs {
        skip,
        default_page_size,
        max_page_size,
    } = match OffsetArgs::parse(attr) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut ast = parse_macro_input!(item as ItemStruct);

    let struct_name = &ast.ident;
    let default_page_fn = format_ident!("default_page_{}", struct_name);
    let default_page_size_fn = format_ident!("default_page_size_{}", struct_name);
    let default_page_fn_str = default_page_fn.to_string();
    let default_page_size_fn_str = default_page_size_fn.to_string();

    let page_size_validate_msg = format!("Page size must be between 1 and {max_page_size}");
    let page_size_doc = format!(
        "The number of items per page (defaults to {default_page_size}, max {max_page_size})."
    );

    // Build conditional attribute token fragments.
    let serde_page_attr = (!skip.serde).then(|| {
        quote! { #[serde(default = #default_page_fn_str)] }
    });
    let validate_page_attr = (!skip.validate).then(|| {
        quote! { #[validate(range(min = 1, message = "Page number must not be less than 1"))] }
    });
    let param_page_attr = (!skip.utoipa).then(|| {
        quote! { #[param(minimum = 1, default = #DEFAULT_PAGE)] }
    });

    let serde_page_size_attr = (!skip.serde).then(|| {
        quote! { #[serde(default = #default_page_size_fn_str)] }
    });
    let validate_page_size_attr = (!skip.validate).then(|| {
        quote! {
            #[validate(range(min = 1, max = #max_page_size, message = #page_size_validate_msg))]
        }
    });
    let param_page_size_attr = (!skip.utoipa).then(|| {
        quote! { #[param(minimum = 1, maximum = #max_page_size, default = #default_page_size)] }
    });

    if let syn::Fields::Named(ref mut fields) = ast.fields {
        fields.named.push(parse_named_field(quote! {
            #[doc = "The page number to retrieve (defaults to 1)."]
            #serde_page_attr
            #validate_page_attr
            #param_page_attr
            pub page: u32
        }));

        fields.named.push(parse_named_field(quote! {
            #[doc = #page_size_doc]
            #serde_page_size_attr
            #validate_page_size_attr
            #param_page_size_attr
            pub page_size: u32
        }));
    } else {
        return TokenStream::from(quote! {
            compile_error!("pagination macro only supports structs with named fields.");
        });
    }

    let helper_fns = (!skip.serde).then(|| {
        quote! {
            #[allow(non_snake_case)]
            fn #default_page_fn() -> u32 {
                #DEFAULT_PAGE
            }

            #[allow(non_snake_case)]
            fn #default_page_size_fn() -> u32 {
                #default_page_size
            }
        }
    });

    TokenStream::from(quote! {
        #helper_fns
        #ast
    })
}
