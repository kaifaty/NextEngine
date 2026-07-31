#![allow(
    unsafe_code,
    reason = "ADR-003 confines graphics API calls and native surface ownership to this crate"
)]

use std::cell::RefCell;
use std::ffi::CString;
use std::fmt::Display;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ash::vk;
use next_contracts::ids::{ContentHash, PersistentId};
use next_contracts::platform::{
    NormalizedControlPhaseV1, PlatformCapabilitySetV1, PlatformEventKindV1, PlatformEventV1,
};
use next_contracts::presentation::PresentationSnapshotV2;
use next_contracts::render_content::RenderContentCatalogV1;
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::{Mod, Scancode};
use sdl3::video::Window;

mod error;
mod run_state;

pub use error::DesktopAdapterError;
use run_state::{AdapterFinalizer, InteractivePacingClock, apply_software_pacing};
pub use run_state::{
    DesktopApplicationFinalization, DesktopFrameTimingSample, DesktopRunOptions, DesktopRunReport,
    MAX_FRAME_PROFILING_SAMPLES,
};
#[cfg(test)]
use run_state::{INTERACTIVE_FRAME_INTERVAL, remaining_frame_budget, software_pacing_delay};

/// Returns the exact engine-owned desktop capability descriptor embedded in
/// every normalized event emitted by this adapter.
pub fn desktop_capability_set() -> Result<PlatformCapabilitySetV1, DesktopAdapterError> {
    lifecycle::desktop_capability_set()
}

/// Returns the canonical hash of [`desktop_capability_set`].
pub fn desktop_capability_set_hash() -> Result<ContentHash, DesktopAdapterError> {
    Ok(desktop_capability_set()?.canonical_hash)
}

pub fn run_interactive(
    snapshot: &PresentationSnapshotV2,
    render_content_catalog: &RenderContentCatalogV1,
    options: &DesktopRunOptions,
) -> Result<DesktopRunReport, DesktopAdapterError> {
    run_interactive_with_frame_source(snapshot, render_content_catalog, options, |_| Ok(None))
}

pub fn run_interactive_with_event_sink(
    snapshot: &PresentationSnapshotV2,
    render_content_catalog: &RenderContentCatalogV1,
    options: &DesktopRunOptions,
    mut event_sink: impl FnMut(&[PlatformEventV1]) -> Result<(), DesktopAdapterError>,
) -> Result<DesktopRunReport, DesktopAdapterError> {
    run_interactive_with_frame_source(snapshot, render_content_catalog, options, |events| {
        if !events.is_empty() {
            event_sink(events)?;
        }
        Ok(None)
    })
}

/// Runs the native desktop loop while allowing the application to advance the
/// simulation and atomically replace the immutable presentation snapshot once
/// per event-loop iteration.
pub fn run_interactive_with_frame_source(
    snapshot: &PresentationSnapshotV2,
    render_content_catalog: &RenderContentCatalogV1,
    options: &DesktopRunOptions,
    mut frame_source: impl FnMut(
        &[PlatformEventV1],
    ) -> Result<Option<PresentationSnapshotV2>, DesktopAdapterError>,
) -> Result<DesktopRunReport, DesktopAdapterError> {
    run_interactive_with_timed_frame_source(
        snapshot,
        render_content_catalog,
        options,
        |events, _elapsed| frame_source(events),
    )
}

/// Runs the native desktop loop with a monotonic elapsed interval that starts
/// after the first successful frame submission and is then available to a
/// fixed-step application scheduler. Vulkan initialization and cold first-frame
/// work cannot become simulation catch-up. The callback is still free to
/// publish no new snapshot, allowing rendering to repeat the latest immutable
/// projection.
pub fn run_interactive_with_timed_frame_source(
    snapshot: &PresentationSnapshotV2,
    render_content_catalog: &RenderContentCatalogV1,
    options: &DesktopRunOptions,
    frame_source: impl FnMut(
        &[PlatformEventV1],
        Duration,
    ) -> Result<Option<PresentationSnapshotV2>, DesktopAdapterError>,
) -> Result<DesktopRunReport, DesktopAdapterError> {
    run_interactive_with_timed_frame_source_and_finalize(
        snapshot,
        render_content_catalog,
        options,
        frame_source,
        || DesktopApplicationFinalization::Complete,
    )
}

