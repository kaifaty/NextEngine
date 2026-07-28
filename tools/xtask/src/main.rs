#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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
        "expected boundary-scan, content-package, host-check, performance, platform, play, physics-collision, physics-backend-parity, persistence-replay, v1-closure or v1-package".to_owned()
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
        "v1-closure" => {
            reject_extra_arguments(arguments)?;
            v1_closure()
        }
        "v1-package" => {
            let output = parse_v1_package_arguments(&mut arguments)?;
            reject_extra_arguments(arguments)?;
            v1_package(&root, &output)
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

fn parse_v1_package_arguments(
    arguments: &mut impl Iterator<Item = String>,
) -> Result<PathBuf, String> {
    let Some(flag) = arguments.next() else {
        return Err("v1-package requires --output <directory>".to_owned());
    };
    if flag != "--output" {
        return Err(format!("unexpected argument: {flag}"));
    }
    let output = arguments
        .next()
        .ok_or_else(|| "--output requires a package directory".to_owned())?;
    Ok(PathBuf::from(output))
}

fn v1_package(root: &Path, requested_output: &Path) -> Result<(), String> {
    let target_triple = shipping_target_triple()?;
    let output = if requested_output.is_absolute() {
        requested_output.to_path_buf()
    } else {
        root.join(requested_output)
    };
    if output.exists() {
        return Err(format!(
            "v1 package output already exists: {}",
            output.display()
        ));
    }
    let parent = output
        .parent()
        .ok_or_else(|| "v1 package output has no parent directory".to_owned())?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create package parent {}: {error}",
            parent.display()
        )
    })?;
    let file_name = output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "v1 package output must have a UTF-8 directory name".to_owned())?;
    let staging = parent.join(format!(".{file_name}.staging-{}", std::process::id()));
    if staging.exists() {
        return Err(format!(
            "v1 package staging directory already exists: {}",
            staging.display()
        ));
    }

    let result = (|| {
        run_checked(
            root,
            "cargo",
            &[
                "build",
                "--release",
                "-p",
                "next_game",
                "-p",
                "next_headless",
                "--features",
                "next_game/desktop-sdl-ash",
            ],
        )?;
        fs::create_dir_all(staging.join("bin"))
            .map_err(|error| format!("failed to create package bin directory: {error}"))?;
        let project_store = next_assets::ContentStore::new(staging.join("project"));
        let source =
            next_project::neutral_vertical_slice_source_v1().map_err(|error| error.to_string())?;
        let cooked = next_project::cook_project_v1(source).map_err(|error| error.to_string())?;
        project_store
            .publish(&cooked.publication().map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
        let activated =
            next_project::activate_project(&project_store).map_err(|error| error.to_string())?;
        if activated.composition_lock.composition_lock_sha256
            != cooked.composition_lock.composition_lock_sha256
        {
            return Err("packaged project activation lock mismatch".to_owned());
        }
        let executable_suffix = if cfg!(target_os = "windows") {
            ".exe"
        } else {
            ""
        };
        let game_name = format!("next_game{executable_suffix}");
        let headless_name = format!("next_headless{executable_suffix}");
        let release_directory = cargo_target_directory(root).join("release");
        let game_source = release_directory.join(&game_name);
        let headless_source = release_directory.join(&headless_name);
        let game_destination = staging.join("bin").join(&game_name);
        let headless_destination = staging.join("bin").join(&headless_name);
        fs::copy(&game_source, &game_destination).map_err(|error| {
            format!(
                "failed to package game binary {}: {error}",
                game_source.display()
            )
        })?;
        fs::copy(&headless_source, &headless_destination).map_err(|error| {
            format!(
                "failed to package headless binary {}: {error}",
                headless_source.display()
            )
        })?;
        let game_hash = file_hash(&game_destination)?;
        let headless_hash = file_hash(&headless_destination)?;
        let project_lock = cooked.composition_lock.composition_lock_sha256.to_hex();
        run_packaged_binary_smoke(
            root,
            &game_source,
            &headless_source,
            &staging.join("project"),
            &project_lock,
        )?;
        let manifest = format!(
            "{{\"composition_lock_sha256\":\"{}\",\"content_manifest_sha256\":\"{}\",\"game_binary\":\"bin/{}\",\"game_binary_sha256\":\"{}\",\"game_launch\":\"PASS\",\"headless_binary\":\"bin/{}\",\"headless_binary_sha256\":\"{}\",\"headless_launch\":\"PASS\",\"mechanics_lock_sha256\":\"{}\",\"schema_registry_sha256\":\"{}\",\"target_triple\":\"{}\",\"world_partition_sha256\":\"{}\"}}",
            project_lock,
            cooked.content_manifest.content_manifest_sha256.to_hex(),
            game_name,
            game_hash.to_hex(),
            headless_name,
            headless_hash.to_hex(),
            cooked
                .rpg_definitions
                .mechanics_lock
                .mechanics_lock_sha256
                .to_hex(),
            cooked
                .schema_registry
                .schema_registry_manifest_sha256
                .to_hex(),
            target_triple,
            cooked
                .world_partition
                .world_partition_manifest_sha256
                .to_hex(),
        );
        let manifest_hash =
            next_contracts::content_hash_from_bytes(next_contracts::sha256(manifest.as_bytes()));
        fs::write(staging.join("package.manifest.jcs"), manifest.as_bytes())
            .map_err(|error| format!("failed to write package manifest: {error}"))?;
        fs::rename(&staging, &output).map_err(|error| {
            format!(
                "failed to atomically publish package {}: {error}",
                output.display()
            )
        })?;
        println!(
            "{{\"status\":\"PASS\",\"target\":\"{}\",\"output\":\"{}\",\"package_manifest_hash\":\"{}\",\"composition_lock_hash\":\"{}\",\"game_binary_hash\":\"{}\",\"headless_binary_hash\":\"{}\",\"game_launch\":\"PASS\",\"headless_launch\":\"PASS\"}}",
            target_triple,
            output.display(),
            manifest_hash.to_hex(),
            cooked.composition_lock.composition_lock_sha256.to_hex(),
            game_hash.to_hex(),
            headless_hash.to_hex(),
        );
        Ok(())
    })();
    if result.is_err() && staging.exists() {
        fs::remove_dir_all(&staging).map_err(|error| {
            format!(
                "v1 package failed and staging cleanup also failed at {}: {error}",
                staging.display()
            )
        })?;
    }
    result
}

