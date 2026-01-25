#![cfg(feature = "derive-deserialize")]

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, Default)]
pub struct CLIConfig {
    /// Database configuration
    database: DatabaseConfig,

    /// Optional encryption
    encryption: Option<EncryptionConfig>,
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config)]
pub enum DatabaseConfig {
    Sqlite(SqliteDatabaseConfig),
    Postgres(PostgresDatabaseConfig),
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self::Sqlite(SqliteDatabaseConfig {
            database_path: ".kamu/db.sqlite".into(),
        })
    }
}

#[derive(setty::Config)]
pub struct SqliteDatabaseConfig {
    /// Path to the database file
    database_path: String,
}

#[derive(setty::Config)]
pub struct PostgresDatabaseConfig {
    /// Name of the schema
    schema_name: String,

    /// Host name
    #[config(default = "localhost")]
    host: String,
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config)]
pub struct EncryptionConfig {
    /// Encryption key
    #[config(required)]
    key: String,

    /// Encryption algorythm
    algo: EncryptionAlgo,
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
    figment2::Jail::expect_with(|_| {
        let cfg: CLIConfig = get_config_figment().extract()?;

        assert_eq!(
            cfg,
            CLIConfig {
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
    figment2::Jail::expect_with(|j| {
        j.create_dir(".kamu")?;
        j.create_file(".kamu/config.yaml", r#""#)?;

        let cfg: CLIConfig = get_config_figment().extract()?;

        assert_eq!(
            cfg,
            CLIConfig {
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
    figment2::Jail::expect_with(|j| {
        j.create_dir(".kamu")?;
        j.create_file(
            ".kamu/config.yaml",
            indoc::indoc!(
                r#"
                database:
                    kind: postgres
                    schemaName: foo
                "#
            ),
        )?;

        let cfg: CLIConfig = get_config_figment().extract()?;

        assert_eq!(
            cfg,
            CLIConfig {
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
    figment2::Jail::expect_with(|j| {
        j.set_env("KAMU_CFG_database__kind", "postgres");
        j.set_env("KAMU_CFG_database__schemaName", "bar");

        let cfg: CLIConfig = get_config_figment().extract()?;

        assert_eq!(
            cfg,
            CLIConfig {
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
    figment2::Jail::expect_with(|j| {
        j.create_dir(".kamu")?;
        j.create_file(
            ".kamu/config.yaml",
            indoc::indoc!(
                r#"
                database:
                    kind: postgres
                    schemaNamez: foo
                "#
            ),
        )?;

        let err = get_config_figment()
            .extract::<CLIConfig>()
            .expect_err("Expected error");

        assert_eq!(
            err.kind,
            figment2::error::Kind::UnknownField("schemaNamez".into(), &["schemaName", "host"])
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
    figment2::Jail::expect_with(|j| {
        j.create_dir(".kamu")?;
        j.create_file(
            ".kamu/config.yaml",
            indoc::indoc!(
                r#"
                encryption:
                    algo: rsa
                "#
            ),
        )?;

        let err = get_config_figment()
            .extract::<CLIConfig>()
            .expect_err("Expected error");

        assert_eq!(err.kind, figment2::error::Kind::MissingField("key".into()));
        assert_eq!(err.path, ["encryption"]);

        Ok(())
    });

    // Required field present
    figment2::Jail::expect_with(|j| {
        j.create_dir(".kamu")?;
        j.create_file(
            ".kamu/config.yaml",
            indoc::indoc!(
                r#"
                encryption:
                    key: secret
                    algo: rsa
                "#
            ),
        )?;

        let cfg: CLIConfig = get_config_figment().extract()?;

        assert_eq!(
            cfg,
            CLIConfig {
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