/// Runs the timed desktop loop and invokes `finalize_application` before any
/// owned SDL or Vulkan adapter resource is released.
///
/// The hook is also invoked when initialization or the event/render loop
/// returns an error. It exists so an application can durably publish its
/// terminal `Closed` state while the platform adapter is still alive. A
/// [`DesktopApplicationFinalization::Retry`] result keeps all adapter resources
/// alive and repeats the hook after a short non-authoritative host delay. The
/// hook must be exact-retry safe and must not call back into this adapter.
pub fn run_interactive_with_timed_frame_source_and_finalize(
    snapshot: &PresentationSnapshotV2,
    render_content_catalog: &RenderContentCatalogV1,
    options: &DesktopRunOptions,
    mut frame_source: impl FnMut(
        &[PlatformEventV1],
        Duration,
    ) -> Result<Option<PresentationSnapshotV2>, DesktopAdapterError>,
    finalize_application: impl FnMut() -> DesktopApplicationFinalization,
) -> Result<DesktopRunReport, DesktopAdapterError> {
    run_interactive_with_shared_timed_frame_source_and_finalize(
        Arc::new(snapshot.clone()),
        render_content_catalog,
        options,
        move |events, elapsed| frame_source(events, elapsed).map(|snapshot| snapshot.map(Arc::new)),
        finalize_application,
    )
}

