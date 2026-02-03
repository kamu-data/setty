#![allow(unused)]

use proc_macro2::Span;
use quote::ToTokens;
use syn::spanned::Spanned;

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct ConfigFieldOpts {
    pub default: Option<Option<syn::Expr>>,
    pub span: Span,
}

impl ConfigFieldOpts {
    fn new(span: Span) -> Self {
        Self {
            default: None,
            span,
        }
    }

    pub fn merge(&mut self, other: Self) -> syn::Result<()> {
        self.span = other.span;

        if other.default.is_some() {
            if self.default.is_some() {
                return Err(syn::Error::new(
                    other.span,
                    "`default` specified more than once",
                ));
            }
            self.default = other.default;
        }

        Ok(())
    }

    pub fn extract_from(field: &mut syn::Field) -> syn::Result<Self> {
        let mut opts = Self::new(proc_macro2::Span::call_site());

        for attr in field.attrs.iter() {
            if attr.path().is_ident("config") {
                let more_opts = Self::parse_from(attr)?;
                opts.merge(more_opts)?;
            }
        }

        field.attrs.retain(|attr| !attr.path().is_ident("config"));

        // Provide a default for `Option<T>`
        if opts.default.is_none() && is_option(&field.ty) {
            opts.default = Some(None);
        }

        Ok(opts)
    }

    fn parse_from(attr: &syn::Attribute) -> syn::Result<Self> {
        let mut opts = Self::new(attr.span());

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("default") {
                if meta.input.peek(syn::Token![=]) {
                    let expr: syn::Expr = meta.value()?.parse()?;

                    // Add `.into()` coersion to everything except int literals
                    let expr = match expr {
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Int(_),
                            attrs: _,
                        }) => expr,
                        _ => syn::parse_quote!(#expr.into()),
                    };

                    opts.default = Some(Some(expr));
                } else {
                    opts.default = Some(None)
                }
            } else if meta.path.is_ident("default_str") {
                let lit: syn::LitStr = meta.value()?.parse()?;
                let expr: syn::Expr = syn::parse_quote!(#lit.parse().unwrap());
                opts.default = Some(Some(expr));
            } else {
                return Err(syn::Error::new(
                    meta.path.span(),
                    format!("unknown config option `{}`", meta.path.to_token_stream()),
                ));
            }
            Ok(())
        })?;

        Ok(opts)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct SerdeTypeOpts {
    pub tag: Option<String>,
    pub span: Span,
}

impl SerdeTypeOpts {
    fn new(span: Span) -> Self {
        Self { tag: None, span }
    }

    fn merge(&mut self, other: Self) -> syn::Result<()> {
        self.span = other.span;

        if other.tag.is_some() {
            self.tag = other.tag;
        }

        Ok(())
    }

    pub fn parse(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut opts = Self::new(proc_macro2::Span::call_site());

        for attr in attrs.iter() {
            if attr.path().is_ident("serde") {
                let more_opts = Self::parse_from(attr)?;
                opts.merge(more_opts)?;
            }
        }

        Ok(opts)
    }

    fn parse_from(attr: &syn::Attribute) -> syn::Result<Self> {
        let mut opts = Self::new(attr.span());

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("tag") {
                let value: syn::LitStr = meta.value()?.parse()?;
                opts.tag = Some(value.value());
            } else {
                let _ = meta.value()?.parse::<syn::Expr>()?;
            }
            Ok(())
        })?;

        Ok(opts)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct SerdeFieldOpts {
    pub rename: Option<String>,
    pub alias: Vec<String>,
    pub span: Span,
}

impl SerdeFieldOpts {
    fn new(span: Span) -> Self {
        Self {
            rename: None,
            alias: Vec::new(),
            span,
        }
    }

    pub fn merge(&mut self, mut other: Self) -> syn::Result<()> {
        self.span = other.span;

        if other.rename.is_some() {
            self.rename = other.rename;
        }

        self.alias.append(&mut other.alias);

        Ok(())
    }

    pub fn parse(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut opts = Self::new(proc_macro2::Span::call_site());

        for attr in attrs.iter() {
            if attr.path().is_ident("serde") {
                let more_opts = Self::parse_from(attr)?;
                opts.merge(more_opts)?;
            }
        }

        Ok(opts)
    }

    fn parse_from(attr: &syn::Attribute) -> syn::Result<Self> {
        let mut opts = Self::new(attr.span());

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let value: syn::LitStr = meta.value()?.parse()?;
                opts.rename = Some(value.value());
            } else if meta.path.is_ident("alias") {
                let value: syn::LitStr = meta.value()?.parse()?;
                opts.alias.push(value.value());
            } else {
                let _ = meta.value()?.parse::<syn::Expr>()?;
            }
            Ok(())
        })?;

        Ok(opts)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn is_option(typ: &syn::Type) -> bool {
    let syn::Type::Path(typ) = typ else {
        panic!("Expected a Type::Path");
    };

    typ.qself.is_none() && &typ.path.segments.last().unwrap().ident == "Option"
}

/////////////////////////////////////////////////////////////////////////////////////////

// TODO: Consider performance of this
pub(crate) fn path_matches(p: &syn::Path, other: &str) -> bool {
    let other: syn::Path = syn::parse_str(other).unwrap();

    p.segments.last().unwrap().ident == other.segments.last().unwrap().ident
}

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn fields_case() -> Option<&'static str> {
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

pub(crate) fn variants_case() -> Option<&'static str> {
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

pub(crate) fn case_permutations(names: &[String]) -> std::collections::BTreeSet<String> {
    let mut ret = std::collections::BTreeSet::new();

    for name in names {
        ret.insert(name.to_owned());
        ret.insert(name.to_lowercase());
        ret.insert(pascal_to_camel(name));
    }

    ret
}

pub(crate) fn pascal_to_camel(s: &str) -> String {
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
