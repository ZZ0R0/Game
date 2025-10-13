//! Performance metrics for terrain generation
//!
//! Tracks detailed timing information for each generation phase

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Detailed metrics for a single generation operation
#[derive(Debug, Clone)]
pub struct GenerationSample {
    pub underground_check_time_us: f32,
    pub underground_fill_time_us: f32,
    pub height_calculation_time_us: f32,
    pub block_placement_time_us: f32,
    pub total_time_us: f32,
    pub was_underground: bool,
    pub timestamp: Instant,
}

/// Thread-safe metrics collector for terrain generation
#[derive(Debug, Clone)]
pub struct GeneratorMetrics {
    samples: Arc<Mutex<VecDeque<GenerationSample>>>,
    max_samples: usize,
}

impl GeneratorMetrics {
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: Arc::new(Mutex::new(VecDeque::with_capacity(max_samples))),
            max_samples,
        }
    }

    /// Add a new sample to the metrics
    pub fn add_sample(&self, sample: GenerationSample) {
        let mut samples = self.samples.lock().unwrap();
        samples.push_back(sample);
        if samples.len() > self.max_samples {
            samples.pop_front();
        }
    }

    /// Get aggregated statistics for the last second
    pub fn get_stats(&self, worker_count: usize) -> GeneratorStats {
        let samples = self.samples.lock().unwrap();
        let now = Instant::now();
        let one_sec_ago = now - Duration::from_secs(1);

        // Filter samples from last second
        let recent_samples: Vec<_> = samples
            .iter()
            .filter(|s| s.timestamp > one_sec_ago)
            .collect();

        if recent_samples.is_empty() {
            return GeneratorStats::default();
        }

        let count = recent_samples.len() as f32;
        let underground_count = recent_samples.iter().filter(|s| s.was_underground).count() as f32;

        // Calculate averages (in microseconds) - same as PerformanceMonitor
        let avg_underground_check_us: f32 = recent_samples.iter().map(|s| s.underground_check_time_us).sum::<f32>() / count;
        let avg_underground_fill_us: f32 = recent_samples.iter().map(|s| s.underground_fill_time_us).sum::<f32>() / count;
        let avg_height_calc_us: f32 = recent_samples.iter().map(|s| s.height_calculation_time_us).sum::<f32>() / count;
        let avg_block_placement_us: f32 = recent_samples.iter().map(|s| s.block_placement_time_us).sum::<f32>() / count;
        let avg_total_us: f32 = recent_samples.iter().map(|s| s.total_time_us).sum::<f32>() / count;

        // Convert to milliseconds for consistency with meshing metrics
        let avg_underground_check_ms = avg_underground_check_us / 1000.0;
        let avg_underground_fill_ms = avg_underground_fill_us / 1000.0;
        let avg_height_calc_ms = avg_height_calc_us / 1000.0;
        let avg_block_placement_ms = avg_block_placement_us / 1000.0;
        let avg_total_ms = avg_total_us / 1000.0;

        // Chunks per second
        let chunks_per_sec = count;

        // SAME CALCULATION AS PerformanceMonitor:
        // total_work_per_sec_ms = chunks_per_sec × avg_time_ms
        let total_underground_check_work_ms = chunks_per_sec * avg_underground_check_ms;
        let total_underground_fill_work_ms = chunks_per_sec * avg_underground_fill_ms;
        let total_height_calc_work_ms = chunks_per_sec * avg_height_calc_ms;
        let total_block_placement_work_ms = chunks_per_sec * avg_block_placement_ms;
        let total_generation_work_ms = chunks_per_sec * avg_total_ms;

        // SAME CALCULATION AS PerformanceMonitor:
        // work_per_worker_ms = total_work_per_sec_ms / worker_count
        let underground_check_work_per_worker_ms = total_underground_check_work_ms / worker_count as f32;
        let underground_fill_work_per_worker_ms = total_underground_fill_work_ms / worker_count as f32;
        let height_calc_work_per_worker_ms = total_height_calc_work_ms / worker_count as f32;
        let block_placement_work_per_worker_ms = total_block_placement_work_ms / worker_count as f32;
        let total_work_per_worker_ms = total_generation_work_ms / worker_count as f32;

        GeneratorStats {
            chunks_per_sec,
            underground_ratio: underground_count / count,
            
            // Average times per chunk (ms)
            avg_underground_check_ms,
            avg_underground_fill_ms,
            avg_height_calc_ms,
            avg_block_placement_ms,
            avg_total_ms,

            // Total work per second (ms/sec)
            total_underground_check_work_ms,
            total_underground_fill_work_ms,
            total_height_calc_work_ms,
            total_block_placement_work_ms,
            total_generation_work_ms,

            // Work per worker (ms/sec per worker)
            underground_check_work_per_worker_ms,
            underground_fill_work_per_worker_ms,
            height_calc_work_per_worker_ms,
            block_placement_work_per_worker_ms,
            total_work_per_worker_ms,

            worker_count,
        }
    }

    /// Clear all samples
    pub fn clear(&self) {
        let mut samples = self.samples.lock().unwrap();
        samples.clear();
    }
}

