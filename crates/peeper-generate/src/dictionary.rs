// SPDX-License-Identifier: MIT
//! A dictionary source: a pre-ordered wordlist replayed as a guess stream.

use crate::CandidateStream;

/// Emits a wordlist that is already sorted in descending likelihood.
pub struct DictionaryStream {
    words: std::vec::IntoIter<Vec<u8>>,
    remaining: usize,
}

impl DictionaryStream {
    /// Builds a stream from a wordlist, taken to be in descending-probability order.
    #[must_use]
    pub fn new(words: Vec<Vec<u8>>) -> Self {
        let remaining = words.len();
        Self {
            words: words.into_iter(),
            remaining,
        }
    }
}

impl CandidateStream for DictionaryStream {
    fn next_candidate(&mut self) -> Option<Vec<u8>> {
        let next = self.words.next();
        if next.is_some() {
            self.remaining -= 1;
        }
        next
    }

    fn remaining_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

#[cfg(test)]
mod tests {
    use super::DictionaryStream;
    use crate::CandidateStream;

    #[test]
    fn emits_words_in_order_then_stops() {
        let mut stream = DictionaryStream::new(vec![b"password".to_vec(), b"123456".to_vec()]);
        assert_eq!(stream.remaining_hint(), Some(2));
        assert_eq!(stream.next_candidate(), Some(b"password".to_vec()));
        assert_eq!(stream.remaining_hint(), Some(1));
        assert_eq!(stream.next_candidate(), Some(b"123456".to_vec()));
        assert_eq!(stream.next_candidate(), None);
        assert_eq!(stream.remaining_hint(), Some(0));
    }
}
