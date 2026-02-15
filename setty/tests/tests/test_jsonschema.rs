#![cfg(feature = "derive-jsonschema")]

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_json_schema_combine_attrs() {
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
                "vec_replace": {
                    "default": [],
                    "items":  {
                        "type": "string",
                    },
                    "type": "array",
                },
                "vec_merge":  {
                    "default": [],
                    "combine": "merge",
                    "items": {
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

#[test]
fn test_json_schema_deprecaion() {
    #[derive(setty::Config)]
    struct Cfg {
        #[deprecated = "Avoid"]
        a: Option<String>,

        #[deprecated(since = "1.1.0", note = "Don't use")]
        b: Option<String>,
    }

    let schema = setty::Config::<Cfg>::new().json_schema();

    pretty_assertions::assert_eq!(
        *schema.as_value(),
        serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "additionalProperties": false,
            "properties": {
                "a":  {
                    "type": [
                        "string",
                        "null",
                    ],
                    "deprecated": true,
                    "deprecation": {
                        "reason": "Avoid",
                    },
                },
                "b":  {
                    "type": [
                        "string",
                        "null",
                    ],
                    "deprecated": true,
                    "deprecation": {
                        "since": "1.1.0",
                        "reason": "Don't use",
                    },
                },
            },
            "title": "Cfg",
            "type": "object",
        }),
    );
}

/////////////////////////////////////////////////////////////////////////////////////////
