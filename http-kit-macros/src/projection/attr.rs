//! Expansion logic for the `#[projection(ResponseField)]` attribute macro.
//!
//! Injects a single query-parameter field onto the target struct:
//!
//! - `fields: Option<::http_kit::projection::Projection<ResponseField>>` — the
//!   field-selection string, parsed and validated by the projection engine.
//!
//! By default the field carries a `#[validate(nested, required(...))]` attribute
//! so that the validator rejects requests that omit the `fields` query parameter.
//! Pass `skip(validate)` to suppress it, or `skip(utoipa)` to suppress the
//! `#[param(...)]` attribute and schema function:
//!
//! ```ignore
//! #[projection(MyResponseField)]                       // validate + utoipa attrs (default)
//! #[projection(MyResponseField, skip(validate))]       // no validate attr
//! #[projection(MyResponseField, skip(utoipa))]         // no #[param(...)] / schema fn
//! ```

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemStruct, Token, Type, parse_macro_input};

use crate::args::{SkipArgs, derives_into_params, parse_named_field};

/// Argument list for `#[projection(...)]`: `Type [, skip(...)]`.
struct ProjectionArgs {
    response_field: Type,
    skip: SkipArgs,
}

impl syn::parse::Parse for ProjectionArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let response_field: Type = input.parse()?;

        let skip = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let remaining: proc_macro2::TokenStream = input.parse()?;
            SkipArgs::parse2(remaining)?
        } else {
            SkipArgs::default()
        };

        Ok(ProjectionArgs {
            response_field,
            skip,
        })
    }
}

pub(crate) fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    if attr.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "projection macro expects a field type, e.g. `#[projection(MyResponseField)]`",
        )
        .to_compile_error()
        .into();
    }

    let ProjectionArgs {
        response_field,
        skip,
    } = match syn::parse::<ProjectionArgs>(attr) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut ast = parse_macro_input!(item as ItemStruct);

    let validate_attr = (!skip.validate).then(|| {
        quote! {
            #[validate(nested, required(message = "Select fields to include in the response"))]
        }
    });

    let emit_utoipa = !skip.utoipa && derives_into_params(&ast);
    let schema_fn_ident = format_ident!("__projection_fields_schema_for_{}", ast.ident);
    // `#[param(required = true)]` cannot be used here: utoipa 5.5 short-circuits on
    // `schema_with` (into_params.rs — `.schema(..).build()` returns before it ever
    // calls `.required(..)`), so the attribute is silently ignored and the parameter
    // is advertised as optional because the field is `Option<_>`. The field must stay
    // `Option<_>` for the `required` validator to fire on an absent `?fields=`.
    // Applications must adjust the generated OpenAPI document if they need its
    // required flag to match validation.
    let param_attr = emit_utoipa.then(|| {
        quote! { #[param(schema_with = #schema_fn_ident)] }
    });

    if let syn::Fields::Named(ref mut fields) = ast.fields {
        fields.named.push(parse_named_field(quote! {
            #[doc = "Comma-separated list of fields to include in the response. (e.g. `?fields=firstName,lastName`)"]
            #param_attr
            #validate_attr
            pub fields: ::std::option::Option<::http_kit::projection::Projection<#response_field>>
        }));
    } else {
        return syn::Error::new_spanned(
            &ast.fields,
            "The projection macro currently only supports structs with named fields.",
        )
        .to_compile_error()
        .into();
    }

    let schema_fn_def = emit_utoipa.then(|| quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #schema_fn_ident() -> ::utoipa::openapi::RefOr<::utoipa::openapi::schema::Schema> {
            let valid_fields = <#response_field as ::http_kit::projection::ProjectionField>::all_field_names()
                .iter()
                .map(|f| ::std::format!("- {}", f))
                .collect::<::std::vec::Vec<_>>()
                .join("\n");

            let description = ::std::format!(
                "Select fields to include in the response.\n\n\
                 - `?fields=id,name` — just these fields\n\
                 - `?fields=all` — all fields\n\
                 - `?fields=-createdAt,-updatedAt,-deletedAt` — all fields except the timestamp fields\n\
                 - `?fields=all,-createdAt,-updatedAt,-deletedAt` — all fields except the timestamp fields\n\
                 - `?fields=id,name,country.iso2` — get `id` and `name`, plus iso2 from the embedded `country`\n\
                 - `?fields=all,country.all` — all fields, and all fields from the embedded `country`\n\n\
                 **Valid fields:**\n{}",
                valid_fields
            );

            ::utoipa::openapi::ObjectBuilder::new()
                .schema_type(::utoipa::openapi::schema::Type::String)
                .description(::core::option::Option::Some(description))
                .build()
                .into()
        }
    });

    quote! {
        #ast
        #schema_fn_def
    }
    .into()
}
