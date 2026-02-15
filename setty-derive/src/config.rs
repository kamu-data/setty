#![allow(unused)]

use crate::attrs::*;
use proc_macro2::TokenStream;
use quote::quote;

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn config_impl(mut input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let mut default_functions: Vec<proc_macro2::TokenStream> = Vec::new();

    let mut item_attrs_overrides = input.attrs;

    let serde_type = SerdeTypeOpts::parse(&item_attrs_overrides)?;

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
            #[serde(crate = "::setty::__internal::serde")]
        });

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
            #[schemars(crate = "::setty::__internal::schemars")]
        });
    }

    match &mut input.data {
        syn::Data::Struct(item) => {
            // Add Validate derive only for structs (enums are not supported by validator crate)
            #[cfg(feature = "derive-validate")]
            {
                add_derive(
                    &mut input.attrs,
                    syn::parse_quote!(::setty::__internal::validator::Validate),
                );
                input.attrs.push(syn::parse_quote! {
                    #[validate(crate = "::setty::__internal::validator")]
                });
            }

            if let Some(case) = fields_case() {
                input.attrs.push(syn::parse_quote! {
                    #[serde(rename_all = #case)]
                });
            }

            for field in &mut item.fields {
                let opts = ConfigFieldOpts::extract_from(field)?;
                let deprecated = DeprecationOpts::parse_from(field)?;

                #[cfg(feature = "derive-jsonschema")]
                if let Some(deprecated) = deprecated {
                    if let Some(since) = deprecated.since
                        && let Some(reason) = deprecated.reason
                    {
                        field.attrs.push(syn::parse_quote! {
                            #[schemars(extend("deprecation" = { "reason": #reason, "since": #since }))]
                        });
                    } else if let Some(reason) = deprecated.reason {
                        field.attrs.push(syn::parse_quote! {
                            #[schemars(extend("deprecation" = { "reason": #reason }))]
                        });
                    }
                }

                #[cfg(feature = "derive-jsonschema")]
                if let Some(combine) = opts.combine {
                    let combine = combine.to_str_lit();
                    field.attrs.push(syn::parse_quote! {
                        #[schemars(extend("combine" = #combine))]
                    });
                }

                if let Some(default) = opts.default {
                    let new_default_attr: syn::Attribute = if let Some(default_expr) = default {
                        let fname =
                            quote::format_ident!("default_{}", field.ident.as_ref().unwrap());
                        let path_str = syn::Lit::Str(syn::LitStr::new(
                            &format!("{}::{}", input.ident, fname),
                            opts.span,
                        ));
                        let rtype = &field.ty;

                        default_functions.push(quote! {
                            fn #fname() -> #rtype { #default_expr }
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

                // Add validation attribute if provided and feature is enabled
                #[cfg(feature = "derive-validate")]
                if let Some(validate) = opts.validate {
                    field.attrs.push(syn::parse_quote! {
                        #[validate(#validate)]
                    });
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
            if !unit_enum && serde_type.tag.is_none() {
                input.attrs.push(syn::parse_quote! {
                    #[serde(tag = "kind")]
                });
            }

            #[cfg(all(
                feature = "case-enums-any",
                any(feature = "derive-deserialize", feature = "derive-serialize")
            ))]
            {
                for variant in &mut item.variants {
                    use std::collections::BTreeSet;

                    let serde_variant = SerdeFieldOpts::parse(&variant.attrs)?;

                    let mut aliases = BTreeSet::new();
                    let name = serde_variant.rename.unwrap_or(variant.ident.to_string());
                    enum_variant_all_case_permutations(&name, &mut aliases);
                    for alias in &serde_variant.alias {
                        enum_variant_all_case_permutations(alias, &mut aliases);
                    }
                    aliases.remove(&name);

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

    Ok(quote! {
        #input

        impl #item_name {
            #(#default_functions)*
        }
    })
}
