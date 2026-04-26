// Benchmark plumbing. The actual GPU run lives in `graph.rs::bench_render`;
// this module just owns the result type and the canonical config list.

#[derive(Debug, Clone)]
pub struct BenchResult {
    pub label: String,
    pub width: u32,
    pub height: u32,
    /// Reserved for future per-config breakdowns (e.g. step-count vs ms
    /// curve plot). Currently surfaced only via the `label` text.
    #[allow(dead_code)]
    pub steps: u32,
    pub ms_median: f32,
}

impl BenchResult {
    pub fn fps(&self) -> f32 {
        if self.ms_median > 0.0 {
            1000.0 / self.ms_median
        } else {
            0.0
        }
    }
}

/// (label, width, height, steps). 2:1 equirect aspect throughout — matches
/// the export sizes a customer is likely to ship.
pub const BENCH_CONFIGS: &[(&str, u32, u32, u32)] = &[
    ("1K / 64",  1024,  512,  64),
    ("1K / 128", 1024,  512, 128),
    ("2K / 96",  2048, 1024,  96),
    ("4K / 96",  4096, 2048,  96),
    ("4K / 128", 4096, 2048, 128),
];

/// Number of warmup frames before timing kicks in (reset pipeline caches /
/// JIT effects).
pub const BENCH_WARMUP: u32 = 3;

/// Number of measured frames per config; we report the median.
pub const BENCH_RUNS: u32 = 7;
