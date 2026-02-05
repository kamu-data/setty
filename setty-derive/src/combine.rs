use crate::attrs::*;
use proc_macro2::{Span, TokenStream};
use quote::quote;

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn combine_impl(input: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let item_name = &input.ident;
    let serde_type = SerdeTypeOpts::parse(&input.attrs)?;

    match &input.data {
        syn::Data::Struct(item) => {
            // if let Some(case) = fields_case() {
            //     input.attrs.push(syn::parse_quote! {
            //         #[serde(rename_all = #case)]
            //     });
            // }

            let mut matches = Vec::new();

            for field in &item.fields {
                let opts = ConfigFieldOpts::parse_from(field)?;
                let field_serde = SerdeFieldOpts::parse(&field.attrs)?;

                let (field_ty_raw, combine) = if let Some(combine) = opts.combine {
                    (&field.ty, combine)
                } else {
                    default_combine_for(&field.ty)
                };

                let aliases = field_aliases(field, &field_serde);

                let expr = match combine {
                    Combine::Keep => quote! {
                        if !lhs.contains_key(&k) {
                            lhs.insert(k, v);
                        }
                    },
                    Combine::Replace => quote! {
                        lhs.insert(k, v);
                    },
                    Combine::Merge => quote! {
                        // TODO: Consider key case permutations and aliases
                        if let Some(ll) = lhs.get_mut(&k) {
                            <#field_ty_raw as ::setty::combine::Combine>::merge(ll, v);
                        } else {
                            lhs.insert(k, v);
                        }
                    },
                };

                matches.push(quote! {
                    #(#aliases)|* => { #expr }
                });
            }

            Ok(quote! {
                impl ::setty::combine::Combine for #item_name {
                    fn merge(lhs: &mut ::setty::__internal::serde_json::Value, rhs: ::setty::__internal::serde_json::Value) {
                        let rhs = match rhs {
                            ::setty::__internal::serde_json::Value::Object(v) => v,
                            _ => {
                                *lhs = rhs;
                                return;
                            }
                        };
                        let Some(lhs) = lhs.as_object_mut() else {
                            *lhs = rhs.into();
                            return;
                        };


                        for (k, v) in rhs {
                            match k.as_str() {
                                #(#matches)*
                                // Fallback to `replace` for unknown values
                                _ => {
                                    lhs.insert(k, v);
                                }
                            }
                        }
                    }
                }
            })
        }

        // Unit enum
        syn::Data::Enum(item)
            if item
                .variants
                .iter()
                .all(|v| matches!(v.fields, syn::Fields::Unit)) =>
        {
            Ok(quote! {
                impl ::setty::combine::Combine for #item_name {
                    fn merge(lhs: &mut ::setty::__internal::serde_json::Value, rhs: ::setty::__internal::serde_json::Value) {
                        *lhs = rhs
                    }
                }
            })
        }

        // Tagged object enum
        syn::Data::Enum(item) => {
            let tag_field = serde_type.tag.unwrap_or("kind".to_string());
            let tag_field = syn::LitStr::new(&tag_field, Span::call_site());

            let mut tag_cmp = Vec::new();
            let mut matches = Vec::new();

            for var in &item.variants {
                let tag_aliases = enum_tag_aliases(var);

                let syn::Fields::Unnamed(fields) = &var.fields else {
                    unreachable!("Expected unnamed enum variant V(T)");
                };

                // TODO: Serde aliases
                assert_eq!(fields.unnamed.len(), 1);
                let var_ty = &fields.unnamed[0].ty;

                tag_cmp.push(quote! {
                    (#(#tag_aliases)|*, #(#tag_aliases)|*) => true,
                });

                matches.push(quote! {
                    #(#tag_aliases)|* => {
                        <#var_ty as ::setty::combine::Combine>::merge(lhs, rhs);
                    }
                });
            }

            Ok(quote! {
                impl ::setty::combine::Combine for #item_name {
                    fn merge(lhs: &mut ::setty::__internal::serde_json::Value, rhs: ::setty::__internal::serde_json::Value) {

                        let Some(lhs_tag) = lhs.get(#tag_field).and_then(|v| v.as_str()).map(|s| s.to_lowercase()) else {
                            *lhs = rhs;
                            return;
                        };
                        let Some(rhs_tag) = rhs.get(#tag_field).and_then(|v| v.as_str()).map(|s| s.to_lowercase()) else {
                            *lhs = rhs;
                            return;
                        };

                        // Compare permutations of tags for equivalence
                        let same_tag = match (lhs_tag.as_str(), rhs_tag.as_str()) {
                            #(#tag_cmp)*
                            _ => false
                        };

                        // Use `replace` upon tag mismatch
                        if !same_tag {
                            *lhs = rhs;
                            return;
                        }

                        match rhs_tag.as_str() {
                            #(#matches)*
                            // Fallback to `replace` for unknown variant
                            _ => {
                                *lhs = rhs;
                            }
                        };
                    }
                }
            })
        }

        _ => Err(syn::Error::new_spanned(
            input,
            "#[derive(Config)] can only be applied to structs and enums",
        )),
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

// TODO: field case variations
// TODO: Serde rename / alias
fn field_aliases(field: &syn::Field, opts: &SerdeFieldOpts) -> Vec<syn::LitStr> {
    let mut aliases = std::collections::BTreeSet::new();

    let name = opts
        .rename
        .clone()
        .unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());

    field_case_permutations(&name, &mut aliases);

    for alias in &opts.alias {
        field_case_permutations(alias, &mut aliases);
    }

    aliases
        .into_iter()
        .map(|s| syn::LitStr::new(&s, Span::call_site()))
        .collect()
}

/////////////////////////////////////////////////////////////////////////////////////////

// TODO: variant case variations
// TODO: Serde rename / alias
fn enum_tag_aliases(var: &syn::Variant) -> Vec<syn::LitStr> {
    let tag = var.ident.to_string().to_lowercase();
    let tag = syn::LitStr::new(&tag, Span::call_site());

    vec![tag]
}

/////////////////////////////////////////////////////////////////////////////////////////

const T_VALUE: &[&str] = &[
    "bool", "char", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128",
    "isize", "f32", "f64", "String",
];

fn default_combine_for(ty: &syn::Type) -> (&syn::Type, Combine) {
    let syn::Type::Path(pty) = ty else {
        panic!("Expected a Type::Path");
    };

    let last = pty.path.segments.last().unwrap();
    let ident = last.ident.to_string();

    if T_VALUE.contains(&ident.as_str()) {
        (ty, Combine::Replace)
    } else if type_matches(ty, "::std::option::Option") {
        let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
            panic!("Cannot parse Option<T> type");
        };
        assert_eq!(args.args.len(), 1);
        let syn::GenericArgument::Type(opt_ty) = &args.args[0] else {
            panic!("Cannot parse Option<T> type");
        };
        default_combine_for(opt_ty)
    } else if type_matches(ty, "::std::vec::Vec")
        || type_matches(ty, "::std::collections::BTreeMap")
    {
        (ty, Combine::Replace)
    } else {
        // Assuming custom type
        // User should ether provide merge implementation or opt out of merging
        (ty, Combine::Merge)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
