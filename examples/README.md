# Usage Examples
We demonstrate use of `setty` through a bunch of small example apps.

Since `setty` is configured via cargo features - each example should be compiled and ran individually like so:

```sh
cargo run -p example-jsonschema
```

## Examples
- [`deprecation`](./deprecation/) - Shows deprecation reporting

- [`example-env`](./env/) - Demonstrates overriding config values via env vars
  - Try running it as `MY_CFG__database__provider='"sqlite"' cargo run -p example-env`
  - Note how overriding the enum tag stops merging in values from `postgres` in config
  - Note the use of `SecretString` for password

- [`example-jsonschema`](./jsonschema/) - Shows generation of JSON Schema and Markdown outputs, combined with `camelCase` renaming of fields and YAML format
  - Notice the generated `config-schema.json` and `config-readme.md` files

- [`example-toml`](./toml/) - Simple TOML config example that demonstrates merging and use of `kebab-case` renaming

- [`example-validate`](./validate/) - Showcase for using `derive-validate` feature for rich validation
