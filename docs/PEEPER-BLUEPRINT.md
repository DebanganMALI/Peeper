# Peeper — Project Blueprint

> **The single source of truth for building Peeper.** This document merges and supersedes `00-master-plan.md` and `01-engineering-standards.md`. If anything here disagrees with those, this wins. (They can be deleted once you're happy with this — tell me and I'll remove them.)
>
> **Thesis:** A distributed, GPU-accelerated, ML-driven credential-recovery engine whose real intelligence is in *candidate ordering* and *attack-cost measurement*, benchmarked by trying to break our own Obscura vault — and proving where the wall is.
>
> **Scope boundary (by design):** operates only on hashes/vaults we own or are explicitly authorized to test. The Obscura self-audit is the flagship use case — a red-team tool validating our own blue-team crypto.

---

## Part I — Strategy

### 1. The honest frame (read first)

One truth anchors the whole project, because it defines what "advanced" means here:

> **Against a correctly-configured memory-hard KDF (Argon2id) protecting a high-entropy master password, no password cracker on Earth wins — not ours, not a nation-state's.** Memory-hardness is engineered to make GPUs and distribution give *sub-linear* returns.

That is not a reason to aim lower; it is the reason to aim at the **right target**. Raw throughput (billions of MD5/s) is solved and commoditized (Hashcat owns it). The genuine frontier, where "most advanced" actually lives, is three places:

1. **Candidate intelligence** — guessing the *right* passwords *first*, so you need $10^6$ guesses, not $10^{15}$. ML/neural models, probabilistic grammars, adaptive strategies. Still an open research area in 2026.
2. **Attack-cost measurement** — turning "did it crack?" into a rigorous metric ($\lambda$-guesswork, cost-to-crack in GPU-years and dollars, Grover-adjusted bounds). This is what makes it publishable and makes the Obscura audit meaningful.
3. **Engineering of the KDF/hash engine + distribution** — the muscle. A means, not the thesis.

The brute-force muscle is table stakes; the **brain** and the **ruler** are the project.

### 2. What we inherit from the reference, and its ceiling

CarterPerez-dev's `Distributed.Password.Cracker.md` is a solid *intermediate* blueprint: GPU hashing (CUDA/OpenCL) for MD5/SHA/bcrypt, a Redis/RabbitMQ job queue with lexicographic keyspace ranges, load balancing + health checks, dictionary/brute-force/pattern/rainbow attacks, a dashboard, a results DB.

We keep its skeleton (GPU engine + distributed queue + dashboard + results DB) and **replace the brain and the ruler**. Its ceiling, which we deliberately pass:
- Candidate generation stops at dictionary + rules + brute-force + rainbow tables (mid-2010s Hashcat). No grammars, no Markov enumeration, no ML.
- No mathematical cost model or entropy metrics — binary "cracked / not."
- Rainbow tables are dead for salted modern hashes.
- Keyspace split by lexicographic range — useless for probability-ordered guessing.
- Zero post-quantum analysis; memory-hard KDFs treated as "slow, oh well."
- No adaptivity — never learns from what it cracked.

### 3. Design philosophy

1. **Probability-ordered, not index-ordered.** Every candidate source emits guesses in decreasing estimated probability; the orchestrator merges streams by *marginal yield* (cracks per unit KDF-cost), not by dividing an alphabet. A chunk of `ZZZZ…` is worthless; a chunk of "most-likely 10M passphrases" is gold.
2. **KDF cost is a first-class citizen.** Every target carries a cost model $c_{\text{KDF}}$ (seconds, memory, parallelism). Scheduling, ETA, and the resistance report compute against it. A fast hash and an Argon2id vault are never treated as the same problem.
3. **Measure, don't just crack.** A run's output is not a password — it is a *curve*: success rate vs. guess number vs. cost. Cracking is the sampling procedure that produces the curve.
4. **Adaptive by default.** The attack reweights its own strategy mid-run from what it is cracking (dynamic dictionaries / EM reweighting). Real attackers do this; so does ours.