/// Aggregated statistics for terrain generation
#[derive(Debug, Clone)]
pub struct GeneratorStats {
    pub chunks_per_sec: f32,
    pub underground_ratio: f32,

    // Average times per chunk (milliseconds)
    pub avg_underground_check_ms: f32,
    pub avg_underground_fill_ms: f32,
    pub avg_height_calc_ms: f32,
    pub avg_block_placement_ms: f32,
    pub avg_total_ms: f32,

    // Total work per second (ms/sec)
    pub total_underground_check_work_ms: f32,
    pub total_underground_fill_work_ms: f32,
    pub total_height_calc_work_ms: f32,
    pub total_block_placement_work_ms: f32,
    pub total_generation_work_ms: f32,

    // Work per worker (ms/sec per worker)
    pub underground_check_work_per_worker_ms: f32,
    pub underground_fill_work_per_worker_ms: f32,
    pub height_calc_work_per_worker_ms: f32,
    pub block_placement_work_per_worker_ms: f32,
    pub total_work_per_worker_ms: f32,

    pub worker_count: usize,
}

impl Default for GeneratorStats {
    fn default() -> Self {
        Self {
            chunks_per_sec: 0.0,
            underground_ratio: 0.0,
            avg_underground_check_ms: 0.0,
            avg_underground_fill_ms: 0.0,
            avg_height_calc_ms: 0.0,
            avg_block_placement_ms: 0.0,
            avg_total_ms: 0.0,
            total_underground_check_work_ms: 0.0,
            total_underground_fill_work_ms: 0.0,
            total_height_calc_work_ms: 0.0,
            total_block_placement_work_ms: 0.0,
            total_generation_work_ms: 0.0,
            underground_check_work_per_worker_ms: 0.0,
            underground_fill_work_per_worker_ms: 0.0,
            height_calc_work_per_worker_ms: 0.0,
            block_placement_work_per_worker_ms: 0.0,
            total_work_per_worker_ms: 0.0,
            worker_count: 0,
        }
    }
}

impl GeneratorStats {
    /// Format stats for display in overlay
    pub fn format_overlay(&self) -> String {
        format!(
            "🔨 GENERATOR BREAKDOWN:\n\
            • Chunks/sec: {:.1}\n\
            • Underground ratio: {:.1}%\n\
            \n\
            ⏱️  AVG TIME PER CHUNK:\n\
            • Underground check: {:.3}ms\n\
            • Underground fill: {:.3}ms\n\
            • Height calculation: {:.3}ms\n\
            • Block placement: {:.3}ms\n\
            • Total: {:.3}ms\n\
            \n\
            💼 TOTAL WORK/SEC:\n\
            • Underground check: {:.1}ms/sec\n\
            • Underground fill: {:.1}ms/sec\n\
            • Height calc: {:.1}ms/sec\n\
            • Block placement: {:.1}ms/sec\n\
            • Total: {:.1}ms/sec\n\
            \n\
            👷 WORK PER WORKER:\n\
            • Underground check: {:.1}ms/sec\n\
            • Underground fill: {:.1}ms/sec\n\
            • Height calc: {:.1}ms/sec\n\
            • Block placement: {:.1}ms/sec\n\
            • Total: {:.1}ms/sec",
            self.chunks_per_sec,
            self.underground_ratio * 100.0,
            self.avg_underground_check_ms,
            self.avg_underground_fill_ms,
            self.avg_height_calc_ms,
            self.avg_block_placement_ms,
            self.avg_total_ms,
            self.total_underground_check_work_ms,
            self.total_underground_fill_work_ms,
            self.total_height_calc_work_ms,
            self.total_block_placement_work_ms,
            self.total_generation_work_ms,
            self.underground_check_work_per_worker_ms,
            self.underground_fill_work_per_worker_ms,
            self.height_calc_work_per_worker_ms,
            self.block_placement_work_per_worker_ms,
            self.total_work_per_worker_ms
        )
    }
}

/// Helper to measure execution time in microseconds
#[inline]
pub fn measure_us<F, R>(f: F) -> (R, f32)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed().as_micros() as f32;
    (result, elapsed)
}
