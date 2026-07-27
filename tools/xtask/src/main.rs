#![forbid(unsafe_code)]

use std::env;
use std::path::Path;
use std::process::{Command, Output};

fn main() {
    if let Err(error) = run() {
        eprintln!("xtask: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let root = env::current_dir().map_err(|error| error.to_string())?;
    let mut arguments = env::args().skip(1);
    let command = arguments.next().ok_or_else(|| {
        "expected boundary-scan, host-check, play, physics-collision or persistence-replay"
            .to_owned()
    })?;
    match command.as_str() {
        "boundary-scan" => {
            reject_extra_arguments(arguments)?;
            xtask::boundary_scan::boundary_scan(&root)
        }
        "host-check" => {
            reject_extra_arguments(arguments)?;
            host_check(&root)
        }
        "persistence-replay" => {
            reject_extra_arguments(arguments)?;
            persistence_replay()
        }
        "play" => {
            reject_extra_arguments(arguments)?;
            play()
        }
        "physics-collision" => {
            reject_extra_arguments(arguments)?;
            physics_collision()
        }
        _ => Err(format!("unknown command: {command}")),
    }
}

fn physics_collision() -> Result<(), String> {
    let report =
        next_verification::run_physics_collision_check().map_err(|error| error.to_string())?;
    let translation = report.final_pose.translation_micrometres;
    println!(
        "{{\"status\":\"PASS\",\"gameplay_ticks\":{},\"physics_substeps\":{},\"contacts\":{{\"begin\":{},\"persist\":{},\"end\":{}}},\"final_pose_um\":[{},{},{}],\"contact_batches_hash\":\"{}\",\"physics_checkpoint_hash\":\"{}\"}}",
        report.gameplay_ticks,
        report.physics_substeps,
        report.begin_contacts,
        report.persist_contacts,
        report.end_contacts,
        translation[0],
        translation[1],
        translation[2],
        report.contact_batches_hash.to_hex(),
        report.physics_checkpoint_hash.to_hex()
    );
    Ok(())
}

fn play() -> Result<(), String> {
    let report = next_verification::run_play_check().map_err(|error| error.to_string())?;
    let translation = report.final_pose.translation_micrometres;
    println!(
        "{{\"status\":\"PASS\",\"ticks\":{},\"final_pose_um\":[{},{},{}],\"events\":{},\"ledger_hash\":\"{}\",\"state_root\":\"{}\"}}",
        report.ticks,
        translation[0],
        translation[1],
        translation[2],
        report.events,
        report.final_command_ledger_hash.to_hex(),
        report.final_state_root.to_hex()
    );
    Ok(())
}

fn persistence_replay() -> Result<(), String> {
    let report =
        next_verification::run_persistence_replay_check().map_err(|error| error.to_string())?;
    println!(
        "{{\"status\":\"PASS\",\"ticks\":{},\"generations\":{},\"final_state_root\":\"{}\",\"final_ledger_root\":\"{}\"}}",
        report.ticks,
        report.generations,
        report.final_state_root.to_hex(),
        report.final_command_ledger_hash.to_hex()
    );
    Ok(())
}

fn reject_extra_arguments(mut arguments: impl Iterator<Item = String>) -> Result<(), String> {
    match arguments.next() {
        Some(argument) => Err(format!("unexpected argument: {argument}")),
        None => Ok(()),
    }
}

fn host_check(root: &Path) -> Result<(), String> {
    let version = run_output(root, "rustc", &["-vV"])?;
    let details = String::from_utf8(version.stdout).map_err(|error| error.to_string())?;
    if !details.lines().any(|line| line == "release: 1.93.0") {
        return Err("rustc release must be exactly 1.93.0".to_owned());
    }
    let host = details
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or_else(|| "rustc did not report host".to_owned())?;
    let supported = [
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
        "x86_64-unknown-linux-gnu",
    ];
    if !supported.contains(&host) {
        return Err(format!("unsupported bootstrap host: {host}"));
    }

    run_checked(root, "cargo", &["fmt", "--all", "--", "--check"])?;
    run_checked(
        root,
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    run_checked(root, "cargo", &["test", "--workspace"])?;
    xtask::boundary_scan::boundary_scan(root)?;
    println!("PASS host-check: host={host}, rustc=1.93.0");
    Ok(())
}

fn run_checked(root: &Path, program: &str, arguments: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(arguments)
        .current_dir(root)
        .status()
        .map_err(|error| format!("failed to run {program}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{program} {} failed with {status}",
            arguments.join(" ")
        ))
    }
}

fn run_output(root: &Path, program: &str, arguments: &[&str]) -> Result<Output, String> {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to run {program}: {error}"))?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(format!(
            "{program} {} failed: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
