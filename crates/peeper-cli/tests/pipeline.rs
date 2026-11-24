// SPDX-License-Identifier: MIT
//! End-to-end integration test: the L3 -> L2 crack loop (precursor to tier T0).

use peeper_cli::crack_with_dictionary;
use peeper_verify::PlainTextTarget;

fn wordlist() -> Vec<Vec<u8>> {
    ["password", "123456", "qwerty", "letmein", "dragon"]
        .iter()
        .map(|w| w.as_bytes().to_vec())
        .collect()
}

#[test]
fn cracks_a_secret_present_in_the_dictionary_at_its_rank() {
    let target = PlainTextTarget::new(b"letmein".to_vec());
    let outcome = crack_with_dictionary(&target, wordlist());
    assert_eq!(outcome.cracked, Some(b"letmein".to_vec()));
    assert_eq!(outcome.guesses_tried, 4);
}

#[test]
fn exhausts_the_dictionary_when_the_secret_is_absent() {
    let target = PlainTextTarget::new(b"correcthorsebatterystaple".to_vec());
    let outcome = crack_with_dictionary(&target, wordlist());
    assert_eq!(outcome.cracked, None);
    assert_eq!(outcome.guesses_tried, 5);
}
