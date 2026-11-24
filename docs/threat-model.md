# Peeper — Threat Model & Authorized Scope

> Phase 0 deliverable. This document defines what Peeper is allowed to run
> against. It is normative: the engine is built to honor it, not to route
> around it. If a use case is not covered here, it is out of scope.

## 1. What Peeper is

Peeper is a **credential-recovery and KDF-strength measurement engine**. Its
purpose is defensive: to measure, in real time and real money, how strong a
password-protected secret is against the best candidate-ordering attacks we can
build — and to feed that measurement back into hardening the defender's choices
(KDF parameters, password policy).

It is a red-team tool whose output is a blue-team artifact: a **resistance
report**, not a stolen password.

## 2. Authorized targets (the allow-list)

Peeper operates **only** on secrets we own or are explicitly authorized to test:

1. **The Obscura self-audit.** Our own password manager's vault, with its own
   master password, on our own hardware. This is the flagship use case.
2. **Synthetic / seeded test-user dumps.** Hash lists we generate ourselves to
   simulate an authorized audit of a user database.
3. **Public research corpora.** Already-leaked wordlists that are standard in
   the cited academic literature (RockYou and similar). Used for *training and
   evaluating* candidate models, and for benchmarking — never as a target to
   re-identify real people.

Anything else — a hash, vault, or wordlist belonging to a third party without
written authorization — is **out of scope** and the tool must not be pointed at
it. There is no "just testing" exception.

## 3. Data handling

- Candidate models train on **public, already-published** research corpora only.
  Never on data scraped from live systems, and never on real user data obtained
  without authorization.
- Real vaults, real hash lists, and any corpus are kept out of version control
  (see `.gitignore`: `/corpora/`, `/vaults/`, `.env`). The repository ships
  code and synthetic fixtures, never secrets or leaked data.
- A cracked secret from an authorized audit is reported to the audit's owner and
  not retained beyond the run.

## 4. Why this is safe to build in the open

Password strength measurement is standard, published security research. Every
offensive capability in Peeper maps to a defensive output:

| Capability | Defensive output |
|---|---|
| Candidate generation + verification | Measured guess-number curve for a given secret |
| KDF cost measurement (L4) | Honest `guesses/sec` for Argon2id on real hardware |
| Entropy / guesswork metrics (L6) | `alpha`-guesswork and cost-to-crack in GPU-years and dollars |
| The Obscura benchmark (T0–T4) | A resistance report that tunes Obscura's Argon2id `(m, t, p)` |
| Grover / weakest-link analysis (L6/§9) | Post-quantum bound on the vault's real weak link |

The honest frame holds: against a correctly-configured memory-hard KDF over a
high-entropy master password, no cracker wins. Peeper's value is in **proving
where that wall is** and showing when a configuration falls short of it.

## 5. Non-goals (explicitly excluded)

- No targeting of systems or accounts we do not own or control.
- No credential stuffing, no live-system attacks, no lateral movement — Peeper
  is compute over owned hashes/vaults, not a network tool.
- No de-anonymization of people in public leak corpora.
- No distribution of leaked data; corpora stay local and untracked.

## 6. Review

This scope is revisited at the start of each phase. Future extensions (the
deferred IoT firmware audit, for example) are in scope only once they are
brought under this same allow-list and data-handling policy.
