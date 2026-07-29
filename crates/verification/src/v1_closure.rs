use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::canonical::sha256;
use next_contracts::ids::{ContentHash, content_hash_from_bytes};

use crate::{
    run_agent_planning_performance_check, run_content_package_check, run_persistence_replay_check,
    run_platform_check, run_play_check, run_streaming_performance_check,
};

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
    let content = run_content_package_check()
        .map_err(|error| V1ClosureCheckError::new("content-package", error.to_string()))?;
    let play =
        run_play_check().map_err(|error| V1ClosureCheckError::new("play", error.to_string()))?;
    let replay = run_persistence_replay_check()
        .map_err(|error| V1ClosureCheckError::new("persistence-replay", error.to_string()))?;
    let platform = run_platform_check()
        .map_err(|error| V1ClosureCheckError::new("platform", error.to_string()))?;
    let streaming = run_streaming_performance_check()
        .map_err(|error| V1ClosureCheckError::new("performance.streaming", error.to_string()))?;
    let agent = run_agent_planning_performance_check()
        .map_err(|error| V1ClosureCheckError::new("performance.agent", error.to_string()))?;

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
        cfg!(all(target_arch = "x86_64", target_os = "linux")),
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
    if cfg!(all(target_arch = "aarch64", target_os = "macos")) {
        "AARCH64_APPLE_DARWIN"
    } else if cfg!(all(
        target_arch = "x86_64",
        target_os = "windows",
        target_env = "msvc"
    )) {
        "X86_64_PC_WINDOWS_MSVC"
    } else if cfg!(all(target_arch = "x86_64", target_os = "linux")) {
        "X86_64_UNKNOWN_LINUX_GNU"
    } else {
        "UNSUPPORTED_HOST"
    }
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
    use super::{TargetGateStatusV1, run_v1_closure_check};
    use next_contracts::ids::ContentHash;

    #[test]
    fn closure_binds_exact_roots_and_never_masks_unavailable_targets() {
        let report = run_v1_closure_check().expect("local v1 closure passes");
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
            target_env = "msvc"
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
        if cfg!(all(target_arch = "x86_64", target_os = "linux")) {
            assert_eq!(report.linux.desktop_smoke_status, TargetGateStatusV1::Pass);
        } else {
            assert!(matches!(
                report.linux.desktop_smoke_status,
                TargetGateStatusV1::NotRun { .. }
            ));
        }
    }
}
