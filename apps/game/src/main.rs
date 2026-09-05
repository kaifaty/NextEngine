#![forbid(unsafe_code)]

#[cfg(feature = "desktop-sdl-ash")]
use std::sync::Arc;

#[cfg(test)]
use next_application::ApplicationError;
use next_application::{
    ApplicationCloseOutcomeV2, ApplicationCoordinator, DiagnosticContextV1, DiagnosticReportV1,
    LaunchRequestV1, ProjectSelectionV1, RunReportV1, default_user_state_root,
};
#[cfg(feature = "desktop-sdl-ash")]
use next_application::{
    InteractiveSimulationWorkerV1, InteractiveWorkerFinalizationV1, PlayerPreferenceLoadOutcomeV1,
    PlayerPreferenceStoreV1, preference_ui_options,
};
#[cfg(feature = "desktop-sdl-ash")]
use next_contracts::preferences::PlayerPreferenceProfileV1;
use next_contracts::session::{CompositionRootV1, PresentationTargetKindV1};

#[cfg(feature = "desktop-sdl-ash")]
mod capture;
mod cli;
#[cfg(feature = "physx-water")]
mod physx_water;
#[cfg(feature = "desktop-sdl-ash")]
mod water_presentation;
#[cfg(feature = "desktop-sdl-ash")]
mod water_waves;

use cli::{AppFailure, GameOptions};

/// `--capture-frame` / `--capture-png`: one rendered frame to write as a
/// diagnostic PNG (plan `continuum-water/09`, human look gate).
#[derive(Clone, Debug, Eq, PartialEq)]
struct CaptureOptions {
    rendered_frame_index: u64,
    png: std::path::PathBuf,
    /// Plan `continuum-water/18`: the `--capture-buffer` name (`color` by default).
    source: String,
    frame_count: u32,
}

fn main() {
    match run(std::env::args().skip(1)) {
        Ok(report) => {
            println!("{}", report.to_json().expect("run report serializes"));
        }
        Err(error) => {
            eprintln!("next_game: {}", error.message);
            let report =
                DiagnosticReportV1::new(error.code, DiagnosticContextV1::message(&error.message));
            println!("{}", report.to_json().expect("diagnostic serializes"));
            std::process::exit(error.exit_code);
        }
    }
}

