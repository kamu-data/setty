//! `setty` is a composable configuration crate.
//!
//! It can be used by **applications** to:
//! - Load and merge config from multiple sources (files, env vars etc.)
//! - Control stylistic choices (case, enum representation) in a single place
//! - Generate documentation and JSON Schema
//! - Edit config with CLI completions.
//!
//! And by **libraries** to define **reusable** config types without needing to anticipate format and style preferences that different applications using the library may choose for their configs.
//!
//! ## Motivation
//! Popular configuration crates like `config` and `figment` deal with **reading** and **merging** values from multiple sources. They leave it up to you to handle **parsing** using `serde` derives.
//!
//! This is a good separation of concerns, but it leaves a lot of important details to you:
//! - Like remembering to put `#[serde(deny_unknown_fields)]` so a small typo in your production config had no effect because of a small typo
//! - Keeping in mind the non-trivial interplay between `#[derive(Default)]`, `Option<T>` fileds, and `#[serde(default = "..")]`
//!
//! You may also need features beyond parsing:
//! - Documentation generation
//! - JSONSchema generation *(e.g. for Helm chart values validation)*
//! - Auto-completion in CLI
//! - Deprecation mechanism
//! - Per-field combine strategies *(e.g. keep first value, replace with latest, merge arrays)*
//!
//! Layering libraries and macros makes your models **very verbose**:
//!
//! ```ignore
//! #[serde_with::skip_serializing_none]
//! #[derive(
//!     Debug,
//!     PartialEq, Eq,
//!     better_default::Default,
//!     serde::Deserialize, serde::Serialize,
//!     validator::Validate,
//!     schemars::JsonSchema,
//! )]
//! #[serde(deny_unknown_fields, rename_all = "camelCase")]
//! struct AppConfig {
//!     /// Need to be explicit about using default for `serde`
//!     #[serde(default)]
//!     database: DatabaseConfig,
//!
//!     /// Note how defaults in `serde` and `Default::default()` are two separate things
//!     #[default(AppConfig::default_hostname())]
//!     #[serde(default = "AppConfig::default_hostname")]
//!     #[validate(min_length = 5)]
//!     hostname: String,
//!
//!     #[default(AppConfig::default_username())]
//!     #[serde(default = "AppConfig::default_username")]
//!     username: Username,
//!
//!     /// !! DO NOT USE !!!
//!     /// Deprecation is done by leaving screamy comments
//!     password: Option<String>
//! }
//!
//! /// No inline default epressions in `serde` - must use functions
//! impl AppConfig {
//!     fn default_hostname() -> String {
//!         "localhost".into()
//!     }
//!
//!     fn default_username() -> Username {
//!         "root".parse().unwrap()
//!     }
//! }
//! ```
//!
//! Even if you power through this in your application - you'll face a **composability problem** - how to surface configuration from the sub-modules you depend on in your app config.
//!
//! If config types defined in a module do not use your ideal set of derive macros, or don't follow your `camelCase` preference - you'll have to write lots of adapter types and mapping logic... yet more boilerplate!
//!
//! ## Proposed Solution
//! Applications and libraries use two simple macro:
//! ```ignore
//! /// Docstrings will appear in Markdown and JSON Schema outputs
//! #[derive(
//!     // Derives serialization, merging logic, schema generation etc.
//!     setty::Config,
//!     // Derives `Default` that is consistent with serde and schemars
//!     setty::Default,
//! )]
//! struct AppConfig {
//!     /// Opt-in into using `Default::default`
//!     #[config(default)]
//!     database: DatabaseConfig,
//!
//!     /// Or specify default values in-line (with full expressions)
//!     #[config(default = "localhost")]
//!     /// Basic validation can be delegated to `validator` crate
//!     #[config(validate(min_length = 5))]
//!     hostname: String,
//!
//!     /// Use `default_str` to parse the value
//!     #[config(default_str = "root")]
//!     username: Username,
//!
//!     /// Use of deptecated values can be reported as warnings or fail strict validation
//!     #[deprecated = "Avoid specifying password in config file"]
//!     password: Option<String>
//! }
//! ```
//!
//! Control what behavior you need via create features:
//! ```toml
//! setty = { version = "*", features = [
//!     # These traits will be derived for all config types in your app AND dependencies
//!     "derive-clone",
//!     "derive-debug",
//!     "derive-partial-eq",
//!     "derive-eq",
//!     "derive-deserialize",
//!     "derive-serialize",
//!     "derive-jsonschema",
//!     "derive-validate",
//!     # Pick one: A case for struct fields (applies `#[serde(renameAll = "...")]`)
//!     "case-fields-lower",
//!     "case-fields-pascal",
//!     "case-fields-camel",
//!     "case-fields-snake",
//!     "case-fields-kebab",
//!     # Pick one: A case for enum variants (applies `#[serde(renameAll = "...")]`)
//!     "case-enums-lower",
//!     "case-enums-pascal",
//!     "case-enums-camel",
//!     "case-enums-snake",
//!     "case-enums-kebab",
//!     "case-enums-any", # Uses one of other cases on write but accepts any on read
//!     # Pick input format(s)
//!     "fmt-toml",
//!     "fmt-json",
//!     "fmt-yaml",
//!     # Pick generation target formats
//!     "gen-jsonschema",
//!     "gen-markdown",
//!     # Extra types support
//!     "types-bigdecimal",
//!     "types-chrono",
//!     "types-duration-string",
//!     "types-url",
//! ] }
//! ```
//!
//! By specifying features **only** at the top-level application crate - the desired derives will be applied to configs of **all crates in your dependency tree** allowing you to directly embed their DTOs. In other words library developers don't have to predict and align every aspect of configuration with the app layer - they can focus only on types.
//!
//! Finally, load the config:
//! ```ignore
//! use setty::format::{Toml, Yaml};
//! use setty::source::{File, Env};
//!
//! let cfg: AppConfig = setty::Config::new()
//!     // Specify sources in priority order. Latter sources replace or merge
//!     // with values in earlier ones (see also `combine()` attribute).
//!     .with_sources(config_paths.iter().map(File::<Toml>::new))
//!     // Env source allows to pass overrides like:
//!     //
//!     //   APP_CONFIG__database__schema_name=my_schema
//!     //   APP_CONFIG__database__connection_timeout=30s
//!     //   APP_CONFIG__encryption='{"algo": "Aes256Gcm", "nonce": "..."}'
//!     //
//!     // I switch to YAML for env vars to avoid excessive quotes in most cases
//!     .with_source(Env::<Yaml>::new("APP_CONFIG__", "__"))
//!     // Merges the values and deserializes to config type
//!     .extract()?;
//! ```
//!
//! ### Known Alternatives
//! - Rolling your own declarative macros (see example in [`datafusion`](https://github.com/apache/datafusion/blob/b463a9f9e3c9603eb2db7113125fea3a1b7f5455/datafusion/common/src/config.rs#L2480))
//!
//!
//! ## Usage Examples
//! See the [`examples`](https://github.com/kamu-data/setty/tree/master/examples) directory.
//!
//!
//! ## Limitations and Future Ideas
//! - Config editing currently does not preserve order, comments, and formatting of files
//! - It's not possible to use different case convention for different formats (e.g. `camelCase` for YAML and `kebab-case` for TOML) - we could support it as a runtime (pre-processing) option
//! - Provide less verbose default syntax like `user: String = "root"` when/if the syntax is [stabilized](https://github.com/rust-lang/rust/issues/132162)
//!   - Currently it's not possible to support this even with a proc macro because how `rustc` rushes to parse the struct definition before handing it over to attribute macro
//! - Ability to provide example values that will appear in JSON Schema and Markdown

mod check_deprecated;
pub mod combine;
pub mod config;
pub mod errors;
pub mod format;
pub mod markdown;
mod merge_with_defaults;
pub mod schema;
pub mod source;
pub mod types;

/////////////////////////////////////////////////////////////////////////////////////////

pub use config::Config;

pub use serde_json::Value;

/////////////////////////////////////////////////////////////////////////////////////////

pub use setty_derive::{Config, Default, derive};

#[doc(hidden)]
pub use setty_derive::__erase;

/////////////////////////////////////////////////////////////////////////////////////////

#[doc(hidden)]
pub mod __internal {

    #[cfg(feature = "derive-jsonschema")]
    pub use schemars;

    #[cfg(any(feature = "derive-deserialize", feature = "derive-serialize"))]
    pub use serde;

    pub use serde_json;

    #[cfg(feature = "derive-serialize")]
    pub use serde_with;

    #[cfg(feature = "derive-validate")]
    pub use validator;
}
