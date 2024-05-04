use std::path::PathBuf;

use figment::value::{Dict, Map};
use serde_default_utils::*;
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, clap::Parser, Serialize)]
pub struct Args {
    /// Config files to load.
    #[arg(long)]
    #[serde(skip)]
    pub config: Vec<PathBuf>,

    #[arg(long)]
    #[serde(skip)]
    pub output: Option<PathBuf>,

    /// The total number of test iterations that will be performed.
    #[arg(long)]
    pub iterations: Option<u64>,

    /// The number of iterations that will be performed before taking a time
    /// measurement.
    ///
    /// Increasing the batch size means that measurements will be more stable
    /// since they will be the average runtime of `batch_size` iterations.
    #[arg(long)]
    pub batch_size: Option<u64>,

    /// The number of threads to use when benchmarking.
    #[arg(long)]
    pub threads: Option<u64>,

    /// The workload that will be run
    #[arg(long)]
    pub workload: Option<WorkloadType>,

    /// The type of mutex that will be benchmarked.
    #[arg(long)]
    pub mutex: Option<MutexType>,

    #[arg(long)]
    #[serde(skip)]
    pub dump_config: bool,
}

impl figment::Provider for Args {
    fn metadata(&self) -> figment::Metadata {
        figment::Metadata::named("command line")
    }

    fn data(&self) -> figment::Result<Map<figment::Profile, Dict>> {
        figment::providers::Serialized::defaults(self.clone()).data()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The total number of test iterations that will be performed.
    pub iterations: u64,

    /// The number of iterations that will be performed before taking a time
    /// measurement.
    ///
    /// Increasing the batch size means that measurements will be more stable
    /// since they will be the average runtime of `batch_size` iterations.
    #[serde(default = "default_u64::<1024>")]
    pub batch_size: u64,

    /// The number of warmup iterations before running the actual test.
    pub warmup: u64,

    /// The number of threads to use when benchmarking.
    pub threads: u64,

    /// The workload that will be run
    pub workload: WorkloadType,

    /// The type of mutex that will be benchmarked.
    pub mutex: MutexType,

    #[serde(default)]
    pub workloads: WorkloadConfig,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkloadType {
    /// The simplest possible workload.
    ///
    /// All it does is increment the mutex state by 1.
    Increment,

    /// Copy a configurable amount of memory into the mutex.
    Memcpy,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MutexType {
    Std,
    ParkingLot,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct WorkloadConfig {
    pub memcpy: MemcpyConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemcpyConfig {
    #[serde(default = "default_usize::<1024>")]
    pub bytes: usize,
}

impl Default for MemcpyConfig {
    fn default() -> Self {
        serde_json::from_value(serde_json::Map::new().into()).unwrap()
    }
}
