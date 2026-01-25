# setty - opitionated application config
`setty` is a facade over several configuration libraries providing **turn-key config system with sane defaults**.

## Problem
Popular configuration crates like `config` and `figment` deal with **reading** and **merging** values from multiple sources. They leave it up to you to handle **parsing** using `serde` derives. This is a good separation of concerns, but it leaves a lot of important details to you. Like remembering to put `#[serde(deny_unknown_fields)]` not to realize that your production config had no effect because of a small typo.

Also, you may need features beyond parsing:
- Documentation generation
- JSONSchema generation *(e.g. for Helm chart values validation)*
- Auto-completion in CLI
- Deprecation mechanism

Layering more libraries and macros makes your models **very verbose**:

```rust
#[serde_with::skip_serializing_none]
#[derive(Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize, serde_valid::Validate, schemars::JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct AppConfig {
    #[serde(default)]
    database: DatabaseConfig,

    #[validate(min_length = 5)]
    username: String,

    #[serde(default = "AppConfig::default_hostname")]
    hostname: String,

    // !! DO NOT USE !!!
    password: Option<String>
}

impl AppConfig {
    fn default_hostname() -> String {
        "localhost".into()
    }
}
```

And even if you power through this problem in your application - you'll face a **composability** problem of surfacing configuration from the modules you depend on. If a config object defined in a module does not use your ideal set of derive macros - you'll be forced to deplicating its structure in a temporary DTO and writing a mapping between them. Yet more boilerplate.

## Solution
Use one simple macro:
```rust
/// Docstrings will appear in Markdown and JSON Schema outputs
#[derive(setty::Config)]
struct AppConfig {
    /// All fields are initialized using `Default::default`
    database: DatabaseConfig,

    /// You can annotate fields that must be specified explicitly
    #[config(required)]
    /// Basic validation can be delegated to `serde_valid` crate
    #[config(validate(min_length = 5))]
    username: String,

    /// Default values can be specified in-line (support full expressions)
    #[config(default = "localhost")]
    hostname: String,

    /// Use of deptecated values can be reported as warnings or fail strict validation
    #[config(deprecated = "Avoid specifying password in config file")]
    password: Option<String>
}
```

Control what behavior you need via create features:
```toml
setty = { 
    version = "*", 
    features = [
        "derive-debug",
        "derive-eq",
        "derive-deserialize",
        "derive-serialize",
        "derive-jsonschema",
        "derive-validate",
        "fmt-toml",
        "fmt-json",
        "fmt-yaml",
    ]
}
```

By specifying features **only** at the top-level application crate - the desired derives will be applied to configs of **all crates in your dependency tree** allowing you to directly embed their DTOs. In other words library developers don't have to predict and align every single aspect of configuration with the app layer - they can focus only on types and validation. 

## Alternatives
- Rolling your own declarative macros (see example in [`datafusion`](https://github.com/apache/datafusion/blob/b463a9f9e3c9603eb2db7113125fea3a1b7f5455/datafusion/common/src/config.rs#L2480))