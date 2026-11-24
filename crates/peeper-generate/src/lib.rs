// SPDX-License-Identifier: MIT
//! Candidate generation: every source emits guesses in decreasing probability.

// shortcut: Phase 0 ships the stream contract and a dictionary source only.
// Rules/masks/PRINCE (Phase 2), Markov/OMEN and PCFG (Phase 2), and the neural
// generators plus the bandit meta-scheduler (Phase 5) implement this same trait.

mod dictionary;

pub use dictionary::DictionaryStream;

/// A source of password guesses, emitted in decreasing estimated probability.
///
/// Merging streams by marginal yield (the orchestrator's job) relies on each
/// source honoring this ordering contract.
pub trait CandidateStream {
    /// Yields the next most-probable candidate, or `None` when exhausted.
    fn next_candidate(&mut self) -> Option<Vec<u8>>;

    /// An upper bound on candidates still to emit, when the source knows it.
    fn remaining_hint(&self) -> Option<usize> {
        None
    }
}
