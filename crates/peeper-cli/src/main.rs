// SPDX-License-Identifier: MIT
//! Peeper CLI entry point.

use clap::{Parser, Subcommand};

use peeper_cli::crack_with_dictionary;
use peeper_core::Target;
use peeper_measure::{cost_to_crack_seconds, guessing_entropy, min_entropy, shannon_entropy};
use peeper_verify::PlainTextTarget;

#[derive(Parser)]
#[command(
    name = "peeper",
    version,
    about = "Cost-aware credential-recovery and KDF-strength engine"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the Phase 0 end-to-end demo: dictionary vs. a plaintext target, with metrics.
    Demo,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Demo => run_demo(),
    }
}

// A tiny, illustrative wordlist in rough descending-likelihood order.
fn sample_wordlist() -> Vec<Vec<u8>> {
    ["password", "123456", "qwerty", "letmein", "dragon"]
        .iter()
        .map(|w| w.as_bytes().to_vec())
        .collect()
}

#[allow(clippy::cast_precision_loss)]
fn run_demo() {
    let words = sample_wordlist();
    let count = words.len();
    let secret = b"123456".to_vec();
    let target = PlainTextTarget::new(secret.clone());

    let outcome = crack_with_dictionary(&target, words.clone());

    println!("Peeper demo - dictionary stream vs. a plaintext target");
    println!("  wordlist size : {count}");
    match &outcome.cracked {
        Some(found) => {
            let shown = String::from_utf8_lossy(found);
            println!(
                "  result        : CRACKED \"{shown}\" at guess {}",
                outcome.guesses_tried
            );
        }
        None => println!(
            "  result        : not found in {} guesses",
            outcome.guesses_tried
        ),
    }

    // Treat the wordlist as a uniform distribution to show the measurement core.
    let probabilities = vec![1.0 / count as f64; count];
    println!("  --- measurement (uniform model over the wordlist) ---");
    println!(
        "  shannon entropy : {:.3} bits",
        shannon_entropy(&probabilities)
    );
    println!(
        "  min-entropy     : {:.3} bits",
        min_entropy(&probabilities)
    );
    println!(
        "  guessing entropy: {:.3} guesses",
        guessing_entropy(&probabilities)
    );

    let seconds = cost_to_crack_seconds(outcome.guesses_tried, &target.cost_model());
    println!(
        "  cost to this crack: {seconds:.3e} s at {:.3e} guesses/s",
        target.cost_model().guesses_per_second()
    );
}
