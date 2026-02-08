#![cfg(feature = "derive-deserialize")]

use super::test_deserialize::*;

/////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "fmt-yaml")]
#[test]
fn test_env_var_override_nested() {
    // Note: Lowercase `postgres` works because of `case-enums-any` feature
    let _a = set_env_var("KAMU_CFG_database__kind", "postgres");
    let _b = set_env_var("KAMU_CFG_database__schema_name", "bar");

    let cfg: MyConfig = setty::Config::new()
        .with_source(setty::source::Env::<setty::format::Yaml>::new(
            "KAMU_CFG_",
            "__",
        ))
        .extract()
        .unwrap();

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
}

/////////////////////////////////////////////////////////////////////////////////////////

fn set_env_var(k: &'static str, v: &'static str) -> Unset {
    let unset = Unset(k);
    unsafe {
        std::env::set_var(k, v);
    }
    unset
}

#[must_use]
struct Unset(&'static str);

impl Drop for Unset {
    fn drop(&mut self) {
        unsafe {
            std::env::remove_var(self.0);
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
