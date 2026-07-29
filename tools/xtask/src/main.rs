#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

mod report;

use report::*;

fn main() {
    if let Err(error) = run() {
        eprintln!("xtask: {error}");
        let report = next_application::DiagnosticReportV1::new(
            diagnostic_code(&error),
            next_application::DiagnosticContextV1::message(&error),
        );
        println!("{}", report.to_json().expect("diagnostic serializes"));
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
            xtask::boundary_scan::boundary_scan(&root)?;
            CommandReportV1::emit(
                "boundary-scan",
                "PASS",
                BoundaryScanDetailsV1 {
                    checks: vec![
                        "source_layout".to_owned(),
                        "publish_policy".to_owned(),
                        "public_contracts".to_owned(),
                        "production_verification_boundary".to_owned(),
                        "importer_boundary".to_owned(),
                        "ffi_policy".to_owned(),
                    ],
                },
            )
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
        let source = next_reference_game::project_source_v2().map_err(|error| error.to_string())?;
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
        let manifest = PackageManifestV1 {
            composition_lock_sha256: project_lock,
            content_manifest_sha256: cooked.content_manifest.content_manifest_sha256.to_hex(),
            game_binary: format!("bin/{game_name}"),
            game_binary_sha256: game_hash.to_hex(),
            game_launch: "PASS".to_owned(),
            headless_binary: format!("bin/{headless_name}"),
            headless_binary_sha256: headless_hash.to_hex(),
            headless_launch: "PASS".to_owned(),
            mechanics_lock_sha256: cooked
                .rpg_definitions
                .mechanics_lock
                .mechanics_lock_sha256
                .to_hex(),
            schema_registry_sha256: cooked
                .schema_registry
                .schema_registry_manifest_sha256
                .to_hex(),
            target_triple: target_triple.to_owned(),
            world_partition_sha256: cooked
                .world_partition
                .world_partition_manifest_sha256
                .to_hex(),
        };
        let manifest = serde_json::to_vec(&manifest).map_err(|error| error.to_string())?;
        let manifest_hash = next_contracts::ids::content_hash_from_bytes(
            next_contracts::canonical::sha256(&manifest),
        );
        fs::write(staging.join("package.manifest.jcs"), &manifest)
            .map_err(|error| format!("failed to write package manifest: {error}"))?;
        fs::rename(&staging, &output).map_err(|error| {
            format!(
                "failed to atomically publish package {}: {error}",
                output.display()
            )
        })?;
        CommandReportV1::emit(
            "v1-package",
            "PASS",
            PackageDetailsV1 {
                target: target_triple.to_owned(),
                output: output.display().to_string(),
                package_manifest_hash: manifest_hash.to_hex(),
                composition_lock_hash: cooked.composition_lock.composition_lock_sha256.to_hex(),
                game_binary_hash: game_hash.to_hex(),
                headless_binary_hash: headless_hash.to_hex(),
                game_launch: "PASS".to_owned(),
                headless_launch: "PASS".to_owned(),
            },
        )
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

fn file_hash(path: &Path) -> Result<next_contracts::ids::ContentHash, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    Ok(next_contracts::ids::content_hash_from_bytes(
        next_contracts::canonical::sha256(&bytes),
    ))
}

fn run_packaged_binary_smoke(
    root: &Path,
    game: &Path,
    headless: &Path,
    project: &Path,
    project_lock: &str,
) -> Result<(), String> {
    let package_root = project
        .parent()
        .ok_or_else(|| "packaged project has no parent directory".to_owned())?;
    run_project_binary(
        root,
        headless,
        &["--project"],
        project,
        &["--lock", project_lock],
        &package_root.join("state-headless"),
        project_lock,
    )?;
    run_project_binary(
        root,
        game,
        &["--interactive", "--maximum-frames", "1", "--project"],
        project,
        &["--lock", project_lock],
        &package_root.join("state-game"),
        project_lock,
    )
}

fn run_project_binary(
    root: &Path,
    binary: &Path,
    arguments_before_project: &[&str],
    project: &Path,
    arguments_after_project: &[&str],
    state_root: &Path,
    project_lock: &str,
) -> Result<(), String> {
    let output = Command::new(binary)
        .args(arguments_before_project)
        .arg(project)
        .args(arguments_after_project)
        .arg("--state-root")
        .arg(state_root)
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
    let report: next_application::RunReportV1 =
        serde_json::from_str(stdout.trim()).map_err(|error| {
            format!(
                "{} emitted an invalid run report: {error}",
                binary.display()
            )
        })?;
    if report.status != "PASS" || report.project_composition_lock_hash != project_lock {
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
    let _ = run_tool_session("tools-v1-closure")?;
    let report = next_verification::run_v1_closure_check().map_err(|error| error.to_string())?;
    let status = if report.shipping_ready {
        "PASS"
    } else {
        "LOCAL_PASS_SHIPPING_TARGETS_NOT_RUN"
    };
    CommandReportV1::emit(
        "v1-closure",
        status,
        V1ClosureDetailsV1 {
            shipping_ready: report.shipping_ready,
            checks: vec![
                "content_package".to_owned(),
                "play".to_owned(),
                "persistence_replay".to_owned(),
                "platform_contract".to_owned(),
                "performance".to_owned(),
                "headless_game_parity".to_owned(),
                "fallback_without_ai_luau_wasm".to_owned(),
            ],
            project_composition_lock_hash: report.project_composition_lock_hash.to_hex(),
            schema_registry_hash: report.schema_registry_hash.to_hex(),
            content_manifest_hash: report.content_manifest_hash.to_hex(),
            mechanics_lock_hash: report.mechanics_lock_hash.to_hex(),
            world_partition_hash: report.world_partition_hash.to_hex(),
            luau_manifest_hash: report.luau_manifest_hash.to_hex(),
            wasm_manifest_hash: report.wasm_manifest_hash.to_hex(),
            wit_v2_hash: report.wit_v2_hash.to_hex(),
            wit_v3_hash: report.wit_v3_hash.to_hex(),
            extension_compatibility_hash: report.extension_compatibility_hash.to_hex(),
            play_state_root: report.play_state_root.to_hex(),
            play_ledger_hash: report.play_ledger_hash.to_hex(),
            replay_state_root: report.replay_state_root.to_hex(),
            replay_ledger_hash: report.replay_ledger_hash.to_hex(),
            windows: TargetGateDetailsV1 {
                target: report.windows.target_triple.to_owned(),
                package_descriptor_hash: report.windows.package_descriptor_hash.to_hex(),
                runtime_check: target_status(&report.windows.runtime_check_status),
                desktop_smoke: target_status(&report.windows.desktop_smoke_status),
            },
            linux: TargetGateDetailsV1 {
                target: report.linux.target_triple.to_owned(),
                package_descriptor_hash: report.linux.package_descriptor_hash.to_hex(),
                runtime_check: target_status(&report.linux.runtime_check_status),
                desktop_smoke: target_status(&report.linux.desktop_smoke_status),
            },
            closure_hash: report.closure_hash.to_hex(),
        },
    )
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
    let _ = run_tool_session("tools-performance")?;
    let streaming =
        next_verification::run_streaming_performance_check().map_err(|error| error.to_string())?;
    let agent = next_verification::run_agent_planning_performance_check()
        .map_err(|error| error.to_string())?;
    CommandReportV1::emit(
        "performance",
        "PASS",
        PerformanceDetailsV1 {
            streaming: StreamingPerformanceDetailsV1 {
                cycles: streaming.cycles,
                staged_asset_references: streaming.staged_asset_references,
                elapsed_microseconds: streaming.elapsed_microseconds,
                final_generation: streaming.final_generation,
                final_world_state_hash: streaming.final_world_state_hash.to_hex(),
            },
            agent_planning: AgentPerformanceDetailsV1 {
                cycles: agent.cycles,
                elapsed_microseconds: agent.elapsed_microseconds,
                final_plan_hash: agent.final_plan_hash.to_hex(),
            },
        },
    )
}

fn platform() -> Result<(), String> {
    let _ = run_tool_session("tools-platform")?;
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
    CommandReportV1::emit(
        "platform",
        "PASS",
        PlatformDetailsV1 {
            portable_contract: "PASS".to_owned(),
            sdl_ash_candidate: candidate_status.to_owned(),
            normalized_events: report.normalized_events,
            rendered_objects: report.rendered_objects,
            presentation_snapshot_hash: report.presentation_snapshot_hash.to_hex(),
            ledger_hash: report.authoritative_ledger_hash.to_hex(),
            state_root: report.authoritative_state_root.to_hex(),
        },
    )
}

fn content_package() -> Result<(), String> {
    let report =
        next_verification::run_content_package_check().map_err(|error| error.to_string())?;
    CommandReportV1::emit(
        "content-package",
        "PASS",
        ContentPackageDetailsV1 {
            records: report.records,
            chunks: report.chunks,
            mechanic_packages: report.mechanic_packages,
            luau_packages: report.luau_packages,
            wasm_plugins: report.wasm_plugins,
            combat_npc_health: report.combat_npc_health,
            scripted_player_health: report.scripted_player_health,
            wasm_player_health: report.wasm_player_health,
            luau_state_hash: report.luau_package_state_hash.to_hex(),
            wasm_state_hash: report.wasm_plugin_state_hash.to_hex(),
            wasm_host_api_major: report.wasm_host_api_major,
            schema_registry_hash: report.schema_registry_hash.to_hex(),
            content_manifest_hash: report.content_manifest_hash.to_hex(),
            mechanics_lock_hash: report.mechanics_lock_hash.to_hex(),
            world_partition_hash: report.world_partition_hash.to_hex(),
            composition_lock_hash: report.composition_lock_hash.to_hex(),
        },
    )
}

fn physics_backend_parity(substeps: u64, permutations: u64) -> Result<(), String> {
    let _ = run_tool_session("tools-physics-backend-parity")?;
    let report = next_verification::run_physics_backend_parity_check(substeps, permutations)
        .map_err(|error| error.to_string())?;
    CommandReportV1::emit(
        "physics-backend-parity",
        "PASS",
        PhysicsParityDetailsV1 {
            compared_substeps: report.compared_substeps,
            registration_permutations: report.registration_permutations,
            world_lifecycle_cycles: report.world_lifecycle_cycles,
        },
    )
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
    let _ = run_tool_session("tools-physics-collision")?;
    let report = next_verification::run_physics_collision_check_with_backend(backend)
        .map_err(|error| error.to_string())?;
    let translation = report.final_pose.translation_micrometres;
    CommandReportV1::emit(
        "physics-collision",
        "PASS",
        PhysicsCollisionDetailsV1 {
            gameplay_ticks: report.gameplay_ticks,
            physics_substeps: report.physics_substeps,
            begin_contacts: report.begin_contacts,
            persist_contacts: report.persist_contacts,
            end_contacts: report.end_contacts,
            final_pose_um: translation,
            contact_batches_hash: report.contact_batches_hash.to_hex(),
            physics_checkpoint_hash: report.physics_checkpoint_hash.to_hex(),
        },
    )
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
    let expected = next_verification::run_play_check().map_err(|error| error.to_string())?;
    let (run, close) = run_tool_session("tools-play")?;
    if run.ticks != expected.ticks
        || run.events != expected.events
        || run.rpg_events != expected.rpg_events
        || run.authoritative_state_root.as_bytes() != expected.final_state_root.as_bytes()
        || run.command_ledger_hash.as_bytes() != expected.final_command_ledger_hash.as_bytes()
    {
        return Err("PLAY_APPLICATION_SESSION_PARITY_MISMATCH".to_owned());
    }
    let report = next_application::RunReportV1::new(
        next_contracts::session::CompositionRootV1::Tools,
        &run,
        &close,
        0,
    )
    .ok_or_else(|| "SESSION_TERMINAL_RECEIPT_MISSING".to_owned())?;
    println!("{}", report.to_json().map_err(|error| error.to_string())?);
    Ok(())
}

fn persistence_replay(backend: next_verification::PersistenceReplayBackend) -> Result<(), String> {
    let _ = run_tool_session("tools-persistence-replay")?;
    let report = next_verification::run_persistence_replay_check_with_backend(backend)
        .map_err(|error| error.to_string())?;
    CommandReportV1::emit(
        "persistence-replay",
        "PASS",
        PersistenceReplayDetailsV1 {
            ticks: report.ticks,
            generations: report.generations,
            rpg_events: report.rpg_events,
            interactive_object_state: report.interactive_object_state.as_str().to_owned(),
            dialogue_node: report.dialogue_node_id.as_str().to_owned(),
            quest_state: report.quest_state_id.as_str().to_owned(),
            npc_player_trust: report.npc_player_trust,
            npc_health: report.npc_health,
            player_health: report.player_health,
            agent_intent: report.agent_intent_id.to_hex(),
            agent_projection: report.agent_projection_hash.to_hex(),
            luau_state_hash: report.luau_package_state_hash.to_hex(),
            wasm_state_hash: report.wasm_plugin_state_hash.to_hex(),
            world_streaming_generation: report.world_streaming_generation,
            current_chunk: report.current_chunk_id.as_str().to_owned(),
            final_state_root: report.final_state_root.to_hex(),
            final_ledger_root: report.final_command_ledger_hash.to_hex(),
        },
    )
}

fn run_tool_session(
    application_id: &str,
) -> Result<
    (
        next_application::ApplicationRunOutcomeV1,
        next_application::ApplicationCloseOutcomeV1,
    ),
    String,
> {
    let state_root = next_application::default_user_state_root(application_id)
        .map_err(|error| format!("{}: {error}", error.diagnostic_code()))?;
    let launch = next_application::LaunchRequestV1::reference(
        state_root,
        next_contracts::session::CompositionRootV1::Tools,
        next_contracts::session::PresentationTargetKindV1::None,
    );
    let mut application = next_application::ApplicationCoordinator::launch_or_resume(launch)
        .map_err(|error| format!("{}: {error}", error.diagnostic_code()))?;
    let run = application
        .run_reference_game(true)
        .map_err(|error| format!("{}: {error}", error.diagnostic_code()))?;
    let close = application
        .close(next_application::CloseExecutionOptionsV1::default())
        .map_err(|error| format!("{}: {error}", error.diagnostic_code()))?;
    Ok((run, close))
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
    CommandReportV1::emit(
        "host-check",
        "PASS",
        HostCheckDetailsV1 {
            host: host.to_owned(),
            rustc_release: "1.93.0".to_owned(),
        },
    )
}

fn run_checked(root: &Path, program: &str, arguments: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to run {program}: {error}"))?;
    if !output.stdout.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{program} {} failed with {}",
            arguments.join(" "),
            output.status,
        ))
    }
}

fn diagnostic_code(error: &str) -> &'static str {
    if error.contains("argument")
        || error.contains("requires")
        || error.contains("unknown command")
        || error.contains("unexpected")
    {
        "CLI_ARGUMENT_INVALID"
    } else if error.contains("PROJECT_LOCK_MISMATCH") {
        "PROJECT_LOCK_MISMATCH"
    } else if error.contains("SESSION_") {
        "SESSION_RUNTIME_FAILED"
    } else {
        "XTASK_COMMAND_FAILED"
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
