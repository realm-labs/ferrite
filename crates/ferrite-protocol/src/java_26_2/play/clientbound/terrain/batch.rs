//! Client chunk-batch timing feedback, matching the Java 26.2 estimator.

const INITIAL_NANOS_PER_CHUNK: f64 = 2_000_000.0;
const TARGET_NANOS_PER_BATCH: f64 = 7_000_000.0;
const MAX_OLD_WEIGHT: i32 = 49;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChunkBatchCalculator {
    aggregated_nanos_per_chunk: f64,
    old_sample_weight: i32,
    batch_start_nanos: i64,
}

impl ChunkBatchCalculator {
    #[must_use]
    pub const fn new(now_nanos: i64) -> Self {
        Self {
            aggregated_nanos_per_chunk: INITIAL_NANOS_PER_CHUNK,
            old_sample_weight: 1,
            batch_start_nanos: now_nanos,
        }
    }

    pub const fn on_batch_start(&mut self, now_nanos: i64) {
        self.batch_start_nanos = now_nanos;
    }

    pub fn on_batch_finished(&mut self, batch_size: i32, now_nanos: i64) -> f32 {
        if batch_size > 0 {
            let elapsed = now_nanos.wrapping_sub(self.batch_start_nanos) as f64;
            let sample = elapsed / f64::from(batch_size);
            let lower = self.aggregated_nanos_per_chunk / 3.0;
            let upper = self.aggregated_nanos_per_chunk * 3.0;
            let clamped = sample.max(lower).min(upper);
            let weight = f64::from(self.old_sample_weight);
            self.aggregated_nanos_per_chunk =
                (self.aggregated_nanos_per_chunk * weight + clamped) / (weight + 1.0);
            self.old_sample_weight = (self.old_sample_weight + 1).min(MAX_OLD_WEIGHT);
        }
        self.desired_chunks_per_tick()
    }

    #[must_use]
    pub fn desired_chunks_per_tick(self) -> f32 {
        (TARGET_NANOS_PER_BATCH / self.aggregated_nanos_per_chunk) as f32
    }

    #[must_use]
    pub const fn old_sample_weight(self) -> i32 {
        self.old_sample_weight
    }

    #[must_use]
    pub const fn batch_start_nanos(self) -> i64 {
        self.batch_start_nanos
    }

    #[must_use]
    pub const fn aggregated_nanos_per_chunk(self) -> f64 {
        self.aggregated_nanos_per_chunk
    }
}
