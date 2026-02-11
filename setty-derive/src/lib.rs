mod attrs;
mod combine;
mod config;
mod default;
mod derive;

/////////////////////////////////////////////////////////////////////////////////////////

/// Our main workhorse. Derives the attributes based on the set of enabled crate features.
///
/// ## Field Attributes
/// These arguments can be specified in `#[config(...)]` field attribute:
/// * `default` - Use `Default::default` value if field is not present
/// * `default = $expr` - Specifies expression used to initialize the value when it's not present in config
/// * `default_str = "$str"` - Shorthand for`default = "$str".parse().unwrap()`
/// * `combine(keep | replace | merge)` - Allows overriding how values are combined across different config files
///   * Possible values:
///     * `keep` - keeps first seen value
///     * `replace` - fully replaces with the new value
///     * `merge` - merges object keys and concatenates arrays, merge is smart and will not merge values across different enums
///   * Default behavior:
///     * `replace` for all known value types
///     * `merge` for unknown types
///       * You will need to implement `setty::combine::Combine` for it to work for custom types
///       * `Config` derive macro automatically implements it for you
///       * If you don't want any merging - simply override to use `combine(replace)`
///
/// ## Interaction with other attributes
/// * `#[serde(...)]` attribute will be propagated and can be used to override default behaviour (e.g. `#[serde(tag = "type")]`)
/// * `#[schemars(...)]` attribute will be propagated
///
#[proc_macro_derive(Config, attributes(config, serde, schemars))]
pub fn derive_config(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match derive_config_impl(input.into()) {
        Ok(output) => proc_macro::TokenStream::from(output),
        Err(err) => err.to_compile_error().into(),
    }
}

pub(crate) fn derive_config_impl(
    input: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let input: syn::DeriveInput = syn::parse2(input)?;
    let combine_output = combine::combine_impl(&input)?;
    let config_output = config::config_impl(input)?;

    Ok(quote::quote! {
        #config_output
        #combine_output
    })
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Special version of built-in `#[derive(Default)]` that recognizes `#[config(default = $expr)]` attributes
#[proc_macro_derive(Default, attributes(config, default))]
pub fn derive_default(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match default::default_impl(input) {
        Ok(output) => proc_macro::TokenStream::from(output),
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
pub fn derive(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match derive::derive_impl(attr.into(), item.into()) {
        Ok(output) => proc_macro::TokenStream::from(output),
        Err(err) => err.to_compile_error().into(),
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[proc_macro_attribute]
pub fn __erase(
    _attr: proc_macro::TokenStream,
    _item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    proc_macro::TokenStream::new()
}

/////////////////////////////////////////////////////////////////////////////////////////
