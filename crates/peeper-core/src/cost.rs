// SPDX-License-Identifier: MIT
//! The per-target cost of a single verification attempt.

/// The measured cost of one `verify` call against a target.
///
/// This is the `c_KDF` that scheduling, ETA, and the resistance report compute
/// against; a fast hash and an Argon2id vault are never the same problem.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CostModel {
    /// Wall-clock seconds for one verification on one unit of parallelism.
    pub seconds_per_guess: f64,
    /// Peak working memory in bytes required by one verification.
    pub memory_bytes: u64,
    /// Independent lanes one verification is designed to occupy.
    pub parallelism: u32,
}

impl CostModel {
    /// Builds a cost model from its measured components.
    #[must_use]
    pub fn new(seconds_per_guess: f64, memory_bytes: u64, parallelism: u32) -> Self {
        Self {
            seconds_per_guess,
            memory_bytes,
            parallelism,
        }
    }

    /// Verifications per second achievable on one unit of parallelism.
    #[must_use]
    pub fn guesses_per_second(&self) -> f64 {
        if self.seconds_per_guess > 0.0 {
            1.0 / self.seconds_per_guess
        } else {
            f64::INFINITY
        }
    }

    /// The time–area product `memory_bytes * seconds_per_guess`, Argon2's cost invariant.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn time_area(&self) -> f64 {
        self.seconds_per_guess * (self.memory_bytes as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::CostModel;

    #[test]
    fn guesses_per_second_inverts_latency() {
        let fast = CostModel::new(1.0e-9, 0, 1);
        assert!((fast.guesses_per_second() - 1.0e9).abs() / 1.0e9 < 1.0e-9);
    }

    #[test]
    fn zero_latency_is_infinite_rate() {
        let c = CostModel::new(0.0, 0, 1);
        assert!(c.guesses_per_second().is_infinite());
    }

    #[test]
    fn time_area_multiplies_memory_and_time() {
        let kdf = CostModel::new(0.5, 64 * 1024 * 1024, 1);
        assert!((kdf.time_area() - 0.5 * 64.0 * 1024.0 * 1024.0).abs() < 1.0);
    }
}
