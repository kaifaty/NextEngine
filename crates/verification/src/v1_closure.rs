use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, content_hash_from_bytes};

use crate::PersistenceReplayBackend;
use crate::agent_performance::run_agent_planning_performance_check_with_scratch;
use crate::content_package::run_content_package_check_with_scratch;
use crate::persistence_replay::run_persistence_replay_check_with_backend_and_scratch;
use crate::platform_check::run_platform_check_with_scratch;
use crate::player_fixture::run_play_check_with_scratch;
use crate::render_performance::run_render_frame_planning_performance_check_with_scratch;
use crate::scratch::ScratchContext;
use crate::streaming_performance::run_streaming_performance_check_with_scratch;

pub const RPG_SCHEMA_UNSUPPORTED_DIAGNOSTIC: &str = "RPG_SCHEMA_UNSUPPORTED";
pub const WIT_N_MINUS_2_UNSUPPORTED_DIAGNOSTIC: &str = "WIT_API_N_MINUS_2_UNSUPPORTED";
pub const OPTIONAL_EXTENSION_DISABLED_DIAGNOSTIC: &str = "OPTIONAL_EXTENSION_DISABLED";
pub const AI_HOST_FALLBACK_DIAGNOSTIC: &str = "AI_HOST_OPTIONAL_FALLBACK";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TargetGateStatusV1 {
    Pass,
    NotRun { reason: String },
}

