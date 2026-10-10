//! Expansion logic for `#[derive(ProjectionField)]`.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::ext::IdentExt;
use syn::punctuated::Punctuated;
use syn::{
    Data, DeriveInput, Expr, ExprLit, Fields, Ident, Lit, Meta, Path, Token, Type, TypePath,
    parse_macro_input,
};

// ── Internal representation ──────────────────────────────────────────────────

struct LeafField {
    /// PascalCase variant identifier, e.g. `CreatedAt`.
    variant_ident: Ident,
    /// camelCase wire name, e.g. `"createdAt"`.
    wire_name: String,
    /// Explicit column path overriding the name-match, e.g. `CountriesColumn::FullName`.
    column_override: Option<Path>,
}

struct NestedField {
    /// PascalCase variant identifier, e.g. `Country`.
    variant_ident: Ident,
    /// snake_case field name, e.g. `country`.
    snake_name: String,
    /// camelCase namespace, e.g. `"country"`.
    namespace: String,
    /// The `*Field` enum type, e.g. `CountryField`.
    field_enum_type: Type,
    /// ALL-UPPER prefix for unique static names, e.g. `COUNTRY`.
    static_prefix: String,
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // --- struct-only constraint ---
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return syn::Error::new_spanned(
                    &input.ident,
                    "ProjectionField can only be derived for structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &input.ident,
                "ProjectionField can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let struct_ident = &input.ident;
    let enum_ident = format_ident!("{}Field", struct_ident);
    let vis = &input.vis;

    // --- resource name + optional backing column enum ---
    let (resource_name, column_ty, skip_utoipa) = match extract_struct_attrs(&input) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    // --- collect fields ---
    let mut leaf_fields: Vec<LeafField> = Vec::new();
    let mut nested_fields: Vec<NestedField> = Vec::new();

    for field in fields {
        match classify_field(field) {
            Ok(Some(Classification::Leaf(lf))) => leaf_fields.push(lf),
            Ok(Some(Classification::Nested(nf))) => nested_fields.push(*nf),
            Ok(None) => {} // projection_skip
            Err(e) => return e.to_compile_error().into(),
        }
    }

    let has_nested = !nested_fields.is_empty();

    // ── IntoColumnSelections (only when `column = …` was given) ─────────────────
    // The own field→column mapping is inlined in the partitioning `match` the
    // same way for every resource; embeds (if any) add a tuple element each.
    let column_impls = match &column_ty {
        None => quote! {},
        Some(column_ty) => {
            let own = format_ident!("__own");

            let own_arms = leaf_fields.iter().map(|lf| {
                let variant = &lf.variant_ident;
                let target = match &lf.column_override {
                    Some(path) => quote! { #path },
                    None => quote! { #column_ty::#variant },
                };
                quote! { #enum_ident::#variant => #own.push(#target) }
            });
            let nested_arms = nested_fields.iter().map(|nf| {
                let variant = &nf.variant_ident;
                let local = format_ident!("{}", nf.snake_name);
                quote! { #enum_ident::#variant(inner) => #local.push(inner) }
            });
            let nested_locals = nested_fields.iter().map(|nf| {
                let local = format_ident!("{}", nf.snake_name);
                let child = &nf.field_enum_type;
                quote! { let mut #local: ::std::vec::Vec<#child> = ::std::vec::Vec::new(); }
            });
            let nested_output_tys = nested_fields.iter().map(|nf| {
                let child = &nf.field_enum_type;
                quote! {
                    ::core::option::Option<
                        <#child as ::http_kit::projection::IntoColumnSelections>::Output
                    >
                }
            });
            let nested_output_vals = nested_fields.iter().map(|nf| {
                let local = format_ident!("{}", nf.snake_name);
                let child = &nf.field_enum_type;
                quote! {
                    (!#local.is_empty()).then(|| {
                        <#child as ::http_kit::projection::IntoColumnSelections>::into_column_selections(#local)
                    })
                }
            });

            // Leaf resources have no embeds: `Output` is the bare selection.
            // Embedding resources return a tuple of (own, one Option per embed).
            let (output_ty, output_val) = if nested_fields.is_empty() {
                (
                    quote! { ::pgkit::projection::ColumnSelection<#column_ty> },
                    quote! { ::pgkit::projection::ColumnSelection::new(#own) },
                )
            } else {
                (
                    quote! {
                        (
                            ::pgkit::projection::ColumnSelection<#column_ty>,
                            #(#nested_output_tys,)*
                        )
                    },
                    quote! {
                        (
                            ::pgkit::projection::ColumnSelection::new(#own),
                            #(#nested_output_vals,)*
                        )
                    },
                )
            };

            quote! {
                #[automatically_derived]
                impl ::http_kit::projection::IntoColumnSelections for #enum_ident {
                    type Output = #output_ty;
                    fn into_column_selections(fields: ::std::vec::Vec<Self>) -> Self::Output {
                        let mut #own: ::std::vec::Vec<#column_ty> = ::std::vec::Vec::new();
                        #(#nested_locals)*
                        for field in fields {
                            match field {
                                #(#own_arms,)*
                                #(#nested_arms,)*
                            }
                        }
                        #output_val
                    }
                }
            }
        }
    };

    // ── enum definition ───────────────────────────────────────────────────────

    let leaf_variant_defs = leaf_fields.iter().map(|lf| {
        let ident = &lf.variant_ident;
        quote! { #ident }
    });
    let nested_variant_defs = nested_fields.iter().map(|nf| {
        let ident = &nf.variant_ident;
        let ty = &nf.field_enum_type;
        quote! { #ident(#ty) }
    });

    // ── parse() ───────────────────────────────────────────────────────────────

    let leaf_parse_arms = leaf_fields.iter().map(|lf| {
        let wire = &lf.wire_name;
        let ident = &lf.variant_ident;
        quote! { #wire => ::std::result::Result::Ok(Self::#ident) }
    });

    let nested_parse_arms = nested_fields.iter().map(|nf| {
        let ns = &nf.namespace;
        let ident = &nf.variant_ident;
        let ty = &nf.field_enum_type;
        quote! {
            #ns => <#ty as ::http_kit::projection::ProjectionField>::parse(field)
                .map(Self::#ident)
                .map_err(|_| ::http_kit::projection::ProjectionError::InvalidField {
                    field: s.to_string(),
                    valid_fields: <#ty as ::http_kit::projection::ProjectionField>::all_field_names().to_vec(),
                })
        }
    });

    let parse_fallthrough = quote! {
        _ => ::std::result::Result::Err(
            ::http_kit::projection::ProjectionError::InvalidField {
                field: ::std::format!("{}.{s}", Self::resource_name()),
                valid_fields: Self::all_field_names().to_vec(),
            }
        )
    };

    let parse_method = if has_nested {
        quote! {
            fn parse(s: &str) -> ::std::result::Result<Self, ::http_kit::projection::ProjectionError> {
                if let ::std::option::Option::Some((ns, field)) = s.split_once('.') {
                    return match ns {
                        #(#nested_parse_arms,)*
                        _ => ::std::result::Result::Err(
                            ::http_kit::projection::ProjectionError::InvalidField {
                                field: s.to_string(),
                                valid_fields: Self::all_field_names().to_vec(),
                            }
                        ),
                    };
                }
                match s {
                    #(#leaf_parse_arms,)*
                    #parse_fallthrough,
                }
            }
        }
    } else {
        quote! {
            fn parse(s: &str) -> ::std::result::Result<Self, ::http_kit::projection::ProjectionError> {
                match s {
                    #(#leaf_parse_arms,)*
                    #parse_fallthrough,
                }
            }
        }
    };

    // ── all_field_names() ─────────────────────────────────────────────────────

    let leaf_wire_names = leaf_fields.iter().map(|lf| lf.wire_name.as_str());

    let all_field_names_method = if has_nested {
        let nested_pushes = nested_fields.iter().map(|nf| {
            let ns = &nf.namespace;
            let ty = &nf.field_enum_type;
            quote! {
                for name in <#ty as ::http_kit::projection::ProjectionField>::all_field_names() {
                    v.push(::std::boxed::Box::leak(
                        ::std::format!("{}.{}", #ns, name).into_boxed_str()
                    ));
                }
            }
        });
        quote! {
            fn all_field_names() -> &'static [&'static str] {
                static NAMES: ::std::sync::LazyLock<::std::boxed::Box<[&'static str]>> =
                    ::std::sync::LazyLock::new(|| {
                        let mut v: ::std::vec::Vec<&'static str> = ::std::vec![#(#leaf_wire_names,)*];
                        #(#nested_pushes)*
                        v.into_boxed_slice()
                    });
                &**NAMES
            }
        }
    } else {
        quote! {
            fn all_field_names() -> &'static [&'static str] {
                &[#(#leaf_wire_names,)*]
            }
        }
    };

    // ── all() ─────────────────────────────────────────────────────────────────

    let leaf_all_items = leaf_fields.iter().map(|lf| {
        let ident = &lf.variant_ident;
        // Use the concrete enum ident (not `Self`) because this token stream may
        // end up inside a `static` initializer where `Self` is not in scope.
        quote! { #enum_ident::#ident }
    });

    let all_method = if has_nested {
        let nested_all_pushes = nested_fields.iter().map(|nf| {
            let ident = &nf.variant_ident;
            let ty = &nf.field_enum_type;
            quote! {
                for f in <#ty as ::http_kit::projection::ProjectionField>::all() {
                    v.push(#enum_ident::#ident(f.clone()));
                }
            }
        });
        quote! {
            fn all() -> &'static [Self] {
                static ALL: ::std::sync::LazyLock<::std::boxed::Box<[#enum_ident]>> =
                    ::std::sync::LazyLock::new(|| {
                        let mut v: ::std::vec::Vec<#enum_ident> = ::std::vec![#(#leaf_all_items,)*];
                        #(#nested_all_pushes)*
                        v.into_boxed_slice()
                    });
                &**ALL
            }
        }
    } else {
        quote! {
            fn all() -> &'static [Self] {
                &[#(#leaf_all_items,)*]
            }
        }
    };

    // ── AsRef<str> ────────────────────────────────────────────────────────────

    let leaf_as_ref_arms = leaf_fields.iter().map(|lf| {
        let ident = &lf.variant_ident;
        let wire = &lf.wire_name;
        quote! { Self::#ident => #wire }
    });

    // Each nested variant gets its own uniquely-named static HashMap.
    let nested_as_ref_arms = nested_fields.iter().map(|nf| {
        let ident = &nf.variant_ident;
        let ty = &nf.field_enum_type;
        let ns = &nf.namespace;
        let static_name = format_ident!("{}_WIRE_NAMES", nf.static_prefix);
        quote! {
            Self::#ident(f) => {
                static #static_name: ::std::sync::LazyLock<
                    ::std::collections::HashMap<#ty, &'static str>
                > = ::std::sync::LazyLock::new(|| {
                    <#ty as ::http_kit::projection::ProjectionField>::all()
                        .iter()
                        .map(|cf| {
                            let s: &'static str = ::std::boxed::Box::leak(
                                ::std::format!("{}.{}", #ns, cf.as_ref()).into_boxed_str()
                            );
                            (cf.clone(), s)
                        })
                        .collect()
                });
                #static_name.get(f).copied().unwrap_or_else(|| {
                    // Fallback: should never be reached for a well-formed enum.
                    unreachable!("ProjectionField::AsRef<str> — variant not found in wire names map")
                })
            }
        }
    });

    // ── Full output ───────────────────────────────────────────────────────────

    // utoipa-gen still checks `T: ComposeSchema + ToSchema` for the
    // `Projection<T>` field type even when the call site overrides the
    // schema via `#[param(schema_with = ...)]`. These stubs satisfy the
    // bound; their content is never rendered. `skip(utoipa)` suppresses them
    // for consumers without a utoipa dependency.
    let utoipa_impls = (!skip_utoipa).then(|| {
        quote! {
            impl ::utoipa::__dev::ComposeSchema for #enum_ident {
                fn compose(
                    _: ::std::vec::Vec<
                        ::utoipa::openapi::RefOr<::utoipa::openapi::schema::Schema>,
                    >,
                ) -> ::utoipa::openapi::RefOr<::utoipa::openapi::schema::Schema> {
                    ::utoipa::openapi::ObjectBuilder::new()
                        .schema_type(::utoipa::openapi::schema::Type::String)
                        .build()
                        .into()
                }
            }

            impl ::utoipa::ToSchema for #enum_ident {
                fn name() -> ::std::borrow::Cow<'static, str> {
                    ::std::borrow::Cow::Borrowed(::std::stringify!(#enum_ident))
                }
            }
        }
    });

    let output = quote! {
        #[derive(
            ::std::fmt::Debug,
            ::std::clone::Clone,
            ::std::hash::Hash,
            ::std::cmp::PartialEq,
            ::std::cmp::Eq,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        #vis enum #enum_ident {
            #(#leaf_variant_defs,)*
            #(#nested_variant_defs,)*
        }

        impl ::http_kit::projection::ProjectionField for #enum_ident {
            #parse_method
            #all_field_names_method
            #all_method

            fn resource_name() -> &'static str {
                #resource_name
            }
        }

        impl ::std::convert::AsRef<str> for #enum_ident {
            fn as_ref(&self) -> &str {
                match self {
                    #(#leaf_as_ref_arms,)*
                    #(#nested_as_ref_arms,)*
                }
            }
        }

        impl ::std::convert::TryFrom<::std::string::String> for #enum_ident {
            type Error = ::http_kit::projection::ProjectionError;

            fn try_from(s: ::std::string::String) -> ::std::result::Result<Self, Self::Error> {
                <Self as ::http_kit::projection::ProjectionField>::parse(&s)
            }
        }

        impl<'__a> ::std::convert::TryFrom<&'__a str> for #enum_ident {
            type Error = ::http_kit::projection::ProjectionError;

            fn try_from(s: &'__a str) -> ::std::result::Result<Self, Self::Error> {
                <Self as ::http_kit::projection::ProjectionField>::parse(s)
            }
        }

        #utoipa_impls

        #column_impls
    };

    output.into()
}

// ── Attribute parsing ─────────────────────────────────────────────────────────

/// Extract `name = "..."`, `column = <Column>`, and `skip(utoipa)` from the
/// struct's `#[projection_field(...)]`. `name` falls back to the lowercased
/// struct name; `column` is `None` unless given; `skip(utoipa)` suppresses the
/// generated utoipa impls for consumers without a utoipa dependency.
fn extract_struct_attrs(input: &DeriveInput) -> syn::Result<(String, Option<Path>, bool)> {
    let mut name: Option<String> = None;
    let mut column: Option<Path> = None;
    let mut skip_utoipa = false;

    for attr in &input.attrs {
        if !attr.path().is_ident("projection_field") {
            continue;
        }
        let list = attr.meta.require_list()?;
        list.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                let lit: Lit = meta.value()?.parse()?;
                if let Lit::Str(s) = lit {
                    name = Some(s.value());
                    Ok(())
                } else {
                    Err(meta.error("expected a string literal for `name`"))
                }
            } else if meta.path.is_ident("column") {
                column = Some(meta.value()?.parse::<Path>()?);
                Ok(())
            } else if meta.path.is_ident("skip") {
                meta.parse_nested_meta(|inner| {
                    if inner.path.is_ident("utoipa") {
                        skip_utoipa = true;
                        Ok(())
                    } else {
                        Err(inner.error("unknown skip target; expected `utoipa`"))
                    }
                })
            } else {
                Err(meta.error(
                    "unrecognised key in `#[projection_field(...)]`; expected `name`, `column`, or `skip(...)`",
                ))
            }
        })?;
    }

    let name = name.unwrap_or_else(|| input.ident.to_string().to_lowercase());
    Ok((name, column, skip_utoipa))
}

// ── Field classification ──────────────────────────────────────────────────────

enum Classification {
    Leaf(LeafField),
    Nested(Box<NestedField>),
}

fn classify_field(field: &syn::Field) -> syn::Result<Option<Classification>> {
    let mut is_skip = false;
    let mut nested_type: Option<Type> = None;
    let mut pf_column: Option<Path> = None; // #[projection_field(column = …)]
    let mut shared_column: Option<Path> = None; // #[column(…)]

    for attr in &field.attrs {
        if attr.path().is_ident("projection_skip") {
            attr.meta.require_path_only()?;
            is_skip = true;
        } else if attr.path().is_ident("projection_field") {
            let list = attr.meta.require_list()?;
            // Bare path → nested resource. Key form → `column = <Column::Variant>`.
            if let Ok(path) = list.parse_args::<Path>() {
                // A lone `column` is a `column = …` with the value left off, not a
                // nested type literally named `column`.
                if path.is_ident("column") {
                    return Err(syn::Error::new_spanned(
                        &path,
                        "`#[projection_field(column)]` is missing a value; write \
                         `#[projection_field(column = <Column::Variant>)]`",
                    ));
                }
                nested_type = Some(Type::Path(TypePath { qself: None, path }));
            } else {
                list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("column") {
                        pf_column = Some(meta.value()?.parse::<Path>()?);
                        Ok(())
                    } else {
                        Err(meta.error(
                            "unrecognised content in field `#[projection_field(...)]`; expected a \
                             nested field type or `column = <Column::Variant>`",
                        ))
                    }
                })?;
            }
        } else if attr.path().is_ident("column") {
            shared_column = Some(attr.meta.require_list()?.parse_args::<Path>()?);
        }
    }

    // Either form may set the column, but not both at once — each macro should
    // set its own, or `#[column(…)]` is the shared one both read.
    if pf_column.is_some() && shared_column.is_some() {
        return Err(syn::Error::new_spanned(
            field,
            "set the column with either `#[projection_field(column = ...)]` or `#[column(...)]`, not both",
        ));
    }
    let column_override = pf_column.or(shared_column);

    if is_skip && (nested_type.is_some() || column_override.is_some()) {
        return Err(syn::Error::new_spanned(
            field,
            "`#[projection_skip]` cannot be combined with `#[projection_field(...)]` or `#[column(...)]`",
        ));
    }

    if is_skip {
        return Ok(None);
    }

    if nested_type.is_some() && column_override.is_some() {
        return Err(syn::Error::new_spanned(
            field,
            "a nested resource field maps to no column; remove the `column = ...` / `#[column(...)]` override",
        ));
    }

    let field_ident = field.ident.as_ref().expect("named field has an identifier");
    // `unraw` so a raw identifier like `r#type` cases to `Type`/`type` instead
    // of panicking on the `r#` prefix.
    let snake = field_ident.unraw().to_string();
    let variant_ident = Ident::new(&snake_to_pascal(&snake), field_ident.span());
    // A field-level `#[serde(rename = "...")]` sets the response key, so it must
    // also be the `?fields=` selection token — otherwise selection and the
    // serialized response disagree. Fall back to the camelCased field name.
    let wire_name = serde_rename(field).unwrap_or_else(|| snake_to_camel(&snake));

    if let Some(field_enum_type) = nested_type {
        let namespace = wire_name.clone(); // camelCase of field name
        let static_prefix = snake.to_uppercase().replace('-', "_");
        Ok(Some(Classification::Nested(Box::new(NestedField {
            variant_ident,
            snake_name: snake,
            namespace,
            field_enum_type,
            static_prefix,
        }))))
    } else {
        Ok(Some(Classification::Leaf(LeafField {
            variant_ident,
            wire_name,
            column_override,
        })))
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// The string from a field's `#[serde(rename = "...")]`, if present.
fn serde_rename(field: &syn::Field) -> Option<String> {
    for attr in &field.attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        let Ok(metas) = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            continue;
        };
        for meta in metas {
            if let Meta::NameValue(nv) = meta
                && nv.path.is_ident("rename")
                && let Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) = nv.value
            {
                return Some(s.value());
            }
        }
    }
    None
}

/// `snake_case` → `PascalCase`
fn snake_to_pascal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut up = true;
    for ch in s.chars() {
        if ch == '_' {
            up = true;
        } else if up {
            out.push(ch.to_ascii_uppercase());
            up = false;
        } else {
            out.push(ch);
        }
    }
    out
}

/// `snake_case` → `camelCase`
fn snake_to_camel(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut up = false;
    for ch in s.chars() {
        if ch == '_' {
            up = true;
        } else if up {
            out.push(ch.to_ascii_uppercase());
            up = false;
        } else {
            out.push(ch);
        }
    }
    out
}
