#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

mod animation_lod_command;
mod native_gate_projection;
mod native_gate_publish;
mod native_gate_runner;
mod native_gate_schedule;
mod native_gate_target;
#[cfg(test)]
mod native_gate_tests;
mod performance_baseline_command;
mod performance_codegen_command;
mod performance_command;
mod physx;
mod visual_smoke;

use serde::{Serialize, Serializer};
use xtask::native_gate::{
    LINUX_TARGET_TRIPLE, NATIVE_GATE_SCHEMA_VERSION, NativeGateCheckNameV1,
    NativeGateCheckRecordV1, NativeGateCheckStatusV1, NativeGateClosureTargetSetV1,
    NativeGateClosureTargetSummaryV1, NativeGateComparableRootsV1, NativeGateDiagnosticV1,
    NativeGatePackageSummaryV1, NativeGatePackagedLaunchSummaryV1, NativeGateRunStatusV1,
    NativeGateTargetExecutionStatusV1, NativeGateTargetReportV1, WINDOWS_TARGET_TRIPLE,
};
use xtask::report::*;

#[derive(Clone, Debug, Eq, PartialEq)]
struct NativeGateIdentity {
    git_commit: String,
    git_object_format: String,
    cargo_lock_sha256: String,
    rustc_release: String,
    rustc_host: String,
    target_triple: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NativeGateCompareArguments {
    windows: PathBuf,
    linux: PathBuf,
    output: PathBuf,
}

struct NativeGateCheckFailure {
    records: Vec<NativeGateCheckRecordV1>,
    error: String,
}

struct NativeGateCheckExecutionFailure {
    record: NativeGateCheckRecordV1,
    error: String,
}

struct NativeGateMatrixSuccess {
    records: Vec<NativeGateCheckRecordV1>,
    closure_targets: NativeGateClosureTargetSetV1,
    comparable_roots: NativeGateComparableRootsV1,
    package: NativeGatePackageSummaryV1,
}

struct NativeGateClosureCheckResult {
    report: CommandReportV1<V1ClosureDetailsV1>,
    targets: NativeGateClosureTargetSetV1,
}

impl Serialize for NativeGateClosureCheckResult {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.report.serialize(serializer)
    }
}

struct NativeGatePackageCheckResult {
    report: CommandReportV1<PackageDetailsV1>,
    closure_targets: NativeGateClosureTargetSetV1,
    comparable_roots: NativeGateComparableRootsV1,
    package: NativeGatePackageSummaryV1,
}

impl Serialize for NativeGatePackageCheckResult {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.report.serialize(serializer)
    }
}

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
        "expected animation-lod, animation-root-motion, boundary-scan, content-package, host-check, native-gate-compare, native-gate-run, performance, performance-baseline, performance-codegen, physx, platform, play, physics-collision, physics-backend-parity, persistence-replay, visual-smoke, v1-closure or v1-package".to_owned()
    })?;
    match command.as_str() {
        "animation-lod" => {
            reject_extra_arguments(arguments)?;
            animation_lod_command::run()
        }
        "animation-root-motion" => {
            reject_extra_arguments(arguments)?;
            animation_root_motion()
        }
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
                        "unsafe_boundary_policy".to_owned(),
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
        "native-gate-run" => {
            let output = parse_native_gate_run_arguments(arguments)?;
            native_gate_runner::native_gate_run(&root, &output)
        }
        "native-gate-compare" => {
            let request = parse_native_gate_compare_arguments(arguments)?;
            native_gate_runner::native_gate_compare(&root, &request)
        }
        "persistence-replay" => {
            let backend = parse_persistence_backend(&mut arguments)?;
            reject_extra_arguments(arguments)?;
            persistence_replay(backend)
        }
        "performance" => {
            let request = performance_command::parse_arguments(arguments)?;
            performance_command::performance(&root, &request)
        }
        "performance-baseline" => {
            let request = performance_baseline_command::parse_arguments(arguments)?;
            performance_baseline_command::performance_baseline(&root, &request)
        }
        "performance-codegen" => {
            let request = performance_codegen_command::parse_arguments(arguments)?;
            performance_codegen_command::performance_codegen(&root, &request)
        }
        "physx" => physx::run(physx::parse_command(arguments)?),
        "platform" => {
            reject_extra_arguments(arguments)?;
            platform()
        }
        "audio-scene" => {
            reject_extra_arguments(arguments)?;
            audio_scene()
        }
        "play" => {
            reject_extra_arguments(arguments)?;
            play()
        }
        "visual-smoke" => {
            let request = visual_smoke::parse_arguments(arguments, &root)?;
            visual_smoke::run(&root, &request)
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

fn parse_native_gate_run_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<PathBuf, String> {
    let Some(flag) = arguments.next() else {
        return Err("native-gate-run requires --output <directory>".to_owned());
    };
    if flag != "--output" {
        return Err(format!("unexpected argument: {flag}"));
    }
    let output = arguments
        .next()
        .ok_or_else(|| "--output requires a native gate directory".to_owned())?;
    reject_extra_arguments(arguments)?;
    Ok(PathBuf::from(output))
}

fn parse_native_gate_compare_arguments(
    mut arguments: impl Iterator<Item = String>,
) -> Result<NativeGateCompareArguments, String> {
    let mut windows = None;
    let mut linux = None;
    let mut output = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("{flag} requires a path"))?;
        let slot = match flag.as_str() {
            "--windows" => &mut windows,
            "--linux" => &mut linux,
            "--output" => &mut output,
            _ => return Err(format!("unexpected argument: {flag}")),
        };
        if slot.replace(PathBuf::from(value)).is_some() {
            return Err(format!("duplicate argument: {flag}"));
        }
    }
    Ok(NativeGateCompareArguments {
        windows: windows.ok_or_else(|| {
            "native-gate-compare requires --windows <target-report.json>".to_owned()
        })?,
        linux: linux.ok_or_else(|| {
            "native-gate-compare requires --linux <target-report.json>".to_owned()
        })?,
        output: output.ok_or_else(|| {
            "native-gate-compare requires --output <cross-target-report.json>".to_owned()
        })?,
    })
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
    v1_package_report(root, requested_output)?.emit_report()
}

