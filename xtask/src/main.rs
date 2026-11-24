// SPDX-License-Identifier: MIT
//! Workspace automation entry point (run via `cargo xtask <task>`).

// shortcut: grows real subcommands (dist, sbom, bench) as phases need them.
fn main() {
    let task = std::env::args().nth(1);
    match task.as_deref() {
        Some(other) => println!("xtask: unknown task {other:?}; none defined yet"),
        None => println!("xtask: usage: cargo xtask <task> (no tasks defined yet)"),
    }
}
