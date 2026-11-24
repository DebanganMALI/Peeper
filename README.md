# Peeper

A distributed, GPU-accelerated, cost-aware **credential-recovery and
KDF-strength measurement engine**, written end-to-end in Rust.

Peeper's real intelligence is not raw throughput (that is a solved, commoditized
problem) but **candidate ordering** and **attack-cost measurement**: guessing
the right passwords first, and turning "did it crack?" into a rigorous curve of
success rate versus guess number versus cost in GPU-years and dollars.

Its flagship use case is a self-audit — benchmarking its own sibling project,
the [Obscura](https://github.com/DebanganMALI) password manager, to prove where
the wall is in Obscura's Argon2id configuration.

## The honest frame

Against a correctly-configured memory-hard KDF (Argon2id) protecting a
high-entropy master password, no password cracker wins — not this one, not a
nation-state's. Memory-hardness makes GPUs and distribution give sub-linear
returns. That is not a reason to aim lower; it is the reason to aim at the right
target: measuring exactly how strong a configuration is, and flagging when it
falls short of that wall.

## Scope

Peeper runs only against secrets we own or are explicitly authorized to test:
the Obscura self-audit, synthetic seeded test dumps, and public research corpora
(RockYou and similar, standard in the cited literature). See
[`docs/threat-model.md`](docs/threat-model.md) for the full authorized-scope
policy. It is a red-team tool whose deliverable is a blue-team resistance report.

## Architecture (six layers)

```
L6  Analytics & Measurement   guess-number curves, alpha-guesswork, cost-to-crack
L5  Orchestration             guess-stream partitioning, adaptive scheduler, workers
L4  Compute Engine            CubeCL GPU kernels: fast hashes AND memory-hard KDFs
L3  Candidate Generation      dict/rules/mask -> Markov/PCFG -> neural -> bandit
L2  Target & Cost Abstraction the Target oracle + per-target cost model (keystone)
L1  Core / Platform           Rust engine, traits, IPC, storage
```

The keystone is the `Target` trait in `peeper-core`: one oracle makes a raw hash
and an Obscura vault identical to everything above L2.

## Workspace layout

| Crate | Layer | Role |
|---|---|---|
| `peeper-core` | L1/L2 | `Target` oracle and `CostModel` |
| `peeper-verify` | L2 | hash/KDF verifiers; the Obscura AEAD-tag oracle |
| `peeper-generate` | L3 | probability-ordered candidate streams |
| `peeper-neural` | L3 | neural generators + the bandit meta-scheduler |
| `peeper-gpu` | L4 | CubeCL kernels (fast hashes + Argon2id) |
| `peeper-orchestrate` | L5 | distributed coordinator and workers |
| `peeper-measure` | L6 | entropy metrics, cost model, resistance reports |
| `peeper-proto` | — | shared gRPC/protobuf contracts |
| `peeper-cli` | — | the `peeper` command-line binary |
| `peeper-gui` | — | Tauri v2 desktop shell (design deferred) |

## Build and run

```sh
cargo build
cargo test
cargo run -p peeper-cli -- demo
```

The `demo` command runs the current end-to-end pipeline (a dictionary stream
against a target, with live entropy and cost metrics) on CPU.

## Status

**Phase 0 — Foundations.** Workspace scaffold, the `Target`/`CostModel` traits,
the measurement core (entropy and guesswork metrics with known-answer tests),
the authorized-scope doc, and an end-to-end CPU pipeline demo. The roadmap
through Phase 7 (the Obscura resistance report) is in
[`docs/PEEPER-BLUEPRINT.md`](docs/PEEPER-BLUEPRINT.md).

## License

MIT. See [`LICENSE`](LICENSE).
