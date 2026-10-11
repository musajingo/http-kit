//! Expansion logic for the `#[order_by(FieldsEnum)]` attribute macro.
//!
//! Injects two query-parameter fields onto the target struct:
//!
//! - `order_by: Option<FieldsEnum>` — which column to sort by. Absent
//!   (`None`) means "no preference"; the service layer is responsible
//!   for picking a fallback (typically the table primary key).
//! - `order_dir: ::restful::ordering::OrderByDirection` — direction (asc / desc).
//!
//! The `order_by` field is wrapped in `Option<_>` deliberately so that
//! `FieldsEnum` does **not** need a `Default` impl. A hardcoded default
//! variant collides with `?fields=` projections that don't include it,
//! producing confusing validation errors even though the client never
//! asked for that column.
//!
//! Consumer crates must depend on `restful` for `OrderByDirection`.
//! Ordering types are available without additional features.
//!
//! An optional `skip(serde)` argument suppresses the `#[serde(default)]`
//! attributes on both fields:
//!
//! ```ignore
//! #[order_by(MyOrderByFields)]                  // serde attrs emitted (default)
//! #[order_by(MyOrderByFields, skip(serde))]     // no serde attrs
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, Token, Type, parse_macro_input};

use crate::args::{SkipArgs, derives_into_params, parse_named_field};

/// Argument list for `#[order_by(...)]`: `Type [, skip(...)]`.
struct OrderByArgs {
    fields_type: Type,
    skip: SkipArgs,
}

impl syn::parse::Parse for OrderByArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fields_type: Type = input.parse()?;

        let skip = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let remaining: proc_macro2::TokenStream = input.parse()?;
            SkipArgs::parse2(remaining)?
        } else {
            SkipArgs::default()
        };

        Ok(OrderByArgs { fields_type, skip })
    }
}

pub(crate) fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    if attr.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "order_by macro expects a type of the fields that can be ordered by, e.g. `#[order_by(MyOrderByFields)]`",
        )
        .to_compile_error()
        .into();
    }

    let OrderByArgs { fields_type, skip } = match syn::parse::<OrderByArgs>(attr) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut ast = parse_macro_input!(item as ItemStruct);

    let serde_attr = (!skip.serde).then(|| quote! { #[serde(default)] });
    let utoipa_attr =
        (!skip.utoipa && derives_into_params(&ast)).then(|| quote! { #[param(inline)] });

    if let syn::Fields::Named(ref mut fields) = ast.fields {
        fields.named.push(parse_named_field(quote! {
            #[doc = "Field to order by (e.g., `name`, `created_at`). \
                     `None` means the caller did not specify a sort column; \
                     the service layer should fall back to the table primary key."]
            #serde_attr
            #utoipa_attr
            pub order_by: ::core::option::Option<#fields_type>
        }));

        fields.named.push(parse_named_field(quote! {
            #[doc = "Direction to order (e.g., `asc`, `desc`). Defaults to OrderByDirection's default value."]
            #serde_attr
            #utoipa_attr
            pub order_dir: ::restful::ordering::OrderByDirection
        }));
    } else {
        return syn::Error::new_spanned(
            &ast.fields,
            "The order_by macro currently only supports structs with named fields.",
        )
        .to_compile_error()
        .into();
    }

    quote! { #ast }.into()
}
