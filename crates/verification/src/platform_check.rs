use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;

use next_contracts::ids::{PersistentId, SchemaId};
use next_contracts::platform::{
    NormalizedControlEventV1, NormalizedControlPhaseV1, PlatformEventKindV1,
    PlatformEventPayloadV1, PlatformEventV1, PresentationTargetKindV1,
};
use next_platform::{PlatformHost, PlatformHostError, ReferencePlatformHost};

use crate::player_fixture::{prepare_game_frame_with_scratch, run_play_check_with_scratch};
use crate::scratch::ScratchContext;
use crate::{GameCheckReport, PlayCheckError};

const MAX_DESKTOP_FRAME_TIMING_SAMPLES: u32 = 65_536;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlatformCheckReport {
    pub normalized_events: usize,
    pub rendered_objects: u32,
    pub authoritative_state_root: next_contracts::ids::StateRoot,
    pub authoritative_ledger_hash: next_contracts::ids::CommandLedgerHash,
    pub presentation_snapshot_hash: next_contracts::ids::ContentHash,
    pub candidate_status: PlatformCandidateStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopFrameTimingSmokeSample {
    pub cpu_extract_and_submit_microseconds: u64,
    pub gpu_duration_microseconds: u64,
    pub event_and_frame_source_update_microseconds: u64,
    pub frame_slot_wait_microseconds: u64,
    pub image_acquire_wait_microseconds: u64,
    pub swapchain_image_wait_microseconds: u64,
    pub frame_plan_microseconds: u64,
    pub command_record_microseconds: u64,
    pub queue_submit_microseconds: u64,
    pub present_wait_microseconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesktopFrameTimingSmokeReport {
    pub samples: Vec<DesktopFrameTimingSmokeSample>,
    pub drawable_extent: [u32; 2],
    pub timestamp_query_count: u64,
    pub dropped_samples: u64,
    pub frame_plan_hash: next_contracts::ids::ContentHash,
    pub frame_plan_cache_hits: u64,
    pub frame_plan_cache_misses: u64,
    pub frame_plan_build_failures: u64,
    pub frame_plan_explicit_invalidations: u64,
    pub software_paced_iterations: u64,
    pub software_pacing_sleep_microseconds: u64,
    pub device_allocation_bytes: u64,
    pub device_allocation_count: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformCandidateStatus {
    Pass,
    NotRunOnDeveloperHost,
    NotRunAdapterDisabled,
}

pub fn run_platform_check() -> Result<PlatformCheckReport, PlatformCheckError> {
    run_platform_check_in(&std::env::temp_dir())
}

pub fn run_platform_check_in(
    scratch_root: &Path,
) -> Result<PlatformCheckReport, PlatformCheckError> {
    let scratch = ScratchContext::new(scratch_root).map_err(platform_scratch_error)?;
    run_platform_check_with_scratch(&scratch)
}

pub fn run_desktop_frame_timing_smoke()
-> Result<Option<DesktopFrameTimingSmokeReport>, PlatformCheckError> {
    run_desktop_frame_timing_smoke_in(&std::env::temp_dir())
}

pub fn run_desktop_frame_timing_smoke_in(
    scratch_root: &Path,
) -> Result<Option<DesktopFrameTimingSmokeReport>, PlatformCheckError> {
    run_desktop_frame_timing_workload_in(scratch_root, 4, [960, 540])
}

pub fn run_desktop_frame_timing_workload_in(
    scratch_root: &Path,
    measured_frames: u32,
    initial_extent: [u32; 2],
) -> Result<Option<DesktopFrameTimingSmokeReport>, PlatformCheckError> {
    if measured_frames == 0
        || measured_frames > MAX_DESKTOP_FRAME_TIMING_SAMPLES
        || initial_extent.contains(&0)
    {
        return Err(PlatformCheckError::DesktopSmokeMismatch);
    }
    let scratch = ScratchContext::new(scratch_root).map_err(platform_scratch_error)?;
    let timing_directory = scratch
        .create_directory("desktop-frame-timing")
        .map_err(platform_scratch_error)?;
    let timing_scratch = timing_directory.context();
    let result =
        run_desktop_frame_timing_smoke_scoped(&timing_scratch, measured_frames, initial_extent);
    timing_directory.finish(result, platform_scratch_error)
}

pub(crate) fn run_platform_check_with_scratch(
    scratch: &ScratchContext,
) -> Result<PlatformCheckReport, PlatformCheckError> {
    let platform_directory = scratch
        .create_directory("platform")
        .map_err(platform_scratch_error)?;
    let platform_scratch = platform_directory.context();
    let result = run_platform_check_scoped(&platform_scratch);
    platform_directory.finish(result, platform_scratch_error)
}

fn run_platform_check_scoped(
    scratch: &ScratchContext,
) -> Result<PlatformCheckReport, PlatformCheckError> {
    let headless_directory = scratch
        .create_directory("headless")
        .map_err(platform_scratch_error)?;
    let headless_result = run_play_check_with_scratch(&headless_directory.context())
        .map_err(PlatformCheckError::from);
    let headless = headless_directory.finish(headless_result, platform_scratch_error)?;
    let frame_directory = scratch
        .create_directory("game-frame")
        .map_err(platform_scratch_error)?;
    let frame_result = prepare_game_frame_with_scratch(&frame_directory.context())
        .map_err(PlatformCheckError::from);
    let prepared = frame_directory.finish(frame_result, platform_scratch_error)?;
    let game = prepared.check;
    verify_authoritative_parity(&headless, &game)?;
    let candidate_status =
        run_desktop_candidate(&prepared.snapshot, &prepared.render_content_catalog)?;

    let mut host = ReferencePlatformHost::interactive()?;
    if host.presentation_target_kind() != PresentationTargetKindV1::Interactive
        || host.drawable_extent() != [960, 540]
    {
        return Err(PlatformCheckError::HostContractMismatch);
    }
    let host_instance = PersistentId::from_bytes([0x71; 16]);
    let window_source = SchemaId::new("nextengine.platform.source.window")?;
    let keyboard_source = SchemaId::new("nextengine.platform.source.keyboard")?;
    let capability_hash = host.capability_set().canonical_hash;
    let events = [
        PlatformEventV1::new(
            host_instance,
            window_source.clone(),
            0,
            0,
            PlatformEventKindV1::FocusChanged,
            PlatformEventPayloadV1::FocusChanged { focused: false },
            capability_hash,
        )?,
        PlatformEventV1::new(
            host_instance,
            window_source.clone(),
            1,
            1,
            PlatformEventKindV1::CapabilityChanged,
            PlatformEventPayloadV1::WindowExtentChanged {
                width: 1_280,
                height: 720,
            },
            capability_hash,
        )?,
        PlatformEventV1::new(
            host_instance,
            window_source,
            2,
            2,
            PlatformEventKindV1::CloseRequested,
            PlatformEventPayloadV1::Reason {
                reason: SchemaId::new("nextengine.platform.reason.user-close")?,
            },
            capability_hash,
        )?,
        PlatformEventV1::new(
            host_instance,
            keyboard_source.clone(),
            0,
            2,
            PlatformEventKindV1::Control,
            PlatformEventPayloadV1::Control(NormalizedControlEventV1::new(
                SchemaId::new("nextengine.input.keyboard")?,
                PersistentId::from_bytes([0x72; 16]),
                SchemaId::new("nextengine.input.key.forward")?,
                NormalizedControlPhaseV1::Started,
                vec![i16::MAX],
                Vec::new(),
                2,
                0,
            )?),
            capability_hash,
        )?,
    ];
    for event in events.into_iter().rev() {
        host.inject_event(event)?;
    }
    let normalized = host.poll_events()?;
    if normalized.len() != 4
        || !normalized
            .iter()
            .any(|event| event.kind == PlatformEventKindV1::Control)
        || !normalized
            .iter()
            .any(|event| event.kind == PlatformEventKindV1::FocusChanged)
        || !normalized
            .iter()
            .any(|event| event.kind == PlatformEventKindV1::CloseRequested)
    {
        return Err(PlatformCheckError::HostContractMismatch);
    }
    let headless_host = ReferencePlatformHost::headless()?;
    if headless_host.presentation_target_kind() != PresentationTargetKindV1::None
        || headless_host.drawable_extent() != [0, 0]
    {
        return Err(PlatformCheckError::HeadlessCreatedPresentationTarget);
    }

    Ok(PlatformCheckReport {
        normalized_events: normalized.len(),
        rendered_objects: game.rendered_object_count,
        authoritative_state_root: game.play.final_state_root,
        authoritative_ledger_hash: game.play.final_command_ledger_hash,
        presentation_snapshot_hash: game.presentation_snapshot_hash,
        candidate_status,
    })
}

fn platform_scratch_error(error: std::io::Error) -> PlatformCheckError {
    PlatformCheckError::Play(PlayCheckError::Fixture(
        crate::NeutralFixtureError::Cleanup(error),
    ))
}

#[cfg(feature = "desktop-sdl-ash")]
fn run_desktop_frame_timing_smoke_scoped(
    scratch: &ScratchContext,
    measured_frames: u32,
    initial_extent: [u32; 2],
) -> Result<Option<DesktopFrameTimingSmokeReport>, PlatformCheckError> {
    if !cfg!(all(
        target_arch = "x86_64",
        any(target_os = "windows", target_os = "linux")
    )) {
        return Ok(None);
    }
    let measured_frames_u64 = u64::from(measured_frames);
    let maximum_event_loop_iterations = measured_frames_u64
        .checked_mul(300)
        .ok_or(PlatformCheckError::DesktopSmokeMismatch)?;
    let prepared = prepare_game_frame_with_scratch(scratch)?;
    let report = next_desktop_sdl_ash::run_interactive(
        &prepared.snapshot,
        &prepared.render_content_catalog,
        &next_desktop_sdl_ash::DesktopRunOptions {
            initial_extent,
            maximum_frames: Some(measured_frames_u64),
            maximum_event_loop_iterations: Some(maximum_event_loop_iterations),
            frame_profiling_sample_capacity: measured_frames,
            ..next_desktop_sdl_ash::DesktopRunOptions::default()
        },
    )?;
    if report.rendered_frames != measured_frames_u64
        || report.frame_timings.len()
            != usize::try_from(measured_frames)
                .map_err(|_| PlatformCheckError::DesktopSmokeMismatch)?
        || report.vulkan_timestamp_queries != measured_frames_u64 * 2
        || report.dropped_frame_timing_samples != 0
        || report.frame_plan_cache_misses != 1
        || report.frame_plan_cache_hits != measured_frames_u64.saturating_sub(1)
        || report.frame_plan_build_failures != 0
        || report.frame_plan_explicit_invalidations != 0
        || report.device_allocation_bytes == 0
        || report.device_allocation_count == 0
    {
        return Err(PlatformCheckError::DesktopSmokeMismatch);
    }
    let frame_plan_hash = report
        .last_frame_plan_hash
        .ok_or(PlatformCheckError::DesktopSmokeMismatch)?;
    let drawable_extent = report
        .last_drawable_extent
        .filter(|extent| !extent.contains(&0))
        .ok_or(PlatformCheckError::DesktopSmokeMismatch)?;
    Ok(Some(DesktopFrameTimingSmokeReport {
        samples: report
            .frame_timings
            .into_iter()
            .map(|sample| DesktopFrameTimingSmokeSample {
                cpu_extract_and_submit_microseconds: sample.cpu_extract_and_submit_microseconds,
                gpu_duration_microseconds: sample.gpu_duration_microseconds,
                event_and_frame_source_update_microseconds: sample
                    .event_and_frame_source_update_microseconds,
                frame_slot_wait_microseconds: sample.frame_slot_wait_microseconds,
                image_acquire_wait_microseconds: sample.image_acquire_wait_microseconds,
                swapchain_image_wait_microseconds: sample.swapchain_image_wait_microseconds,
                frame_plan_microseconds: sample.frame_plan_microseconds,
                command_record_microseconds: sample.command_record_microseconds,
                queue_submit_microseconds: sample.queue_submit_microseconds,
                present_wait_microseconds: sample.present_wait_microseconds,
            })
            .collect(),
        drawable_extent,
        timestamp_query_count: report.vulkan_timestamp_queries,
        dropped_samples: report.dropped_frame_timing_samples,
        frame_plan_hash,
        frame_plan_cache_hits: report.frame_plan_cache_hits,
        frame_plan_cache_misses: report.frame_plan_cache_misses,
        frame_plan_build_failures: report.frame_plan_build_failures,
        frame_plan_explicit_invalidations: report.frame_plan_explicit_invalidations,
        software_paced_iterations: report.software_paced_iterations,
        software_pacing_sleep_microseconds: report.software_pacing_sleep_microseconds,
        device_allocation_bytes: report.device_allocation_bytes,
        device_allocation_count: report.device_allocation_count,
    }))
}

#[cfg(not(feature = "desktop-sdl-ash"))]
fn run_desktop_frame_timing_smoke_scoped(
    _scratch: &ScratchContext,
    _measured_frames: u32,
    _initial_extent: [u32; 2],
) -> Result<Option<DesktopFrameTimingSmokeReport>, PlatformCheckError> {
    Ok(None)
}

#[cfg(feature = "desktop-sdl-ash")]
fn run_desktop_candidate(
    snapshot: &next_contracts::presentation::PresentationSnapshotV2,
    render_content_catalog: &next_contracts::render_content::RenderContentCatalogV1,
) -> Result<PlatformCandidateStatus, PlatformCheckError> {
    if !cfg!(all(
        target_arch = "x86_64",
        any(target_os = "windows", target_os = "linux")
    )) {
        return Ok(PlatformCandidateStatus::NotRunOnDeveloperHost);
    }
    let report = next_desktop_sdl_ash::run_interactive(
        snapshot,
        render_content_catalog,
        &next_desktop_sdl_ash::DesktopRunOptions {
            maximum_frames: Some(1),
            maximum_event_loop_iterations: Some(600),
            inject_device_loss_after_frames: Some(0),
            inject_startup_lifecycle_probe: true,
            ..next_desktop_sdl_ash::DesktopRunOptions::default()
        },
    )?;
    let drawable_extent = report
        .last_drawable_extent
        .ok_or(PlatformCheckError::DesktopSmokeMismatch)?;
    let target_revision = report
        .last_target_revision
        .ok_or(PlatformCheckError::DesktopSmokeMismatch)?;
    let expected_plan = next_render::build_b0_frame_plan(
        snapshot,
        render_content_catalog,
        next_render::RenderTargetV1 {
            extent: drawable_extent,
            target_revision,
        },
    )
    .map_err(|_| PlatformCheckError::DesktopSmokeMismatch)?;
    if report.rendered_frames != 1
        || report.rendered_objects != u64::from(expected_plan.visible_object_count)
        || report.indexed_draws != u64::from(expected_plan.indexed_draw_count)
        || report.fallback_material_draws != u64::from(expected_plan.fallback_material_draw_count)
        || report.last_frame_plan_hash != Some(expected_plan.frame_plan_hash)
        || report.control_events != 4
        || report.resize_events < 1
        || report.focus_events < 2
        || report.fullscreen_events != 1
        || report.device_loss_events != 1
        || report.device_recoveries != 1
        || !report.b0_capabilities_verified
    {
        return Err(PlatformCheckError::DesktopSmokeMismatch);
    }
    Ok(PlatformCandidateStatus::Pass)
}

#[cfg(not(feature = "desktop-sdl-ash"))]
fn run_desktop_candidate(
    _snapshot: &next_contracts::presentation::PresentationSnapshotV2,
    _render_content_catalog: &next_contracts::render_content::RenderContentCatalogV1,
) -> Result<PlatformCandidateStatus, PlatformCheckError> {
    if cfg!(all(
        target_arch = "x86_64",
        any(target_os = "windows", target_os = "linux")
    )) {
        Ok(PlatformCandidateStatus::NotRunAdapterDisabled)
    } else {
        Ok(PlatformCandidateStatus::NotRunOnDeveloperHost)
    }
}

fn verify_authoritative_parity(
    headless: &crate::PlayCheckReport,
    game: &GameCheckReport,
) -> Result<(), PlatformCheckError> {
    if headless.final_state_root != game.play.final_state_root
        || headless.final_command_ledger_hash != game.play.final_command_ledger_hash
        || headless.ticks != game.play.ticks
        || headless.rpg_events != game.play.rpg_events
    {
        return Err(PlatformCheckError::AuthoritativeParityMismatch);
    }
    Ok(())
}

#[derive(Debug)]
#[non_exhaustive]
pub enum PlatformCheckError {
    Play(PlayCheckError),
    Platform(PlatformHostError),
    Contract(next_contracts::platform::PlatformContractError),
    Identifier(next_contracts::ids::IdentifierError),
    AuthoritativeParityMismatch,
    HostContractMismatch,
    HeadlessCreatedPresentationTarget,
    DesktopSmokeMismatch,
    #[cfg(feature = "desktop-sdl-ash")]
    Desktop(next_desktop_sdl_ash::DesktopAdapterError),
}

impl Display for PlatformCheckError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Play(error) => write!(formatter, "{error}"),
            Self::Platform(error) => write!(formatter, "{error}"),
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::Identifier(error) => write!(formatter, "{error}"),
            Self::AuthoritativeParityMismatch => {
                formatter.write_str("game/headless authoritative hashes diverged")
            }
            Self::HostContractMismatch => {
                formatter.write_str("platform host lifecycle normalization mismatch")
            }
            Self::HeadlessCreatedPresentationTarget => {
                formatter.write_str("headless created a presentation target")
            }
            Self::DesktopSmokeMismatch => {
                formatter.write_str("desktop adapter did not render the bounded B0 smoke frame")
            }
            #[cfg(feature = "desktop-sdl-ash")]
            Self::Desktop(error) => write!(formatter, "desktop adapter smoke failed: {error}"),
        }
    }
}

