#![cfg(feature = "derive-serialize")]

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_json_schema() {
    use super::test_deserialize::CLIConfig;

    let cfg = CLIConfig::default();

    pretty_assertions::assert_eq!(
        serde_json::to_value(&cfg).unwrap(),
        serde_json::json!({
            "database": {
                "kind": "sqlite",
                "databasePath": ".kamu/db.sqlite",
            },
            // None is not serialized
            // "encryption": null,
        }),
    );
}

/////////////////////////////////////////////////////////////////////////////////////////
