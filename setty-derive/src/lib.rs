#![allow(unused)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::spanned::Spanned;

/////////////////////////////////////////////////////////////////////////////////////////

#[proc_macro_derive(Config, attributes(config, serde, schemars))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match config_impl(input) {
        Ok(output) => output,
        Err(err) => err.to_compile_error().into(),
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Special version of `#[derive(Default)]` that recognizes `#[config(default = $expr)]` attributes
#[proc_macro_derive(Default, attributes(config, default))]
pub fn derive_default(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match default_impl(input) {
        Ok(output) => output,
        Err(err) => err.to_compile_error().into(),
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Special version of `#[derive(..)]` macro. Works just like the standard one, except it
/// will de-duplicate the derives expanded from [`Config`] and explicit ones.
///
/// Thus declaration such as `#[setty::derive(Config, Clone, serde::Deserialize)]` will
/// always derive `Clone` and `Deserialize` even if those are not configured via top-level features,
/// and will not duplicate implementations if those features were enabled.
#[proc_macro_attribute]
pub fn derive(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    let mut derives = Vec::new();

    let args: syn::Attribute = syn::parse_quote!(#[derive(#attr)]);

    args.parse_args_with(|input: syn::parse::ParseStream| {
        while !input.is_empty() {
            let p: syn::Path = input.parse()?;
            derives.push(p);

            let _ = input.parse::<syn::Token![,]>();
        }
        Ok(())
    })
    .unwrap();

    let derives_config = derives.iter().any(|p| path_matches(p, "setty::Config"));

    derives.retain(|p| {
        if derives_config {
            #[cfg(feature = "derive-clone")]
            if path_matches(p, "std::clone::Clone") {
                return false;
            }

            #[cfg(feature = "derive-debug")]
            if path_matches(p, "std::fmt::Debug") {
                return false;
            }

            #[cfg(feature = "derive-partial-eq")]
            if path_matches(p, "std::cmp::PartialEq") {
                return false;
            }

            #[cfg(feature = "derive-eq")]
            if path_matches(p, "std::cmp::Eq") {
                return false;
            }
        }

        true
    });

    TokenStream::from(quote! {
        #[::std::prelude::v1::derive(#(#derives,)*)]
        #item
    })
}

/////////////////////////////////////////////////////////////////////////////////////////

#[proc_macro_attribute]
pub fn __erase(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    TokenStream::new()
}

/////////////////////////////////////////////////////////////////////////////////////////

fn config_impl(mut input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let mut default_functions: Vec<proc_macro2::TokenStream> = Vec::new();

    let mut item_attrs_overrides = input.attrs;

    input.attrs = Vec::new();

    let add_derive = |attrs: &mut Vec<syn::Attribute>, derive: syn::Path| {
        attrs.push(syn::parse_quote! { #[derive(#derive)] });
    };

    #[cfg(feature = "derive-clone")]
    add_derive(&mut input.attrs, syn::parse_quote!(Clone));

    #[cfg(feature = "derive-debug")]
    add_derive(&mut input.attrs, syn::parse_quote!(Debug));

    #[cfg(feature = "derive-partial-eq")]
    add_derive(&mut input.attrs, syn::parse_quote!(PartialEq));

    #[cfg(feature = "derive-eq")]
    add_derive(&mut input.attrs, syn::parse_quote!(Eq));

    #[cfg(feature = "derive-deserialize")]
    {
        add_derive(
            &mut input.attrs,
            syn::parse_quote!(::setty::__internal::serde::Deserialize),
        );

        input.attrs.push(syn::parse_quote! {
            #[serde(deny_unknown_fields)]
        });
    }

    #[cfg(feature = "derive-serialize")]
    {
        input.attrs.push(syn::parse_quote! {
            #[::setty::__internal::serde_with::skip_serializing_none]
        });
        add_derive(
            &mut input.attrs,
            syn::parse_quote!(::setty::__internal::serde::Serialize),
        );
    }

    #[cfg(feature = "derive-jsonschema")]
    {
        add_derive(
            &mut input.attrs,
            syn::parse_quote!(::setty::__internal::schemars::JsonSchema),
        );
        input.attrs.push(syn::parse_quote! {
            #[schemars(crate = "setty::__internal::schemars")]
        });
    }

    match &mut input.data {
        syn::Data::Struct(item) => {
            if let Some(case) = fields_case() {
                input.attrs.push(syn::parse_quote! {
                    #[serde(rename_all = #case)]
                });
            }

            for field in &mut item.fields {
                let opts = ConfigFieldOpts::extract_from(&mut field.attrs)?;

                if !opts.required.unwrap_or_default() {
                    let new_default_attr: syn::Attribute =
                        if opts.default.is_some() || opts.default_parse.is_some() {
                            let expr = if let Some(expr) = opts.default {
                                match &expr {
                                    syn::Expr::Lit(syn::ExprLit {
                                        lit: syn::Lit::Int(_),
                                        attrs: _,
                                    }) => quote! { #expr },
                                    _ => quote! { #expr.into() },
                                }
                            } else if let Some(lit) = opts.default_parse {
                                quote!( #lit.parse().unwrap() )
                            } else {
                                unreachable!()
                            };

                            let fname =
                                quote::format_ident!("__default_{}", field.ident.as_ref().unwrap());
                            let path_str = syn::Lit::Str(syn::LitStr::new(
                                &format!("{}::{}", input.ident, fname),
                                opts.span,
                            ));
                            let rtype = &field.ty;

                            default_functions.push(quote! {
                                fn #fname() -> #rtype { #expr }
                            });

                            syn::parse_quote! {
                                #[serde(default = #path_str)]
                            }
                        } else {
                            syn::parse_quote!(#[serde(default)])
                        };

                    #[cfg(feature = "derive-deserialize")]
                    field.attrs.push(new_default_attr);
                }
            }
        }

        syn::Data::Enum(item) => {
            if let Some(case) = variants_case() {
                input.attrs.push(syn::parse_quote! {
                    #[serde(rename_all = #case)]
                });
            }

            let unit_enum = item
                .variants
                .iter()
                .all(|v| matches!(v.fields, syn::Fields::Unit));

            #[cfg(any(feature = "derive-deserialize", feature = "derive-serialize"))]
            if !unit_enum {
                let serde =
                    if let Some(overrides) = extract_serde_overrides(&mut item_attrs_overrides) {
                        overrides
                    } else {
                        syn::parse_quote! {
                            #[serde(tag = "kind")]
                        }
                    };

                input.attrs.push(serde);
            }

            #[cfg(all(
                feature = "case-enums-any",
                any(feature = "derive-deserialize", feature = "derive-serialize")
            ))]
            {
                for variant in &mut item.variants {
                    let name = variant.ident.to_string();
                    let aliases = case_permutations(&name);
                    for alias in aliases {
                        variant.attrs.push(syn::parse_quote! {
                            #[serde(alias = #alias)]
                        });
                    }
                }
            }
        }

        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "#[derive(Config)] can only be applied to structs and enums",
            ));
        }
    }

    input.attrs.extend(item_attrs_overrides);

    // NOTE: Since derive macros are additive (only emit new code) we add a special
    // `__erase` proc macro call to erase the emitted type and avoid having duplicate types.
    //  i.e. the emitted type will exist only long enough for newly emitted derive macros
    // to do their work.
    input.attrs.push(syn::parse_quote! { #[::setty::__erase] });

    let item_name = &input.ident;

    Ok(TokenStream::from(quote! {
        #input

        impl #item_name {
            #(#default_functions)*
        }
    }))
}

/////////////////////////////////////////////////////////////////////////////////////////

fn default_impl(mut input: syn::DeriveInput) -> syn::Result<TokenStream> {
    match &mut input.data {
        syn::Data::Struct(item) => {
            let mut defaults = Vec::new();

            for field in &mut item.fields {
                let opts = ConfigFieldOpts::extract_from(&mut field.attrs)?;

                if opts.required.unwrap_or_default() {
                    return Err(syn::Error::new_spanned(
                        input,
                        "Cannot derive Default for a struct with required fields",
                    ));
                }

                let expr = if opts.default.is_some() || opts.default_parse.is_some() {
                    let fname = quote::format_ident!("__default_{}", field.ident.as_ref().unwrap());
                    quote! { Self::#fname() }
                } else {
                    quote! { ::std::default::Default::default() }
                };

                let fname = field.ident.as_ref().unwrap();

                defaults.push(quote! { #fname: #expr, });
            }

            let item_name = input.ident;
            Ok(TokenStream::from(quote! {
                impl ::std::default::Default for #item_name {
                    fn default() -> Self {
                        Self {
                            #(#defaults)*
                        }
                    }
                }
            }))
        }

        syn::Data::Enum(item) => {
            let mut default = None;
            for variant in &item.variants {
                if variant.attrs.iter().any(is_default) {
                    default = Some(variant);
                }
            }

            let Some(default) = default else {
                return Err(syn::Error::new_spanned(
                    input,
                    "Tag the default variant with `#[default]`",
                ));
            };

            let variant_name = &default.ident;
            let variant_value = match default.fields {
                syn::Fields::Unit => quote! {},
                syn::Fields::Named(_) | syn::Fields::Unnamed(_) => {
                    quote! { (::std::default::Default::default()) }
                }
            };

            let item_name = input.ident;
            Ok(TokenStream::from(quote! {
                impl ::std::default::Default for #item_name {
                    fn default() -> Self {
                        Self:: #variant_name #variant_value
                    }
                }
            }))
        }

        _ => Err(syn::Error::new_spanned(
            input,
            "#[derive(Default)] can only be applied to structs and enums",
        )),
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

struct ConfigFieldOpts {
    required: Option<bool>,
    default: Option<syn::Expr>,
    default_parse: Option<syn::LitStr>,
    span: Span,
}

impl ConfigFieldOpts {
    fn new(span: Span) -> Self {
        Self {
            required: None,
            default: None,
            default_parse: None,
            span,
        }
    }

    fn merge(&mut self, other: Self) -> syn::Result<()> {
        self.span = other.span;

        if other.required.is_some() {
            if self.required.is_some() {
                return Err(syn::Error::new(
                    other.span,
                    "`required` specified more than once",
                ));
            }
            self.required = other.required;
        }

        if other.default.is_some() {
            if self.default.is_some() || self.default_parse.is_some() {
                return Err(syn::Error::new(
                    other.span,
                    "`default` specified more than once",
                ));
            }
            self.default = other.default;
        }

        if other.default_parse.is_some() {
            if self.default.is_some() || self.default_parse.is_some() {
                return Err(syn::Error::new(
                    other.span,
                    "`default` specified more than once",
                ));
            }
            self.default_parse = other.default_parse;
        }

        if self.required == Some(true) && self.default.is_some() {
            return Err(syn::Error::new(
                other.span,
                "can't be both `required` and specify a `default`",
            ));
        }

        Ok(())
    }

    fn extract_from(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut opts = Self::new(proc_macro2::Span::call_site());

        for attr in attrs.iter() {
            if attr.path().is_ident("config") {
                let more_opts = Self::parse_from(attr)?;
                opts.merge(more_opts)?;
            }
        }

        attrs.retain(|attr| !attr.path().is_ident("config"));

        Ok(opts)
    }

    fn parse_from(attr: &syn::Attribute) -> syn::Result<Self> {
        let mut opts = Self::new(attr.span());

        attr.parse_args_with(|input: syn::parse::ParseStream| {
            while !input.is_empty() {
                let ident: syn::Ident = input.parse()?;

                match ident.to_string().as_str() {
                    "required" => {
                        opts.required = Some(true);
                    }

                    "default" => {
                        input.parse::<syn::Token![=]>()?;
                        let expr: syn::Expr = input.parse()?;
                        opts.default = Some(expr);
                    }

                    "default_parse" => {
                        input.parse::<syn::Token![=]>()?;
                        let lit: syn::LitStr = input.parse()?;
                        opts.default_parse = Some(lit);
                    }

                    _ => {
                        return Err(syn::Error::new(
                            attr.span(),
                            format!("unknown config option `{}`", ident),
                        ));
                    }
                }

                // Optional trailing comma
                let _ = input.parse::<syn::Token![,]>();
            }

            Ok(())
        })?;

        Ok(opts)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

fn extract_serde_overrides(attrs: &mut Vec<syn::Attribute>) -> Option<syn::Attribute> {
    let mut ret = None;

    for attr in attrs.iter() {
        if attr.path().is_ident("serde") {
            ret = Some(attr.clone());
        }
    }

    if ret.is_some() {
        attrs.retain(|a| !a.path().is_ident("serde"));
    }

    ret
}

/////////////////////////////////////////////////////////////////////////////////////////

fn is_default(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("default")
}

/////////////////////////////////////////////////////////////////////////////////////////

// TODO: Consider performance of this
fn path_matches(p: &syn::Path, other: &str) -> bool {
    let other: syn::Path = syn::parse_str(other).unwrap();

    p.segments.last().unwrap().ident == other.segments.last().unwrap().ident
}

/////////////////////////////////////////////////////////////////////////////////////////

fn fields_case() -> Option<&'static str> {
    let case: Option<&'static str> = None;

    #[cfg(feature = "case-fields-lower")]
    let case = Some("lowercase");

    #[cfg(feature = "case-fields-pascal")]
    let case = Some("PascalCase");

    #[cfg(feature = "case-fields-camel")]
    let case = Some("camelCase");

    #[cfg(feature = "case-fields-kebab")]
    let case = Some("kebab-case");

    // Default for `--all-features`
    #[cfg(feature = "case-fields-snake")]
    let case = Some("snake_case");

    case
}

/////////////////////////////////////////////////////////////////////////////////////////

fn variants_case() -> Option<&'static str> {
    let case: Option<&'static str> = None;

    #[cfg(feature = "case-enums-lower")]
    let case = Some("lowercase");

    #[cfg(feature = "case-enums-camel")]
    let case = Some("camelCase");

    #[cfg(feature = "case-enums-kebab")]
    let case = Some("kebab-case");

    #[cfg(feature = "case-enums-snake")]
    let case = Some("snake_case");

    // Default for `--all-features`
    #[cfg(feature = "case-enums-pascal")]
    let case = Some("PascalCase");

    case
}

fn case_permutations(name: &str) -> std::collections::BTreeSet<String> {
    let mut ret = std::collections::BTreeSet::new();

    ret.insert(name.to_owned());
    ret.insert(name.to_lowercase());

    ret.insert(pascal_to_camel(name));

    ret
}

fn pascal_to_camel(s: &str) -> String {
    let mut chars = s.chars();

    match chars.next() {
        None => String::new(),
        Some(first) => {
            let mut result = String::new();
            result.extend(first.to_lowercase());
            result.extend(chars);
            result
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