/// Shared-snapshot variant for composition roots that publish immutable
/// presentation generations from a simulation worker.
///
/// Ownership transfer is `Arc`-only on the frame boundary, so a large
/// projection is not copied merely to move it from the simulation worker to
/// the render thread. Snapshot validation and monotonic transition checks are
/// identical to [`run_interactive_with_timed_frame_source_and_finalize`].
pub fn run_interactive_with_shared_timed_frame_source_and_finalize(
    snapshot: Arc<PresentationSnapshotV2>,
    render_content_catalog: &RenderContentCatalogV1,
    options: &DesktopRunOptions,
    mut frame_source: impl FnMut(
        &[PlatformEventV1],
        Duration,
    )
        -> Result<Option<Arc<PresentationSnapshotV2>>, DesktopAdapterError>,
    finalize_application: impl FnMut() -> DesktopApplicationFinalization,
) -> Result<DesktopRunReport, DesktopAdapterError> {
    // These owners are declared before the guard so Rust's reverse local drop
    // order always runs application finalization before platform teardown,
    // including every `?`/early-return path below.
    let sdl;
    let video;
    let mut window;
    let mut events;
    let mut graphics;
    let mut finalizer = AdapterFinalizer::new(finalize_application);

    snapshot.validate()?;
    let current_snapshot = RefCell::new(snapshot);
    if options.initial_extent[0] == 0 || options.initial_extent[1] == 0 {
        return Err(DesktopAdapterError::InvalidExtent);
    }
    sdl = sdl3::init().map_err(sdl_error)?;
    video = sdl.video().map_err(sdl_error)?;
    window = video
        .window(
            &options.title,
            options.initial_extent[0],
            options.initial_extent[1],
        )
        .vulkan()
        .resizable()
        .position_centered()
        .build()
        .map_err(|error| DesktopAdapterError::Sdl(error.to_string()))?;
    events = sdl.event_pump().map_err(sdl_error)?;
    if options.inject_startup_lifecycle_probe {
        native_events::inject_startup_lifecycle_probe(
            &sdl.event().map_err(sdl_error)?,
            window.id(),
            options.initial_extent,
        )?;
    }
    if options.frame_profiling_sample_capacity > MAX_FRAME_PROFILING_SAMPLES {
        return Err(DesktopAdapterError::GpuProfilingSampleCapacityExceeded {
            maximum: MAX_FRAME_PROFILING_SAMPLES,
        });
    }
    graphics = Some(GraphicsContext::new(
        &window,
        render_content_catalog,
        options.frame_profiling_sample_capacity,
    )?);
    let mut normalizer = lifecycle::DesktopEventNormalizer::new(options.host_instance_id)?;
    let mut event_stats = DesktopEventStats::default();
    {
        let mut resume_sink = |events: &[PlatformEventV1], elapsed: Duration| {
            apply_frame_source_result(&current_snapshot, &mut frame_source, events, elapsed)
        };
        publish_fresh_host_resume_if_requested(
            options.resume_suspended_application,
            &mut normalizer,
            &mut resume_sink,
            &mut event_stats,
        )?
    };
    let pacing_clock = RefCell::new(InteractivePacingClock::default());
    let mut event_sink = |events: &[PlatformEventV1]| {
        let elapsed = pacing_clock.borrow_mut().elapsed_for_pump(Instant::now());
        apply_frame_source_result(&current_snapshot, &mut frame_source, events, elapsed)
    };
    let mut rendered_frames = 0_u64;
    let mut rendered_objects = 0_u64;
    let mut indexed_draws = 0_u64;
    let mut fallback_material_draws = 0_u64;
    let mut last_frame_plan_hash = None;
    let mut last_drawable_extent = None;
    let mut last_target_revision = None;
    let mut close_requested = false;
    let mut rendering_suspended = false;
    let mut fullscreen = false;
    let mut device_recoveries = 0_u64;
    let mut injected_device_loss = false;
    let mut event_loop_iterations = 0_u64;
    let mut software_paced_iterations = 0_u64;
    let mut software_pacing_sleep_microseconds = 0_u64;

    'application: loop {
        let frame_started = Instant::now();
        event_loop_iterations = advance_event_loop_iteration(
            event_loop_iterations,
            options.maximum_event_loop_iterations,
        )?;
        let mut observations = Vec::new();
        let mut swapchain_dirty = false;
        let mut fullscreen_toggle_count = 0_u64;
        let mut native_events: Vec<_> = events.poll_iter().collect();
        native_events.sort_by_key(native_events::sort_key);
        for event in native_events {
            let platform_sample_tick = event.get_timestamp();
            match event {
                Event::Quit { .. } | Event::AppTerminating { .. } => {
                    close_requested = true;
                    observations.push(close_observation(platform_sample_tick));
                }
                Event::Window {
                    win_event: WindowEvent::CloseRequested,
                    ..
                } => {
                    close_requested = true;
                    observations.push(close_observation(platform_sample_tick));
                }
                Event::Window {
                    win_event:
                        WindowEvent::Resized(width, height)
                        | WindowEvent::PixelSizeChanged(width, height),
                    ..
                } => {
                    swapchain_dirty = true;
                    if width > 0 && height > 0 {
                        let width =
                            u32::try_from(width).map_err(|_| DesktopAdapterError::InvalidExtent)?;
                        let height = u32::try_from(height)
                            .map_err(|_| DesktopAdapterError::InvalidExtent)?;
                        observations.push(lifecycle::DesktopObservation {
                            source: lifecycle::DesktopEventSource::Window,
                            platform_sample_tick,
                            kind: lifecycle::DesktopObservationKind::WindowExtentChanged {
                                width,
                                height,
                            },
                        });
                    }
                }
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::FocusGained | WindowEvent::FocusLost => {
                        observations.push(lifecycle::DesktopObservation {
                            source: lifecycle::DesktopEventSource::Window,
                            platform_sample_tick,
                            kind: lifecycle::DesktopObservationKind::FocusChanged {
                                focused: win_event == WindowEvent::FocusGained,
                            },
                        });
                    }
                    WindowEvent::Hidden | WindowEvent::Minimized | WindowEvent::Occluded => {
                        if !rendering_suspended {
                            rendering_suspended = true;
                            observations.push(suspend_observation(
                                platform_sample_tick,
                                "nextengine.platform.reason.window-minimized",
                            ));
                        }
                    }
                    WindowEvent::Exposed | WindowEvent::Restored | WindowEvent::Shown => {
                        if rendering_suspended {
                            rendering_suspended = false;
                            swapchain_dirty = true;
                            observations.push(resume_observation(
                                platform_sample_tick,
                                "nextengine.platform.reason.window-restored",
                            ));
                        }
                    }
                    _ => {}
                },
                Event::AppWillEnterBackground { .. } | Event::AppDidEnterBackground { .. } => {
                    if !rendering_suspended {
                        rendering_suspended = true;
                        observations.push(suspend_observation(
                            platform_sample_tick,
                            "nextengine.platform.reason.application-backgrounded",
                        ));
                    }
                }
                Event::AppWillEnterForeground { .. } | Event::AppDidEnterForeground { .. } => {
                    if rendering_suspended {
                        rendering_suspended = false;
                        swapchain_dirty = true;
                        observations.push(resume_observation(
                            platform_sample_tick,
                            "nextengine.platform.reason.application-foregrounded",
                        ));
                    }
                }
                Event::KeyDown {
                    scancode,
                    keymod,
                    repeat,
                    which,
                    ..
                } => {
                    if repeat {
                        continue;
                    }
                    if is_fullscreen_shortcut(scancode, keymod) {
                        fullscreen_toggle_count = fullscreen_toggle_count
                            .checked_add(1)
                            .ok_or(DesktopAdapterError::CounterOverflow)?;
                    }
                    if let Some(observation) = keyboard_observation(
                        platform_sample_tick,
                        scancode,
                        keymod,
                        which,
                        NormalizedControlPhaseV1::Started,
                    ) {
                        observations.push(observation);
                    }
                }
                Event::KeyUp {
                    scancode,
                    keymod,
                    repeat,
                    which,
                    ..
                } => {
                    if repeat {
                        continue;
                    }
                    if let Some(observation) = keyboard_observation(
                        platform_sample_tick,
                        scancode,
                        keymod,
                        which,
                        NormalizedControlPhaseV1::Completed,
                    ) {
                        observations.push(observation);
                    }
                }
                Event::MouseMotion {
                    which, xrel, yrel, ..
                } => {
                    if let Some(observation) =
                        mouse_motion_observation(platform_sample_tick, which, xrel, yrel)
                    {
                        observations.push(observation);
                    }
                }
                _ => {}
            }
        }

        let published = publish_observations(
            &mut normalizer,
            observations,
            &mut event_sink,
            &mut event_stats,
        )?;
        if !published {
            event_sink(&[])?;
        }

        if fullscreen_toggle_count % 2 == 1 {
            fullscreen = !fullscreen;
            window.set_fullscreen(fullscreen).map_err(sdl_error)?;
            swapchain_dirty = true;
        }
        event_stats.fullscreen_events = event_stats
            .fullscreen_events
            .checked_add(fullscreen_toggle_count)
            .ok_or(DesktopAdapterError::CounterOverflow)?;

        if close_requested {
            break 'application;
        }
        if rendering_suspended {
            apply_software_pacing(
                frame_started.elapsed(),
                false,
                &mut software_paced_iterations,
                &mut software_pacing_sleep_microseconds,
            )?;
            continue;
        }

        if swapchain_dirty {
            let recreate = graphics
                .as_mut()
                .ok_or(DesktopAdapterError::GraphicsContextMissing)?
                .recreate_swapchain(&window);
            if let Err(error) = recreate {
                if error.is_recoverable_presentation_loss() {
                    recover_graphics(
                        &mut graphics,
                        &window,
                        render_content_catalog,
                        options.frame_profiling_sample_capacity,
                        &mut normalizer,
                        &mut event_sink,
                        &mut event_stats,
                        &mut device_recoveries,
                        options.maximum_device_recoveries,
                        event_timestamp_fallback(rendered_frames),
                        error.presentation_loss_reason(),
                    )?;
                } else {
                    return Err(error);
                }
            }
        }

        if !injected_device_loss && options.inject_device_loss_after_frames == Some(rendered_frames)
        {
            recover_graphics(
                &mut graphics,
                &window,
                render_content_catalog,
                options.frame_profiling_sample_capacity,
                &mut normalizer,
                &mut event_sink,
                &mut event_stats,
                &mut device_recoveries,
                options.maximum_device_recoveries,
                event_timestamp_fallback(rendered_frames),
                "nextengine.platform.reason.injected-device-loss",
            )?;
            injected_device_loss = true;
        }

        let event_and_frame_source_update_microseconds =
            if options.frame_profiling_sample_capacity == 0 {
                0
            } else {
                u64::try_from(frame_started.elapsed().as_micros())
                    .map_err(|_| DesktopAdapterError::CounterOverflow)?
            };
        let current_snapshot = current_snapshot.borrow();
        let render_result = graphics
            .as_mut()
            .ok_or(DesktopAdapterError::GraphicsContextMissing)?
            .render(
                current_snapshot.as_ref(),
                &window,
                event_and_frame_source_update_microseconds,
            );
        drop(current_snapshot);
        let submitted = match render_result {
            Ok(submitted) => submitted,
            Err(error) if error.is_recoverable_presentation_loss() => {
                recover_graphics(
                    &mut graphics,
                    &window,
                    render_content_catalog,
                    options.frame_profiling_sample_capacity,
                    &mut normalizer,
                    &mut event_sink,
                    &mut event_stats,
                    &mut device_recoveries,
                    options.maximum_device_recoveries,
                    event_timestamp_fallback(rendered_frames),
                    error.presentation_loss_reason(),
                )?;
                None
            }
            Err(error) => return Err(error),
        };
        if let Some(submitted) = submitted {
            rendered_frames = rendered_frames
                .checked_add(1)
                .ok_or(DesktopAdapterError::CounterOverflow)?;
            rendered_objects = submitted.rendered_objects;
            indexed_draws = submitted.indexed_draws;
            fallback_material_draws = submitted.fallback_material_draws;
            last_frame_plan_hash = Some(submitted.frame_plan_hash);
            last_drawable_extent = Some(submitted.drawable_extent);
            last_target_revision = Some(submitted.target_revision);
            pacing_clock
                .borrow_mut()
                .observe_frame_submission(Instant::now());
        }
        if options
            .maximum_frames
            .is_some_and(|maximum| rendered_frames >= maximum)
        {
            break;
        }
        apply_software_pacing(
            frame_started.elapsed(),
            submitted.is_some(),
            &mut software_paced_iterations,
            &mut software_pacing_sleep_microseconds,
        )?;
    }
    let (frame_profiling, device_allocation_bytes, device_allocation_count, frame_plan_metrics) = {
        let graphics = graphics
            .as_mut()
            .ok_or(DesktopAdapterError::GraphicsContextMissing)?;
        graphics.wait_idle()?;
        let (bytes, allocations) = graphics.device_allocation_stats()?;
        (
            graphics.take_frame_profiling(),
            bytes,
            allocations,
            graphics.frame_plan_metrics(),
        )
    };
    let report = DesktopRunReport {
        rendered_frames,
        rendered_objects,
        indexed_draws,
        fallback_material_draws,
        last_frame_plan_hash,
        last_drawable_extent,
        last_target_revision,
        normalized_events: event_stats.normalized_events,
        control_events: event_stats.control_events,
        lifecycle_events: event_stats.lifecycle_events,
        resize_events: event_stats.resize_events,
        focus_events: event_stats.focus_events,
        fullscreen_events: event_stats.fullscreen_events,
        device_loss_events: event_stats.device_loss_events,
        device_recoveries,
        close_requested,
        api_version: vk::API_VERSION_1_3,
        b0_capabilities_verified: true,
        capability_set_hash: normalizer.capability_set_hash(),
        timebase_hash: normalizer.timebase_hash(),
        last_platform_event_id: event_stats.last_platform_event_id,
        frame_timings: frame_profiling.samples,
        vulkan_timestamp_queries: frame_profiling.timestamp_query_count,
        dropped_frame_timing_samples: frame_profiling.dropped_samples,
        software_paced_iterations,
        software_pacing_sleep_microseconds,
        frame_plan_cache_hits: frame_plan_metrics.cache_hits,
        frame_plan_cache_misses: frame_plan_metrics.cache_misses,
        frame_plan_build_failures: frame_plan_metrics.build_failures,
        frame_plan_explicit_invalidations: frame_plan_metrics.explicit_invalidations,
        device_allocation_bytes,
        device_allocation_count,
    };
    finalizer.finish();
    Ok(report)
}

