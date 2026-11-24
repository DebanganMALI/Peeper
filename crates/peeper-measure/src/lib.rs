// SPDX-License-Identifier: MIT
//! Measurement core: turn a password distribution into entropy and guesswork metrics.

mod cost;
mod distribution;

pub use cost::{cost_to_crack_seconds, expected_attack_work};
pub use distribution::{alpha_work_factor, guessing_entropy, min_entropy, shannon_entropy};