fn run(arguments: impl Iterator<Item = String>) -> Result<RunReportV1, AppFailure> {
    let options = GameOptions::parse(arguments)?;
    #[cfg(feature = "physical-sound-lab")]
    eprintln!("next_game: SPEC-45 experimental physical-sound-lab enabled (presentation only)");
    #[cfg(feature = "physical-sound-selected-glass")]
    eprintln!(
        "next_game: selected 16-mode Q30 thin-container glass candidate enabled (experiment only)"
    );
    if options.help {
        eprintln!(
            "usage: next_game [--interactive [--maximum-frames <positive-integer>]] [--project <cooked-store>] [--lock <sha256>] [--state-root <directory>]"
        );
        return Err(AppFailure::help());
    }
    if options.interactive && !cfg!(feature = "desktop-sdl-ash") {
        return Err(AppFailure::cli(
            "PLATFORM_INTERACTIVE_ADAPTER_UNAVAILABLE",
            "interactive desktop adapter is not enabled",
        ));
    }
    let state_root = options
        .state_root
        .unwrap_or(default_user_state_root("game").map_err(AppFailure::application)?);
    let target = if options.interactive {
        PresentationTargetKindV1::Interactive
    } else {
        PresentationTargetKindV1::None
    };
    #[cfg(feature = "desktop-sdl-ash")]
    let platform_capability_set = if options.interactive {
        Some(
            next_desktop_sdl_ash::desktop_capability_set()
                .map_err(|error| AppFailure::cli(error.diagnostic_code(), error.to_string()))?,
        )
    } else {
        None
    };
    #[cfg(not(feature = "desktop-sdl-ash"))]
    let platform_capability_set = None;
    let launch = LaunchRequestV1 {
        project: options.project.map_or(
            ProjectSelectionV1::Reference,
            ProjectSelectionV1::PublishedStateRoot,
        ),
        expected_project_lock: options.expected_lock,
        state_root,
        composition_root: CompositionRootV1::Game,
        presentation_target: target,
        platform_capability_set,
        spawn_override: None,
    };
    if options.interactive {
        let source = options
            .capture_buffer
            .clone()
            .unwrap_or_else(|| "color".to_owned());
        let capture = options.capture_frame.zip(options.capture_png.clone()).map(
            |(rendered_frame_index, png)| CaptureOptions {
                rendered_frame_index,
                png,
                source: source.clone(),
                frame_count: options.capture_frames.unwrap_or(1),
            },
        );
        let mut launch = launch;
        if options.start_at_water {
            // Plan 11: a fresh session in front of the basin, looking at it.
            launch.spawn_override = Some(next_reference_game::ReferenceSpawnOverrideV1::at_water());
        }
        if options.start_at_pond {
            // Plan 32: on the pond floor, the camera under the level.
            launch.spawn_override = Some(next_reference_game::ReferenceSpawnOverrideV1::in_pond());
        }
        if options.start_at_lake {
            // Plan 39: on the west bank's top, the lake ahead.
            launch.spawn_override = Some(next_reference_game::ReferenceSpawnOverrideV1::at_lake());
        }
        if options.start_at_falls {
            // Plan 39: south of the pond, the stream and the dam ahead.
            launch.spawn_override = Some(next_reference_game::ReferenceSpawnOverrideV1::at_falls());
        }
        if options.start_at_vessels {
            // Plan 36: south of the vessels, in reach of the gate lever.
            launch.spawn_override =
                Some(next_reference_game::ReferenceSpawnOverrideV1::at_vessels());
        }
        if options.physx_water {
            // Plan 24: the PhysX water demo pours into the basin, seen from
            // the water start.
            launch.spawn_override = Some(next_reference_game::ReferenceSpawnOverrideV1::at_water());
        }
        return run_interactive_session(
            launch,
            options.maximum_frames,
            capture,
            InteractiveSessionOptions {
                projection_jitter: options.projection_jitter,
                physx_water: options.physx_water,
                physx_water_pour: options.physx_water_pour,
                physx_water_fail_after: options.physx_water_fail_after,
                inject_device_loss_after_frames: options.inject_device_loss_after_frames,
            },
        );
    }

    let mut application =
        ApplicationCoordinator::launch_or_resume(launch).map_err(AppFailure::application)?;
    eprintln!(
        "next_game: session {} active",
        application.state().session_id.to_hex()
    );
    let run = application
        .run_reference_game(true)
        .map_err(AppFailure::application)?;
    let close = application.close().map_err(AppFailure::application)?;
    if !matches!(close, ApplicationCloseOutcomeV2::Closed { .. }) {
        return Err(AppFailure::cli(
            "SESSION_FINAL_SAVE_FAILED",
            "application close did not reach a terminal receipt",
        ));
    }
    RunReportV1::new(CompositionRootV1::Game, &run, &close, 0).ok_or_else(|| {
        AppFailure::cli(
            "SESSION_TERMINAL_RECEIPT_MISSING",
            "closed application has no terminal receipt",
        )
    })
}

#[cfg(test)]
fn begin_or_resume_reference_game_live(
    application: &mut ApplicationCoordinator,
) -> Result<next_application::ApplicationRunOutcomeV1, AppFailure> {
    match application.current_live_run() {
        Ok(run) => Ok(run),
        Err(ApplicationError::NoLiveRun) => application
            .begin_reference_game_live(true)
            .map_err(AppFailure::application),
        Err(error) => Err(AppFailure::application(error)),
    }
}