fn validate_snapshot_transition(
    current: &PresentationSnapshotV2,
    next: &PresentationSnapshotV2,
) -> Result<(), DesktopAdapterError> {
    next.validate()?;
    if next.project_composition_lock_hash != current.project_composition_lock_hash
        || next.content_manifest_hash != current.content_manifest_hash
        || next.presentation_profile_hash != current.presentation_profile_hash
        || next.simulation_tick < current.simulation_tick
    {
        return Err(DesktopAdapterError::SnapshotTransitionInvalid);
    }
    if next.snapshot_epoch == current.snapshot_epoch {
        if next.snapshot_sequence <= current.snapshot_sequence {
            return Err(DesktopAdapterError::SnapshotTransitionInvalid);
        }
    } else if next.snapshot_sequence != 0 || next.camera_records().any(|camera| !camera.cut) {
        return Err(DesktopAdapterError::SnapshotTransitionInvalid);
    }
    Ok(())
}

fn apply_frame_source_result(
    current_snapshot: &RefCell<Arc<PresentationSnapshotV2>>,
    frame_source: &mut impl FnMut(
        &[PlatformEventV1],
        Duration,
    )
        -> Result<Option<Arc<PresentationSnapshotV2>>, DesktopAdapterError>,
    events: &[PlatformEventV1],
    elapsed: Duration,
) -> Result<(), DesktopAdapterError> {
    if let Some(next_snapshot) = frame_source(events, elapsed)? {
        validate_snapshot_transition(current_snapshot.borrow().as_ref(), next_snapshot.as_ref())?;
        *current_snapshot.borrow_mut() = next_snapshot;
    }
    Ok(())
}

