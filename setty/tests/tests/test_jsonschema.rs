#![cfg(feature = "derive-jsonschema")]

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
#[serial_test::serial]
fn test_json_schema() {
    use super::test_deserialize::MyConfig;

    let schema = setty::Config::<MyConfig>::new().json_schema();
    let schema = serde_json::to_string_pretty(&schema).unwrap();
    std::fs::write("tests/tests/gen/jsonschema.json", schema).unwrap();
}

/////////////////////////////////////////////////////////////////////////////////////////
