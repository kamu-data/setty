use crate::attrs::*;
use proc_macro2::TokenStream;
use quote::quote;

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn default_impl(mut input: syn::DeriveInput) -> syn::Result<TokenStream> {
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
                    let fname = quote::format_ident!("default_{}", field.ident.as_ref().unwrap());
                    quote! { Self::#fname() }
                } else {
                    quote! { ::std::default::Default::default() }
                };

                let fname = field.ident.as_ref().unwrap();

                defaults.push(quote! { #fname: #expr, });
            }

            let item_name = input.ident;
            Ok(quote! {
                impl ::std::default::Default for #item_name {
                    fn default() -> Self {
                        Self {
                            #(#defaults)*
                        }
                    }
                }
            })
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
            Ok(quote! {
                impl ::std::default::Default for #item_name {
                    fn default() -> Self {
                        Self:: #variant_name #variant_value
                    }
                }
            })
        }

        _ => Err(syn::Error::new_spanned(
            input,
            "#[derive(Default)] can only be applied to structs and enums",
        )),
    }
}