/// The interactive session's switches beyond the launch request.
#[cfg_attr(
    not(feature = "desktop-sdl-ash"),
    allow(dead_code, reason = "read only by the desktop session")
)]
struct InteractiveSessionOptions {
    projection_jitter: bool,
    physx_water: bool,
    physx_water_pour: bool,
    /// Plan 29: inject a fluid failure after this many lane frames.
    physx_water_fail_after: Option<u64>,
    /// Plan 29: the adapter's injected device loss.
    inject_device_loss_after_frames: Option<u64>,
}

#[cfg(feature = "desktop-sdl-ash")]
fn run_interactive_session(
    launch: LaunchRequestV1,
    maximum_frames: Option<u64>,
    capture: Option<CaptureOptions>,
    session: InteractiveSessionOptions,
) -> Result<RunReportV1, AppFailure> {
    let InteractiveSessionOptions {
        projection_jitter,
        physx_water,
        physx_water_pour,
        physx_water_fail_after,
        inject_device_loss_after_frames,
    } = session;
    let capture = capture.map(|capture| capture::CaptureRequest {
        rendered_frame_index: capture.rendered_frame_index,
        png: capture.png,
        source: capture::parse_capture_source(&capture.source).unwrap_or_default(),
        frame_count: capture.frame_count,
    });
    let state_root = launch.state_root.clone();
    let (worker, ready) =
        InteractiveSimulationWorkerV1::spawn(launch).map_err(AppFailure::interactive_worker)?;
    let platform_close_event = std::cell::RefCell::new(None);
    let worker_result = std::cell::RefCell::new(None);
    let worker = std::cell::RefCell::new(worker);
    let mut finalization_retries = 0_u64;
    let mut last_rendered_generation = (
        ready.initial_snapshot.snapshot_epoch,
        ready.initial_snapshot.snapshot_sequence,
    );
    let mut last_audio_sequence = 0_u64;

    // Local PresentationOnly preference profile: a missing file yields bounded
    // defaults, an unreadable one was quarantined by the store and also yields
    // defaults; a storage error can never block the game (SPEC-18).
    let (ui_locale, ui_text_scale_milli, ui_subtitles_enabled) =
        match PlayerPreferenceStoreV1::new(&state_root).load() {
            Ok(load) => {
                if let PlayerPreferenceLoadOutcomeV1::Quarantined { diagnostic_code } = load.outcome
                {
                    eprintln!("next_game: {diagnostic_code}: preference profile quarantined");
                }
                preference_ui_options(&load.profile)
            }
            Err(error) => {
                eprintln!(
                    "next_game: {}: preference store unavailable, using defaults",
                    error.diagnostic_code()
                );
                preference_ui_options(&PlayerPreferenceProfileV1::bounded_defaults())
            }
        };

    // Plan 09 (ADR-101/102): the three authored water quads are declared as
    // dynamic surfaces and the gate jet as the one particle surface. Both
    // are presentation-only feeds of the committed checkpoint.
    let mut water_feed =
        water_presentation::WaterPresentationFeed::new(&ready.render_content_catalog)
            .map_err(|message| AppFailure::cli("GAME_WATER_PRESENTATION_INVALID", message))?;
    // Plan 25 (ADR-106): the PhysX water lane, optional and fail-closed to
    // the stage's droplets.
    #[cfg(feature = "physx-water")]
    let mut physx_lane_fallback: Option<String> = None;
    #[cfg(feature = "physx-water")]
    let physx_lane = std::cell::RefCell::new(if physx_water {
        match physx_water::PhysxWaterLane::new(
            water_feed.particle_bounds(),
            physx_water_pour,
            physx_water_fail_after,
        ) {
            Ok(lane) => {
                eprintln!("next_game: PHYSX_WATER: lane active over the basin");
                Some(lane)
            }
            Err(reason) => {
                eprintln!("next_game: PHYSX_WATER_FALLBACK: {reason}");
                physx_lane_fallback = Some(reason);
                None
            }
        }
    } else {
        None
    });
    #[cfg(not(feature = "physx-water"))]
    let _ = (physx_water_pour, physx_water_fail_after);
    #[cfg(not(feature = "physx-water"))]
    if physx_water {
        return Err(AppFailure::cli(
            "GAME_PHYSX_WATER_UNAVAILABLE",
            "--physx-water needs a build with --features physx-water",
        ));
    }
    #[cfg(feature = "physx-water")]
    let particle_profile = physx_lane.borrow().as_ref().map_or_else(
        || water_feed.particle_surface_profile(),
        physx_water::PhysxWaterLane::particle_surface_profile,
    );
    #[cfg(not(feature = "physx-water"))]
    let particle_profile = water_feed.particle_surface_profile();
    let adapter = next_desktop_sdl_ash::run_interactive_with_shared_frame_publication_and_finalize(
        Arc::clone(&ready.initial_snapshot),
        &ready.render_content_catalog,
        &next_desktop_sdl_ash::DesktopRunOptions {
            maximum_frames,
            host_instance_id: ready.host_instance_id,
            resume_suspended_application: ready.resume_suspended_application,
            ui_text_catalogs: ready.text_catalogs.clone(),
            ui_locale: ui_locale.clone(),
            ui_text_scale_milli,
            ui_subtitles_enabled,
            dynamic_surfaces: water_feed.dynamic_surface_profiles(),
            particle_surface: Some(particle_profile),
            frame_capture: capture
                .as_ref()
                .map(capture::CaptureRequest::adapter_request),
            scripted_input: Vec::new(),
            projection_jitter,
            inject_device_loss_after_frames,
            // Scene look L1 (plan `look/01`): a bounded timestamp buffer so
            // the closing report carries the GPU frame time.
            frame_profiling_sample_capacity: 4_096,
            ..next_desktop_sdl_ash::DesktopRunOptions::default()
        },
        |events, elapsed, audio| {
            let mut worker = worker.borrow_mut();
            if let Some(failure) = worker.try_take_failure() {
                eprintln!(
                    "next_game: simulation worker failed before desktop finalization: {}: {}",
                    failure.code, failure.message
                );
                return Err(next_desktop_sdl_ash::DesktopAdapterError::client(
                    failure.code,
                    failure.message,
                ));
            }
            let scheduler_events = platform_events_before_close_boundary(
                events,
                &mut platform_close_event.borrow_mut(),
            );
            worker
                .submit_advance(elapsed, scheduler_events)
                .map_err(|failure| {
                    next_desktop_sdl_ash::DesktopAdapterError::client(failure.code, failure.message)
                })?;
            if let Some(failure) = worker.try_take_failure() {
                eprintln!(
                    "next_game: simulation worker failed before desktop finalization: {}: {}",
                    failure.code, failure.message
                );
                return Err(next_desktop_sdl_ash::DesktopAdapterError::client(
                    failure.code,
                    failure.message,
                ));
            }

            // Baseline audio (A4): queue exactly the canonical PCM windows the
            // worker published since the previous pump; output failure degrades
            // to silence inside the adapter and never fails the frame.
            if let Ok(Some(frame)) = worker.read_latest_audio()
                && frame.audio_sequence != last_audio_sequence
            {
                last_audio_sequence = frame.audio_sequence;
                audio.queue_pcm(&frame.pcm);
            }

            let read = worker.read_latest_snapshot().map_err(|failure| {
                next_desktop_sdl_ash::DesktopAdapterError::client(failure.code, failure.message)
            })?;
            let latest = read.snapshot;
            let generation = (latest.snapshot_epoch, latest.snapshot_sequence);
            // Plan 24: the PhysX fluid publishes every frame, replacing the
            // stage's droplets while the demo runs.
            #[cfg(feature = "physx-water")]
            let demo_particles = match physx_lane.borrow_mut().as_mut() {
                Some(lane) => {
                    if generation != last_rendered_generation
                        && let Some(frame) = read.water.as_deref()
                    {
                        lane.observe_frame(frame);
                    }
                    lane.advance(elapsed, water_feed.next_sequence())?
                }
                None => None,
            };
            #[cfg(not(feature = "physx-water"))]
            let demo_particles: Option<
                Arc<next_desktop_sdl_ash::ParticleSurfaceUpdateV1>,
            > = None;
            if generation == last_rendered_generation {
                Ok(next_desktop_sdl_ash::DesktopFramePublicationV1 {
                    snapshot: None,
                    dynamic_surface_updates: Vec::new(),
                    particle_surface_update: demo_particles,
                })
            } else {
                last_rendered_generation = generation;
                let (dynamic_surface_updates, particle_surface_update) =
                    water_feed.publication(read.water.as_deref())?;
                Ok(next_desktop_sdl_ash::DesktopFramePublicationV1 {
                    snapshot: Some(latest),
                    dynamic_surface_updates,
                    particle_surface_update: demo_particles.or(particle_surface_update),
                })
            }
        },
        || {
            let finalization = worker
                .borrow_mut()
                .shutdown_attempt(platform_close_event.borrow().clone(), 0);
            if let InteractiveWorkerFinalizationV1::Closed { result, .. } = finalization {
                *worker_result.borrow_mut() =
                    Some((*result).map_err(AppFailure::interactive_worker));
                return next_desktop_sdl_ash::DesktopApplicationFinalization::Complete;
            }

            finalization_retries = finalization_retries.saturating_add(1);
            if finalization_retries.is_power_of_two() {
                let message = match &finalization {
                    InteractiveWorkerFinalizationV1::Retry(failure) => {
                        format!("{}: {}", failure.code, failure.message)
                    }
                    InteractiveWorkerFinalizationV1::Closed { .. } => {
                        "simulation worker did not confirm durable Closed".to_owned()
                    }
                };
                eprintln!(
                    "next_game: keeping desktop adapter alive for exact close retry {}: {}",
                    finalization_retries, message
                );
            }
            next_desktop_sdl_ash::DesktopApplicationFinalization::Retry
        },
    );

    let mut worker_report = worker_result.into_inner().ok_or_else(|| {
        AppFailure::cli(
            "SESSION_RUNTIME_FAILED",
            "desktop adapter released without finalizing the simulation worker",
        )
    })??;
    let adapter =
        adapter.map_err(|error| AppFailure::cli(error.diagnostic_code(), error.to_string()))?;
    worker_report.interactive_host_object_count = adapter.rendered_objects;
    // Plan 09 G5 evidence: one catalog and one frame plan per run, the ring
    // and the particle pass fed once per published snapshot.
    eprintln!(
        "next_game: water presentation: dynamic_surface_publications={}, dynamic_surface_uploads={}, dynamic_surface_draws={}, particle_surface_available={}, particle_surface_publications={}, particle_surface_frames={}, frame_plan_cache_misses={}, frame_plan_explicit_invalidations={}",
        adapter.dynamic_surface_publications,
        adapter.dynamic_surface_uploads,
        adapter.dynamic_surface_draws,
        adapter.particle_surface_available,
        adapter.particle_surface_publications,
        adapter.particle_surface_frames,
        adapter.frame_plan_cache_misses,
        adapter.frame_plan_explicit_invalidations,
    );
    if let Some(capture) = &capture {
        capture.write(&adapter.captured_frames)?;
        eprintln!(
            "next_game: captured rendered frame {} to {}",
            capture.rendered_frame_index,
            capture.png.display()
        );
    }

    // Plan 28: the lane in the run report, absent when not requested.
    #[cfg(feature = "physx-water")]
    if physx_water {
        worker_report.presentation_fluid = Some(match physx_lane.borrow().as_ref() {
            Some(lane) => {
                let stats = lane.stats();
                // Plan 29: a demoted lane keeps its statistics up to the
                // failure and carries the reason.
                next_application::PresentationFluidReportV1 {
                    lane: "physx-pbd".to_owned(),
                    active: lane.failure().is_none(),
                    fallback_reason: lane.failure().map(str::to_owned),
                    frames: stats.frames,
                    peak_particles: stats.peak_particles as u64,
                    emitted: stats.emitted,
                    absorbed: stats.absorbed,
                    last_particles: stats.last_particles as u64,
                    cost_mean_us: u64::try_from(
                        stats.cost_total_microseconds / u128::from(stats.frames.max(1)),
                    )
                    .unwrap_or(u64::MAX),
                    cost_max_us: u64::try_from(stats.cost_max_microseconds).unwrap_or(u64::MAX),
                    analysis_mean_us: u64::try_from(
                        stats.analysis_total_microseconds / u128::from(stats.frames.max(1)),
                    )
                    .unwrap_or(u64::MAX),
                    inside_colliders_max: stats.inside_colliders_max as u64,
                    spray_fraction_max_permille: stats.spray_fraction_max_permille,
                    recoveries: stats.recoveries,
                }
            }
            None => next_application::PresentationFluidReportV1 {
                lane: "physx-pbd".to_owned(),
                active: false,
                fallback_reason: Some(physx_lane_fallback.clone().unwrap_or_default()),
                frames: 0,
                peak_particles: 0,
                emitted: 0,
                absorbed: 0,
                last_particles: 0,
                cost_mean_us: 0,
                cost_max_us: 0,
                analysis_mean_us: 0,
                inside_colliders_max: 0,
                spray_fraction_max_permille: 0,
                recoveries: 0,
            },
        });
    }
    #[cfg(feature = "physx-water")]
    if let Some(lane) = physx_lane.borrow().as_ref() {
        let stats = lane.stats();
        eprintln!(
            "next_game: PHYSX_WATER: frames={}, peak_particles={}, emitted={}, absorbed={}, last_particles={}, cost_mean_us={}, cost_max_us={}, inside_colliders_max={}, analysis_mean_us={}, analysis_max_us={}, spray_max_permille={}, spray_last_permille={}, kernels_last_permille={}, kernels_blended_permille={}, kernel_change_permille={}, recoveries={}",
            stats.frames,
            stats.peak_particles,
            stats.emitted,
            stats.absorbed,
            stats.last_particles,
            stats.cost_total_microseconds / u128::from(stats.frames.max(1)),
            stats.cost_max_microseconds,
            stats.inside_colliders_max,
            stats.analysis_total_microseconds / u128::from(stats.frames.max(1)),
            stats.analysis_max_microseconds,
            stats.spray_fraction_max_permille,
            stats.spray_fraction_last_permille,
            stats.kernels_last_permille,
            stats.kernels_blended_permille,
            stats.kernel_change_permille,
            stats.recoveries
        );
    }
    eprintln!(
        "next_game: desktop session closed: frames={}, particle_frames={}, submerged_frames={}, platform_events={}, controls={}, resizes={}, focus_events={}, fullscreen={}, recoveries={}, audio_queued={}, audio_dropped={}, audio_underruns={}, audio_faults={}, audio_reopens={}, audio_active={}, device_allocation_bytes={}",
        adapter.rendered_frames,
        adapter.particle_surface_frames,
        adapter.submerged_frames,
        adapter.normalized_events,
        adapter.control_events,
        adapter.resize_events,
        adapter.focus_events,
        adapter.fullscreen_events,
        adapter.device_recoveries,
        adapter.audio_queued_samples,
        adapter.audio_dropped_samples,
        adapter.audio_callback_underruns,
        adapter.audio_device_faults,
        adapter.audio_device_reopens,
        adapter.audio_output_active,
        adapter.device_allocation_bytes,
    );
    // Scene look L1 (plan `look/01`): the GPU frame time from the adapter's
    // timestamps, mean and 95th percentile over the recorded samples.
    {
        let mut gpu: Vec<u64> = adapter
            .frame_timings
            .iter()
            .map(|sample| sample.gpu_duration_microseconds)
            .filter(|value| *value > 0)
            .collect();
        gpu.sort_unstable();
        let mean = gpu.iter().sum::<u64>() / u64::try_from(gpu.len().max(1)).unwrap_or(1);
        let p95 = gpu
            .get((gpu.len() * 95 / 100).min(gpu.len().saturating_sub(1)))
            .copied()
            .unwrap_or(0);
        eprintln!(
            "next_game: render timing: gpu_samples={}, gpu_frame_mean_us={mean}, gpu_frame_p95_us={p95}",
            gpu.len()
        );
    }
    Ok(worker_report)
}