impl Error for PlatformCheckError {}

impl From<PlayCheckError> for PlatformCheckError {
    fn from(error: PlayCheckError) -> Self {
        Self::Play(error)
    }
}

impl From<PlatformHostError> for PlatformCheckError {
    fn from(error: PlatformHostError) -> Self {
        Self::Platform(error)
    }
}

impl From<next_contracts::platform::PlatformContractError> for PlatformCheckError {
    fn from(error: next_contracts::platform::PlatformContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<next_contracts::ids::IdentifierError> for PlatformCheckError {
    fn from(error: next_contracts::ids::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

#[cfg(feature = "desktop-sdl-ash")]
impl From<next_desktop_sdl_ash::DesktopAdapterError> for PlatformCheckError {
    fn from(error: next_desktop_sdl_ash::DesktopAdapterError) -> Self {
        Self::Desktop(error)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_DESKTOP_FRAME_TIMING_SAMPLES, PlatformCheckError, run_desktop_frame_timing_workload_in,
        run_platform_check,
    };

    #[test]
    fn game_headless_platform_and_presentation_contracts_match() {
        let report = run_platform_check().expect("platform check");
        assert_eq!(report.normalized_events, 4);
        assert_eq!(report.rendered_objects, 5);
    }

    #[test]
    fn desktop_timing_workload_rejects_zero_and_unbounded_sample_counts() {
        for sample_count in [0, MAX_DESKTOP_FRAME_TIMING_SAMPLES + 1] {
            assert!(matches!(
                run_desktop_frame_timing_workload_in(
                    &std::env::temp_dir(),
                    sample_count,
                    [960, 540],
                ),
                Err(PlatformCheckError::DesktopSmokeMismatch)
            ));
        }
        assert!(matches!(
            run_desktop_frame_timing_workload_in(&std::env::temp_dir(), 1, [0, 540]),
            Err(PlatformCheckError::DesktopSmokeMismatch)
        ));
    }
}
