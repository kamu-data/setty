use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::spanned::Spanned;

/////////////////////////////////////////////////////////////////////////////////////////

#[proc_macro_derive(Config, attributes(config))]
pub fn derive_config(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match config_impl(input) {
        Ok(output) => output,
        Err(err) => err.to_compile_error().into(),
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[proc_macro_attribute]
pub fn __erase(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    TokenStream::new()
}

/////////////////////////////////////////////////////////////////////////////////////////

fn config_impl(mut input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let mut default_functions = Vec::new();

    #[cfg(feature = "derive-debug")]
    input.attrs.push(syn::parse_quote! {
        #[derive(Debug)]
    });

    #[cfg(feature = "derive-eq")]
    input.attrs.push(syn::parse_quote! {
        #[derive(PartialEq, Eq)]
    });

    #[cfg(feature = "derive-deserialize")]
    {
        input.attrs.push(syn::parse_quote! {
            #[derive(::serde::Deserialize)]
        });
        input.attrs.push(syn::parse_quote! {
            #[serde(deny_unknown_fields, rename_all = "camelCase")]
        });
    }

    #[cfg(feature = "derive-serialize")]
    {
        input.attrs.push(syn::parse_quote! {
            #[::serde_with::skip_serializing_none]
        });
        input.attrs.push(syn::parse_quote! {
            #[derive(::serde::Serialize)]
        });
    }

    #[cfg(feature = "derive-jsonschema")]
    input.attrs.push(syn::parse_quote! {
        #[derive(::schemars::JsonSchema)]
    });

    match &mut input.data {
        syn::Data::Struct(item) => {
            for field in &mut item.fields {
                let opts = ConfigFieldOpts::extract_from(&mut field.attrs)?;

                if !opts.required.unwrap_or_default() {
                    let new_default_attr = if let Some(expr) = opts.default {
                        let fname =
                            quote::format_ident!("__default_{}", field.ident.as_ref().unwrap());
                        let path_str = syn::Lit::Str(syn::LitStr::new(
                            &format!("{}::{}", input.ident, fname),
                            opts.span,
                        ));

                        default_functions.push(quote! {
                            fn #fname() -> String {
                                #expr.into()
                            }
                        });

                        syn::parse_quote! {
                            #[serde(default = #path_str)]
                        }
                    } else {
                        syn::parse_quote!(#[serde(default)])
                    };

                    field.attrs.push(new_default_attr);
                }
            }
        }

        syn::Data::Enum(item) => {
            let unit_enum = item
                .variants
                .iter()
                .all(|v| matches!(v.fields, syn::Fields::Unit));

            if !unit_enum {
                input.attrs.push(syn::parse_quote! {
                    #[serde(tag = "kind")]
                });
            }
        }

        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "#[derive(Config)] can only be applied to structs and enums",
            ));
        }
    }

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

struct ConfigFieldOpts {
    required: Option<bool>,
    default: Option<syn::Expr>,
    span: Span,
}

impl ConfigFieldOpts {
    fn new(span: Span) -> Self {
        Self {
            required: None,
            default: None,
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
            if self.default.is_some() {
                return Err(syn::Error::new(
                    other.span,
                    "`default` specified more than once",
                ));
            }
            self.default = other.default;
        }

        if self.required == Some(true) && self.default.is_some() {
            return Err(syn::Error::new(
                other.span,
                "can't be both `required` and specify a `default`",
            ));
        }

        Ok(())
    }

    fn extract_from(attrs: &mut Vec<syn::Attribute>) -> syn::Result<ConfigFieldOpts> {
        let mut opts = ConfigFieldOpts::new(proc_macro2::Span::call_site());

        for attr in attrs.iter() {
            if attr.path().is_ident("config") {
                let more_opts = Self::parse_from(attr)?;
                opts.merge(more_opts)?;
            }
        }

        attrs.retain(|attr| !attr.path().is_ident("config"));

        Ok(opts)
    }

    fn parse_from(attr: &syn::Attribute) -> syn::Result<ConfigFieldOpts> {
        let mut opts = ConfigFieldOpts::new(attr.span());

        attr.parse_args_with(|input: syn::parse::ParseStream| {
            while !input.is_empty() {
                let ident: syn::Ident = input.parse()?;

                match ident.to_string().as_str() {
                    "required" => {
                        if opts.required.is_some() {
                            return Err(syn::Error::new(
                                attr.span(),
                                "`required` specified more than once",
                            ));
                        }
                        opts.required = Some(true);
                    }

                    "default" => {
                        if opts.required.is_some() {
                            return Err(syn::Error::new(
                                attr.span(),
                                "`default` specified more than once",
                            ));
                        }
                        input.parse::<syn::Token![=]>()?;
                        let expr: syn::Expr = input.parse()?;
                        opts.default = Some(expr);
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
