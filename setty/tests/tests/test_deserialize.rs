#![cfg(feature = "derive-deserialize")]

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
pub struct MyConfig {
    /// Database configuration
    pub database: DatabaseConfig,

    /// Optional encryption
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
    #[config(required)]
    pub key: String,

    /// Encryption algorythm
    pub algo: EncryptionAlgo,
}

#[derive(Default, setty::Config)]
pub enum EncryptionAlgo {
    #[default]
    Aes,
    Rsa,
}

/////////////////////////////////////////////////////////////////////////////////////////

fn get_config_figment() -> figment2::Figment {
    use figment2::providers::{Env, Format, Yaml};

    figment2::Figment::new()
        .merge(Yaml::file(".kamu/config.yaml"))
        .merge(Env::prefixed("KAMU_CFG_").split("__").lowercase(false))
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
#[serial_test::serial]
fn test_defaults() {
    setty::test::Jail::expect_with(|_| {
        let cfg: MyConfig = get_config_figment().extract()?;

        assert_eq!(
            cfg,
            MyConfig {
                database: DatabaseConfig::default(),
                encryption: None,
            }
        );
        Ok(())
    });
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
#[serial_test::serial]
fn test_empty_config() {
    setty::test::Jail::expect_with(|j| {
        j.create_dir(".kamu")?;
        j.create_file(".kamu/config.yaml", r#""#)?;

        let cfg: MyConfig = get_config_figment().extract()?;

        assert_eq!(
            cfg,
            MyConfig {
                database: DatabaseConfig::default(),
                encryption: None,
            }
        );
        Ok(())
    });
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
#[serial_test::serial]
fn test_one_config() {
    setty::test::Jail::expect_with(|j| {
        j.create_dir(".kamu")?;
        j.create_file(
            ".kamu/config.yaml",
            indoc::indoc!(
                r#"
                database:
                    kind: Postgres
                    schema_name: foo
                "#
            ),
        )?;

        let cfg: MyConfig = get_config_figment().extract()?;

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
        Ok(())
    });
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
#[serial_test::serial]
fn test_env_var_override_nested() {
    setty::test::Jail::expect_with(|j| {
        // Note: Lowercase `postgres` works because of `case-enums-any` feature
        j.set_env("KAMU_CFG_database__kind", "postgres");
        j.set_env("KAMU_CFG_database__schema_name", "bar");

        let cfg: MyConfig = get_config_figment().extract()?;

        assert_eq!(
            cfg,
            MyConfig {
                database: DatabaseConfig::Postgres(PostgresDatabaseConfig {
                    schema_name: "bar".into(),
                    host: "localhost".into(),
                }),
                encryption: None,
            }
        );
        Ok(())
    });
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
#[serial_test::serial]
fn test_unrecognized_field_rejected() {
    setty::test::Jail::expect_with(|j| {
        j.create_dir(".kamu")?;
        j.create_file(
            ".kamu/config.yaml",
            indoc::indoc!(
                r#"
                database:
                    kind: Postgres
                    schema_namez: foo
                "#
            ),
        )?;

        let err = get_config_figment()
            .extract::<MyConfig>()
            .expect_err("Expected error");

        assert_eq!(
            err.kind,
            figment2::error::Kind::UnknownField("schema_namez".into(), &["schema_name", "host"])
        );
        assert_eq!(err.path, ["database"]);
        Ok(())
    });
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
#[serial_test::serial]
fn test_required_field() {
    // Required field missing
    setty::test::Jail::expect_with(|j| {
        j.create_dir(".kamu")?;
        j.create_file(
            ".kamu/config.yaml",
            indoc::indoc!(
                r#"
                encryption:
                    algo: Rsa
                "#
            ),
        )?;

        let err = get_config_figment()
            .extract::<MyConfig>()
            .expect_err("Expected error");

        assert_eq!(err.kind, figment2::error::Kind::MissingField("key".into()));
        assert_eq!(err.path, ["encryption"]);

        Ok(())
    });

    // Required field present
    setty::test::Jail::expect_with(|j| {
        j.create_dir(".kamu")?;
        j.create_file(
            ".kamu/config.yaml",
            indoc::indoc!(
                r#"
                encryption:
                    key: secret
                    algo: Rsa
                "#
            ),
        )?;

        let cfg: MyConfig = get_config_figment().extract()?;

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
        Ok(())
    });
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_default() {
    #[derive(setty::Config, setty::Default)]
    struct Config {
        #[config(default = 42)]
        foo: u32,

        #[config(default_parse = "42")]
        bar: u32,

        baz: Baz,
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
            baz: Baz::Y
        }
    );

    let cfg: Config = figment2::Figment::new().extract().unwrap();
    assert_eq!(
        cfg,
        Config {
            foo: 42,
            bar: 42,
            baz: Baz::Y
        }
    );
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_serde_rename_unit() {
    #[derive(setty::Config)]
    struct A {
        #[config(required)]
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
        .with_source_str::<setty::format::Yaml>(r#"b: baz"#)
        .extract()
        .unwrap();

    assert_eq!(cfg, A { b: B::Bar });
}

/////////////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_serde_rename_enum() {
    #[derive(setty::Config)]
    struct A {
        #[config(required)]
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
        .with_source_str::<setty::format::Yaml>(
            r#"
            b:
                type: baz
            "#,
        )
        .extract()
        .unwrap();

    assert_eq!(cfg, A { b: B::Bar(C {}) });

    // Setty will generate case permutations for alias as well
    let cfg: A = setty::Config::new()
        .with_source_str::<setty::format::Yaml>(
            r#"
            b:
                type: foobar
            "#,
        )
        .extract()
        .unwrap();

    assert_eq!(cfg, A { b: B::Bar(C {}) });
}

/////////////////////////////////////////////////////////////////////////////////////////
