# setty - opitionated application config
`setty` is a facade over several configuration libraries providing **turn-key config system with sane defaults**.

## Motivation
Popular configuration crates like `config` and `figment` deal with **reading** and **merging** values from multiple sources. They leave it up to you to handle **parsing** using `serde` derives. This is a good separation of concerns, but it leaves a lot of important details to you. Like remembering to put `#[serde(deny_unknown_fields)]` not to realize that your production config had no effect because of a small typo.

Also, you may need features beyond parsing:
- Documentation generation
- JSONSchema generation *(e.g. for Helm chart values validation)*
- Auto-completion in CLI
- Deprecation mechanism
- Per-field combine strategies *(e.g. keep first value, replace with latest, merge arrays)*

Layering more libraries and macros makes your models **very verbose**:

```rust
#[serde_with::skip_serializing_none]
#[derive(
    Debug,
    PartialEq, Eq,
    better_default::Default,
    serde::Deserialize, serde::Serialize,
    serde_valid::Validate,
    schemars::JsonSchema,
)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct AppConfig {
    /// Need to be explicit about using default for `serde`
    #[serde(default)]
    database: DatabaseConfig,

    /// Note how defaults in `serde` and `Default::default()` are two separate things
    #[default(AppConfig::default_hostname())]
    #[serde(default = "AppConfig::default_hostname")]
    #[validate(min_length = 5)]
    hostname: String,

    #[default(AppConfig::default_username())]
    #[serde(default = "AppConfig::default_username")]
    username: Username,

    /// !! DO NOT USE !!!
    /// Deprecation is done by leaving screamy comments
    password: Option<String>
}

/// No inline default epressions in `serde` - must use functions
impl AppConfig {
    fn default_hostname() -> String {
        "localhost".into()
    }

    fn default_username() -> Username {
        "root".parse().unwrap()
    }
}
```

And even if you power through this problem in your application - you'll face a **composability** problem - how to surface configuration from the sub-modules you depend on in your app config. If a config types defined in a module do not use your ideal set of derive macros - you'll be forced to define a temporary DTO and write lots of mapping logic... yet more boilerplate!

## Solution
Use one simple macro:
```rust
/// Docstrings will appear in Markdown and JSON Schema outputs
#[derive(
    // Implements all serde and schema stuff
    setty::Config,
    // Reuses same defaults to implement `Default` trait 
    setty::Default,
)]
struct AppConfig {
    /// Opt-in into using `Default::default`
    #[config(default)]
    database: DatabaseConfig,

    /// Or specify default values in-line (support full expressions)
    /// This default will be consistent between `Default::default()`, `serde(default)`, and `schemars`
    #[config(default = "localhost")]
    /// Basic validation can be delegated to `serde_valid` crate
    #[config(validate(min_length = 5))]
    hostname: String,

    /// Use `default_str` to parse the value
    #[config(default_str = "root")]
    username: Username,

    /// Use of deptecated values can be reported as warnings or fail strict validation
    #[deprecated = "Avoid specifying password in config file"]
    password: Option<String>
}
```

Control what behavior you need via create features:
```toml
setty = { 
    version = "*", 
    features = [
        # These traits will be derived for all types
        "derive-clone",
        "derive-debug",
        "derive-partial-eq",
        "derive-eq",
        "derive-deserialize",
        "derive-serialize",
        "derive-jsonschema",
        "derive-validate",
        # Pick a case for struct fields (applies `#[serde(renameAll = "...")]`)
        "case-fields-lower",
        "case-fields-pascal",
        "case-fields-camel",
        "case-fields-snake",
        "case-fields-kebab",
        # Pick a case for enum variants (applies `#[serde(renameAll = "...")]`)
        "case-enums-lower",
        "case-enums-pascal",
        "case-enums-camel",
        "case-enums-snake",
        "case-enums-kebab",
        "case-enums-any", # Uses one of other cases on write but accepts any on read
        # Pick input format(s)
        "fmt-toml",
        "fmt-json",
        "fmt-yaml",
        # Pick generation target formats
        "gen-jsonschema",
        "gen-markdown",
    ]
}
```

By specifying features **only** at the top-level application crate - the desired derives will be applied to configs of **all crates in your dependency tree** allowing you to directly embed their DTOs. In other words library developers don't have to predict and align every single aspect of configuration with the app layer - they can focus only on types and validation. 

## Alternatives
- Rolling your own declarative macros (see example in [`datafusion`](https://github.com/apache/datafusion/blob/b463a9f9e3c9603eb2db7113125fea3a1b7f5455/datafusion/common/src/config.rs#L2480))


## Proc Macros
* `derive` - a replacement for standard `#[derive(...)]` macro that will de-duplicate derivations - this is most useful for e.g. `#[setty::derive(setty::Config, Clone)]` which allows type to implement `Clone` even when top-level feature `derive-clone` is disable, and not hit duplicate trait impl error when feature is enabled.

## Derive Macros
* `Config` - main workhorse
* `Debug` - same as `std::Debug` but recognizes defaults provided via `#[config(default = $expr)]` attributes

## Field Attributes
These arguments can be specified in `#[config(...)]` field attribute:
* `default` - Use `Default::default` value if field is not present
* `default = $expr` - Specifies expression used to initialize the value when it's not present in config
* `default_str = "$str"` - Shorthand for`default = "$str".parse().unwrap()`
* `combine(keep | replace | merge)` - Allows overriding how values are combined across different config files
  * Possible values:
    * `keep` - keeps first seen value
    * `replace` - fully replaces with the new value
    * `merge` - merges object keys and concatenates arrays, merge is smart and will not merge values across different enums
  * Default behavior:
    * `replace` for all known value types
    * `merge` for unknown types
      * You will need to implement `setty::combine::Combine` for it to work for custom types
      * `Config` derive macro automatically implements it for you
      * If you don't want any merging - simply override to use `combine(replace)`

## Interaction with other attributes
* `#[serde(...)]` attribute will be propagated and can be used to override default behaviour (e.g. `#[serde(tag = "type")]`)
* `#[schemars(...)]` attribute will be propagated
