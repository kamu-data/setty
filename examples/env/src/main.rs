#![allow(unused)]

use setty::format::{Json, Toml};
use setty::source::{Env, File};

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
struct Cfg {
    /// Database config
    #[config(default = DatabaseCfg::Sqlite(DatabaseCfgSqlite::default()))]
    database: DatabaseCfg,

    /// Connection timeout
    #[config(default_str = "15s")]
    connection_timeout: setty::types::DurationString,
}

#[derive(setty::Config)]
#[serde(tag = "provider")]
enum DatabaseCfg {
    /// Sqlite provider
    Sqlite(DatabaseCfgSqlite),

    /// Postgres provider
    Postgres(DatabaseCfgPostgres),
}

#[derive(setty::Config, setty::Default)]
struct DatabaseCfgSqlite {
    /// Location of DB file
    #[config(default = "db.sqlite")]
    #[schemars(with = "String")]
    path: std::path::PathBuf,
}

#[setty::derive(setty::Config)]
struct DatabaseCfgPostgres {
    /// Host URL
    host: String,

    /// Username
    user: String,

    /// Password
    #[schemars(with = "String")]
    password: secrecy::SecretString,
}

/////////////////////////////////////////////////////////////////////////////////////////

fn main() -> color_eyre::Result<()> {
    let base_path = std::path::PathBuf::from("examples/env");

    let cfg: Cfg = setty::Config::new()
        .with_source(File::<Toml>::new(base_path.join("config.toml")))
        .with_source(Env::<Json>::new("MY_CFG__", "__"))
        .extract()?;

    eprintln!("Loaded config:\n{cfg:#?}");

    Ok(())
}
