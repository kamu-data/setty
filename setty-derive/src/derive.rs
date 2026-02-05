use crate::attrs::*;
use proc_macro2::TokenStream;
use quote::quote;

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn derive_impl(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let derives = deduplicated_derives(attr)?;

    Ok(quote! {
        #[::std::prelude::v1::derive(#(#derives,)*)]
        #item
    })
}

/////////////////////////////////////////////////////////////////////////////////////////

fn deduplicated_derives(attr: TokenStream) -> syn::Result<Vec<syn::Path>> {
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

    #[allow(unused)]
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

    Ok(derives)
}

/////////////////////////////////////////////////////////////////////////////////////////
