// SPDX-License-Identifier: MIT
//! Library side of the CLI: the minimal end-to-end crack pipeline (Phase 0).

use peeper_core::Target;
use peeper_generate::{CandidateStream, DictionaryStream};

/// The result of running a guess stream against a target.
pub struct CrackOutcome {
    /// The matching candidate, if one was found.
    pub cracked: Option<Vec<u8>>,
    /// How many candidates were verified before stopping.
    pub guesses_tried: usize,
}

/// Runs a dictionary stream against a target, stopping at the first match.
///
/// This is the whole L3 -> L2 loop in miniature: generate a candidate, ask the
/// oracle, count the cost. GPU, distribution, and smarter generators slot in later.
#[must_use]
pub fn crack_with_dictionary(target: &dyn Target, words: Vec<Vec<u8>>) -> CrackOutcome {
    let mut stream = DictionaryStream::new(words);
    let mut guesses_tried = 0;
    while let Some(candidate) = stream.next_candidate() {
        guesses_tried += 1;
        if target.verify(&candidate) {
            return CrackOutcome {
                cracked: Some(candidate),
                guesses_tried,
            };
        }
    }
    CrackOutcome {
        cracked: None,
        guesses_tried,
    }
}
