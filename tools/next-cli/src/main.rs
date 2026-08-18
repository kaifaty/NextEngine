#![forbid(unsafe_code)]

use std::io::{self, Write};

fn main() {
    let report = next_cli::execute(std::env::args_os().skip(1));
    let passed = report.is_pass();
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &report).expect("creator report serializes");
    stdout
        .write_all(b"\n")
        .expect("creator report writes to stdout");
    if !passed {
        eprintln!("next: command failed; inspect the JSON diagnostic on stdout");
        std::process::exit(2);
    }
}
