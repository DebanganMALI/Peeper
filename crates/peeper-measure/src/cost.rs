// SPDX-License-Identifier: MIT
//! Turn guesswork into wall-clock cost against a target's measured KDF cost.

use peeper_core::CostModel;

use crate::distribution::guessing_entropy;

/// Expected attack work in seconds: `E[W] = (sum p_i * i) * c_KDF`.
///
/// The guessing entropy (optimal-order expected guesses) scaled by the measured
/// per-guess cost. This unifies a distribution and a KDF into one time figure.
#[must_use]
pub fn expected_attack_work(probabilities: &[f64], cost: &CostModel) -> f64 {
    guessing_entropy(probabilities) * cost.seconds_per_guess
}

/// Cost in seconds to reach a given alpha-work-factor: `C(alpha) = mu_alpha * c_KDF`.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn cost_to_crack_seconds(work_factor: usize, cost: &CostModel) -> f64 {
    (work_factor as f64) * cost.seconds_per_guess
}

#[cfg(test)]
mod tests {
    use super::{cost_to_crack_seconds, expected_attack_work};
    use peeper_core::CostModel;

    const EPS: f64 = 1.0e-9;

    #[test]
    fn expected_work_scales_guessing_entropy_by_kdf_cost() {
        // uniform4 guessing entropy = 2.5; at 0.5 s/guess -> 1.25 s.
        let uniform4 = [0.25, 0.25, 0.25, 0.25];
        let cost = CostModel::new(0.5, 0, 1);
        assert!((expected_attack_work(&uniform4, &cost) - 1.25).abs() < EPS);
    }

    #[test]
    fn fast_hash_dwarfs_memory_hard_kdf_for_same_guess_count() {
        let guesses = 1_000_000;
        let fast = cost_to_crack_seconds(guesses, &CostModel::new(1.0e-9, 0, 1));
        let argon = cost_to_crack_seconds(guesses, &CostModel::new(0.5, 64 << 20, 1));
        assert!(argon / fast > 1.0e8);
    }
}