impl TargetGateStatusV1 {
    #[must_use]
    pub const fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct V1TargetGateV1 {
    pub target_triple: &'static str,
    pub package_descriptor_hash: ContentHash,
    pub runtime_check_status: TargetGateStatusV1,
    pub desktop_smoke_status: TargetGateStatusV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct V1ClosureCheckReport {
    pub project_composition_lock_hash: ContentHash,
    pub schema_registry_hash: ContentHash,
    pub content_manifest_hash: ContentHash,
    pub mechanics_lock_hash: ContentHash,
    pub world_partition_hash: ContentHash,
    pub luau_manifest_hash: ContentHash,
    pub wasm_manifest_hash: ContentHash,
    pub wit_v2_hash: ContentHash,
    pub wit_v3_hash: ContentHash,
    pub extension_compatibility_hash: ContentHash,
    pub play_state_root: next_contracts::ids::StateRoot,
    pub play_ledger_hash: next_contracts::ids::CommandLedgerHash,
    pub replay_state_root: next_contracts::ids::StateRoot,
    pub replay_ledger_hash: next_contracts::ids::CommandLedgerHash,
    pub streaming_performance_hash: ContentHash,
    pub agent_performance_hash: ContentHash,
    pub headless_game_parity: bool,
    pub no_ai_host_fallback: bool,
    pub no_luau_fallback: bool,
    pub no_wasm_fallback: bool,
    pub windows: V1TargetGateV1,
    pub linux: V1TargetGateV1,
    pub shipping_ready: bool,
    pub closure_hash: ContentHash,
}

pub fn run_v1_closure_check() -> Result<V1ClosureCheckReport, V1ClosureCheckError> {
    run_v1_closure_check_in(&std::env::temp_dir())
}

pub fn run_v1_closure_check_in(
    scratch_root: &Path,
) -> Result<V1ClosureCheckReport, V1ClosureCheckError> {
    let scratch = ScratchContext::new(scratch_root)
        .map_err(|error| V1ClosureCheckError::new("scratch root", error.to_string()))?;
    run_v1_closure_check_with_scratch(&scratch)
}

pub(crate) fn run_v1_closure_check_with_scratch(
    scratch: &ScratchContext,
) -> Result<V1ClosureCheckReport, V1ClosureCheckError> {
    let directory = scratch
        .create_directory("v1-closure")
        .map_err(|error| V1ClosureCheckError::new("v1-closure scratch", error.to_string()))?;
    let result = run_v1_closure_check_scoped(&directory.context());
    directory.finish(result, |error| {
        V1ClosureCheckError::new("remove v1-closure scratch", error.to_string())
    })
}

fn run_v1_closure_check_scoped(
    scratch: &ScratchContext,
) -> Result<V1ClosureCheckReport, V1ClosureCheckError> {
    let content_directory = scratch
        .create_directory("content-package")
        .map_err(|error| V1ClosureCheckError::new("content-package scratch", error.to_string()))?;
    let content_result = run_content_package_check_with_scratch(&content_directory.context())
        .map_err(|error| V1ClosureCheckError::new("content-package", error.to_string()));
    let content = content_directory.finish(content_result, |error| {
        V1ClosureCheckError::new("remove content-package scratch", error.to_string())
    })?;

    let play_directory = scratch
        .create_directory("play")
        .map_err(|error| V1ClosureCheckError::new("play scratch", error.to_string()))?;
    let play_result = run_play_check_with_scratch(&play_directory.context())
        .map_err(|error| V1ClosureCheckError::new("play", error.to_string()));
    let play = play_directory.finish(play_result, |error| {
        V1ClosureCheckError::new("remove play scratch", error.to_string())
    })?;

    let replay_directory = scratch
        .create_directory("persistence-replay")
        .map_err(|error| V1ClosureCheckError::new("persistence scratch", error.to_string()))?;
    let replay_result = run_persistence_replay_check_with_backend_and_scratch(
        PersistenceReplayBackend::Reference,
        &replay_directory.context(),
    )
    .map_err(|error| V1ClosureCheckError::new("persistence-replay", error.to_string()));
    let replay = replay_directory.finish(replay_result, |error| {
        V1ClosureCheckError::new("remove persistence scratch", error.to_string())
    })?;

    let platform_directory = scratch
        .create_directory("platform")
        .map_err(|error| V1ClosureCheckError::new("platform scratch", error.to_string()))?;
    let platform_result = run_platform_check_with_scratch(&platform_directory.context())
        .map_err(|error| V1ClosureCheckError::new("platform", error.to_string()));
    let platform = platform_directory.finish(platform_result, |error| {
        V1ClosureCheckError::new("remove platform scratch", error.to_string())
    })?;

    let streaming_directory = scratch
        .create_directory("performance-streaming")
        .map_err(|error| V1ClosureCheckError::new("streaming scratch", error.to_string()))?;
    let streaming_result =
        run_streaming_performance_check_with_scratch(&streaming_directory.context())
            .map_err(|error| V1ClosureCheckError::new("performance.streaming", error.to_string()));
    let streaming = streaming_directory.finish(streaming_result, |error| {
        V1ClosureCheckError::new("remove streaming scratch", error.to_string())
    })?;

    let agent_directory = scratch
        .create_directory("performance-agent")
        .map_err(|error| V1ClosureCheckError::new("agent scratch", error.to_string()))?;
    let agent_result =
        run_agent_planning_performance_check_with_scratch(&agent_directory.context())
            .map_err(|error| V1ClosureCheckError::new("performance.agent", error.to_string()));
    let agent = agent_directory.finish(agent_result, |error| {
        V1ClosureCheckError::new("remove agent scratch", error.to_string())
    })?;

    let render_directory = scratch
        .create_directory("performance-render")
        .map_err(|error| V1ClosureCheckError::new("render scratch", error.to_string()))?;
    let render_result =
        run_render_frame_planning_performance_check_with_scratch(&render_directory.context())
            .map_err(|error| V1ClosureCheckError::new("performance.render", error.to_string()));
    let _render = render_directory.finish(render_result, |error| {
        V1ClosureCheckError::new("remove render scratch", error.to_string())
    })?;

    if platform.authoritative_state_root != play.final_state_root
        || platform.authoritative_ledger_hash != play.final_command_ledger_hash
    {
        return Err(V1ClosureCheckError::condition(
            "game/headless authoritative parity",
        ));
    }
    if content.luau_package_state_hash != replay.luau_package_state_hash
        || content.wasm_plugin_state_hash != replay.wasm_plugin_state_hash
    {
        return Err(V1ClosureCheckError::condition(
            "extension state save/load parity",
        ));
    }
    if play.npc_health != 75
        || play.player_health != 75
        || replay.npc_health != 75
        || replay.player_health != 75
        || content.combat_npc_health != 75
        || content.scripted_player_health != 75
        || content.wasm_player_health != 75
    {
        return Err(V1ClosureCheckError::condition(
            "data-only Luau Wasm gameplay closure",
        ));
    }

    let luau_manifest =
        next_script_luau::reference_scripted_melee_manifest_v1().map_err(|error| {
            V1ClosureCheckError::new("Luau compatibility manifest", error.to_string())
        })?;
    let wasm_manifest = next_plugin_host::reference_wasm_manifest_v1(false).map_err(|error| {
        V1ClosureCheckError::new("Wasm compatibility manifest", error.to_string())
    })?;
    let wit_v2_hash = content_hash_from_bytes(sha256(next_plugin_host::WIT_V2.as_bytes()));
    let wit_v3_hash = content_hash_from_bytes(sha256(next_plugin_host::WIT_V3.as_bytes()));
    let extension_compatibility_hash = extension_compatibility_hash(
        luau_manifest.manifest_hash,
        wasm_manifest.manifest_hash,
        wit_v2_hash,
        wit_v3_hash,
    );

    let windows = target_gate(
        "x86_64-pc-windows-msvc",
        cfg!(all(
            target_arch = "x86_64",
            target_os = "windows",
            target_env = "msvc"
        )),
        platform.candidate_status,
        &content,
        extension_compatibility_hash,
    );
    let linux = target_gate(
        "x86_64-unknown-linux-gnu",
        cfg!(all(
            target_arch = "x86_64",
            target_os = "linux",
            target_env = "gnu"
        )) && !current_host_is_wsl(),
        platform.candidate_status,
        &content,
        extension_compatibility_hash,
    );
    let shipping_ready = windows.runtime_check_status.is_pass()
        && windows.desktop_smoke_status.is_pass()
        && linux.runtime_check_status.is_pass()
        && linux.desktop_smoke_status.is_pass();

    let closure_hash = closure_hash(
        &content,
        &play,
        &replay,
        streaming.final_world_state_hash,
        agent.final_plan_hash,
        extension_compatibility_hash,
        windows.package_descriptor_hash,
        linux.package_descriptor_hash,
    );
    Ok(V1ClosureCheckReport {
        project_composition_lock_hash: content.composition_lock_hash,
        schema_registry_hash: content.schema_registry_hash,
        content_manifest_hash: content.content_manifest_hash,
        mechanics_lock_hash: content.mechanics_lock_hash,
        world_partition_hash: content.world_partition_hash,
        luau_manifest_hash: luau_manifest.manifest_hash,
        wasm_manifest_hash: wasm_manifest.manifest_hash,
        wit_v2_hash,
        wit_v3_hash,
        extension_compatibility_hash,
        play_state_root: play.final_state_root,
        play_ledger_hash: play.final_command_ledger_hash,
        replay_state_root: replay.final_state_root,
        replay_ledger_hash: replay.final_command_ledger_hash,
        streaming_performance_hash: streaming.final_world_state_hash,
        agent_performance_hash: agent.final_plan_hash,
        headless_game_parity: true,
        no_ai_host_fallback: true,
        no_luau_fallback: true,
        no_wasm_fallback: true,
        windows,
        linux,
        shipping_ready,
        closure_hash,
    })
}

fn target_gate(
    target_triple: &'static str,
    running_on_target: bool,
    platform_candidate_status: crate::PlatformCandidateStatus,
    content: &crate::ContentPackageCheckReport,
    extension_compatibility_hash: ContentHash,
) -> V1TargetGateV1 {
    let package_descriptor_hash =
        target_package_descriptor_hash(target_triple, content, extension_compatibility_hash);
    if running_on_target && platform_candidate_status == crate::PlatformCandidateStatus::Pass {
        V1TargetGateV1 {
            target_triple,
            package_descriptor_hash,
            runtime_check_status: TargetGateStatusV1::Pass,
            desktop_smoke_status: TargetGateStatusV1::Pass,
        }
    } else {
        let reason = if running_on_target {
            match platform_candidate_status {
                crate::PlatformCandidateStatus::Pass => "TARGET_GATE_INCONSISTENT".to_owned(),
                crate::PlatformCandidateStatus::NotRunOnDeveloperHost => {
                    "TARGET_PLATFORM_SMOKE_NOT_RUN".to_owned()
                }
                crate::PlatformCandidateStatus::NotRunAdapterDisabled => {
                    "TARGET_DESKTOP_ADAPTER_DISABLED".to_owned()
                }
            }
        } else {
            format!("TARGET_EXECUTION_UNAVAILABLE_ON_{}", current_host_label())
        };
        V1TargetGateV1 {
            target_triple,
            package_descriptor_hash,
            runtime_check_status: TargetGateStatusV1::NotRun {
                reason: reason.clone(),
            },
            desktop_smoke_status: TargetGateStatusV1::NotRun { reason },
        }
    }
}

fn target_package_descriptor_hash(
    target_triple: &str,
    content: &crate::ContentPackageCheckReport,
    extension_compatibility_hash: ContentHash,
) -> ContentHash {
    let mut bytes = b"nextengine.v1-target-package-descriptor.v1\0".to_vec();
    extend_text(&mut bytes, target_triple);
    extend_text(&mut bytes, env!("CARGO_PKG_VERSION"));
    bytes.extend_from_slice(content.composition_lock_hash.as_bytes());
    bytes.extend_from_slice(content.schema_registry_hash.as_bytes());
    bytes.extend_from_slice(content.content_manifest_hash.as_bytes());
    bytes.extend_from_slice(content.mechanics_lock_hash.as_bytes());
    bytes.extend_from_slice(content.world_partition_hash.as_bytes());
    bytes.extend_from_slice(extension_compatibility_hash.as_bytes());
    extend_text(&mut bytes, "next_game");
    extend_text(&mut bytes, "next_headless");
    content_hash_from_bytes(sha256(&bytes))
}

fn extension_compatibility_hash(
    luau_manifest_hash: ContentHash,
    wasm_manifest_hash: ContentHash,
    wit_v2_hash: ContentHash,
    wit_v3_hash: ContentHash,
) -> ContentHash {
    let mut bytes = b"nextengine.v1-extension-compatibility.v1\0".to_vec();
    extend_text(&mut bytes, next_script_luau::LUAU_ADAPTER_VERSION);
    extend_text(&mut bytes, next_plugin_host::WASMTIME_ADAPTER_VERSION);
    extend_text(
        &mut bytes,
        next_plugin_host::WASM_COMPONENT_MODEL_FEATURE_SET,
    );
    bytes.extend_from_slice(luau_manifest_hash.as_bytes());
    bytes.extend_from_slice(wasm_manifest_hash.as_bytes());
    bytes.extend_from_slice(wit_v2_hash.as_bytes());
    bytes.extend_from_slice(wit_v3_hash.as_bytes());
    bytes.extend_from_slice(&next_contracts::extension::WASM_HOST_CURRENT_API_MAJOR.to_le_bytes());
    bytes.extend_from_slice(&next_contracts::extension::WASM_HOST_PREVIOUS_API_MAJOR.to_le_bytes());
    extend_text(&mut bytes, RPG_SCHEMA_UNSUPPORTED_DIAGNOSTIC);
    extend_text(&mut bytes, WIT_N_MINUS_2_UNSUPPORTED_DIAGNOSTIC);
    extend_text(&mut bytes, OPTIONAL_EXTENSION_DISABLED_DIAGNOSTIC);
    extend_text(&mut bytes, AI_HOST_FALLBACK_DIAGNOSTIC);
    content_hash_from_bytes(sha256(&bytes))
}

#[allow(
    clippy::too_many_arguments,
    reason = "the closure root intentionally binds every independently checked product root"
)]
fn closure_hash(
    content: &crate::ContentPackageCheckReport,
    play: &crate::PlayCheckReport,
    replay: &crate::PersistenceReplayCheckReport,
    streaming_hash: ContentHash,
    agent_hash: ContentHash,
    extension_compatibility_hash: ContentHash,
    windows_package_hash: ContentHash,
    linux_package_hash: ContentHash,
) -> ContentHash {
    let mut bytes = b"nextengine.v1-closure.v1\0".to_vec();
    bytes.extend_from_slice(content.composition_lock_hash.as_bytes());
    bytes.extend_from_slice(content.schema_registry_hash.as_bytes());
    bytes.extend_from_slice(content.content_manifest_hash.as_bytes());
    bytes.extend_from_slice(content.mechanics_lock_hash.as_bytes());
    bytes.extend_from_slice(content.world_partition_hash.as_bytes());
    bytes.extend_from_slice(play.final_state_root.as_bytes());
    bytes.extend_from_slice(play.final_command_ledger_hash.as_bytes());
    bytes.extend_from_slice(replay.final_state_root.as_bytes());
    bytes.extend_from_slice(replay.final_command_ledger_hash.as_bytes());
    bytes.extend_from_slice(streaming_hash.as_bytes());
    bytes.extend_from_slice(agent_hash.as_bytes());
    bytes.extend_from_slice(extension_compatibility_hash.as_bytes());
    bytes.extend_from_slice(windows_package_hash.as_bytes());
    bytes.extend_from_slice(linux_package_hash.as_bytes());
    content_hash_from_bytes(sha256(&bytes))
}

