#![cfg(feature = "derive-jsonschema")]

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
#[serial_test::serial]
fn test_markdown() {
    use super::test_deserialize::MyConfig;

    let md = setty::Config::<MyConfig>::new().markdown();
    std::fs::write("tests/tests/gen/markdown.md", md).unwrap();
}

/////////////////////////////////////////////////////////////////////////////////////////
