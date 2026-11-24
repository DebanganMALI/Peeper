// SPDX-License-Identifier: MIT
//! The `Target` oracle: the one trait that makes every attackable thing uniform.

use crate::cost::CostModel;

/// What class of target this is, which fixes the shape of its cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    /// MD5/SHA/NTLM and friends: verify is hash-and-compare, cost near nanoseconds.
    FastHash,
    /// Raw Argon2id/bcrypt/scrypt: verify is derive-and-compare, cost in milliseconds and MiB.
    MemoryHardKdf,
    /// A vault envelope (Obscura): the AEAD tag is the oracle.
    VaultEnvelope,
}

/// The oracle every layer above L2 is written against.
///
/// One implementation of this trait makes "crack a raw NTLM hash" and "crack an
/// Obscura vault" identical to the generators, scheduler, and measurement code.
pub trait Target: Send + Sync {
    /// Returns true iff `candidate` is the secret this target guards.
    fn verify(&self, candidate: &[u8]) -> bool;

    /// The measured cost of a single `verify` call on this target.
    fn cost_model(&self) -> CostModel;

    /// Which class of target this is.
    fn kind(&self) -> TargetKind;
}