fn extend_text(bytes: &mut Vec<u8>, value: &str) {
    let length =
        u32::try_from(value.len()).expect("static closure labels fit canonical u32 lengths");
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn current_host_label() -> &'static str {
    if current_host_is_wsl() {
        "WSL"
    } else if cfg!(all(target_arch = "aarch64", target_os = "macos")) {
        "AARCH64_APPLE_DARWIN"
    } else if cfg!(all(
        target_arch = "x86_64",
        target_os = "windows",
        target_env = "msvc"
    )) {
        "X86_64_PC_WINDOWS_MSVC"
    } else if cfg!(all(
        target_arch = "x86_64",
        target_os = "linux",
        target_env = "gnu"
    )) {
        "X86_64_UNKNOWN_LINUX_GNU"
    } else {
        "UNSUPPORTED_HOST"
    }
}

#[cfg(target_os = "linux")]
fn current_host_is_wsl() -> bool {
    let os_release = std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default();
    wsl_markers_present(
        std::env::var_os("WSL_INTEROP").is_some(),
        std::env::var_os("WSL_DISTRO_NAME").is_some(),
        &os_release,
    )
}

#[cfg(not(target_os = "linux"))]
const fn current_host_is_wsl() -> bool {
    false
}

#[cfg(any(target_os = "linux", test))]
fn wsl_markers_present(has_wsl_interop: bool, has_wsl_distro_name: bool, os_release: &str) -> bool {
    has_wsl_interop || has_wsl_distro_name || os_release.to_ascii_lowercase().contains("microsoft")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct V1ClosureCheckError {
    context: &'static str,
    detail: String,
}

impl V1ClosureCheckError {
    fn new(context: &'static str, detail: impl Into<String>) -> Self {
        Self {
            context,
            detail: detail.into(),
        }
    }

    fn condition(context: &'static str) -> Self {
        Self::new(context, "acceptance condition was false")
    }
}

impl Display for V1ClosureCheckError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for V1ClosureCheckError {}

#[cfg(test)]
mod tests {
    use super::{
        TargetGateStatusV1, current_host_is_wsl, run_v1_closure_check_in,
        run_v1_closure_check_with_scratch, wsl_markers_present,
    };
    use crate::scratch::ScratchContext;
    use next_contracts::ids::ContentHash;
    use std::collections::HashSet;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn closure_binds_exact_roots_and_never_masks_unavailable_targets() {
        let root = std::env::temp_dir().join(format!(
            "nextengine-v1-closure-scratch-test-{}-{}",
            std::process::id(),
            TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("create closure scratch root");
        let canonical_root = fs::canonicalize(&root).expect("canonical closure scratch root");
        let scratch = ScratchContext::new(&root).expect("closure scratch context");
        let report = run_v1_closure_check_with_scratch(&scratch).expect("local v1 closure passes");
        let allocations = scratch.allocated_paths();
        assert!(!allocations.is_empty());
        assert!(
            allocations
                .iter()
                .all(|path| path.starts_with(&canonical_root))
        );
        assert_eq!(
            allocations.iter().collect::<HashSet<_>>().len(),
            allocations.len()
        );
        for label in [
            "content-package",
            "play-project",
            "persistence-replay",
            "save-restore",
            "physics-fallback",
            "platform",
            "headless",
            "game-frame",
            "performance-streaming",
            "performance-agent",
            "performance-render",
        ] {
            assert!(
                allocations.iter().any(|path| path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().contains(label))),
                "missing isolated scratch allocation for {label}"
            );
        }
        assert_eq!(
            fs::read_dir(&root)
                .expect("read cleaned closure root")
                .count(),
            0
        );
        fs::remove_dir(&root).expect("remove closure scratch root");
        assert_ne!(report.closure_hash, ContentHash::default());
        assert_ne!(
            report.windows.package_descriptor_hash,
            report.linux.package_descriptor_hash
        );
        assert!(report.headless_game_parity);
        assert!(report.no_ai_host_fallback);
        assert!(report.no_luau_fallback);
        assert!(report.no_wasm_fallback);
        if cfg!(all(
            target_arch = "x86_64",
            target_os = "windows",
            target_env = "msvc",
            feature = "desktop-sdl-ash"
        )) {
            assert_eq!(
                report.windows.runtime_check_status,
                TargetGateStatusV1::Pass
            );
        } else {
            assert!(matches!(
                report.windows.runtime_check_status,
                TargetGateStatusV1::NotRun { .. }
            ));
        }
        if cfg!(all(
            target_arch = "x86_64",
            target_os = "linux",
            target_env = "gnu",
            feature = "desktop-sdl-ash"
        )) && !current_host_is_wsl()
        {
            assert_eq!(report.linux.desktop_smoke_status, TargetGateStatusV1::Pass);
        } else {
            assert!(matches!(
                report.linux.desktop_smoke_status,
                TargetGateStatusV1::NotRun { .. }
            ));
        }
    }

    #[test]
    fn explicit_closure_scratch_root_must_already_exist() {
        let missing = std::env::temp_dir().join(format!(
            "nextengine-v1-closure-missing-test-{}-{}",
            std::process::id(),
            TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        assert!(run_v1_closure_check_in(&missing).is_err());
        assert!(!missing.exists());
    }

    #[test]
    fn wsl_markers_never_count_as_native_linux_evidence() {
        assert!(wsl_markers_present(true, false, "6.6.0-generic"));
        assert!(wsl_markers_present(false, true, "6.6.0-generic"));
        assert!(wsl_markers_present(
            false,
            false,
            "5.15.153.1-microsoft-standard-WSL2"
        ));
        assert!(!wsl_markers_present(false, false, "6.8.0-31-generic"));
    }
}
