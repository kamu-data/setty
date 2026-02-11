# Usage Examples
We demonstrate use of `setty` through a bunch of small example apps.

Since `setty` is configured via cargo features - each example should be compiled and ran individually like so:

```sh
cargo run -p example-jsonschema
```

## Examples
- `example-jsonschema` - Shows generation of JSON Schema and Markdown outputs, combined with `camelCase` renaming of fields and YAML format.
- `example-toml` - Simple TOML config example that demonstrates merging and use of `kebab-case` renaming.