mod gpu_content;
mod graphics;
mod lifecycle;
mod native_events;
mod shader_assets;

use graphics::GraphicsContext;

#[derive(Default)]
struct DesktopEventStats {
    normalized_events: u64,
    control_events: u64,
    lifecycle_events: u64,
    resize_events: u64,
    focus_events: u64,
    fullscreen_events: u64,
    device_loss_events: u64,
    last_platform_event_id: Option<ContentHash>,
}

impl DesktopEventStats {
    fn observe(&mut self, events: &[PlatformEventV1]) -> Result<(), DesktopAdapterError> {
        let event_count =
            u64::try_from(events.len()).map_err(|_| DesktopAdapterError::CounterOverflow)?;
        self.normalized_events = self
            .normalized_events
            .checked_add(event_count)
            .ok_or(DesktopAdapterError::CounterOverflow)?;
        for event in events {
            match event.kind {
                PlatformEventKindV1::Control => {
                    self.control_events = self
                        .control_events
                        .checked_add(1)
                        .ok_or(DesktopAdapterError::CounterOverflow)?;
                }
                PlatformEventKindV1::FocusChanged => {
                    self.focus_events = self
                        .focus_events
                        .checked_add(1)
                        .ok_or(DesktopAdapterError::CounterOverflow)?;
                    self.lifecycle_events = self
                        .lifecycle_events
                        .checked_add(1)
                        .ok_or(DesktopAdapterError::CounterOverflow)?;
                }
                PlatformEventKindV1::CapabilityChanged => {
                    self.resize_events = self
                        .resize_events
                        .checked_add(1)
                        .ok_or(DesktopAdapterError::CounterOverflow)?;
                    self.lifecycle_events = self
                        .lifecycle_events
                        .checked_add(1)
                        .ok_or(DesktopAdapterError::CounterOverflow)?;
                }
                PlatformEventKindV1::PresentationDeviceLost => {
                    self.device_loss_events = self
                        .device_loss_events
                        .checked_add(1)
                        .ok_or(DesktopAdapterError::CounterOverflow)?;
                    self.lifecycle_events = self
                        .lifecycle_events
                        .checked_add(1)
                        .ok_or(DesktopAdapterError::CounterOverflow)?;
                }
                _ => {
                    self.lifecycle_events = self
                        .lifecycle_events
                        .checked_add(1)
                        .ok_or(DesktopAdapterError::CounterOverflow)?;
                }
            }
            self.last_platform_event_id = Some(event.platform_event_id);
        }
        Ok(())
    }
}

