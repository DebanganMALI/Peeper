# Cross-crate tests

Workspace-level integration and known-answer tests that span several crates.
Crate-local tests live inside each crate (`src` unit tests and that crate's own
`tests/`). The first end-to-end pipeline test lives in `crates/peeper-cli/tests`.
