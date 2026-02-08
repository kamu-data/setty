#![cfg(feature = "derive-deserialize")]

use serde_json::json;

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct MyConfig {
    /// Database configuration
    #[config(default)]
    pub database: DatabaseConfig,

    /// Optional encryption
    #[config(default)]
    pub encryption: Option<EncryptionConfig>,
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub enum DatabaseConfig {
    #[default]
    Sqlite(SqliteDatabaseConfig),
    Postgres(PostgresDatabaseConfig),
}

/// Sqlite driver
#[derive(setty::Config, setty::Default)]
pub struct SqliteDatabaseConfig {
    /// Path to the database file
    #[config(default = ".kamu/db.sqlite")]
    pub database_path: String,
}

/// Postgres driver
#[derive(setty::Config)]
pub struct PostgresDatabaseConfig {
    /// Name of the schema
    pub schema_name: String,

    /// Host name
    #[config(default = "localhost")]
    pub host: String,
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config)]
pub struct EncryptionConfig {
    /// Encryption key
    pub key: String,

    /// Encryption algorythm
    #[config(default)]
    pub algo: EncryptionAlgo,
}

#[derive(Default, setty::Config)]
pub enum EncryptionAlgo {
    #[default]
    Aes,
    Rsa,
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_defaults() {
    let cfg: MyConfig = setty::Config::new().extract().unwrap();

    assert_eq!(
        cfg,
        MyConfig {
            database: DatabaseConfig::default(),
            encryption: None,
        }
    );
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_empty_config() {
    let cfg: MyConfig = setty::Config::new()
        .with_source(json!({}))
        .extract()
        .unwrap();

    assert_eq!(
        cfg,
        MyConfig {
            database: DatabaseConfig::default(),
            encryption: None,
        }
    );
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_one_config() {
    let cfg: MyConfig = setty::Config::new()
        .with_source(json!({
            "database": {
                "kind": "Postgres",
                "schema_name": "foo",
            }
        }))
        .extract()
        .unwrap();

    assert_eq!(
        cfg,
        MyConfig {
            database: DatabaseConfig::Postgres(PostgresDatabaseConfig {
                schema_name: "foo".into(),
                host: "localhost".into(),
            }),
            encryption: None,
        }
    );
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_unrecognized_field_rejected() {
    let err = setty::Config::<MyConfig>::new()
        .with_source(json!({
            "database": {
                "kind": "Postgres",
                "schema_namez": "foo",
            }
        }))
        .extract()
        .expect_err("Expected error");

    pretty_assertions::assert_eq!(
        err.to_string(),
        "unknown field `schema_namez`, expected `schema_name` or `host`"
    );
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_required_field() {
    // Required field missing
    let err = setty::Config::<MyConfig>::new()
        .with_source(json!({
            "encryption": {
                "algo": "Rsa",
            }
        }))
        .extract()
        .expect_err("Expected error");

    pretty_assertions::assert_eq!(err.to_string(), "missing field `key`");

    // Required field present
    let cfg: MyConfig = setty::Config::new()
        .with_source(json!({
            "encryption": {
                "algo": "Rsa",
                "key": "secret",
            }
        }))
        .extract()
        .unwrap();

    assert_eq!(
        cfg,
        MyConfig {
            database: DatabaseConfig::default(),
            encryption: Some(EncryptionConfig {
                key: "secret".into(),
                algo: EncryptionAlgo::Rsa
            }),
        }
    );
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_default() {
    #[derive(setty::Config, setty::Default)]
    struct Config {
        #[config(default = 42)]
        foo: u32,

        #[config(default_str = "42")]
        bar: u32,

        #[config(default)]
        baz: Baz,

        opt: Option<u32>,
    }

    #[derive(setty::Config, setty::Default)]
    enum Baz {
        X,
        #[default]
        Y,
    }

    assert_eq!(
        Config::default(),
        Config {
            foo: 42,
            bar: 42,
            baz: Baz::Y,
            opt: None,
        }
    );

    let cfg: Config = setty::Config::new().extract().unwrap();
    assert_eq!(
        cfg,
        Config {
            foo: 42,
            bar: 42,
            baz: Baz::Y,
            opt: None,
        }
    );
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_serde_enum_mixed() {
    #[derive(setty::Config)]
    struct A {
        b: B,
    }

    #[derive(setty::Config)]
    enum B {
        Foo,
        Bar(C),
    }

    #[derive(setty::Config)]
    struct C {
        x: u32,
    }

    let cfg: A = setty::Config::new()
        .with_source(json!({
            "b": {
                "kind": "foo",
            }
        }))
        .extract()
        .unwrap();

    assert_eq!(cfg, A { b: B::Foo });

    let cfg: A = setty::Config::new()
        .with_source(json!({
            "b": {
                "kind": "bar",
                "x": 42,
            }
        }))
        .extract()
        .unwrap();

    assert_eq!(
        cfg,
        A {
            b: B::Bar(C { x: 42 })
        }
    );
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_serde_rename_unit() {
    #[derive(setty::Config)]
    struct A {
        b: B,
    }

    #[derive(setty::Config)]
    enum B {
        Foo,

        #[serde(rename = "Baz")]
        Bar,
    }

    // Setty will consider new name `Baz` and apply the case aliases to it, not to `Bar`
    let cfg: A = setty::Config::new()
        .with_source(json!({ "b": "baz" }))
        .extract()
        .unwrap();

    assert_eq!(cfg, A { b: B::Bar });
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_serde_rename_enum() {
    #[derive(setty::Config)]
    struct A {
        b: B,
    }

    #[derive(setty::Config)]
    #[serde(tag = "type")]
    enum B {
        Foo(C),

        #[serde(rename = "Baz", alias = "Foobar")]
        Bar(C),
    }

    #[derive(setty::Config)]
    struct C {}

    // Setty will consider new name `Baz` and apply the case aliases to it, not to `Bar`
    let cfg: A = setty::Config::new()
        .with_source(json!({
            "b": {
                "type": "baz",
            }
        }))
        .extract()
        .unwrap();

    assert_eq!(cfg, A { b: B::Bar(C {}) });

    // Setty will generate case permutations for alias as well
    let cfg: A = setty::Config::new()
        .with_source(json!({
            "b": {
                "type": "foobar",
            }
        }))
        .extract()
        .unwrap();

    assert_eq!(cfg, A { b: B::Bar(C {}) });
}

/////////////////////////////////////////////////////////////////////////////////////////