fn publish_observations(
    normalizer: &mut lifecycle::DesktopEventNormalizer,
    observations: Vec<lifecycle::DesktopObservation>,
    event_sink: &mut impl FnMut(&[PlatformEventV1]) -> Result<(), DesktopAdapterError>,
    event_stats: &mut DesktopEventStats,
) -> Result<bool, DesktopAdapterError> {
    let events = normalizer.normalize(observations)?;
    if events.is_empty() {
        return Ok(false);
    }
    event_sink(&events)?;
    event_stats.observe(&events)?;
    Ok(true)
}

fn publish_fresh_host_resume_if_requested(
    requested: bool,
    normalizer: &mut lifecycle::DesktopEventNormalizer,
    event_sink: &mut impl FnMut(&[PlatformEventV1], Duration) -> Result<(), DesktopAdapterError>,
    event_stats: &mut DesktopEventStats,
) -> Result<bool, DesktopAdapterError> {
    if !requested {
        return Ok(false);
    }
    let events = normalizer.normalize(vec![resume_observation(
        0,
        "nextengine.platform.reason.fresh-host-ready",
    )])?;
    event_sink(&events, Duration::ZERO)?;
    event_stats.observe(&events)?;
    Ok(true)
}

#[allow(
    clippy::too_many_arguments,
    reason = "the private recovery boundary keeps the bounded policy, event publication and graphics ownership explicit"
)]
fn recover_graphics(
    graphics: &mut Option<GraphicsContext>,
    window: &Window,
    render_content_catalog: &RenderContentCatalogV1,
    frame_profiling_sample_capacity: u32,
    normalizer: &mut lifecycle::DesktopEventNormalizer,
    event_sink: &mut impl FnMut(&[PlatformEventV1]) -> Result<(), DesktopAdapterError>,
    event_stats: &mut DesktopEventStats,
    recovery_count: &mut u64,
    maximum_recoveries: u16,
    platform_sample_tick: u64,
    loss_reason: &'static str,
) -> Result<(), DesktopAdapterError> {
    if *recovery_count >= u64::from(maximum_recoveries) {
        return Err(DesktopAdapterError::DeviceRecoveryLimitExceeded {
            maximum: maximum_recoveries,
        });
    }
    let _ = publish_observations(
        normalizer,
        vec![lifecycle::DesktopObservation {
            source: lifecycle::DesktopEventSource::Graphics,
            platform_sample_tick,
            kind: lifecycle::DesktopObservationKind::PresentationDeviceLost {
                reason: loss_reason,
            },
        }],
        event_sink,
        event_stats,
    )?;
    let prior = graphics
        .take()
        .ok_or(DesktopAdapterError::GraphicsContextMissing)?;
    drop(prior);
    *graphics = Some(GraphicsContext::new(
        window,
        render_content_catalog,
        frame_profiling_sample_capacity,
    )?);
    *recovery_count = recovery_count
        .checked_add(1)
        .ok_or(DesktopAdapterError::CounterOverflow)?;
    publish_observations(
        normalizer,
        vec![lifecycle::DesktopObservation {
            source: lifecycle::DesktopEventSource::Graphics,
            platform_sample_tick,
            kind: lifecycle::DesktopObservationKind::PresentationDeviceRestored {
                reason: "nextengine.platform.reason.device-restored",
            },
        }],
        event_sink,
        event_stats,
    )
    .map(|_| ())
}