### 4. Architecture (six layers)

```
L6  Analytics & Measurement   guess-number curves, λ-guesswork, cost-to-crack,
                              Grover bounds, Obscura resistance report
L5  Orchestration             guess-stream partitioning, adaptive scheduler,
                              heterogeneous workers, fault tolerance, telemetry
L4  Compute Engine            CPU core + CubeCL GPU kernels: fast hashes AND
                              memory-hard KDFs (Argon2id/scrypt/bcrypt/PBKDF2)
L3  Candidate Generation      "the brain": dict/rules/mask/PRINCE · Markov/OMEN ·
                              PCFG · neural (GAN→transformer) · adaptive · bandit
L2  Target & Cost Abstraction hash parsers, Obscura envelope parser,
                              verify(candidate)->bool, per-target cost model
L1  Core / Platform           Rust engine, plugin traits, IPC, storage
```

**L2 is the keystone.** One trait makes "crack a raw NTLM hash" and "crack an Obscura vault" identical to everything above it:

```rust
trait Target {
    fn verify(&self, candidate: &[u8]) -> bool;   // the oracle
    fn cost_model(&self) -> CostModel;            // time, memory, parallelism per guess
    fn kind(&self) -> TargetKind;                 // FastHash | MemoryHardKDF | VaultEnvelope
}
```

- **FastHash** (MD5/SHA/NTLM): `verify` = hash-and-compare; cost ≈ nanoseconds.
- **MemoryHardKDF** (raw Argon2id/bcrypt/scrypt): `verify` = derive-and-compare; cost = hundreds of ms + tens-to-hundreds of MiB.
- **VaultEnvelope** (Obscura): `verify` = Argon2id(salt, params) → attempt AEAD decrypt of a known header → success iff the tag validates. The AEAD tag *is* the oracle.

Everything above L2 is written once and works on all three.

### 5. Layer 3 — the brain (candidate generation)

The differentiator. Research lineage, oldest → newest, and what we take:

