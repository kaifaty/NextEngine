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
        "expected boundary-scan, content-package, host-check, performance, platform, play, physics-collision, physics-backend-parity or persistence-replay".to_owned()
    })?;
    match command.as_str() {
        "boundary-scan" => {
            reject_extra_arguments(arguments)?;
            xtask::boundary_scan::boundary_scan(&root)
        }
        "content-package" => {
            reject_extra_arguments(arguments)?;
            content_package()
        }
        "host-check" => {
            reject_extra_arguments(arguments)?;
            host_check(&root)
        }
        "persistence-replay" => {
            let backend = parse_persistence_backend(&mut arguments)?;
            reject_extra_arguments(arguments)?;
            persistence_replay(backend)
        }
        "performance" => {
            reject_extra_arguments(arguments)?;
            performance()
        }
        "platform" => {
            reject_extra_arguments(arguments)?;
            platform()
        }
        "play" => {
            reject_extra_arguments(arguments)?;
            play()
        }
        "physics-collision" => {
            let backend = parse_physics_backend(&mut arguments)?;
            reject_extra_arguments(arguments)?;
            physics_collision(backend)
        }
        "physics-backend-parity" => {
            let (substeps, permutations) = parse_parity_counts(arguments)?;
            physics_backend_parity(substeps, permutations)
        }
        _ => Err(format!("unknown command: {command}")),
    }
}

fn performance() -> Result<(), String> {
    let streaming =
        next_verification::run_streaming_performance_check().map_err(|error| error.to_string())?;
    let agent = next_verification::run_agent_planning_performance_check()
        .map_err(|error| error.to_string())?;
    println!(
        "{{\"status\":\"PASS\",\"streaming\":{{\"cycles\":{},\"staged_asset_references\":{},\"elapsed_microseconds\":{},\"final_generation\":{},\"final_world_state_hash\":\"{}\"}},\"agent_planning\":{{\"cycles\":{},\"elapsed_microseconds\":{},\"final_plan_hash\":\"{}\"}}}}",
        streaming.cycles,
        streaming.staged_asset_references,
        streaming.elapsed_microseconds,
        streaming.final_generation,
        streaming.final_world_state_hash.to_hex(),
        agent.cycles,
        agent.elapsed_microseconds,
        agent.final_plan_hash.to_hex(),
    );
    Ok(())
}

fn platform() -> Result<(), String> {
    let report = next_verification::run_platform_check().map_err(|error| error.to_string())?;
    println!(
        "{{\"status\":\"PASS\",\"portable_contract\":\"PASS\",\"sdl_ash_candidate\":\"NOT_RUN_DEVELOPER_HOST\",\"normalized_events\":{},\"rendered_objects\":{},\"presentation_snapshot_hash\":\"{}\",\"ledger_hash\":\"{}\",\"state_root\":\"{}\"}}",
        report.normalized_events,
        report.rendered_objects,
        report.presentation_snapshot_hash.to_hex(),
        report.authoritative_ledger_hash.to_hex(),
        report.authoritative_state_root.to_hex(),
    );
    Ok(())
}

fn content_package() -> Result<(), String> {
    let report =
        next_verification::run_content_package_check().map_err(|error| error.to_string())?;
    println!(
        "{{\"status\":\"PASS\",\"records\":{},\"chunks\":{},\"mechanic_packages\":{},\"combat_npc_health\":{},\"schema_registry_hash\":\"{}\",\"content_manifest_hash\":\"{}\",\"world_partition_hash\":\"{}\",\"composition_lock_hash\":\"{}\"}}",
        report.records,
        report.chunks,
        report.mechanic_packages,
        report.combat_npc_health,
        report.schema_registry_hash.to_hex(),
        report.content_manifest_hash.to_hex(),
        report.world_partition_hash.to_hex(),
        report.composition_lock_hash.to_hex(),
    );
    Ok(())
}

fn physics_backend_parity(substeps: u64, permutations: u64) -> Result<(), String> {
    let report = next_verification::run_physics_backend_parity_check(substeps, permutations)
        .map_err(|error| error.to_string())?;
    println!(
        "{{\"status\":\"PASS\",\"compared_substeps\":{},\"registration_permutations\":{},\"world_lifecycle_cycles\":{}}}",
        report.compared_substeps, report.registration_permutations, report.world_lifecycle_cycles,
    );
    Ok(())
}

fn parse_parity_counts(mut arguments: impl Iterator<Item = String>) -> Result<(u64, u64), String> {
    let mut substeps = 100_000;
    let mut permutations = 10_000;
    let mut saw_substeps = false;
    let mut saw_permutations = false;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a positive integer"))?;
        let value = value
            .parse::<u64>()
            .map_err(|_| format!("{flag} requires a positive integer"))?;
        if value == 0 {
            return Err(format!("{flag} requires a positive integer"));
        }
        match flag.as_str() {
            "--substeps" if !saw_substeps => {
                substeps = value;
                saw_substeps = true;
            }
            "--permutations" if !saw_permutations => {
                permutations = value;
                saw_permutations = true;
            }
            "--substeps" | "--permutations" => {
                return Err(format!("duplicate argument: {flag}"));
            }
            _ => return Err(format!("unexpected argument: {flag}")),
        }
    }
    Ok((substeps, permutations))
}

