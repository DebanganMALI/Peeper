// SPDX-License-Identifier: MIT
//! GPU compute kernels via CubeCL (Phase 3).

// shortcut: deferred to Phase 3. One CubeCL kernel source compiled per backend
// (CUDA day one, then ROCm/HIP and WGPU). Fast-hash kernels first, then a
// correct constant-memory Argon2id kernel reporting honest guesses/sec.
