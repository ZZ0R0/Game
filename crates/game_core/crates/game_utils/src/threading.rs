//! Threader global + file bornée + profilage par label.
//! - n_threads = available_parallelism() (fallback num_cpus)
//! - qcap = 4096
//! - Macros: job!(label, expr) → Receiver<T>, job_do!(label, expr) → fire-and-forget
//! - Reports: get_job_report(label), get_all_job_reports(), reset_*()

use crossbeam_channel::{bounded, Receiver, Sender};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use rayon::ThreadPoolBuilder;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

type Job = Box<dyn FnOnce() + Send + 'static>;

// ---------- Profilage ----------
#[derive(Clone, Debug)]
pub struct JobReport {
    pub label: String,
    pub runs: u64,
    pub avg_ms: f64,
    pub max_ms: f64,
    pub total_ms: f64,
}

#[derive(Default, Debug)]
struct RawStats {
    runs: u64,
    total_ns: u128,
    max_ns: u128,
}

static STATS: Lazy<DashMap<String, RawStats>> = Lazy::new(|| DashMap::new());

#[inline]
fn record_duration(label: &str, dur: Duration) {
    let ns = dur.as_nanos();
    let mut e = STATS
        .entry(label.to_string())
        .or_insert_with(RawStats::default);
    e.runs += 1;
    e.total_ns += ns;
    if ns > e.max_ns {
        e.max_ns = ns;
    }
}

#[inline]
fn to_report(label: &str, s: &RawStats) -> JobReport {
    let total_ms = s.total_ns as f64 / 1e6;
    let avg_ms = if s.runs > 0 {
        total_ms / s.runs as f64
    } else {
        0.0
    };
    let max_ms = s.max_ns as f64 / 1e6;
    JobReport {
        label: label.to_string(),
        runs: s.runs,
        avg_ms,
        max_ms,
        total_ms,
    }
}

pub fn get_job_report(label: &str) -> Option<JobReport> {
    STATS.get(label).map(|g| to_report(label, &*g))
}

pub fn get_all_job_reports() -> Vec<JobReport> {
    STATS.iter().map(|kv| to_report(kv.key(), &*kv)).collect()
}

pub fn reset_job_report(label: &str) {
    STATS.remove(label);
}

pub fn reset_all_job_reports() {
    STATS.clear();
}

fn auto_threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or_else(|_| num_cpus::get())
        .max(1)
}

// ---------- Threader ----------
pub struct Threader {
    tx: Sender<Job>,
    _dispatcher: JoinHandle<()>,
}

impl Threader {
    fn new(queue_cap: usize) -> Self {
        static POOL_INIT: Lazy<()> = Lazy::new(|| {
            let n = auto_threads();
            ThreadPoolBuilder::new()
                .num_threads(n)
                .build_global()
                .expect("rayon global pool");
        });
        let _ = *POOL_INIT;

        let (tx, rx) = bounded::<Job>(queue_cap);

        let dispatcher = thread::spawn(move || {
            while let Ok(job) = rx.recv() {
                // FIFO pour limiter la famine en burst
                rayon::spawn_fifo(job);
            }
        });

        Self {
            tx,
            _dispatcher: dispatcher,
        }
    }

    /// Envoi simple (bloque si file pleine).
    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.tx.send(Box::new(f)).expect("dispatcher stopped");
    }

    /// Envoi avec résultat.
    pub fn submit_result<F, T>(&self, f: F) -> Receiver<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (rtx, rrx) = bounded(1);
        self.submit(move || {
            let out = f();
            let _ = rtx.send(out);
        });
        rrx
    }

    /// Envoi profilé sans résultat.
    pub fn submit_profiled_do<F>(&self, label: &'static str, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.submit(move || {
            let t0 = Instant::now();
            f();
            record_duration(label, t0.elapsed());
        });
    }

    /// Envoi profilé avec résultat.
    pub fn submit_profiled_result<F, T>(&self, label: &'static str, f: F) -> Receiver<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (rtx, rrx) = bounded(1);
        self.submit(move || {
            let t0 = Instant::now();
            let out = f();
            record_duration(label, t0.elapsed());
            let _ = rtx.send(out);
        });
        rrx
    }
}

// Global singleton: qcap = 4096
static GLOBAL_THREADER: Lazy<Threader> = Lazy::new(|| Threader::new(4096));

pub fn global() -> &'static Threader {
    &*GLOBAL_THREADER
}

// ---------- Macros ergonomiques ----------

#[macro_export]
macro_rules! job {
    ($label:expr, $e:expr) => {{
        $crate::threading::global().submit_profiled_result($label, move || $e)
    }};
}

#[macro_export]
macro_rules! job_do {
    ($label:expr, $e:expr) => {{
        $crate::threading::global().submit_profiled_do($label, move || $e)
    }};
}
