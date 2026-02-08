#![cfg(feature = "derive-jsonschema")]

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_json_schema_gen() {
    use super::test_deserialize::MyConfig;

    let schema = setty::Config::<MyConfig>::new().json_schema();
    let schema = serde_json::to_string_pretty(&schema).unwrap();
    std::fs::write("tests/tests/gen/jsonschema.json", schema).unwrap();
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_json_schema_combine_attrs() {
    // Always derives Clone, deduplicating it with one from Config
    #[derive(setty::Config)]
    struct Cfg {
        #[config(default)]
        vec_replace: Vec<String>,

        #[config(default, combine(merge))]
        vec_merge: Vec<String>,
    }

    let schema = setty::Config::<Cfg>::new().json_schema();

    pretty_assertions::assert_eq!(
        *schema.as_value(),
        serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "additionalProperties": false,
            "properties": {
                "vec_replace":  {
                    "default":  [],

                    "items":  {
                        "type": "string",
                    },
                    "type": "array",
                },
                "vec_merge":  {
                    "default":  [],
                    "combine": "merge",
                    "items":  {
                        "type": "string",
                    },
                    "type": "array",
                },
            },
            "title": "Cfg",
            "type": "object",
        }),
    );
}

/////////////////////////////////////////////////////////////////////////////////////////
