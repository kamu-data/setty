mod attrs;
mod config;
mod default;
mod derive;

/////////////////////////////////////////////////////////////////////////////////////////

#[proc_macro_derive(Config, attributes(config, serde, schemars))]
pub fn derive_config(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match config::config_impl(input) {
        Ok(output) => proc_macro::TokenStream::from(output),
        Err(err) => err.to_compile_error().into(),
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Special version of `#[derive(Default)]` that recognizes `#[config(default = $expr)]` attributes
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