fn file_hash(path: &Path) -> Result<next_contracts::ContentHash, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    Ok(next_contracts::content_hash_from_bytes(
        next_contracts::sha256(&bytes),
    ))
}

fn run_packaged_binary_smoke(
    root: &Path,
    game: &Path,
    headless: &Path,
    project: &Path,
    project_lock: &str,
) -> Result<(), String> {
    run_project_binary(
        root,
        headless,
        &["--project"],
        project,
        &["--lock", project_lock],
        project_lock,
    )?;
    run_project_binary(
        root,
        game,
        &["--interactive", "--maximum-frames", "1", "--project"],
        project,
        &["--lock", project_lock],
        project_lock,
    )
}

fn run_project_binary(
    root: &Path,
    binary: &Path,
    arguments_before_project: &[&str],
    project: &Path,
    arguments_after_project: &[&str],
    project_lock: &str,
) -> Result<(), String> {
    let output = Command::new(binary)
        .args(arguments_before_project)
        .arg(project)
        .args(arguments_after_project)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to launch {}: {error}", binary.display()))?;
    if !output.status.success() {
        return Err(format!(
            "{} release smoke failed: {}",
            binary.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("{} emitted non-UTF-8 output: {error}", binary.display()))?;
    let expected_lock = format!("\"project_lock\":\"{project_lock}\"");
    if !stdout.contains("\"status\":\"PASS\"") || !stdout.contains(&expected_lock) {
        return Err(format!(
            "{} release smoke did not report the exact activated project lock",
            binary.display()
        ));
    }
    Ok(())
}

fn cargo_target_directory(root: &Path) -> PathBuf {
    env::var_os("CARGO_TARGET_DIR").map_or_else(
        || root.join("target"),
        |configured| {
            let configured = PathBuf::from(configured);
            if configured.is_absolute() {
                configured
            } else {
                root.join(configured)
            }
        },
    )
}

fn shipping_target_triple() -> Result<&'static str, String> {
    if cfg!(all(
        target_arch = "x86_64",
        target_os = "windows",
        target_env = "msvc"
    )) {
        Ok("x86_64-pc-windows-msvc")
    } else if cfg!(all(target_arch = "x86_64", target_os = "linux")) {
        Ok("x86_64-unknown-linux-gnu")
    } else {
        Err("TARGET_PACKAGE_REQUIRES_NATIVE_WINDOWS_OR_LINUX_X86_64".to_owned())
    }
}

fn v1_closure() -> Result<(), String> {
    let report = next_verification::run_v1_closure_check().map_err(|error| error.to_string())?;
    let status = if report.shipping_ready {
        "PASS"
    } else {
        "LOCAL_PASS_SHIPPING_TARGETS_NOT_RUN"
    };
    println!(
        "{{\"status\":\"{}\",\"shipping_ready\":{},\"checks\":{{\"content_package\":\"PASS\",\"play\":\"PASS\",\"persistence_replay\":\"PASS\",\"platform_contract\":\"PASS\",\"performance\":\"PASS\",\"headless_game_parity\":\"PASS\",\"fallback_without_ai_luau_wasm\":\"PASS\"}},\"project_composition_lock_hash\":\"{}\",\"schema_registry_hash\":\"{}\",\"content_manifest_hash\":\"{}\",\"mechanics_lock_hash\":\"{}\",\"world_partition_hash\":\"{}\",\"luau_manifest_hash\":\"{}\",\"wasm_manifest_hash\":\"{}\",\"wit_v2_hash\":\"{}\",\"wit_v3_hash\":\"{}\",\"extension_compatibility_hash\":\"{}\",\"play_state_root\":\"{}\",\"play_ledger_hash\":\"{}\",\"replay_state_root\":\"{}\",\"replay_ledger_hash\":\"{}\",\"windows\":{{\"target\":\"{}\",\"package_descriptor_hash\":\"{}\",\"runtime_check\":\"{}\",\"desktop_smoke\":\"{}\"}},\"linux\":{{\"target\":\"{}\",\"package_descriptor_hash\":\"{}\",\"runtime_check\":\"{}\",\"desktop_smoke\":\"{}\"}},\"closure_hash\":\"{}\"}}",
        status,
        report.shipping_ready,
        report.project_composition_lock_hash.to_hex(),
        report.schema_registry_hash.to_hex(),
        report.content_manifest_hash.to_hex(),
        report.mechanics_lock_hash.to_hex(),
        report.world_partition_hash.to_hex(),
        report.luau_manifest_hash.to_hex(),
        report.wasm_manifest_hash.to_hex(),
        report.wit_v2_hash.to_hex(),
        report.wit_v3_hash.to_hex(),
        report.extension_compatibility_hash.to_hex(),
        report.play_state_root.to_hex(),
        report.play_ledger_hash.to_hex(),
        report.replay_state_root.to_hex(),
        report.replay_ledger_hash.to_hex(),
        report.windows.target_triple,
        report.windows.package_descriptor_hash.to_hex(),
        target_status(&report.windows.runtime_check_status),
        target_status(&report.windows.desktop_smoke_status),
        report.linux.target_triple,
        report.linux.package_descriptor_hash.to_hex(),
        target_status(&report.linux.runtime_check_status),
        target_status(&report.linux.desktop_smoke_status),
        report.closure_hash.to_hex(),
    );
    Ok(())
}

fn target_status(status: &next_verification::TargetGateStatusV1) -> String {
    match status {
        next_verification::TargetGateStatusV1::Pass => "PASS".to_owned(),
        next_verification::TargetGateStatusV1::NotRun { reason } => {
            format!("NOT_RUN({reason})")
        }
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
    let candidate_status = match report.candidate_status {
        next_verification::PlatformCandidateStatus::Pass => "PASS",
        next_verification::PlatformCandidateStatus::NotRunOnDeveloperHost => {
            "NOT_RUN_DEVELOPER_HOST"
        }
        next_verification::PlatformCandidateStatus::NotRunAdapterDisabled => {
            "NOT_RUN_ADAPTER_DISABLED"
        }
    };
    println!(
        "{{\"status\":\"PASS\",\"portable_contract\":\"PASS\",\"sdl_ash_candidate\":\"{}\",\"normalized_events\":{},\"rendered_objects\":{},\"presentation_snapshot_hash\":\"{}\",\"ledger_hash\":\"{}\",\"state_root\":\"{}\"}}",
        candidate_status,
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
        "{{\"status\":\"PASS\",\"records\":{},\"chunks\":{},\"mechanic_packages\":{},\"luau_packages\":{},\"wasm_plugins\":{},\"combat_npc_health\":{},\"scripted_player_health\":{},\"wasm_player_health\":{},\"luau_state_hash\":\"{}\",\"wasm_state_hash\":\"{}\",\"wasm_host_api_major\":{},\"schema_registry_hash\":\"{}\",\"content_manifest_hash\":\"{}\",\"mechanics_lock_hash\":\"{}\",\"world_partition_hash\":\"{}\",\"composition_lock_hash\":\"{}\"}}",
        report.records,
        report.chunks,
        report.mechanic_packages,
        report.luau_packages,
        report.wasm_plugins,
        report.combat_npc_health,
        report.scripted_player_health,
        report.wasm_player_health,
        report.luau_package_state_hash.to_hex(),
        report.wasm_plugin_state_hash.to_hex(),
        report.wasm_host_api_major,
        report.schema_registry_hash.to_hex(),
        report.content_manifest_hash.to_hex(),
        report.mechanics_lock_hash.to_hex(),
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
        "{{\"status\":\"PASS\",\"ticks\":{},\"generations\":{},\"rpg_events\":{},\"interactive_object_state\":\"{}\",\"dialogue_node\":\"{}\",\"quest_state\":\"{}\",\"npc_player_trust\":{},\"npc_health\":{},\"player_health\":{},\"agent_intent\":\"{}\",\"agent_projection\":\"{}\",\"luau_state_hash\":\"{}\",\"wasm_state_hash\":\"{}\",\"world_streaming_generation\":{},\"current_chunk\":\"{}\",\"final_state_root\":\"{}\",\"final_ledger_root\":\"{}\"}}",
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
        report.luau_package_state_hash.to_hex(),
        report.wasm_plugin_state_hash.to_hex(),
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
