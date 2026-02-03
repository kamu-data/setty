#![cfg(feature = "derive-serialize")]

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_dont_serialize_none() {
    use super::test_deserialize::MyConfig;

    let cfg = MyConfig::default();

    pretty_assertions::assert_eq!(
        serde_json::to_value(&cfg).unwrap(),
        serde_json::json!({
            "database": {
                "kind": "Sqlite",
                "database_path": ".kamu/db.sqlite",
            },
            // None is not serialized
            // "encryption": null,
        }),
    );
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_tag_override() {
    #[derive(setty::Config, setty::Default)]
    struct A {
        #[config(default)]
        b: B,
    }

    #[derive(setty::Config, setty::Default)]
    #[serde(tag = "provider")]
    enum B {
        X(X),
        #[default]
        Y(Y),
    }

    #[derive(setty::Config, setty::Default)]
    struct X {
        #[config(default)]
        a: u32,
    }

    #[derive(setty::Config, setty::Default)]
    struct Y {
        #[config(default)]
        a: u32,
    }

    let cfg = A::default();

    pretty_assertions::assert_eq!(
        serde_json::to_value(&cfg).unwrap(),
        serde_json::json!({
            "b": {
                "provider": "Y",
                "a": 0,
            },
        }),
    );
}

/////////////////////////////////////////////////////////////////////////////////////////
