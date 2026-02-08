#![cfg(feature = "derive-deserialize")]

use super::test_deserialize::*;

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "fmt-json")]
#[test]
fn test_format_json() {
    let cfg: MyConfig = setty::Config::new()
        .with_source(setty::source::RawData::<setty::format::Json>::new(
            indoc::indoc!(
                r#"
                {
                    "database": {
                        "kind": "Postgres",
                        "schema_name": "my_schema",
                        "host": "my_host"
                    },
                    "encryption": {
                        "algo": "Rsa",
                        "key": "secret"
                    }
                }
                "#,
            ),
        ))
        .extract()
        .unwrap();

    assert_eq!(
        cfg,
        MyConfig {
            database: DatabaseConfig::Postgres(PostgresDatabaseConfig {
                schema_name: "my_schema".into(),
                host: "my_host".into()
            }),
            encryption: Some(EncryptionConfig {
                key: "secret".into(),
                algo: EncryptionAlgo::Rsa
            }),
        }
    );
}

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "fmt-yaml")]
#[test]
fn test_format_yaml() {
    let cfg: MyConfig = setty::Config::new()
        .with_source(setty::source::RawData::<setty::format::Yaml>::new(
            indoc::indoc!(
                r#"
                database:
                    kind: Postgres
                    schema_name: my_schema
                    host: my_host
                encryption:
                    algo: Rsa
                    key: secret
                "#,
            ),
        ))
        .extract()
        .unwrap();

    assert_eq!(
        cfg,
        MyConfig {
            database: DatabaseConfig::Postgres(PostgresDatabaseConfig {
                schema_name: "my_schema".into(),
                host: "my_host".into()
            }),
            encryption: Some(EncryptionConfig {
                key: "secret".into(),
                algo: EncryptionAlgo::Rsa
            }),
        }
    );
}

/////////////////////////////////////////////////////////////////////////////////////////

// Test to ensure we don't emit the dreaded `$serde_json::private::Number`
#[cfg(feature = "fmt-json")]
#[cfg(feature = "fmt-yaml")]
#[test]
fn test_format_yaml_to_json_precision() {
    use setty::format::Format;

    #[derive(setty::Config)]
    struct Cfg {
        num: usize,
    }

    let data = setty::Config::<Cfg>::new()
        .with_source(setty::source::RawData::<setty::format::Yaml>::new(
            "num: 42",
        ))
        .data(false)
        .unwrap();

    pretty_assertions::assert_eq!(
        setty::format::JsonPretty::serialize(&data).unwrap(),
        indoc::indoc!(
            r#"
            {
              "num": 42
            }"#
        ),
    );

    pretty_assertions::assert_eq!(
        setty::format::Yaml::serialize(&data).unwrap(),
        indoc::indoc!(
            r#"
            num: 42
            "#
        ),
    );
}

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "fmt-toml")]
#[test]
fn test_format_toml() {
    let cfg: MyConfig = setty::Config::new()
        .with_source(setty::source::RawData::<setty::format::Toml>::new(
            indoc::indoc!(
                r#"
                [database]
                kind = "Postgres"
                schema_name = "my_schema"
                host = "my_host"

                [encryption]
                algo = "Rsa"
                key = "secret"
                "#,
            ),
        ))
        .extract()
        .unwrap();

    assert_eq!(
        cfg,
        MyConfig {
            database: DatabaseConfig::Postgres(PostgresDatabaseConfig {
                schema_name: "my_schema".into(),
                host: "my_host".into()
            }),
            encryption: Some(EncryptionConfig {
                key: "secret".into(),
                algo: EncryptionAlgo::Rsa
            }),
        }
    );
}

/////////////////////////////////////////////////////////////////////////////////////////
