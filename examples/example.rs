use std::{collections::BTreeMap, path::PathBuf};

use clap::Parser;
use clap_verbosity_flag::Verbosity;
use storecfg::{StoreArgs, StoreConfigCollector};

#[derive(Debug, Parser)]
struct Cli {
    #[command(flatten)]
    store: StoreArgs,
    #[clap(long, short)]
    config: Vec<PathBuf>,
    #[command(flatten)]
    verbosity: Verbosity,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbosity.into())
        .init();

    let mut collector = StoreConfigCollector::default();
    collector.add_config("", cli.store.into());

    for path in cli.config {
        log::info!("Reading config file: {}", path.display());

        let contents = std::fs::read_to_string(path)?;
        let configs: BTreeMap<String, BTreeMap<String, String>> = toml::from_str(&contents)?;
        for (name, config) in configs {
            for (key, value) in config {
                log::debug!("Adding config: {name} {key}={value}");
                collector.add(&name, &key, value);
            }
        }
    }
    collector.add_from_env("STORAGE")?;

    let (configs, invalids) = collector.into_configs(true);
    if !invalids.is_empty() {
        eprintln!("Missing URI: {invalids:?}");
    }
    for (name, config) in configs {
        println!("Service: {name}");
        println!("\turi: {}", config.uri);
        for (k, v) in &config.options {
            println!("\t{k}: {v}");
        }
        match opendal::Operator::from_uri(config.into_tuple()) {
            Ok(op) => println!("\tVALID CONFIG: {}", op.info().scheme()),
            Err(e) => println!("\tINVALID CONFIG: {e}"),
        }
    }
    Ok(())
}
