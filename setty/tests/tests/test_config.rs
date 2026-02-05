#![cfg(feature = "derive-deserialize")]

use super::test_deserialize::*;

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
#[serial_test::serial]
fn test_config_ops() {
    setty::test::Jail::expect_with(|j| {
        j.create_file(
            "1.yaml",
            indoc::indoc!(
                r#"
                database:
                    kind: Postgres
                    schema_name: foo
                "#
            ),
        )?;
        j.create_file(
            "2.yaml",
            indoc::indoc!(
                r#"
                encryption:
                    key: secret
                "#
            ),
        )?;

        let fig = setty::Config::<MyConfig>::new()
            .with_source_file::<setty::format::Yaml>("1.yaml")
            .with_source_file::<setty::format::Yaml>("2.yaml");

        // Extract
        let cfg = fig.extract().unwrap();

        pretty_assertions::assert_eq!(
            cfg,
            MyConfig {
                database: DatabaseConfig::Postgres(PostgresDatabaseConfig {
                    schema_name: "foo".into(),
                    host: "localhost".into(),
                }),
                encryption: Some(EncryptionConfig {
                    key: "secret".into(),
                    algo: EncryptionAlgo::Aes,
                })
            }
        );

        // Data
        let value = fig.data(false).unwrap();
        let value_json = serde_json::to_value(&value).unwrap();
        pretty_assertions::assert_eq!(
            value_json,
            serde_json::json!({
                "database":  {
                    "kind": "Postgres",
                    "schema_name": "foo",
                },
                "encryption": {
                    "key": "secret",
                }
            }),
        );

        let value = fig.data(true).unwrap();
        let value_json = serde_json::to_value(&value).unwrap();
        pretty_assertions::assert_eq!(
            value_json,
            serde_json::json!({
                "database":  {
                    "host": "localhost",
                    "kind": "Postgres",
                    "schema_name": "foo",
                },
                "encryption": {
                    "algo": "Aes",
                    "key": "secret",
                }
            }),
        );

        // Get value
        pretty_assertions::assert_eq!(fig.get_value("database.host", false).unwrap(), None);
        pretty_assertions::assert_eq!(
            fig.get_value("database.host", true).unwrap(),
            Some(setty::Value::from("localhost")),
        );

        // Set value
        fig.set_value::<setty::format::Yaml>("encryption.key", "bar", "2.yaml")
            .unwrap();

        // Re-init figment to reset its cache
        let fig = setty::Config::<MyConfig>::new()
            .with_source_file::<setty::format::Yaml>("1.yaml")
            .with_source_file::<setty::format::Yaml>("2.yaml");

        pretty_assertions::assert_eq!(
            fig.get_value("encryption.key", false).unwrap(),
            Some(setty::Value::from("bar"))
        );

        // Complete path
        let mut completions = fig.complete_path("data");
        completions.sort();

        pretty_assertions::assert_eq!(
            completions,
            [
                "database",
                "database.database_path",
                "database.host",
                "database.kind",
                "database.kind",
                "database.schema_name",
            ]
        );

        Ok(())
    })
}

/////////////////////////////////////////////////////////////////////////////////////////
