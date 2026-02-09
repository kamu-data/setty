use setty::format::Yaml;
use setty::source::File;

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
struct Cfg {
    /// Connection timeout
    #[config(default_str = "15s")]
    connection_timeout: setty::types::DurationString,

    /// List of hosts to index
    #[config(default, combine(merge))]
    hosts: Vec<url::Url>,

    /// Indexing mode
    #[config(default = Mode::Serial)]
    mode: Mode,
}

#[derive(setty::Config)]
enum Mode {
    /// Index in a single thread
    Serial,

    /// Index in multiple threads
    Parallel(ModeParallel),
}

#[derive(setty::Config, setty::Default)]
struct ModeParallel {
    /// Maximum concurrent requests
    #[config(default = 10)]
    concurrency: usize,
}

/////////////////////////////////////////////////////////////////////////////////////////

fn main() -> color_eyre::Result<()> {
    let base_path = std::path::PathBuf::from("examples/jsonschema");

    let cfg: Cfg = setty::Config::new()
        .with_source(File::<Yaml>::new(base_path.join("config.yaml")))
        .extract()?;

    eprintln!("Loaded config:\n{cfg:#?}");

    eprintln!("Generating JSON Schema");
    let json_schema = setty::Config::<Cfg>::new().json_schema().to_string_pretty();
    std::fs::write(base_path.join("config-schema.json"), json_schema)?;

    eprintln!("Generating Markdown");
    let markdown = setty::Config::<Cfg>::new().markdown();
    std::fs::write(base_path.join("config-readme.md"), markdown)?;

    Ok(())
}
