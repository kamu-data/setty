#![allow(unused)]

use proc_macro2::Span;
use quote::ToTokens;
use syn::spanned::Spanned;

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct ConfigFieldOpts {
    pub required: Option<bool>,
    pub default: Option<syn::Expr>,
    pub default_parse: Option<syn::LitStr>,
    pub span: Span,
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

    pub fn merge(&mut self, other: Self) -> syn::Result<()> {
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

    pub fn extract_from(attrs: &mut Vec<syn::Attribute>) -> syn::Result<Self> {
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

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("required") {
                opts.required = Some(true);
            } else if meta.path.is_ident("default") {
                let value: syn::Expr = meta.value()?.parse()?;
                opts.default = Some(value);
            } else if meta.path.is_ident("default_parse") {
                let value: syn::LitStr = meta.value()?.parse()?;
                opts.default_parse = Some(value);
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

pub(crate) fn is_default(attr: &syn::Attribute) -> bool {
    attr.path().is_ident("default")
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
