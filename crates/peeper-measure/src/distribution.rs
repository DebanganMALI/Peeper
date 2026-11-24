// SPDX-License-Identifier: MIT
//! Entropy and guesswork metrics over a discrete password distribution.

// All functions take a slice of probabilities. They do not assume it is sorted
// or exactly normalized; callers own the modeling, these own the arithmetic.

/// Shannon entropy in bits: a weak-attacker model, kept for contrast.
#[must_use]
pub fn shannon_entropy(probabilities: &[f64]) -> f64 {
    probabilities
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * p.log2())
        .sum()
}

/// Min-entropy in bits: `-log2(max p_i)`, the strength against one best guess.
#[must_use]
pub fn min_entropy(probabilities: &[f64]) -> f64 {
    let top = probabilities.iter().copied().fold(0.0_f64, f64::max);
    if top > 0.0 {
        -top.log2()
    } else {
        0.0
    }
}

/// Guessing entropy: expected guesses under optimal (descending-probability) order.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn guessing_entropy(probabilities: &[f64]) -> f64 {
    let mut sorted = probabilities.to_vec();
    sorted.sort_by(|a, b| b.total_cmp(a));
    sorted
        .iter()
        .enumerate()
        .map(|(index, &p)| ((index + 1) as f64) * p)
        .sum()
}

/// The alpha-work-factor: guesses needed to cover probability mass `alpha`.
///
/// This is the quantity our success-vs-guesses curve estimates directly (Bonneau).
/// Returns `None` when the distribution cannot reach `alpha`.
#[must_use]
pub fn alpha_work_factor(probabilities: &[f64], alpha: f64) -> Option<usize> {
    if alpha <= 0.0 {
        return Some(0);
    }
    let mut sorted = probabilities.to_vec();
    sorted.sort_by(|a, b| b.total_cmp(a));
    let mut cumulative = 0.0;
    for (index, &p) in sorted.iter().enumerate() {
        cumulative += p;
        if cumulative >= alpha {
            return Some(index + 1);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{alpha_work_factor, guessing_entropy, min_entropy, shannon_entropy};

    const EPS: f64 = 1.0e-9;

    fn uniform4() -> [f64; 4] {
        [0.25, 0.25, 0.25, 0.25]
    }

    #[test]
    fn shannon_of_uniform4_is_two_bits() {
        assert!((shannon_entropy(&uniform4()) - 2.0).abs() < EPS);
    }

    #[test]
    fn shannon_of_certainty_is_zero() {
        assert!(shannon_entropy(&[1.0]).abs() < EPS);
    }

    #[test]
    fn min_entropy_of_uniform4_is_two_bits() {
        assert!((min_entropy(&uniform4()) - 2.0).abs() < EPS);
    }

    #[test]
    fn min_entropy_tracks_the_single_best_guess() {
        // One dominant password: min-entropy is -log2(0.9), far below Shannon.
        let skewed = [0.9, 0.1];
        let expected = -(0.9_f64.log2());
        assert!((min_entropy(&skewed) - expected).abs() < EPS);
    }

    #[test]
    fn guessing_entropy_of_uniform4() {
        // (1+2+3+4) * 0.25 = 2.5
        assert!((guessing_entropy(&uniform4()) - 2.5).abs() < EPS);
    }

    #[test]
    fn guessing_entropy_ignores_input_order() {
        let a = guessing_entropy(&[0.1, 0.6, 0.3]);
        let b = guessing_entropy(&[0.6, 0.3, 0.1]);
        assert!((a - b).abs() < EPS);
    }

    #[test]
    fn alpha_work_factor_half_of_uniform4_is_two() {
        assert_eq!(alpha_work_factor(&uniform4(), 0.5), Some(2));
    }

    #[test]
    fn alpha_work_factor_zero_needs_no_guesses() {
        assert_eq!(alpha_work_factor(&uniform4(), 0.0), Some(0));
    }

    #[test]
    fn alpha_work_factor_unreachable_is_none() {
        assert_eq!(alpha_work_factor(&[0.3, 0.3], 0.9), None);
    }
}
