// SPDX-License-Identifier: MIT
//! Neural candidate generation and the bandit meta-scheduler (Phase 5).

// shortcut: deferred to Phase 5. PassGAN -> character transformer (guided) ->
// dynamic dictionaries + EM, then the UCB bandit over generators under a bounded
// KDF budget. Training with Burn, in-engine inference with Candle, no Python.
