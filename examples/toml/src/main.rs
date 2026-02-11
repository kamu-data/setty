use setty::format::Toml;
use setty::source::File;

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(setty::Config, setty::Default)]
struct Cfg {
    /// Connection timeout
    #[config(default_str = "15s")]
    connection_timeout: setty::types::DurationString,

    /// Indexing mode
    #[config(default = Mode::Serial)]
    mode: Mode,

    /// List of targets to index
    #[config(default, combine(merge))]
    targets: Vec<Target>,
}

#[derive(setty::Config)]
struct Target {
    /// Host URL
    host: String,

    #[config(default = 100)]
    priority: usize,
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
    let base_path = std::path::PathBuf::from("examples/toml");

    let cfg: Cfg = setty::Config::new()
        .with_source(File::<Toml>::new(base_path.join("config1.toml")))
        .with_source(File::<Toml>::new(base_path.join("config2.toml")))
        .extract()?;

    eprintln!("Loaded config:\n{cfg:#?}");

    Ok(())
}
