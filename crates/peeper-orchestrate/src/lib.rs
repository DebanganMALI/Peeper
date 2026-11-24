// SPDX-License-Identifier: MIT
//! Distributed orchestration: guess-stream work units and scheduling (Phase 4).

// shortcut: deferred to Phase 4. Work unit = a slice of a guess stream (not an
// alphabet range); memory-aware adaptive scheduling so a 4 GB node never takes a
// high-memory Argon2 chunk; chunk-timeout reassignment; telemetry. tokio + tonic.