fn close_observation(platform_sample_tick: u64) -> lifecycle::DesktopObservation {
    lifecycle::DesktopObservation {
        source: lifecycle::DesktopEventSource::Window,
        platform_sample_tick,
        kind: lifecycle::DesktopObservationKind::CloseRequested {
            reason: "nextengine.platform.reason.user-close",
        },
    }
}

fn suspend_observation(
    platform_sample_tick: u64,
    reason: &'static str,
) -> lifecycle::DesktopObservation {
    lifecycle::DesktopObservation {
        source: lifecycle::DesktopEventSource::Window,
        platform_sample_tick,
        kind: lifecycle::DesktopObservationKind::SuspendRequested { reason },
    }
}

fn resume_observation(
    platform_sample_tick: u64,
    reason: &'static str,
) -> lifecycle::DesktopObservation {
    lifecycle::DesktopObservation {
        source: lifecycle::DesktopEventSource::Window,
        platform_sample_tick,
        kind: lifecycle::DesktopObservationKind::ResumeRequested { reason },
    }
}

const fn event_timestamp_fallback(rendered_frames: u64) -> u64 {
    rendered_frames
}

fn advance_event_loop_iteration(
    completed_iterations: u64,
    maximum_iterations: Option<u64>,
) -> Result<u64, DesktopAdapterError> {
    if let Some(maximum) = maximum_iterations
        && completed_iterations >= maximum
    {
        return Err(DesktopAdapterError::EventLoopIterationLimitExceeded { maximum });
    }
    completed_iterations
        .checked_add(1)
        .ok_or(DesktopAdapterError::CounterOverflow)
}

fn is_fullscreen_shortcut(scancode: Option<Scancode>, modifiers: Mod) -> bool {
    scancode == Some(Scancode::F11)
        || (scancode == Some(Scancode::Return) && modifiers.intersects(Mod::LALTMOD | Mod::RALTMOD))
}

