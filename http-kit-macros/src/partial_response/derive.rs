//! Expansion logic for `#[derive(PartialResponse)]`.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Data, DeriveInput, Field, Fields, GenericArgument, Ident, Lit, Meta, Path,
    PathArguments, Token, Type, TypePath, parse_macro_input, parse_quote,
};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Generics are not supported.
    if !input.generics.params.is_empty() {
        let mut combined: Option<syn::Error> = None;
        for param in &input.generics.params {
            let msg = match param {
                syn::GenericParam::Lifetime(_) => {
                    "PartialResponse cannot be derived on a type with lifetime parameters. \
                     Replace borrowed fields (e.g. `&'a str`) with their owned counterparts \
                     (`String`) before deriving."
                }
                syn::GenericParam::Type(_) => {
                    "PartialResponse cannot be derived on a generic type. Derive on a concrete \
                     type or write the `Partial*` companion by hand."
                }
                syn::GenericParam::Const(_) => {
                    "PartialResponse cannot be derived on a type with const generic \
                     parameters. Derive on a concrete instantiation instead, or write the \
                     `Partial*` companion by hand."
                }
            };
            let err = syn::Error::new_spanned(param, msg);
            match &mut combined {
                Some(acc) => acc.combine(err),
                slot => *slot = Some(err),
            }
        }
        return combined
            .expect("non-empty generics produces at least one error")
            .to_compile_error()
            .into();
    }

    // Only structs with named fields are supported.
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return syn::Error::new_spanned(
                    &input.ident,
                    "PartialResponse can only be derived for structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &input.ident,
                "PartialResponse can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let original_ident = &input.ident;
    let (partial_name, row_ty) = match extract_struct_attrs(&input) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };
    let partial_ident = Ident::new(&partial_name, original_ident.span());
    let vis = &input.vis;

    // Pass through every `#[serde(...)]` attribute from the original struct.
    let serde_attrs: Vec<&Attribute> = input
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("serde"))
        .collect();

    let mut partial_fields: Vec<TokenStream2> = Vec::with_capacity(fields.len());
    let mut infos: Vec<FieldInfo> = Vec::with_capacity(fields.len());
    for field in fields {
        match process_field(field) {
            Ok(Some((pf, info))) => {
                partial_fields.push(pf);
                infos.push(info);
            }
            Ok(None) => {} // #[partial_response(skip)]
            Err(e) => return e.to_compile_error().into(),
        }
    }

    // `From<row>` for the partial — only when `#[partial_response(row = …)]` is given.
    let from_partial_row = match &row_ty {
        Some(row_ty) => {
            let arms = infos.iter().map(|info| {
                let ident = &info.ident;
                let source = &info.source_ident;
                // Embedded fields are converted over their `Option` depth —
                // via `Into`, or the `with = …` function when given; leaf
                // fields move directly (same type as the row field).
                let convert = match &info.convert {
                    Some(path) => quote! { #path },
                    None => quote! { ::core::convert::Into::into },
                };
                let expr = match (info.is_nested, info.was_option) {
                    (true, true) => {
                        quote! { row.#source.map(|inner| inner.map(#convert)) }
                    }
                    (true, false) => quote! { row.#source.map(#convert) },
                    (false, _) => quote! { row.#source },
                };
                quote! { #ident: #expr }
            });
            quote! {
                #[automatically_derived]
                impl ::core::convert::From<#row_ty> for #partial_ident {
                    fn from(row: #row_ty) -> Self {
                        Self {
                            #(#arms,)*
                        }
                    }
                }
            }
        }
        None => quote! {},
    };

    let output = quote! {
        // `<T>PartialResponse` companion struct. Owns the response shape
        // with every field wrapped in an extra `Option`.
        #[derive(::std::fmt::Debug, ::std::default::Default, ::serde::Serialize)]
        #(#serde_attrs)*
        #[allow(dead_code)]
        #vis struct #partial_ident {
            #(#partial_fields,)*
        }

        #from_partial_row
    };

    output.into()
}

/// Per-field data needed to build the `From<row>` arms.
struct FieldInfo {
    /// The partial-response field name (target).
    ident: Ident,
    /// The row field to read from — the field name unless overridden with
    /// `#[partial_response(row_field = …)]`.
    source_ident: Ident,
    /// Whether the field is an embedded resource (`#[partial_include(...)]`).
    is_nested: bool,
    /// Whether the original field was itself `Option<_>` (controls embed depth).
    was_option: bool,
    /// Row→field conversion from `#[partial_include(T, with = path)]`; `Into`
    /// when absent.
    convert: Option<Path>,
}

// Returns the partial field token stream plus the data the `From` arm needs, or
// `None` when the field is `#[partial_response(skip)]`-ed out of the partial.
fn process_field(field: &Field) -> syn::Result<Option<(TokenStream2, FieldInfo)>> {
    let attrs = parse_field_partial_response(field)?;
    let partial_include = parse_partial_include(field)?;

    if attrs.skip {
        if attrs.row_field.is_some() || partial_include.is_some() {
            return Err(syn::Error::new_spanned(
                field,
                "`#[partial_response(skip)]` must be used on its own — not with `row_field` or `#[partial_include(...)]`",
            ));
        }
        return Ok(None);
    }

    let ident = field.ident.clone().expect("named field has an identifier");
    let vis = &field.vis;

    // Carry the field's own `#[serde(...)]` attributes (e.g. `rename = "type"`)
    // onto the partial so the projected response shape matches the full one. The
    // macro supplies its own `skip_serializing_if`, so a forwarded one would
    // duplicate it — strip any from the carried attributes.
    let serde_attrs = forwarded_serde_attrs(field)?;

    let original_ty = &field.ty;
    let (was_option, _inner_ty) = peel_option(original_ty);

    // If this is an embedded resource (annotated `#[partial_include(TheType)]`).
    let partial_field = if let Some((partial_inner, _)) = &partial_include {
        let partial_ty: Type = if was_option {
            parse_quote! { ::std::option::Option<::std::option::Option<#partial_inner>> }
        } else {
            parse_quote! { ::std::option::Option<#partial_inner> }
        };
        quote! {
            #(#serde_attrs)*
            #[serde(skip_serializing_if = "::std::option::Option::is_none")]
            #vis #ident: #partial_ty
        }
    } else {
        // Plain leaf: wrap in one extra `Option`.
        let partial_ty: Type = if was_option {
            let inner_ty = peel_option(original_ty).1;
            parse_quote! { ::std::option::Option<::std::option::Option<#inner_ty>> }
        } else {
            parse_quote! { ::std::option::Option<#original_ty> }
        };
        quote! {
            #(#serde_attrs)*
            #[serde(skip_serializing_if = "::std::option::Option::is_none")]
            #vis #ident: #partial_ty
        }
    };

    let source_ident = attrs.row_field.unwrap_or_else(|| ident.clone());
    let info = FieldInfo {
        ident,
        source_ident,
        is_nested: partial_include.is_some(),
        was_option,
        convert: partial_include.and_then(|(_, with)| with),
    };

    Ok(Some((partial_field, info)))
}

/// Extracts `name = "..."` and the optional `row = <PartialRow>` from the
/// struct's `#[partial_response(...)]`. `name` falls back to
/// `{Struct}PartialResponse`; `row` is `None` unless given.
fn extract_struct_attrs(input: &DeriveInput) -> syn::Result<(String, Option<Path>)> {
    let mut name: Option<String> = None;
    let mut row: Option<Path> = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("partial_response") {
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
            } else if meta.path.is_ident("row") {
                row = Some(meta.value()?.parse::<Path>()?);
                Ok(())
            } else {
                Err(meta.error(
                    "unrecognised key in `#[partial_response(...)]`; expected `name` or `row`",
                ))
            }
        })?;
    }

    let name = name.unwrap_or_else(|| format!("Partial{}", input.ident));
    Ok((name, row))
}

/// Rebuilds a field's `#[serde(...)]` attributes for the partial field, dropping
/// any `skip_serializing_if` entry (the macro emits its own). An attribute that
/// held only `skip_serializing_if` is dropped entirely. `serialize_with` / `with`
/// are rejected: the partial wraps the field in an extra `Option`, so a custom
/// serializer's signature would no longer match.
fn forwarded_serde_attrs(field: &Field) -> syn::Result<Vec<Attribute>> {
    let mut out = Vec::new();
    for attr in &field.attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        let metas = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in &metas {
            if meta.path().is_ident("serialize_with") || meta.path().is_ident("with") {
                return Err(syn::Error::new_spanned(
                    meta,
                    "`serde(serialize_with/with)` is not supported by PartialResponse: \
                     the partial wraps the field in an extra `Option`, so the custom \
                     serializer's signature no longer matches. Implement `Serialize` \
                     on the field's type instead, or skip the field.",
                ));
            }
        }
        let kept: Punctuated<Meta, Token![,]> = metas
            .into_iter()
            .filter(|m| !m.path().is_ident("skip_serializing_if"))
            .collect();
        if !kept.is_empty() {
            out.push(parse_quote! { #[serde(#kept)] });
        }
    }
    Ok(out)
}

/// Returns the explicit partial type from `#[partial_include(TheType)]` and the
/// optional row→field conversion from `#[partial_include(TheType, with = path)]`,
/// or `None`.
fn parse_partial_include(field: &Field) -> syn::Result<Option<(Type, Option<Path>)>> {
    for attr in &field.attrs {
        if attr.path().is_ident("partial_include") {
            let list = attr.meta.require_list()?;
            let args: PartialIncludeArgs = list.parse_args().map_err(|_| {
                syn::Error::new_spanned(
                    attr,
                    "`#[partial_include]` requires a type argument, e.g. \
                     `#[partial_include(PartialCountry)]`, optionally followed by \
                     `with = <conversion fn>`",
                )
            })?;
            let ty = Type::Path(TypePath {
                qself: None,
                path: args.ty,
            });
            return Ok(Some((ty, args.with)));
        }
    }
    Ok(None)
}

/// The arguments of `#[partial_include(TheType)]` /
/// `#[partial_include(TheType, with = path)]`.
struct PartialIncludeArgs {
    ty: Path,
    with: Option<Path>,
}

impl syn::parse::Parse for PartialIncludeArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty: Path = input.parse()?;
        let mut with = None;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let kw: Ident = input.parse()?;
            if kw != "with" {
                return Err(syn::Error::new_spanned(kw, "expected `with`"));
            }
            input.parse::<Token![=]>()?;
            with = Some(input.parse()?);
        }
        Ok(Self { ty, with })
    }
}

/// Field-level `#[partial_response(...)]` options.
#[derive(Default)]
struct FieldPartialResponse {
    /// `#[partial_response(skip)]` — drop the field from the partial entirely.
    skip: bool,
    /// `#[partial_response(row_field = <ident>)]` — read from this row field.
    row_field: Option<Ident>,
}

/// Reads `skip` and `row_field = …` from every field-level
/// `#[partial_response(...)]` (accumulated, so repeated attributes all apply).
fn parse_field_partial_response(field: &Field) -> syn::Result<FieldPartialResponse> {
    let mut out = FieldPartialResponse::default();
    for attr in &field.attrs {
        if !attr.path().is_ident("partial_response") {
            continue;
        }
        attr.meta.require_list()?.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                out.skip = true;
                Ok(())
            } else if meta.path.is_ident("row_field") {
                out.row_field = Some(meta.value()?.parse::<Ident>()?);
                Ok(())
            } else {
                Err(meta.error(
                    "unrecognised key in field `#[partial_response(...)]`; expected `skip` or `row_field`",
                ))
            }
        })?;
    }
    Ok(out)
}

/// Peel one `Option` layer if present.
///
/// Returns `(true, inner)` if `ty` is `Option<T>`, otherwise `(false, ty)`.
fn peel_option(ty: &Type) -> (bool, &Type) {
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return (false, ty);
    };
    let Some(last) = path.segments.last() else {
        return (false, ty);
    };
    if last.ident != "Option" {
        return (false, ty);
    }
    let PathArguments::AngleBracketed(args) = &last.arguments else {
        return (false, ty);
    };
    if args.args.len() != 1 {
        return (false, ty);
    }
    match args.args.first() {
        Some(GenericArgument::Type(inner)) => (true, inner),
        _ => (false, ty),
    }
}
