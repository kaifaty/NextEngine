use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::time::Instant;

use next_contracts::ids::{CommandLedgerHash, ContentHash, StateRoot};
use next_contracts::localization::TextCatalogV1;
use next_contracts::presentation::PresentationSnapshotV3;
use next_contracts::project::domain_hash;
use next_contracts::render_content::RenderContentCatalogV1;
use next_presentation::PresentationExtractorV1;
use next_reference_game::{ReferenceDialogueChoiceV1, ReferenceRunOutcomeV2};

use crate::platform_check::{
    DesktopFrameTimingSmokeReport, prepare_desktop_frame_timing_workload_for_inputs_in,
};
use crate::player_fixture::prepare_fixture_project_package_with_scratch;
use crate::scratch::ScratchContext;

pub const R2_ALPHA_RENDER_WARMUP_FRAMES_PER_WINDOW: u32 = 600;
pub const R2_ALPHA_RENDER_MEASURED_FRAMES_PER_WINDOW: u32 = 3_600;
pub const R2_ALPHA_RENDER_WINDOW_COUNT: usize = 3;
pub const R2_ALPHA_RENDER_PROFILE_COUNT: usize = 2;

const LOGICAL_SCENE_RECORD_BYTES: u64 = 256;
const LOGICAL_CAMERA_RECORD_BYTES: u64 = 256;
const LOGICAL_UI_RECORD_BYTES: u64 = 512;
const LOGICAL_SKINNING_RECORD_BYTES: u64 = 1_024;
const LOGICAL_BATCH_BYTES: u64 = 64;
const LOGICAL_SNAPSHOT_BYTES: u64 = 256;
const LOGICAL_TIMING_SAMPLE_BYTES: u64 = 80;
const LOGICAL_TIMING_REPORT_BYTES: u64 = 4_096;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum R2AlphaRenderWindowV1 {
    Exploration,
    Combat,
    UiDialogue,
}

impl R2AlphaRenderWindowV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Exploration => "exploration",
            Self::Combat => "combat",
            Self::UiDialogue => "ui-dialogue",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum R2AlphaRenderProfileV1 {
    Primary1080p,
    Safe720p30,
}

