// SPDX-License-Identifier: MIT
//! Verifiers: concrete `Target` implementations behind the L2 oracle.

// shortcut: Phase 0 ships only PlainTextTarget so the pipeline runs on CPU.
// Real MD5/SHA/NTLM (FastHash) and Argon2id/bcrypt/scrypt (MemoryHardKdf)
// verifiers land in Phase 1/3 via RustCrypto with known-answer vectors.

mod envelope;
mod plaintext;

pub use envelope::{EnvelopeError, ObscuraEnvelope};
pub use plaintext::PlainTextTarget;