fn physics_collision(backend: next_verification::PhysicsCollisionBackend) -> Result<(), String> {
    let report = next_verification::run_physics_collision_check_with_backend(backend)
        .map_err(|error| error.to_string())?;
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

fn parse_physics_backend(
    arguments: &mut impl Iterator<Item = String>,
) -> Result<next_verification::PhysicsCollisionBackend, String> {
    let Some(flag) = arguments.next() else {
        return Ok(next_verification::PhysicsCollisionBackend::Reference);
    };
    if flag != "--backend" {
        return Err(format!("unexpected argument: {flag}"));
    }
    match arguments.next().as_deref() {
        Some("reference") => Ok(next_verification::PhysicsCollisionBackend::Reference),
        Some("physx") => Ok(next_verification::PhysicsCollisionBackend::PhysX),
        Some("compare") => Ok(next_verification::PhysicsCollisionBackend::Compare),
        Some(value) => Err(format!("unknown physics backend: {value}")),
        None => Err("--backend requires reference, physx or compare".to_owned()),
    }
}

fn play() -> Result<(), String> {
    let report = next_verification::run_play_check().map_err(|error| error.to_string())?;
    let translation = report.final_pose.translation_micrometres;
    println!(
        "{{\"status\":\"PASS\",\"ticks\":{},\"final_pose_um\":[{},{},{}],\"events\":{},\"rpg_events\":{},\"interactive_object_state\":\"{}\",\"dialogue_node\":\"{}\",\"quest_state\":\"{}\",\"npc_player_trust\":{},\"npc_health\":{},\"player_health\":{},\"agent_intent\":\"{}\",\"agent_projection\":\"{}\",\"world_streaming_generation\":{},\"current_chunk\":\"{}\",\"ledger_hash\":\"{}\",\"state_root\":\"{}\"}}",
        report.ticks,
        translation[0],
        translation[1],
        translation[2],
        report.events,
        report.rpg_events,
        report.interactive_object_state.as_str(),
        report.dialogue_node_id.as_str(),
        report.quest_state_id.as_str(),
        report.npc_player_trust,
        report.npc_health,
        report.player_health,
        report.agent_intent_id.to_hex(),
        report.agent_projection_hash.to_hex(),
        report.world_streaming_generation,
        report.current_chunk_id.as_str(),
        report.final_command_ledger_hash.to_hex(),
        report.final_state_root.to_hex()
    );
    Ok(())
}

fn persistence_replay(backend: next_verification::PersistenceReplayBackend) -> Result<(), String> {
    let report = next_verification::run_persistence_replay_check_with_backend(backend)
        .map_err(|error| error.to_string())?;
    println!(
        "{{\"status\":\"PASS\",\"ticks\":{},\"generations\":{},\"rpg_events\":{},\"interactive_object_state\":\"{}\",\"dialogue_node\":\"{}\",\"quest_state\":\"{}\",\"npc_player_trust\":{},\"npc_health\":{},\"player_health\":{},\"agent_intent\":\"{}\",\"agent_projection\":\"{}\",\"world_streaming_generation\":{},\"current_chunk\":\"{}\",\"final_state_root\":\"{}\",\"final_ledger_root\":\"{}\"}}",
        report.ticks,
        report.generations,
        report.rpg_events,
        report.interactive_object_state,
        report.dialogue_node_id,
        report.quest_state_id,
        report.npc_player_trust,
        report.npc_health,
        report.player_health,
        report.agent_intent_id.to_hex(),
        report.agent_projection_hash.to_hex(),
        report.world_streaming_generation,
        report.current_chunk_id.as_str(),
        report.final_state_root.to_hex(),
        report.final_command_ledger_hash.to_hex()
    );
    Ok(())
}

fn parse_persistence_backend(
    arguments: &mut impl Iterator<Item = String>,
) -> Result<next_verification::PersistenceReplayBackend, String> {
    let Some(flag) = arguments.next() else {
        return Ok(next_verification::PersistenceReplayBackend::Reference);
    };
    if flag != "--backend" {
        return Err(format!("unexpected argument: {flag}"));
    }
    match arguments.next().as_deref() {
        Some("reference") => Ok(next_verification::PersistenceReplayBackend::Reference),
        Some("physx") => Ok(next_verification::PersistenceReplayBackend::PhysX),
        Some(value) => Err(format!("unknown persistence backend: {value}")),
        None => Err("--backend requires reference or physx".to_owned()),
    }
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
