#![cfg(feature = "derive-jsonschema")]

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
#[serial_test::serial]
fn test_json_schema() {
    use super::test_deserialize::CLIConfig;

    let schema = schemars::schema_for!(CLIConfig);
    let schema = serde_json::to_string_pretty(&schema).unwrap();
    std::fs::write("tests/tests/generated-schema.json", schema).unwrap();
}

/////////////////////////////////////////////////////////////////////////////////////////