| Generation | Technique | Key paper / tool | What we borrow |
|---|---|---|---|
| Rule-based | Dictionary + rules, masks, combinator, hybrid, PRINCE | Hashcat; Steube | Baseline streams; rule + mask engine |
| Statistical (Markov) | Ordered Markov enumeration in probability order | OMEN (Dürmuth et al., ESSoS'15) | Streamed probability-ordered output; beats JtR-Markov ~8× to equal yield |
| Statistical (grammar) | PCFG — segment into $L/D/S$ structures | Weir et al.; PCFGv4 | Strong on structured/policy passwords; leads at low guess counts (~$10^6$) |
| Neural (GAN) | Generator learns the password manifold | PassGAN (Hitaj et al., ACNS'19) | Representation-learning baseline |
| Neural (repr. learning) | Conditional guessing + EM dynamic adaptation | Pasquini et al., S&P'21; AdaMs, USENIX'21 | Conditional generation + dynamic dictionaries that adapt mid-attack |
| Neural (transformer) | Autoregressive char LM; guided generation | PassGPT (2023); PagPassGPT (DSN'24) | Best unseen yield (≈2× PassGAN); constraint-guided sampling |
| LLM-adaptation | LoRA-adapted LLMs for targeted guessing | PassLLM (USENIX Sec'25) | Optional targeted-guessing add-on |

> **Honest caveat (2025):** the empirical study "When Intelligence Fails" found *general-purpose* off-the-shelf LLMs underperform traditional rule/Markov/PCFG methods at password guessing — fluency ≠ task reasoning. The win is in *small, fine-tuned, character-level transformers* (PassGPT-style), not a giant chatbot. We build the former; the LLM route stays an optional add-on. Keeps us honest and keeps compute sane on a Legion laptop.

**5.1 The bandit meta-scheduler (headline novelty).** Each generator $g_i$ is a stream with running empirical yield $\hat{y}_i(t)$ = cracks per KDF-second. Treat generator selection as a **multi-armed bandit**: spend the next batch of expensive KDF evaluations on the arm with the best upper-confidence-bound yield, with forced exploration so a slow-starting grammar still gets sampled:

$$g^\star = \arg\max_i \left[\, \hat{y}_i(t) + c\sqrt{\frac{\ln t}{n_i(t)}}\,\right]$$

Existing crackers run strategies in fixed phases; framing selection as online regret-minimization against measured cost is, as far as I can find, open. **Level-up (agreed):** carry this to a *formal regret bound* — that is what turns "a tool" into "a paper."

**5.2 Adaptive layer.** Every cracked password feeds back: reweights Markov/PCFG priors, joins a dynamic dictionary the rule engine mangles, and seeds the conditional neural generator. Matters most against a leaked hash list (e.g. auditing a dump of our own test users).

### 6. Layer 4 — the compute engine (honest about physics)

**Fast hashes** (MD5/SHA/NTLM): GPU-bound, embarrassingly parallel; saturate the Legion GPU. Real lessons (coalesced memory, minimal host↔device transfer, on-GPU candidate generation), known ceiling.

**Memory-hard KDFs** (the real target): the physics change and we *show* the math.
- **bcrypt**: 4 KiB, cache-thrashing; GPUs do poorly, FPGAs better.
- **scrypt**: tunable memory; GPU-attackable only at small parameters.
- **Argon2id** (Obscura's choice): memory-bandwidth-bound. Benchmarks to reproduce — a password that takes **1 s** under SHA-256 takes **~14 years** under Argon2id on the same rig (Specops, 2025, 8× RTX 5090); reference Argon2id ≈ 0.5 s standard / 2.3 s paranoid per derivation (single-digit guesses/sec/core).

The engine exposes the *actual achievable* $g/s$ per target on our hardware — the whole resistance report depends on it. A correct constant-memory Argon2id kernel reporting honest throughput beats a fast MD5 kernel here.

### 7. Layer 5 — distributed orchestration

- **Work unit = a slice of a guess stream**, not an alphabet range (e.g. "indices $[a,b)$ of OMEN level-$k$"), each with a precomputed keyspace for exact progress.
- **Adaptive scheduling:** chunk size auto-tunes to a target completion time per worker (Fitcrack-style), so a Pi-class node and the Legion both stay busy.
- **Fault tolerance:** chunk timeout → reassignment; workers register hashrate **and memory** — a 4 GB node cannot take high-memory Argon2 chunks, so the scheduler must know.
- **Workers:** heterogeneous — Legion GPU + the Hyper-V Kali/BlackArch VMs as the demo cluster. No extra hardware.

### 8. Layer 6 — the mathematical core (what makes it publishable)

**8.1 Entropy metrics.** For a password distribution with sorted probabilities $p_1 \ge p_2 \ge \dots$:
- Shannon entropy $H = -\sum_i p_i \log_2 p_i$ (weak attacker model, for contrast).
- Min-entropy $H_\infty = -\log_2 p_1$ (best single guess).
- Guessing entropy $G = \sum_i i \cdot p_i$ (expected guesses, optimal order).
- **$\alpha$-guesswork $\mu_\alpha$** (Bonneau) — guesses to crack a fraction $\alpha$ of accounts. This is what our success-vs-guesses curve estimates directly.

**8.2 Expected attack work** — the equation that unifies everything:

$$\mathbb{E}[W] = \Big(\sum_{i=1}^{N} p_i \cdot i\Big)\cdot c_{\text{KDF}}, \qquad C(\alpha) = \mu_\alpha \cdot c_{\text{KDF}}\;[\text{s}], \qquad C_\$(\alpha) = C(\alpha)\cdot \frac{\$\,\text{per GPU-hour}}{3600}$$

with $c_{\text{KDF}}$ the measured per-guess cost from L4. **This is the resistance report** — it converts "is Obscura safe?" into a time and a dollar figure per strength tier.

**8.3 Why distribution/GPUs don't rescue the attacker.** Argon2's security rests on the time–area product $AT$: halving time by doubling parallelism doubles memory, so cost is invariant. Distribution buys *linear* speedup at best, never the super-linear collapse low-entropy passwords suffer. The report proves this on our own curves (cost vs. nodes).

**8.4 The cracker as a strength meter.** Because L6 produces $\mu_\alpha$ for any password, the engine doubles as a simulated-cracking strength meter for Obscura's "new master password" screen — a citable contribution and a direct feature gift back to Obscura.

### 9. The post-quantum dimension (two distinct surfaces)

**Surface A — Grover search over the password space.** Grover searches size-$N$ space in $\sim\sqrt{N}$ evaluations, halving effective bit-strength ($2^b \to 2^{b/2}$). But human passwords are already $\le 2^{40}$ and far from uniform, and each Grover iteration must evaluate Argon2id reversibly in superposition — astronomically expensive for decades. We build a **Quantum Cost Estimator**: given KDF cost + distribution, report the Grover-adjusted bound ($\mu_\alpha^{\text{quantum}} \approx \sqrt{\mu_\alpha}$, with the honest oracle-cost caveat) alongside the classical one. Optional **toy Qiskit Grover oracle** on a 4–8 bit toy hash — illustrative, and a tie-in to your IBM Quantum badge.

**Surface B — Obscura's PQ hybrid key wrapping (the one that matters).** Obscura wraps the vault key with ML-KEM/Kyber + a classical curve. A **weakest-link analyzer** models both attack costs ($C_{\text{password-path}}$ vs. $C_{\text{KEM-path}}$) and reports $\min(\cdot)$. The near-universal finding — *the password is always the weak link* — is the compelling Obscura lesson.

> **Net honest claim:** we are not "breaking post-quantum crypto." We (a) quantify the quadratic Grover erosion of password entropy and (b) prove the password governs a PQ-hybrid vault. Both true, defensible, genuinely advanced.

### 10. Flagship benchmark — the Obscura resistance ladder

Structured so it always yields a meaningful result (a crack *or* a proven wall):

| Tier | Vault config | Expected | Proves |
|---|---|---|---|
| T0 | master password `password123` (known weak) | cracks in seconds | end-to-end pipeline: parse → derive → AEAD-tag oracle |
| T1 | rockyou-class human password (`Summer2024!`) | cracks in a bounded budget; record $\mu$ | real per-generator yield vs. Obscura's actual params |
| T2 | policy-compliant, patterned (name+year+symbol) | crackable but expensive; record curve | where "looks strong" sits in GPU-hours |
| T3 | Obscura default Argon2id + 4–5 word diceware | infeasible | the wall: $C_\$(\alpha)$ in GPU-years/dollars + Grover bound |
| T4 | T3 + PQ hybrid wrap | password path dominates | PQ wrapping sound; password governs — the headline |

The deliverable is a **resistance report**, not "we cracked Obscura": curves and dollar figures showing exactly how strong the defaults are, and whether the Argon2id params ($m,t,p$) are high enough. If T2 cracks too cheaply, that's an actionable fix that hardens Obscura — the projects feed each other. Paper/patent material.

### 11. Where the genuine novelty is (paper/patent angles)

1. **Yield-optimal bandit scheduling** across heterogeneous generators under a bounded KDF budget, with a formal regret bound (§5.1).
2. **A unified cost-aware attack-and-measurement framework** — the same engine attacks and emits $\alpha$-guesswork resistance reports in real currency/time (§8).
3. **Weakest-link analysis for PQ-hybrid vaults** (§9B), applied to a real shipping product.
4. **The Obscura resistance study itself** (§10) — an end-to-end empirical KDF-parameter evaluation with a feedback loop into the product's defaults.

(1)+(2) together are a strong workshop-paper angle.

---

## Part II — Engineering standards

### 12. Locked decisions (ADR log)

| # | Decision | Status |
|---|---|---|
| 1 | Name: **Peeper** | Locked |
| 2 | Language: **Rust, end-to-end** (no Python, no C/C++) | Locked |
| 3 | GPU: **CubeCL** (pure Rust, multi-backend); no hand-written CUDA-C/OpenCL-C | Locked |
| 4 | ML: **Burn** (train) + **Candle** (in-engine inference) | Locked |
| 5 | Async + networking: **tokio** + **tonic** (gRPC) | Locked |
| 6 | State/infra: **Redis** (queue) + **Postgres** (durable state) | Locked |
| 7 | **CLI + GUI**, both shipped; GUI = **Tauri v2** (same stack as Obscura), thin shell over the engine | Locked |
| 8 | Release targets: **Windows + Linux**, GPU on both (one CubeCL source) | Locked |
| 9 | Comments: **single-line only**; one-line `///` on public API; no block comments | Locked |
| 10 | No AI attribution anywhere in code, commits, or PRs | Standing rule |
| 11 | CI workflows never deleted without explicit per-item confirmation | Standing rule |
| 12 | Adopt **Ponytail** (MIT, `DietrichGebert/ponytail`) as governing coding philosophy, level **full** | Locked |
| 13 | Git: **sole contributor (Saheb)**; atomic **Conventional Commits**, committed often | Standing rule |
| 14 | GUI visual design deferred to a later **Figma** session (sci-fi/tech look) | Parked |

### 13. Final technology stack

| Component | Choice |
|---|---|
| Core engine, verifiers, orchestrator | **Rust** |
| GPU kernels (hashes + Argon2id) | **CubeCL** → CUDA (NVIDIA, day one), ROCm/HIP (AMD), WGPU/SPIR-V (rest) from one source |
| KDF/crypto reference | RustCrypto `argon2`/`scrypt`/`bcrypt`/`pbkdf2`, `aes-gcm`/`chacha20poly1305` (correctness + known-answer vectors) |
| ML models | **Burn** (training) + **Candle** (inference), no Python |
| Async / networking | **tokio** + **tonic** (gRPC) |
| Queue / state | **Redis** + **Postgres** |
| CLI | `peeper-cli` (Salt-miner-style presentation) |
| GUI | `peeper-gui` — **Tauri v2**, thin shell over the engine (design later) |
| Distributed workers | Hyper-V Kali/BlackArch VMs + Legion GPU |
| Packaging / CI | Docker + docker-compose; GitHub Actions (build Windows + Linux from day one) |

**GPU portability (the Option-A payoff):** one Rust kernel, compiled per backend via a generic `Runtime` + feature flag. CUDA is just the NVIDIA backend CubeCL emits — the same source runs on Windows and Linux and, with a flag, on AMD/others. Per-backend work is only tuning constants, never forked kernels.

### 14. Workspace structure

```
peeper/
├── Cargo.toml                 # workspace manifest (shared deps, lints)
├── rust-toolchain.toml        # pinned toolchain
├── rustfmt.toml  clippy.toml  # formatting + lint config
├── deny.toml                  # cargo-deny: supply-chain + license policy
├── crates/
│   ├── peeper-core/           # domain types; Target + CostModel traits (L1/L2)
│   ├── peeper-verify/         # hash + KDF verifiers; Obscura AEAD-tag oracle
│   ├── peeper-generate/       # candidate generators: dict/rules/mask/Markov/PCFG
│   ├── peeper-neural/         # Burn/Candle models + the bandit meta-scheduler
│   ├── peeper-gpu/            # CubeCL kernels (fast hashes + Argon2id)
│   ├── peeper-orchestrate/    # tokio/tonic coordinator + worker, scheduling
│   ├── peeper-measure/        # entropy metrics, cost model, resistance reports
│   ├── peeper-proto/          # shared gRPC/protobuf contracts
│   ├── peeper-cli/            # the binary + UX
│   └── peeper-gui/            # Tauri v2 desktop shell (design deferred)
├── xtask/                     # build/release automation
├── benches/                   # criterion benchmarks
├── tests/                     # cross-crate integration + known-answer tests
├── docs/                      # all long-form explanation lives here
└── .github/workflows/         # CI — never deleted without explicit say-so
```

A type's home crate is decided by *what it is*: a new generator goes in `peeper-generate`, a new metric in `peeper-measure`, even if you're editing elsewhere at the time.

### 15. Code standards

**Naming.** Idiomatic Rust `snake_case` (vars/fns/modules), `CamelCase` (types/traits), `SCREAMING_SNAKE_CASE` (consts) — enforced by rustfmt/clippy; non-idiomatic naming reads as unprofessional. The goal is *no ugly names*, via discipline, not removing underscores:

| Bad | Good |
|---|---|
| `cand_strm`, `kdf_cst` | `candidate_stream`, `kdf_cost` |
| `vec_guesses`, `str_name` | `guesses`, `name` |
| `data`, `info`, `manager` | `envelope`, `yield_rate`, `scheduler` |
| `process2`, `handlerX` | `verify_batch`, `decrypt_header` |

Verbs for functions (`derive_key`), nouns for types (`CostModel`); short locals (`i`, `n`, `buf`) fine, public items descriptive.

**Comments.** Single-line only. One-line `///` on public items (feeds `cargo doc`); `//` inline only where intent is non-obvious; deferred shortcuts as `// shortcut: <limit>, <when to upgrade>`. Everything longer lives in `/docs`.

**Toolchain & gates.** rustfmt (CI-checked) · clippy `pedantic`, warnings denied · pinned `rust-toolchain.toml` · Conventional Commits + SemVer · cargo-audit + cargo-deny · SPDX headers + CycloneDX SBOM · tests = unit + integration + criterion + proptest + **known-answer vectors for every crypto primitive** · CI order `fmt → clippy → test → audit`.

**Standards framing.** No ISO governs code style — the real bar is the **Rust API Guidelines** + the toolchain above. ISO/IEC 25010 is the quality lens (performance efficiency, maintainability, reliability, security); SEI CERT + OWASP for secure coding (and your ISO 27001 interest).

### 16. Ponytail — governing coding philosophy

Adopted from `DietrichGebert/ponytail` (MIT), level **full**. Essence: *solve the whole problem with the least new code, and never cut validation, error handling, security, accessibility, or tests.*

The ladder before writing anything (first rung that fully works): does it need to exist → reuse what's in the repo → stdlib/platform → existing dependency → one clear line → minimum code that works. Non-negotiable: finish every caller/test/fixture the change touches; non-trivial logic (branch, loop, parser, crypto/security) leaves a test; never cut trust-boundary validation, data-loss handling, security, accessibility; end each code-delivery reply with what was skipped + any risk.

**No contradiction with the multi-crate structure:** the crates are *justified structure* for a large system (a logged architecture decision), not speculative abstraction. Ponytail governs how we implement *inside* each crate — reuse-first, smallest complete diff. Native install for the always-on enforcement + `/ponytail-review` and `/ponytail-audit`: `/plugin marketplace add DietrichGebert/ponytail` then `/plugin install ponytail@ponytail`.

### 17. Git & commit policy

- **Sole contributor: Saheb.** Every commit under his GitHub-verified identity; no `Co-Authored-By`, no "Generated with…", no AI name anywhere.
- **Attribution correctness:** `git config user.email` must be a GitHub-verified address on his account (or his GitHub `noreply`) so commits count on his graph and attribute only to him. Confirm before the first commit.
- **Style:** atomic, self-contained commits, each a single logical change, Conventional Commits message; commit at every natural checkpoint. A high daily count is fine as a byproduct — not by fragmenting one change or committing filler.
- **Deletions:** any delete command is named in plain words and confirmed first; Actions workflows/run history never deleted without an explicit, specific request.

---

## Part III — Execution

### 18. Phase roadmap

- **Phase 0 — Foundations.** Threat model + authorized-scope doc; workspace scaffold; `Target`/`CostModel` traits; the measurement harness + curve plotting (build the *ruler* first); Obscura envelope parser. CPU-only.
- **Phase 1 — Correct single-node engine.** Hash + KDF verifiers; Obscura AEAD-tag oracle; a dictionary attack end-to-end; **T0 passes** on CPU. *Milestone: the pipeline is real.*
- **Phase 2 — Classical brains.** Rules + masks + combinator + PRINCE → Markov/OMEN → PCFG; comparison on a held-out set. *Milestone: beats naive dictionary at low guess counts.*
- **Phase 3 — GPU acceleration.** Fast-hash kernels, then a correct constant-memory Argon2id kernel with honest $g/s$. *Milestone: real cost numbers on real hardware.*
- **Phase 4 — Distribution.** Guess-stream work units; memory-aware adaptive scheduler; fault tolerance; telemetry; lab-VM workers. *Milestone: linear scaling, plotted.*
- **Phase 5 — Neural + adaptive.** PassGAN → character-transformer (guided) → dynamic dictionaries + EM → the bandit meta-scheduler (+ the regret-bound write-up). *Milestone: neural beats classical on unseen passwords.*
- **Phase 6 — Math & PQ core.** $\alpha$-guesswork, cost-to-crack report, time–area proof; Quantum Cost Estimator + toy Qiskit Grover oracle; weakest-link analyzer. *Milestone: the resistance-report generator.*
- **Phase 7 — Obscura benchmark + writeup.** Run T0–T4; produce the resistance report; feed findings into Obscura's Argon2 params; draft paper/patent + README (Salt-miner style). *Milestone: a publishable result + portfolio centerpiece.*

First-cut build order **0 → 1 → 2 → 3** already gives a working, honest, GPU-accelerated cracker with real measurement and T0–T2 results; the novelty concentrates in 5–6, sitting on that base.

### 19. Future extensions (deliberately deferred — not in Phases 0–7)

Captured so the ambition is on record without derailing the core. Each is opt-in, *after* the engine proves out:

- **IoT firmware credential audit** — extract and audit password hashes/default credentials from firmware images we own. Coherent and advanced, but a different problem domain (firmware unpacking) and overlapping your PQC-IoT project. A Phase 8 module at most, or better placed near that project. (IoT devices as *cracking workers* is rejected: no GPU, negligible compute.)
- **FPGA backend** for bcrypt (cache-thrashing, FPGA-friendly) — real research value, only if hardware access appears.
- **eBPF performance telemetry** — profile the cracking process at the kernel level. Observability only, marginal to the thesis, and overlapping your ransomware-detection eBPF project. Not core.

Note on "kernel level": the deep low-level work in Peeper is the **GPU compute kernel** (CubeCL) — that's the real systems flex. An **OS-kernel** component is architecturally wrong for a cracker (userspace, compute-bound; kernel code adds privilege + risk for zero benefit) and is not planned.

### 20. Scope & ethics (built in, not bolted on)

- Only targets we own or are authorized to test: the Obscura self-audit, synthetic/seeded test-user dumps, and public research corpora (rockyou and similar, standard in every cited paper).
- Candidate models train on public, already-leaked research corpora — never on data scraped from live systems.
- Every offensive capability maps to a defensive output (resistance report, strength meter, hardened Obscura params). That is the story for the README, the paper, and the CV.

### 21. Before we start — prerequisites

**Locked and ready:** name, language, GPU approach, ML stack, interfaces, targets, structure, standards, Ponytail, git policy.

**Still needed from you:**
1. **The code folder + the repo** you'll dedicate to Peeper (this project folder stays the research/docs home). Point me at them and I scaffold Phase 0.
2. **Obscura's real vault envelope format** (Argon2id params layout, AEAD choice, how the PQ wrap is stored). Not a Phase-0 blocker — needed by Phase 1 for the envelope parser; until then I work from the assumed AEAD + Argon2id + PQ-hybrid shape.

**First practical move once the repo exists:** dev-environment setup (Rust toolchain, CUDA toolkit on the Legion, Docker for Redis/Postgres) — I'll walk this one step at a time.