fn keyboard_control_path(scancode: Scancode) -> Option<&'static str> {
    match scancode {
        Scancode::W => Some("nextengine.input.keyboard.w"),
        Scancode::A => Some("nextengine.input.keyboard.a"),
        Scancode::S => Some("nextengine.input.keyboard.s"),
        Scancode::D => Some("nextengine.input.keyboard.d"),
        Scancode::Q => Some("nextengine.input.keyboard.q"),
        Scancode::E => Some("nextengine.input.keyboard.e"),
        Scancode::R => Some("nextengine.input.keyboard.r"),
        Scancode::F => Some("nextengine.input.keyboard.f"),
        Scancode::I => Some("nextengine.input.keyboard.i"),
        Scancode::J => Some("nextengine.input.keyboard.j"),
        Scancode::K => Some("nextengine.input.keyboard.k"),
        Scancode::L => Some("nextengine.input.keyboard.l"),
        Scancode::Up => Some("nextengine.input.keyboard.arrow-up"),
        Scancode::Down => Some("nextengine.input.keyboard.arrow-down"),
        Scancode::Left => Some("nextengine.input.keyboard.arrow-left"),
        Scancode::Right => Some("nextengine.input.keyboard.arrow-right"),
        Scancode::Space => Some("nextengine.input.keyboard.space"),
        Scancode::Tab => Some("nextengine.input.keyboard.tab"),
        Scancode::Return => Some("nextengine.input.keyboard.enter"),
        Scancode::Escape => Some("nextengine.input.keyboard.escape"),
        Scancode::LShift => Some("nextengine.input.keyboard.shift-left"),
        Scancode::RShift => Some("nextengine.input.keyboard.shift-right"),
        Scancode::F11 => Some("nextengine.input.keyboard.f11"),
        _ => None,
    }
}

fn keyboard_observation(
    platform_sample_tick: u64,
    scancode: Option<Scancode>,
    modifiers: Mod,
    device_id: u32,
    phase: NormalizedControlPhaseV1,
) -> Option<lifecycle::DesktopObservation> {
    let control_path = scancode.and_then(keyboard_control_path)?;
    Some(lifecycle::DesktopObservation {
        source: lifecycle::DesktopEventSource::Keyboard,
        platform_sample_tick,
        kind: lifecycle::DesktopObservationKind::Control {
            control_path,
            device_class: "nextengine.input.keyboard",
            device_instance_nonce: lifecycle::keyboard_device_nonce(device_id),
            modifier_set: normalized_modifiers(modifiers),
            phase,
            quantized_value: vec![if phase == NormalizedControlPhaseV1::Started {
                i16::MAX
            } else {
                0
            }],
        },
    })
}

fn mouse_motion_observation(
    platform_sample_tick: u64,
    device_id: u32,
    x_relative: f32,
    y_relative: f32,
) -> Option<lifecycle::DesktopObservation> {
    let quantized_value = [
        quantize_mouse_delta(x_relative)?,
        quantize_mouse_delta(y_relative)?,
    ];
    if quantized_value == [0, 0] {
        return None;
    }
    Some(lifecycle::DesktopObservation {
        source: lifecycle::DesktopEventSource::Mouse,
        platform_sample_tick,
        kind: lifecycle::DesktopObservationKind::Control {
            control_path: "nextengine.input.mouse.delta",
            device_class: "nextengine.input.mouse",
            device_instance_nonce: lifecycle::mouse_device_nonce(device_id),
            modifier_set: Vec::new(),
            phase: NormalizedControlPhaseV1::Changed,
            quantized_value: quantized_value.to_vec(),
        },
    })
}

fn quantize_mouse_delta(value: f32) -> Option<i16> {
    if !value.is_finite() {
        return None;
    }
    Some(value.round().clamp(-32_767.0, 32_767.0) as i16)
}

fn normalized_modifiers(modifiers: Mod) -> Vec<&'static str> {
    let mut normalized = Vec::new();
    for (mask, identifier) in [
        (
            Mod::LSHIFTMOD | Mod::RSHIFTMOD,
            "nextengine.input.modifier.shift",
        ),
        (
            Mod::LCTRLMOD | Mod::RCTRLMOD,
            "nextengine.input.modifier.control",
        ),
        (Mod::LALTMOD | Mod::RALTMOD, "nextengine.input.modifier.alt"),
        (Mod::LGUIMOD | Mod::RGUIMOD, "nextengine.input.modifier.gui"),
        (Mod::CAPSMOD, "nextengine.input.modifier.caps-lock"),
        (Mod::NUMMOD, "nextengine.input.modifier.num-lock"),
        (Mod::MODEMOD, "nextengine.input.modifier.mode"),
    ] {
        if modifiers.intersects(mask) {
            normalized.push(identifier);
        }
    }
    normalized
}

fn sdl_error(error: impl Display) -> DesktopAdapterError {
    DesktopAdapterError::Sdl(error.to_string())
}

#[cfg(test)]
mod tests;