impl R2AlphaRenderProfileV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Primary1080p => "primary-1080p",
            Self::Safe720p30 => "b0-safe-720p30",
        }
    }

    #[must_use]
    pub const fn extent(self) -> [u32; 2] {
        match self {
            Self::Primary1080p => [1_920, 1_080],
            Self::Safe720p30 => [1_280, 720],
        }
    }

    #[must_use]
    pub fn presentation_profile_hash(self) -> ContentHash {
        match self {
            Self::Primary1080p => next_reference_game::reference_b0_presentation_profile_hash(),
            Self::Safe720p30 => domain_hash(
                "nextengine.presentation-profile.b0-safe-720p30.v1",
                b"sdr-reference-no-optional-features:1280x720:30fps",
            ),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct R2AlphaRenderWindowReportV1 {
    pub window: R2AlphaRenderWindowV1,
    pub profile: R2AlphaRenderProfileV1,
    pub presentation_snapshot_hash: ContentHash,
    pub elapsed_microseconds: u64,
    pub timing: DesktopFrameTimingSmokeReport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct R2AlphaRenderPerformanceReportV1 {
    pub windows: Vec<R2AlphaRenderWindowReportV1>,
    pub authoritative_state_root: StateRoot,
    pub command_ledger_hash: CommandLedgerHash,
    pub accounting_profile_hash: ContentHash,
    pub authoritative_state_bytes: u64,
    pub required_staging_bytes: u64,
    pub reconstructible_host_cache_bytes: u64,
    pub presentation_transient_bytes: u64,
    pub tooling_transient_bytes: u64,
}

impl R2AlphaRenderPerformanceReportV1 {
    #[must_use]
    pub fn vulkan_timestamp_queries(&self) -> u64 {
        self.windows
            .iter()
            .map(|window| window.timing.timestamp_query_count)
            .sum()
    }

    #[must_use]
    pub fn device_allocation_ceiling_bytes(&self) -> u64 {
        self.windows
            .iter()
            .map(|window| window.timing.device_allocation_bytes)
            .max()
            .unwrap_or(0)
    }

    #[must_use]
    pub fn device_allocation_ceiling_count(&self) -> u64 {
        self.windows
            .iter()
            .map(|window| window.timing.device_allocation_count)
            .max()
            .unwrap_or(0)
    }
}

#[derive(Debug)]
pub struct PreparedR2AlphaRenderPerformanceCheckV1 {
    scratch_root: PathBuf,
    render_content_catalog: RenderContentCatalogV1,
    text_catalogs: Vec<TextCatalogV1>,
    windows: Vec<PreparedWindowV1>,
    authoritative_state_root: StateRoot,
    command_ledger_hash: CommandLedgerHash,
    accounting_profile_hash: ContentHash,
    authoritative_state_bytes: u64,
    reconstructible_host_cache_bytes: u64,
    presentation_transient_bytes: u64,
    tooling_transient_bytes: u64,
    run_started: bool,
}

#[derive(Debug)]
struct PreparedWindowV1 {
    window: R2AlphaRenderWindowV1,
    profile: R2AlphaRenderProfileV1,
    snapshot: PresentationSnapshotV3,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct R2AlphaRenderPerformanceErrorV1 {
    diagnostic: String,
}

impl Display for R2AlphaRenderPerformanceErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.diagnostic)
    }
}

impl Error for R2AlphaRenderPerformanceErrorV1 {}

pub fn run_r2_alpha_render_performance_check_in(
    scratch_root: &Path,
) -> Result<Option<R2AlphaRenderPerformanceReportV1>, R2AlphaRenderPerformanceErrorV1> {
    prepare_r2_alpha_render_performance_check_in(scratch_root)?.run_measured()
}

pub fn prepare_r2_alpha_render_performance_check_in(
    scratch_root: &Path,
) -> Result<PreparedR2AlphaRenderPerformanceCheckV1, R2AlphaRenderPerformanceErrorV1> {
    let scratch =
        ScratchContext::new(scratch_root).map_err(|error| workload_error("scratch-root", error))?;
    let prepared_package = prepare_fixture_project_package_with_scratch(
        &scratch,
        next_reference_game::REFERENCE_GAME_PROJECT_ID,
    )
    .map_err(|error| workload_error("project-activation", error))?;
    let render_content_catalog = prepared_package
        .package
        .project
        .render_content_catalog
        .clone();
    let text_catalogs = prepared_package.package.project.text_catalogs.clone();
    let scenarios = (|| {
        let exploration =
            next_reference_game::run_reference_game(prepared_package.package.clone(), false)
                .map_err(|error| workload_error("exploration-scenario", error))?;
        let combat =
            next_reference_game::run_reference_game(prepared_package.package.clone(), true)
                .map_err(|error| workload_error("combat-scenario", error))?;
        Ok((exploration, combat))
    })();
    let (exploration, combat) =
        prepared_package.finish(scenarios, |error| workload_error("project-cleanup", error))?;

    let (checkpoint, components) = combat
        .runtime
        .world_checkpoint_with_canonical_components()
        .map_err(|error| workload_error("authoritative-checkpoint", error))?;
    let authoritative_state_bytes = checked_sum([
        components.runtime_snapshot_bytes().len(),
        components.rpg_snapshot_bytes().len(),
        components.physics_checkpoint_bytes().len(),
    ])?;
    let command_ledger_hash = components
        .command_ledger_hash()
        .map_err(|error| workload_error("command-ledger-hash", error))?;
    let mut host_cache_lengths = vec![
        render_content_catalog
            .canonical_bytes()
            .map_err(|error| workload_error("render-content-catalog", error))?
            .len(),
    ];
    for catalog in &text_catalogs {
        host_cache_lengths.push(
            catalog
                .canonical_bytes()
                .map_err(|error| workload_error("text-catalog", error))?
                .len(),
        );
    }
    let reconstructible_host_cache_bytes = checked_sum(host_cache_lengths)?;

    let mut windows =
        Vec::with_capacity(R2_ALPHA_RENDER_WINDOW_COUNT * R2_ALPHA_RENDER_PROFILE_COUNT);
    for profile in [
        R2AlphaRenderProfileV1::Primary1080p,
        R2AlphaRenderProfileV1::Safe720p30,
    ] {
        windows.push(PreparedWindowV1 {
            window: R2AlphaRenderWindowV1::Exploration,
            profile,
            snapshot: extract_window_snapshot(&exploration, profile, false)?,
        });
        windows.push(PreparedWindowV1 {
            window: R2AlphaRenderWindowV1::Combat,
            profile,
            snapshot: extract_window_snapshot(&combat, profile, false)?,
        });
        windows.push(PreparedWindowV1 {
            window: R2AlphaRenderWindowV1::UiDialogue,
            profile,
            snapshot: extract_window_snapshot(&combat, profile, true)?,
        });
    }
    let mut snapshot_hashes = windows
        .iter()
        .map(|window| window.snapshot.canonical_hash)
        .collect::<Vec<_>>();
    snapshot_hashes.sort_unstable();
    snapshot_hashes.dedup();
    if snapshot_hashes.len() != windows.len() {
        return Err(invalid_workload(
            "semantic/profile snapshots are not distinct",
        ));
    }
    let presentation_transient_bytes = windows
        .iter()
        .map(|window| logical_snapshot_charge(&window.snapshot))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .max()
        .unwrap_or(0);
    let retained_before_last = u64::from(R2_ALPHA_RENDER_MEASURED_FRAMES_PER_WINDOW)
        .checked_mul(
            u64::try_from(windows.len().saturating_sub(1))
                .map_err(|error| workload_error("tooling-retained-window-count", error))?,
        )
        .ok_or_else(|| invalid_workload("tooling sample count overflow"))?;
    let active_total = u64::from(R2_ALPHA_RENDER_WARMUP_FRAMES_PER_WINDOW)
        .checked_add(u64::from(R2_ALPHA_RENDER_MEASURED_FRAMES_PER_WINDOW))
        .and_then(|value| value.checked_add(u64::from(R2_ALPHA_RENDER_MEASURED_FRAMES_PER_WINDOW)))
        .ok_or_else(|| invalid_workload("tooling sample count overflow"))?;
    let tooling_transient_bytes = retained_before_last
        .checked_add(active_total)
        .and_then(|samples| samples.checked_mul(LOGICAL_TIMING_SAMPLE_BYTES))
        .and_then(|bytes| bytes.checked_add(LOGICAL_TIMING_REPORT_BYTES))
        .ok_or_else(|| invalid_workload("tooling charge overflow"))?;

    Ok(PreparedR2AlphaRenderPerformanceCheckV1 {
        scratch_root: scratch_root.to_path_buf(),
        render_content_catalog,
        text_catalogs,
        windows,
        authoritative_state_root: checkpoint.state_root,
        command_ledger_hash,
        accounting_profile_hash: domain_hash(
            "nextengine.performance.logical-resource-accounting.r2-alpha-render.v1",
            b"authority=checkpoint-components;staging=zero-prepared;host-cache=canonical-render-and-text;device=max-engine-owned-bound-vulkan;presentation=max-fixed-record-charge-256-256-512-batch64-snapshot256;tooling=retained-plus-active-frame-samples-80-report4096",
        ),
        authoritative_state_bytes,
        reconstructible_host_cache_bytes,
        presentation_transient_bytes,
        tooling_transient_bytes,
        run_started: false,
    })
}

impl PreparedR2AlphaRenderPerformanceCheckV1 {
    #[must_use]
    pub const fn authoritative_state_root(&self) -> StateRoot {
        self.authoritative_state_root
    }

    #[must_use]
    pub const fn command_ledger_hash(&self) -> CommandLedgerHash {
        self.command_ledger_hash
    }

    pub fn run_measured(
        &mut self,
    ) -> Result<Option<R2AlphaRenderPerformanceReportV1>, R2AlphaRenderPerformanceErrorV1> {
        if self.run_started {
            return Err(invalid_workload("prepared workload was already started"));
        }
        self.run_started = true;
        let mut reports = Vec::with_capacity(self.windows.len());
        for window in &self.windows {
            let mut prepared = prepare_desktop_frame_timing_workload_for_inputs_in(
                &self.scratch_root,
                &window.snapshot,
                &self.render_content_catalog,
                &self.text_catalogs,
                R2_ALPHA_RENDER_WARMUP_FRAMES_PER_WINDOW,
                R2_ALPHA_RENDER_MEASURED_FRAMES_PER_WINDOW,
                window.profile.extent(),
                &format!(
                    "Next Engine — R2 {} / {}",
                    window.window.as_str(),
                    window.profile.as_str()
                ),
            )
            .map_err(|error| workload_error("desktop-prepare", error))?;
            let started = Instant::now();
            let measurement = prepared.run_measured();
            let elapsed_microseconds = u64::try_from(started.elapsed().as_micros())
                .map_err(|error| workload_error("desktop-elapsed", error))?;
            let Some(timing) = prepared
                .finish(measurement)
                .map_err(|error| workload_error("desktop-finish", error))?
            else {
                return Ok(None);
            };
            if timing.samples.len()
                != usize::try_from(R2_ALPHA_RENDER_MEASURED_FRAMES_PER_WINDOW)
                    .map_err(|error| workload_error("measured-frame-count", error))?
                || timing.drawable_extent != window.profile.extent()
            {
                return Err(invalid_workload(
                    "desktop report does not match its declared profile",
                ));
            }
            reports.push(R2AlphaRenderWindowReportV1 {
                window: window.window,
                profile: window.profile,
                presentation_snapshot_hash: window.snapshot.canonical_hash,
                elapsed_microseconds,
                timing,
            });
        }
        Ok(Some(R2AlphaRenderPerformanceReportV1 {
            windows: reports,
            authoritative_state_root: self.authoritative_state_root,
            command_ledger_hash: self.command_ledger_hash,
            accounting_profile_hash: self.accounting_profile_hash,
            authoritative_state_bytes: self.authoritative_state_bytes,
            required_staging_bytes: 0,
            reconstructible_host_cache_bytes: self.reconstructible_host_cache_bytes,
            presentation_transient_bytes: self.presentation_transient_bytes,
            tooling_transient_bytes: self.tooling_transient_bytes,
        }))
    }
}

fn extract_window_snapshot(
    scenario: &ReferenceRunOutcomeV2,
    profile: R2AlphaRenderProfileV1,
    dialogue: bool,
) -> Result<PresentationSnapshotV3, R2AlphaRenderPerformanceErrorV1> {
    let mut extractor = PresentationExtractorV1::new(
        scenario.project_composition_lock_hash,
        profile.presentation_profile_hash(),
        8,
    )
    .map_err(|error| workload_error("presentation-extractor", error))?;
    let rpg = scenario.runtime.rpg_snapshot();
    let mut ui_records = next_reference_game::read_only_screen_semantic_ui_records_for_ids(
        extractor.snapshot_epoch(),
        scenario.player_character_id,
        scenario.quest_id,
        &[scenario.pickup_item_id, scenario.npc_weapon_item_id],
        &scenario.item_display_text_id,
        &scenario.quest_display_text_id,
        &rpg,
    )
    .map_err(|error| workload_error("read-only-ui", error))?;
    if dialogue {
        ui_records.extend(
            next_reference_game::dialogue_semantic_ui_records_for_ids(
                extractor.snapshot_epoch(),
                scenario.dialogue_id,
                &rpg,
                ReferenceDialogueChoiceV1::Accept,
            )
            .map_err(|error| workload_error("dialogue-ui", error))?,
        );
    }
    extractor
        .extract_with_character_skinning(
            scenario.ticks,
            scenario.project_composition_lock_hash,
            scenario.content_manifest_hash,
            scenario.runtime.physics_snapshot(),
            &scenario.presentation_bindings,
            &[],
            ui_records,
            scenario.character_skinning_records.clone(),
        )
        .cloned()
        .map_err(|error| workload_error("presentation-extraction", error))
}

fn logical_snapshot_charge(
    snapshot: &PresentationSnapshotV3,
) -> Result<u64, R2AlphaRenderPerformanceErrorV1> {
    let scene_records = snapshot
        .scene_batches
        .iter()
        .map(|batch| batch.records.len())
        .sum::<usize>();
    let camera_records = snapshot
        .camera_batches
        .iter()
        .map(|batch| batch.records.len())
        .sum::<usize>();
    let ui_records = snapshot
        .semantic_ui_batches
        .iter()
        .map(|batch| batch.records.len())
        .sum::<usize>();
    let skinning_records = snapshot
        .character_skinning_batches
        .iter()
        .map(|batch| batch.records.len())
        .sum::<usize>();
    let batches = snapshot
        .scene_batches
        .len()
        .checked_add(snapshot.camera_batches.len())
        .and_then(|value| value.checked_add(snapshot.semantic_ui_batches.len()))
        .and_then(|value| value.checked_add(snapshot.character_skinning_batches.len()))
        .ok_or_else(|| invalid_workload("presentation batch count overflow"))?;
    let scene_charge = checked_charge(scene_records, LOGICAL_SCENE_RECORD_BYTES)?;
    let camera_charge = checked_charge(camera_records, LOGICAL_CAMERA_RECORD_BYTES)?;
    let ui_charge = checked_charge(ui_records, LOGICAL_UI_RECORD_BYTES)?;
    let skinning_charge = checked_charge(skinning_records, LOGICAL_SKINNING_RECORD_BYTES)?;
    let batch_charge = checked_charge(batches, LOGICAL_BATCH_BYTES)?;
    scene_charge
        .checked_add(camera_charge)
        .and_then(|bytes| bytes.checked_add(ui_charge))
        .and_then(|bytes| bytes.checked_add(skinning_charge))
        .and_then(|bytes| bytes.checked_add(batch_charge))
        .and_then(|bytes| bytes.checked_add(LOGICAL_SNAPSHOT_BYTES))
        .ok_or_else(|| invalid_workload("presentation charge overflow"))
}

fn checked_charge(
    count: usize,
    bytes_per_item: u64,
) -> Result<u64, R2AlphaRenderPerformanceErrorV1> {
    checked_u64(count)?
        .checked_mul(bytes_per_item)
        .ok_or_else(|| invalid_workload("presentation charge overflow"))
}

fn checked_sum(
    values: impl IntoIterator<Item = usize>,
) -> Result<u64, R2AlphaRenderPerformanceErrorV1> {
    values.into_iter().try_fold(0_u64, |total, value| {
        total
            .checked_add(checked_u64(value)?)
            .ok_or_else(|| invalid_workload("logical byte charge overflow"))
    })
}

fn checked_u64(value: usize) -> Result<u64, R2AlphaRenderPerformanceErrorV1> {
    u64::try_from(value).map_err(|error| workload_error("logical-byte-conversion", error))
}

fn workload_error(context: &str, error: impl Display) -> R2AlphaRenderPerformanceErrorV1 {
    R2AlphaRenderPerformanceErrorV1 {
        diagnostic: format!("R2_ALPHA_RENDER_WORKLOAD_INVALID: {context}: {error}"),
    }
}

fn invalid_workload(details: &str) -> R2AlphaRenderPerformanceErrorV1 {
    R2AlphaRenderPerformanceErrorV1 {
        diagnostic: format!("R2_ALPHA_RENDER_WORKLOAD_INVALID: {details}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_profiles_are_independent_and_exact() {
        assert_eq!(
            R2AlphaRenderProfileV1::Primary1080p.extent(),
            [1_920, 1_080]
        );
        assert_eq!(R2AlphaRenderProfileV1::Safe720p30.extent(), [1_280, 720]);
        assert_ne!(
            R2AlphaRenderProfileV1::Primary1080p.presentation_profile_hash(),
            R2AlphaRenderProfileV1::Safe720p30.presentation_profile_hash()
        );
        assert_eq!(R2_ALPHA_RENDER_WARMUP_FRAMES_PER_WINDOW, 600);
        assert_eq!(R2_ALPHA_RENDER_MEASURED_FRAMES_PER_WINDOW, 3_600);
    }

    #[cfg(not(feature = "desktop-sdl-ash"))]
    #[test]
    fn disabled_desktop_adapter_returns_not_run_after_validating_production_inputs() {
        let report = run_r2_alpha_render_performance_check_in(&std::env::temp_dir())
            .expect("valid production R2 workload");
        assert!(report.is_none());
    }
}