#[cfg(any(feature = "desktop-sdl-ash", test))]
fn platform_events_before_close_boundary(
    events: &[next_contracts::platform::PlatformEventV1],
    platform_close_event: &mut Option<next_contracts::platform::PlatformEventV1>,
) -> Vec<next_contracts::platform::PlatformEventV1> {
    if platform_close_event.is_none() {
        *platform_close_event = events
            .iter()
            .filter(|event| {
                event.kind == next_contracts::platform::PlatformEventKindV1::CloseRequested
            })
            .min_by(|left, right| {
                (
                    left.host_instance_id,
                    &left.source_class,
                    left.source_sequence,
                    left.platform_event_id,
                )
                    .cmp(&(
                        right.host_instance_id,
                        &right.source_class,
                        right.source_sequence,
                        right.platform_event_id,
                    ))
            })
            .cloned();
    }
    let Some(close) = platform_close_event.as_ref() else {
        return events.to_vec();
    };
    events
        .iter()
        .filter(|event| {
            event.kind != next_contracts::platform::PlatformEventKindV1::CloseRequested
                && !(event.host_instance_id == close.host_instance_id
                    && event.source_class == close.source_class
                    && event.source_sequence >= close.source_sequence)
        })
        .cloned()
        .collect()
}

#[cfg(not(feature = "desktop-sdl-ash"))]
fn run_interactive_session(
    _launch: LaunchRequestV1,
    _maximum_frames: Option<u64>,
    _capture: Option<CaptureOptions>,
    _session: InteractiveSessionOptions,
) -> Result<RunReportV1, AppFailure> {
    Err(AppFailure::cli(
        "PLATFORM_INTERACTIVE_ADAPTER_UNAVAILABLE",
        "interactive desktop adapter is not enabled",
    ))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        GameOptions, begin_or_resume_reference_game_live, platform_events_before_close_boundary,
    };
    use next_application::{
        ApplicationCloseOutcomeV2, ApplicationCoordinator, FixedStepLiveSchedulerV1,
        LaunchRequestV1,
    };
    use next_contracts::ids::SchemaId;
    use next_contracts::platform::{PlatformEventKindV1, PlatformEventPayloadV1, PlatformEventV1};
    use next_contracts::session::{CompositionRootV1, PresentationTargetKindV1};

    #[test]
    fn bounded_interactive_mode_requires_a_positive_frame_limit() {
        let options = GameOptions::parse(
            ["--interactive", "--maximum-frames", "1"]
                .into_iter()
                .map(str::to_owned),
        )
        .expect("bounded interactive options");
        assert!(options.interactive);
        assert_eq!(options.maximum_frames, Some(1));
        assert!(
            GameOptions::parse(["--maximum-frames", "1"].into_iter().map(str::to_owned)).is_err()
        );
    }

    #[test]
    fn desktop_launch_continues_an_existing_live_session_instead_of_beginning_twice() {
        let state_root = unique_test_directory("restart-live");
        let launch = LaunchRequestV1::reference(
            state_root.clone(),
            CompositionRootV1::Game,
            PresentationTargetKindV1::Interactive,
        );
        let mut first =
            ApplicationCoordinator::launch_or_resume(launch.clone()).expect("first launch");
        let initial =
            begin_or_resume_reference_game_live(&mut first).expect("begin first live run");
        let session_id = first.state().session_id;
        drop(first);

        let mut restarted =
            ApplicationCoordinator::launch_or_resume(launch).expect("restart existing live run");
        let resumed =
            begin_or_resume_reference_game_live(&mut restarted).expect("continue recovered run");

        assert_eq!(restarted.state().session_id, session_id);
        assert_eq!(resumed.session_id, initial.session_id);
        assert_eq!(
            resumed.project_composition_lock_hash,
            initial.project_composition_lock_hash
        );
        assert_eq!(resumed.ticks, initial.ticks);
        assert_eq!(resumed.events, initial.events);
        assert_eq!(resumed.rpg_events, initial.rpg_events);
        assert_eq!(
            resumed.authoritative_revision,
            initial.authoritative_revision
        );
        assert_eq!(
            resumed.authoritative_state_root,
            initial.authoritative_state_root
        );
        assert_eq!(resumed.command_archive_root, initial.command_archive_root);
        assert_eq!(
            resumed.command_identity_index_root,
            initial.command_identity_index_root
        );
        assert_eq!(resumed.command_ledger_hash, initial.command_ledger_hash);
        assert_eq!(
            resumed.presentation_input_count,
            initial.presentation_input_count
        );
        let initial_presentation = initial
            .presentation_snapshot
            .expect("initial presentation snapshot");
        let resumed_presentation = resumed
            .presentation_snapshot
            .expect("resumed presentation snapshot");
        assert_eq!(
            resumed_presentation.snapshot_epoch,
            initial_presentation.snapshot_epoch
        );
        assert_eq!(resumed_presentation.snapshot_sequence, 0);
        assert!(
            resumed_presentation
                .camera_records()
                .all(|camera| camera.cut)
        );
        std::fs::remove_dir_all(state_root).expect("remove test state");
    }

    #[test]
    fn close_boundary_does_not_admit_same_source_suffix_before_close_transaction() {
        let state_root = unique_test_directory("close-boundary");
        let launch = LaunchRequestV1::reference(
            state_root.clone(),
            CompositionRootV1::Game,
            PresentationTargetKindV1::Interactive,
        );
        let capabilities = launch
            .platform_capability_set
            .clone()
            .expect("interactive capabilities");
        let mut application = ApplicationCoordinator::launch(launch).expect("launch");
        application
            .begin_reference_game_live(true)
            .expect("begin live game");
        let host_instance_id = application
            .register_platform_host(&capabilities)
            .expect("register platform host");
        let source_class =
            SchemaId::new("nextengine.platform.source.window-test").expect("source class");
        let event = |sequence, kind, payload| {
            PlatformEventV1::new(
                host_instance_id,
                source_class.clone(),
                sequence,
                sequence,
                kind,
                payload,
                capabilities.canonical_hash,
            )
            .expect("platform event")
        };
        let before = event(
            0,
            PlatformEventKindV1::FocusChanged,
            PlatformEventPayloadV1::FocusChanged { focused: false },
        );
        let close = event(
            1,
            PlatformEventKindV1::CloseRequested,
            PlatformEventPayloadV1::Reason {
                reason: SchemaId::new("nextengine.platform.reason.window-close")
                    .expect("close reason"),
            },
        );
        let after = event(
            2,
            PlatformEventKindV1::FocusChanged,
            PlatformEventPayloadV1::FocusChanged { focused: true },
        );
        let mut selected_close = None;
        let scheduler_events = platform_events_before_close_boundary(
            &[before.clone(), close.clone(), after],
            &mut selected_close,
        );
        assert_eq!(scheduler_events, vec![before]);
        assert_eq!(selected_close, Some(close.clone()));

        let mut scheduler = FixedStepLiveSchedulerV1::reference_game_v1();
        scheduler
            .advance_reference_game(&mut application, Duration::ZERO, &scheduler_events)
            .expect("admit only the pre-close source prefix");
        let closed = application
            .close_from_platform_event(&close)
            .expect("close remains the exact next source event");
        assert!(matches!(closed, ApplicationCloseOutcomeV2::Closed { .. }));
        std::fs::remove_dir_all(state_root).expect("remove test state");
    }

    fn unique_test_directory(label: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};

        static SEQUENCE: AtomicU64 = AtomicU64::new(0);

        std::env::temp_dir().join(format!(
            "nextengine-game-{label}-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ))
    }
}
