#![cfg(feature = "derive-deserialize")]
#![cfg(feature = "derive-jsonschema")]

use super::test_deserialize::*;

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_config_ops() {
    let tmp = tempfile::tempdir().unwrap();

    let path1 = tmp.path().join("1.yaml");
    let path2 = tmp.path().join("2.yaml");

    std::fs::write(
        &path1,
        indoc::indoc!(
            r#"
            database:
                kind: Postgres
                schema_name: foo
            "#
        ),
    )
    .unwrap();

    std::fs::write(
        &path2,
        indoc::indoc!(
            r#"
            encryption:
                key: secret
            "#
        ),
    )
    .unwrap();

    let fig = setty::Config::<MyConfig>::new()
        .with_source(setty::source::File::<setty::format::Yaml>::new(&path1))
        .with_source(setty::source::File::<setty::format::Yaml>::new(&path2));

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
        Some("localhost".into()),
    );

    // Set value
    fig.set_value::<setty::format::Yaml>("encryption.key", "bar", &path2)
        .unwrap();

    pretty_assertions::assert_eq!(
        fig.get_value("encryption.key", false).unwrap(),
        Some("bar".into())
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
}

/////////////////////////////////////////////////////////////////////////////////////////
