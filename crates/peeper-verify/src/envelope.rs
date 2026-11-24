// SPDX-License-Identifier: MIT
//! The Obscura vault envelope: parser and AEAD-tag oracle (placeholder).

// shortcut: the real envelope byte layout (Argon2id m/t/p, AEAD choice, PQ-wrap
// storage) is an external input, needed by Phase 1 (blueprint section 21).
// Until it is supplied, `parse` refuses rather than guessing a format.

use std::fmt;

/// Why an envelope failed to parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvelopeError {
    /// The real Obscura envelope format has not been wired in yet.
    NotYetSpecified,
    /// The input was shorter than any valid envelope.
    TooShort,
}

impl fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotYetSpecified => write!(f, "obscura envelope format not yet specified"),
            Self::TooShort => write!(f, "input too short to be an obscura envelope"),
        }
    }
}

impl std::error::Error for EnvelopeError {}

/// A parsed Obscura vault envelope.
///
/// The AEAD tag is the oracle: a guess is correct iff Argon2id over the guess
/// derives a key whose AEAD decrypt of the known header validates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObscuraEnvelope {
    /// The Argon2id salt bytes.
    pub salt: Vec<u8>,
    /// Argon2id memory cost in KiB (`m`).
    pub memory_kib: u32,
    /// Argon2id time cost / iterations (`t`).
    pub iterations: u32,
    /// Argon2id parallelism lanes (`p`).
    pub lanes: u32,
}

impl ObscuraEnvelope {
    /// Parses an envelope from its on-disk bytes.
    ///
    /// # Errors
    /// Returns `TooShort` on empty input, and `NotYetSpecified` until the real
    /// Obscura format is wired in (blueprint section 21).
    pub fn parse(bytes: &[u8]) -> Result<Self, EnvelopeError> {
        if bytes.is_empty() {
            return Err(EnvelopeError::TooShort);
        }
        Err(EnvelopeError::NotYetSpecified)
    }
}

#[cfg(test)]
mod tests {
    use super::{EnvelopeError, ObscuraEnvelope};

    #[test]
    fn empty_input_is_too_short() {
        assert_eq!(ObscuraEnvelope::parse(&[]), Err(EnvelopeError::TooShort));
    }

    #[test]
    fn nonempty_input_awaits_the_real_format() {
        assert_eq!(
            ObscuraEnvelope::parse(&[1, 2, 3]),
            Err(EnvelopeError::NotYetSpecified)
        );
    }
}
