//! `setty` is an opinionated configuration crate.
//!
//! It can be used by:
//! - Applications - to load and merge config from multiple sources and formats and generate documentation and JSON Schema
//! - Libraries - to define their own config DTOs without needing to anticipate or align application-level config format and style preferences
//!
//! ## Problem Statement
//! Popular configuration crates like `config` and `figment` deal with **reading** and **merging** values from multiple sources. They leave it up to you to handle **parsing** using `serde` derives. This is a good separation of concerns, but it leaves a lot of important details to you. Like remembering to put `#[serde(deny_unknown_fields)]` not to realize that your production config had no effect because of a small typo.
//!
//! Also, you may need features beyond parsing:
//! - Documentation generation
//! - JSONSchema generation *(e.g. for Helm chart values validation)*
//! - Auto-completion in CLI
//! - Deprecation mechanism
//! - Per-field combine strategies *(e.g. keep first value, replace with latest, merge arrays)*
//!
//! Layering more libraries and macros makes your models **very verbose**:
//!
//! ```ignore
//! #[serde_with::skip_serializing_none]
//! #[derive(
//!     Debug,
//!     PartialEq, Eq,
//!     better_default::Default,
//!     serde::Deserialize, serde::Serialize,
//!     serde_valid::Validate,
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
//! And even if you power through this problem in your application - you'll face a **composability** problem - how to surface configuration from the sub-modules you depend on in your app config. If a config types defined in a module do not use your ideal set of derive macros - you'll be forced to define a temporary DTO and write lots of mapping logic... yet more boilerplate!
//!
//! ## Solution
//! Use one simple macro:
//! ```ignore
//! /// Docstrings will appear in Markdown and JSON Schema outputs
//! #[derive(setty::Config)]
//! struct AppConfig {
//!     /// Opt-in into using `Default::default`
//!     #[config(default)]
//!     database: DatabaseConfig,
//!
//!     /// Or specify default values in-line (support full expressions)
//!     /// This default will be consistent between `Default::default()`, `serde(default)`, and `schemars`
//!     #[config(default = "localhost")]
//!     /// Basic validation can be delegated to `serde_valid` crate
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
//!     # These traits will be derived for all types
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
//! By specifying features **only** at the top-level application crate - the desired derives will be applied to configs of **all crates in your dependency tree** allowing you to directly embed their DTOs. In other words library developers don't have to predict and align every aspect of configuration with the app layer - they can focus only on DTO types.
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
//! ## Usage Examples
//! See the [`examples`](https://github.com/kamu-data/setty/tree/master/examples) directory.

pub mod combine;
pub mod config;
pub mod errors;
pub mod format;
pub mod markdown;
pub mod merge_with_defaults;
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
}
