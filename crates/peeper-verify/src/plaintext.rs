// SPDX-License-Identifier: MIT
//! A trivial, crypto-free target used to exercise the pipeline end-to-end.

use peeper_core::{CostModel, Target, TargetKind};

/// A fixture target whose secret is compared in the clear.
///
/// Not a real oracle; it lets the generators, scheduler, and measurement run on
/// CPU before any crypto lands. Treated as a `FastHash` for cost purposes.
pub struct PlainTextTarget {
    secret: Vec<u8>,
}

impl PlainTextTarget {
    /// Wraps a known secret as a target.
    #[must_use]
    pub fn new(secret: impl Into<Vec<u8>>) -> Self {
        Self {
            secret: secret.into(),
        }
    }
}

impl Target for PlainTextTarget {
    fn verify(&self, candidate: &[u8]) -> bool {
        candidate == self.secret.as_slice()
    }

    fn cost_model(&self) -> CostModel {
        CostModel::new(1.0e-9, 0, 1)
    }

    fn kind(&self) -> TargetKind {
        TargetKind::FastHash
    }
}

#[cfg(test)]
mod tests {
    use super::PlainTextTarget;
    use peeper_core::{Target, TargetKind};

    #[test]
    fn accepts_the_secret_and_rejects_others() {
        let target = PlainTextTarget::new(b"hunter2".to_vec());
        assert!(target.verify(b"hunter2"));
        assert!(!target.verify(b"hunter3"));
        assert!(!target.verify(b""));
    }

    #[test]
    fn reports_fast_hash_kind() {
        let target = PlainTextTarget::new("x");
        assert_eq!(target.kind(), TargetKind::FastHash);
    }
}
