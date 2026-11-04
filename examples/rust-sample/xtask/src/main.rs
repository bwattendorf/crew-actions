//! `cargo xtask ci`: the one gate command. Runs each step in order, stops at
//! the first failure and prints a pass/fail/skip summary.

use std::process::{Command, ExitCode};

const STEPS: &[(&str, &[&str])] = &[
    ("format", &["fmt", "--all", "--check"]),
    ("lint", &["clippy", "--all-targets", "--", "-D", "warnings"]),
    ("test", &["test", "--workspace"]),
];

fn main() -> ExitCode {
    let task = std::env::args().nth(1).unwrap_or_default();
    if task != "ci" {
        eprintln!("usage: cargo xtask ci");
        return ExitCode::from(2);
    }
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut results = Vec::new();
    let mut failed = false;
    for (name, args) in STEPS {
        if failed {
            results.push((*name, "skip"));
            continue;
        }
        println!("== {name}: cargo {}", args.join(" "));
        let ok = Command::new(&cargo)
            .args(*args)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        results.push((*name, if ok { "pass" } else { "FAIL" }));
        failed = !ok;
    }
    println!("== summary");
    for (name, result) in &results {
        println!("  {result:<4}  {name}");
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
