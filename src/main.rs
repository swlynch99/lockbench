use anyhow::Context;
use clap::Parser;
use config::WorkloadType;
use figment::providers::Format;

use crate::bench::{Bencher, Workload};
use crate::config::{Args, Config, MutexType};

#[macro_use]
extern crate serde;

mod bench;
mod config;
mod mutex;
mod workload;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut config = figment::Figment::new();

    for path in &args.config {
        config = config.admerge(figment::providers::Toml::file_exact(path));
    }
    config = config.admerge(args.clone());

    let config: Config = config.extract()?;
    if args.dump_config {
        let config = toml::to_string_pretty(&config) //
            .context("failed to serialize the config to toml")?;

        println!("{config}");
    }

    let bencher = match config.workload {
        WorkloadType::Increment => run_with_mutex::<workload::Increment>(&config),
        WorkloadType::Memcpy => run_with_mutex::<workload::Memcpy>(&config),
    }?;

    let report = bencher.report();

    print!("{report}");

    if let Some(output) = &args.output {
        let json = serde_json::to_string_pretty(&report)?;

        std::fs::write(&output, json).with_context(|| {
            format!("failed to write the report json to `{}`", output.display())
        })?;
    }

    Ok(())
}

fn run_with_mutex<W: Workload>(config: &Config) -> anyhow::Result<Bencher> {
    let mut bencher = Bencher::new()?;

    match config.mutex {
        MutexType::Std => bencher.bench::<std::sync::Mutex<W::State>, W>(config),
        MutexType::ParkingLot => bencher.bench::<parking_lot::Mutex<W::State>, W>(config),
    }

    Ok(bencher)
}