fn v1_package_report(
    root: &Path,
    requested_output: &Path,
) -> Result<CommandReportV1<PackageDetailsV1>, String> {
    let package = xtask::package::build_v1_package(root, requested_output)?;
    Ok(package_command_report(
        &package,
        package.output.display().to_string(),
    ))
}

fn package_command_report(
    package: &xtask::package::PackageBuildResult,
    output: String,
) -> CommandReportV1<PackageDetailsV1> {
    let manifest = &package.manifest;
    CommandReportV1::new(
        "v1-package",
        "PASS",
        PackageDetailsV1 {
            target: manifest.target_triple.clone(),
            output,
            package_manifest_hash: package.package_manifest_sha256.clone(),
            composition_lock_hash: manifest.target_neutral_roots.project_lock_sha256.clone(),
            game_binary_hash: manifest.binaries.game.binary_sha256.clone(),
            headless_binary_hash: manifest.binaries.headless.binary_sha256.clone(),
            game_launch: manifest.binaries.game.launch_status.clone(),
            headless_launch: manifest.binaries.headless.launch_status.clone(),
        },
    )
}

fn v1_closure() -> Result<(), String> {
    v1_closure_report(None)?.emit_report()
}

fn v1_closure_report(
    state_root: Option<&Path>,
) -> Result<CommandReportV1<V1ClosureDetailsV1>, String> {
    let _ = run_tool_session("tools-v1-closure-v2", state_root)?;
    let report = match state_root {
        Some(root) => next_verification::run_v1_closure_check_in(root),
        None => next_verification::run_v1_closure_check(),
    }
    .map_err(|error| error.to_string())?;
    let status = if report.shipping_ready {
        "PASS"
    } else {
        "LOCAL_PASS_SHIPPING_TARGETS_NOT_RUN"
    };
    Ok(CommandReportV1::new(
        "v1-closure",
        status,
        V1ClosureDetailsV1 {
            shipping_ready: report.shipping_ready,
            checks: vec![
                "content_package".to_owned(),
                "play".to_owned(),
                "persistence_replay".to_owned(),
                "audio_scene".to_owned(),
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
            audio_scene_pcm_digest: report.audio_scene_pcm_digest.to_hex(),
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
    ))
}

fn target_status(status: &next_verification::TargetGateStatusV1) -> String {
    match status {
        next_verification::TargetGateStatusV1::Pass => "PASS".to_owned(),
        next_verification::TargetGateStatusV1::NotRun { reason } => {
            format!("NOT_RUN({reason})")
        }
    }
}

fn platform() -> Result<(), String> {
    platform_report(None)?.emit_report()
}

fn platform_report(
    state_root: Option<&Path>,
) -> Result<CommandReportV1<PlatformDetailsV1>, String> {
    let _ = run_tool_session("tools-platform-v2", state_root)?;
    let report = match state_root {
        Some(root) => next_verification::run_platform_check_in(root),
        None => next_verification::run_platform_check(),
    }
    .map_err(|error| error.to_string())?;
    let candidate_status = match report.candidate_status {
        next_verification::PlatformCandidateStatus::Pass => "PASS",
        next_verification::PlatformCandidateStatus::NotRunOnDeveloperHost => {
            "NOT_RUN_DEVELOPER_HOST"
        }
        next_verification::PlatformCandidateStatus::NotRunAdapterDisabled => {
            "NOT_RUN_ADAPTER_DISABLED"
        }
    };
    Ok(CommandReportV1::new(
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
    ))
}

fn content_package() -> Result<(), String> {
    content_package_report(None)?.emit_report()
}

fn content_package_report(
    state_root: Option<&Path>,
) -> Result<CommandReportV1<ContentPackageDetailsV1>, String> {
    let report = match state_root {
        Some(root) => next_verification::run_content_package_check_in(root),
        None => next_verification::run_content_package_check(),
    }
    .map_err(|error| error.to_string())?;
    Ok(CommandReportV1::new(
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
    ))
}

fn physics_backend_parity(substeps: u64, permutations: u64) -> Result<(), String> {
    let _ = run_tool_session("tools-physics-backend-parity-v2", None)?;
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
    let _ = run_tool_session("tools-physics-collision-v2", None)?;
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

fn animation_root_motion() -> Result<(), String> {
    let report = next_verification::run_root_motion_conformance_check()
        .map_err(|error| error.to_string())?;
    CommandReportV1::emit(
        "animation-root-motion",
        "PASS",
        RootMotionConformanceDetailsV1 {
            cycles: report.cycles,
            accepted_cycles: report.accepted_cycles,
            rejected_cycles: report.rejected_cycles,
            retried_cycles: report.retried_cycles,
            save_load_cycles: report.save_load_cycles,
            lod_cycles: report.lod_cycles,
            canonical_proposal_round_trips: report.canonical_proposal_round_trips,
            motor_safety_decisions: report.motor_safety_decisions,
            replayed_cycles: report.replayed_cycles,
            full_motion_outcomes: report.full_motion_outcomes,
            clipped_motion_outcomes: report.clipped_motion_outcomes,
            blocked_motion_outcomes: report.blocked_motion_outcomes,
            fault_no_mutation_outcomes: report.fault_no_mutation_outcomes,
            lod_full_projection_probes: report.lod_full_projection_probes,
            lod_fallback_projection_probes: report.lod_fallback_projection_probes,
            final_pose_um: report.final_pose.translation_micrometres,
            final_physics_checkpoint_hash: report.final_physics_checkpoint_hash.to_hex(),
            final_command_ledger_hash: report.final_command_ledger_hash.to_hex(),
            matrix_digest: report.matrix_digest.to_hex(),
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
    // AUDIO-02 displayless audio gate runs as part of the audio routing
    // row's `play` product check: scene/PCM/acoustic-fact determinism and
    // cue acceptance must hold before the play report is emitted.
    let audio = next_verification::run_audio_scene_check().map_err(|error| error.to_string())?;
    if !audio.repeated_run_identical {
        return Err("AUDIO_SCENE_DETERMINISM_MISMATCH".to_owned());
    }
    let report = play_report(None)?;
    println!("{}", report.to_json().map_err(|error| error.to_string())?);
    Ok(())
}

#[derive(serde::Serialize)]
struct AudioSceneDetailsV1 {
    ticks: u64,
    cue_count: u64,
    acoustic_fact_count: u64,
    non_silent_windows: u64,
    pcm_frames: u64,
    pcm_digest: String,
    canonical_wav_digest: String,
    final_state_root: String,
    repeated_run_identical: bool,
}

fn audio_scene() -> Result<(), String> {
    let report = next_verification::run_audio_scene_check().map_err(|error| error.to_string())?;
    CommandReportV1::new(
        "audio-scene",
        "PASS",
        AudioSceneDetailsV1 {
            ticks: report.ticks,
            cue_count: report.cue_count,
            acoustic_fact_count: report.acoustic_fact_count,
            non_silent_windows: report.non_silent_windows,
            pcm_frames: report.pcm_frames,
            pcm_digest: report.pcm_digest.to_hex(),
            canonical_wav_digest: report.canonical_wav_digest.to_hex(),
            final_state_root: report.final_state_root.to_hex(),
            repeated_run_identical: report.repeated_run_identical,
        },
    )
    .emit_report()
}

fn play_report(state_root: Option<&Path>) -> Result<next_application::RunReportV1, String> {
    let expected = match state_root {
        Some(root) => next_verification::run_play_check_in(root),
        None => next_verification::run_play_check(),
    }
    .map_err(|error| error.to_string())?;
    let (run, close) = run_tool_session("tools-play-v2", state_root)?;
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
    Ok(report)
}

fn persistence_replay(backend: next_verification::PersistenceReplayBackend) -> Result<(), String> {
    persistence_replay_report(backend, None)?.emit_report()
}

fn persistence_replay_report(
    backend: next_verification::PersistenceReplayBackend,
    state_root: Option<&Path>,
) -> Result<CommandReportV1<PersistenceReplayDetailsV1>, String> {
    let _ = run_tool_session("tools-persistence-replay-v2", state_root)?;
    let report = match state_root {
        Some(root) => {
            next_verification::run_persistence_replay_check_with_backend_in(backend, root)
        }
        None => next_verification::run_persistence_replay_check_with_backend(backend),
    }
    .map_err(|error| error.to_string())?;
    Ok(CommandReportV1::new(
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
    ))
}

fn run_tool_session(
    application_id: &str,
    requested_state_root: Option<&Path>,
) -> Result<
    (
        next_application::ApplicationRunOutcomeV1,
        next_application::ApplicationCloseOutcomeV2,
    ),
    String,
> {
    let state_root = match requested_state_root {
        Some(state_root) => state_root.to_path_buf(),
        None => next_application::default_user_state_root(application_id)
            .map_err(|error| format!("{}: {error}", error.diagnostic_code()))?,
    };
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
        .close()
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
    host_check_report(root, None)?.emit_report()
}

fn host_check_report(
    root: &Path,
    state_root: Option<&Path>,
) -> Result<CommandReportV1<HostCheckDetailsV1>, String> {
    let version = run_output_with_state(root, "rustc", &["-vV"], state_root)?;
    let details = String::from_utf8(version.stdout).map_err(|error| error.to_string())?;
    if !details.lines().any(|line| line == "release: 1.97.1") {
        return Err("rustc release must be exactly 1.97.1".to_owned());
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

    run_checked(
        root,
        "cargo",
        &["fmt", "--all", "--", "--check"],
        state_root,
    )?;
    run_checked(
        root,
        "cargo",
        &host_check_clippy_arguments(state_root, cfg!(feature = "desktop-sdl-ash")),
        state_root,
    )?;
    run_checked(
        root,
        "cargo",
        &host_check_test_arguments(state_root, cfg!(feature = "desktop-sdl-ash")),
        state_root,
    )?;
    xtask::boundary_scan::boundary_scan(root)?;
    Ok(CommandReportV1::new(
        "host-check",
        "PASS",
        HostCheckDetailsV1 {
            host: host.to_owned(),
            rustc_release: "1.97.1".to_owned(),
        },
    ))
}

const DESKTOP_HOST_CHECK_FEATURES: &str =
    "xtask/desktop-sdl-ash,next_verification/desktop-sdl-ash,next_game/desktop-sdl-ash";

fn host_check_clippy_arguments(
    state_root: Option<&Path>,
    desktop_sdl_ash: bool,
) -> Vec<&'static str> {
    let mut arguments = vec!["clippy"];
    if state_root.is_some() {
        arguments.push("--locked");
    }
    arguments.extend(["--workspace", "--all-targets"]);
    if desktop_sdl_ash {
        arguments.extend(["--features", DESKTOP_HOST_CHECK_FEATURES]);
    }
    arguments.extend(["--", "-D", "warnings"]);
    arguments
}

fn host_check_test_arguments(
    state_root: Option<&Path>,
    desktop_sdl_ash: bool,
) -> Vec<&'static str> {
    let mut arguments = vec!["test"];
    if state_root.is_some() {
        arguments.push("--locked");
    }
    arguments.push("--workspace");
    if desktop_sdl_ash {
        arguments.extend(["--features", DESKTOP_HOST_CHECK_FEATURES]);
    }
    arguments
}

fn run_checked(
    root: &Path,
    program: &str,
    arguments: &[&str],
    state_root: Option<&Path>,
) -> Result<(), String> {
    let output = run_output_with_state(root, program, arguments, state_root)?;
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

fn run_output_with_state(
    root: &Path,
    program: &str,
    arguments: &[&str],
    state_root: Option<&Path>,
) -> Result<Output, String> {
    let mut command = Command::new(program);
    command.args(arguments).current_dir(root);
    if let Some(state_root) = state_root {
        let local_app_data = state_root.join("local-app-data");
        let xdg_state_home = state_root.join("xdg-state");
        let roaming_app_data = state_root.join("roaming-app-data");
        let temporary = state_root.join("temp");
        for directory in [
            &local_app_data,
            &xdg_state_home,
            &roaming_app_data,
            &temporary,
        ] {
            fs::create_dir_all(directory).map_err(|error| {
                format!(
                    "failed to create isolated host-check directory {}: {error}",
                    directory.display()
                )
            })?;
        }
        command
            .env("LOCALAPPDATA", local_app_data)
            .env("APPDATA", roaming_app_data)
            .env("XDG_STATE_HOME", xdg_state_home)
            .env("TMP", &temporary)
            .env("TEMP", &temporary)
            .env("TMPDIR", temporary);
    }
    command
        .output()
        .map_err(|error| format!("failed to run {program}: {error}"))
}

fn diagnostic_code(error: &str) -> &'static str {
    if error.starts_with("PHYSX_SDK_INCOMPLETE") {
        "PHYSX_SDK_NOT_PREPARED"
    } else if error.starts_with("PHYSX_MANIFEST_MISMATCH")
        || error.starts_with("PHYSX_VERSION_MISMATCH")
        || error.starts_with("PHYSX_ARCHIVE_HASH_MISMATCH")
    {
        "PHYSX_PROFILE_MISMATCH"
    } else if error.starts_with("PHYSX_") {
        "PHYSX_SETUP_FAILED"
    } else if error.starts_with("NATIVE_GATE_WORKTREE_DIRTY") {
        "NATIVE_GATE_WORKTREE_DIRTY"
    } else if error.starts_with("NATIVE_GATE_HEAD_CHANGED") {
        "NATIVE_GATE_HEAD_CHANGED"
    } else if error.starts_with("NATIVE_GATE_UNSUPPORTED_TARGET") {
        "NATIVE_GATE_UNSUPPORTED_TARGET"
    } else if error.starts_with("NATIVE_GATE_DESKTOP_ADAPTER_DISABLED") {
        "NATIVE_GATE_DESKTOP_ADAPTER_DISABLED"
    } else if error.starts_with("NATIVE_GATE_CHECK_FAILED") {
        "NATIVE_GATE_CHECK_FAILED"
    } else if error.starts_with("NATIVE_GATE_REPORT_INVALID") {
        "NATIVE_GATE_REPORT_INVALID"
    } else if error.starts_with("NATIVE_GATE_TARGET_SET_INVALID") {
        "NATIVE_GATE_TARGET_SET_INVALID"
    } else if error.starts_with("NATIVE_GATE_COMMIT_MISMATCH") {
        "NATIVE_GATE_COMMIT_MISMATCH"
    } else if error.starts_with("NATIVE_GATE_ROOT_MISMATCH") {
        "NATIVE_GATE_ROOT_MISMATCH"
    } else if error.starts_with("NATIVE_GATE_PACKAGE_RUNTIME_PROFILE_INVALID") {
        "NATIVE_GATE_PACKAGE_RUNTIME_PROFILE_INVALID"
    } else if error.starts_with("NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING") {
        "NATIVE_GATE_PACKAGE_RUNTIME_DEPENDENCY_MISSING"
    } else if error.starts_with("NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED") {
        "NATIVE_GATE_PACKAGE_RUNTIME_ABI_UNSUPPORTED"
    } else if error.starts_with("NATIVE_GATE_PACKAGE_SMOKE_TIMEOUT") {
        "NATIVE_GATE_PACKAGE_SMOKE_TIMEOUT"
    } else if error.starts_with("NATIVE_GATE_PACKAGE_INVALID") {
        "NATIVE_GATE_PACKAGE_INVALID"
    } else if error.starts_with("NATIVE_GATE_OUTPUT_EXISTS") {
        "NATIVE_GATE_OUTPUT_EXISTS"
    } else if error.contains("argument")
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
