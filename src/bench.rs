use std::fmt;
use std::sync::Barrier;
use std::time::{Duration, Instant};

use anyhow::Context;
use histogram::{AtomicHistogram, SparseHistogram};

use crate::config::Config;

pub trait Workload {
    type State: Send;

    fn new(config: &Config) -> Self;
    fn new_state(config: &Config) -> Self::State;

    fn work(&mut self, state: &mut Self::State);
}

pub trait Mutex<M> {
    fn new(value: M) -> Self;
    fn with<F: FnOnce(&mut M)>(&self, f: F);
}

pub struct Bencher {
    pub latencies: AtomicHistogram,
}

impl Bencher {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            latencies: AtomicHistogram::new(8, 64)
                .context("failed to construct latency histogram")?,
        })
    }

    pub fn bench<M, W>(&mut self, config: &Config)
    where
        W: Workload,
        M: Mutex<W::State> + Sync,
    {
        for _ in 0..config.warmup {
            self.bench_one::<M, W>(config);
        }

        for _ in 0..config.iterations {
            let duration = self.bench_one::<M, W>(config);
            let average = ((duration.as_nanos() * 1000) / config.batch_size as u128) as u64;

            self.latencies
                .increment(average)
                .expect("failed to increment latency bucket");
        }
    }

    fn bench_one<M, W>(&mut self, config: &Config) -> Duration
    where
        W: Workload,
        M: Mutex<W::State> + Sync,
    {
        let mutex = M::new(W::new_state(config));
        let barrier = Barrier::new(config.threads as usize);

        std::thread::scope(|scope| {
            let mut threads = Vec::new();
            for index in 0..config.threads {
                threads.push(scope.spawn({
                    let index = Box::new(index);
                    || {
                        let index = index;
                        self.run_bench::<M, W>(config, &mutex, &barrier, *index)
                    }
                }))
            }

            let mut total = Duration::ZERO;
            for thread in threads {
                let duration = match thread.join() {
                    Ok(duration) => duration,
                    Err(payload) => std::panic::resume_unwind(payload),
                };

                total += duration;
            }

            total
        })
    }

    fn run_bench<M, W>(&self, config: &Config, mutex: &M, barrier: &Barrier, index: u64) -> Duration
    where
        W: Workload,
        M: Mutex<W::State>,
    {
        let mut workload = W::new(config);

        barrier.wait();
        let start = Instant::now();
        for _ in (index..config.batch_size).step_by(config.threads as _) {
            mutex.with(|state| workload.work(state));
        }

        start.elapsed()
    }

    pub fn report(&self) -> Report {
        let snapshot = self.latencies.load();

        Report {
            latencies: (&snapshot).into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub latencies: SparseHistogram,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Latency Distribution")?;
        writeln!(
            f,
            "  p01:   {}",
            DisplayBucket(self.latencies.percentile(1.0).unwrap())
        )?;
        writeln!(
            f,
            "  p10:   {}",
            DisplayBucket(self.latencies.percentile(10.0).unwrap())
        )?;
        writeln!(
            f,
            "  p50:   {}",
            DisplayBucket(self.latencies.percentile(50.0).unwrap())
        )?;
        writeln!(
            f,
            "  p90:   {}",
            DisplayBucket(self.latencies.percentile(90.0).unwrap())
        )?;
        writeln!(
            f,
            "  p95:   {}",
            DisplayBucket(self.latencies.percentile(95.0).unwrap())
        )?;
        writeln!(
            f,
            "  p99:   {}",
            DisplayBucket(self.latencies.percentile(99.0).unwrap())
        )?;

        Ok(())
    }
}

struct DisplayBucket(histogram::Bucket);

impl fmt::Display for DisplayBucket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ps = self.0.end();

        match ps {
            _ if ps < 1000 => write!(f, "{ps} ps"),
            _ if ps < 100000 => write!(f, "{}.{:02} ns", ps / 1000, ((ps % 1000) + 5) / 10),
            _ => {
                let dur = Duration::from_nanos(self.0.end());
                write!(f, "{}", humantime::format_duration(dur / 1000))
            }
        }
    }
}
