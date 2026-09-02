use std::sync::atomic::{AtomicU64, Ordering};

use super::*;
use sdl3::event::EventType;
use sdl3::{EventPump, EventSubsystem};

const PREPARED_RUN_INVALID_CODE: &str = "PERF_DESKTOP_PREPARED_RUN_INVALID";
static NEXT_PREPARATION_ID: AtomicU64 = AtomicU64::new(1);

/// Bounded wait for the initial compositor configure after a borderless
/// fullscreen start. Wayland/X11 compositors apply the fullscreen state
/// asynchronously; presentation must not create its swapchain until the
/// window settles at the declared extent or the frame plan would rebuild
/// mid-run when the surface changes under it.
const FULLSCREEN_START_STABILIZATION_TIMEOUT: Duration = Duration::from_millis(1_000);
const FULLSCREEN_START_POLL_INTERVAL: Duration = Duration::from_millis(10);
const FULLSCREEN_START_REQUIRED_STABLE_POLLS: u32 = 2;
/// SDL3 flattens every `SDL_EVENT_WINDOW_*` subtype into one contiguous type
/// range that ends below the keyboard block (`0x300`). Flushing this range
/// after stabilization drops only pre-run window noise (configure/resize/
/// expose transitions absorbed by the fullscreen switch), never input events.
const SDL_EVENT_TYPE_WINDOW_LAST: u32 = 0x2FF;

fn extent_matches_pixel_pair(extent: [u32; 2], width: u32, height: u32) -> bool {
    extent[0] == width && extent[1] == height
}

fn display_bounds_match_extent(window: &Window, requested_extent: [u32; 2]) -> bool {
    let Ok(display) = window.get_display() else {
        return false;
    };
    let Ok(bounds) = display.get_bounds() else {
        return false;
    };
    extent_matches_pixel_pair(requested_extent, bounds.width(), bounds.height())
}

fn enter_stable_borderless_fullscreen(
    window: &mut Window,
    events: &mut EventPump,
    event_subsystem: &EventSubsystem,
    requested_extent: [u32; 2],
) -> Result<(), DesktopAdapterError> {
    window.set_fullscreen(true).map_err(sdl_error)?;
    let deadline = Instant::now() + FULLSCREEN_START_STABILIZATION_TIMEOUT;
    let mut stable_polls = 0_u32;
    while Instant::now() < deadline {
        events.pump_events();
        let (width, height) = window.size_in_pixels();
        if extent_matches_pixel_pair(requested_extent, width, height) {
            stable_polls += 1;
            if stable_polls >= FULLSCREEN_START_REQUIRED_STABLE_POLLS {
                event_subsystem
                    .flush_events(EventType::WindowShown as u32, SDL_EVENT_TYPE_WINDOW_LAST);
                return Ok(());
            }
        } else {
            stable_polls = 0;
        }
        std::thread::sleep(FULLSCREEN_START_POLL_INTERVAL);
    }
    let (width, height) = window.size_in_pixels();
    Err(DesktopAdapterError::FullscreenStartExtentUnavailable {
        requested: requested_extent,
        observed: [width, height],
    })
}

/// Opaque owner for an initialized SDL/Vulkan interactive run.
///
/// Construction performs snapshot validation and all native adapter setup.
/// [`Self::run_measured`] executes only the bounded event/render loop, while
/// [`Self::finish`] waits for the device, collects the report, finalizes the
/// application hook, and then releases native resources.
pub struct PreparedDesktopRun {
    preparation_id: u64,
    core: InteractiveRunCore<fn() -> DesktopApplicationFinalization>,
}

/// Opaque proof that one prepared desktop loop completed successfully.
#[derive(Debug)]
pub struct DesktopRunMeasurement {
    preparation_id: u64,
    _private: (),
}

/// Initializes SDL, the native window, and Vulkan before a caller-controlled
/// measurement window begins.
pub fn prepare_interactive(
    snapshot: &PresentationSnapshotV3,
    render_content_catalog: &RenderContentCatalogV1,
    options: &DesktopRunOptions,
) -> Result<PreparedDesktopRun, DesktopAdapterError> {
    let preparation_id = next_preparation_id()?;
    let core = InteractiveRunCore::prepare(
        Arc::new(snapshot.clone()),
        render_content_catalog.clone(),
        options.clone(),
        complete_application_finalization as fn() -> DesktopApplicationFinalization,
        &mut |_, _, _| Ok(DesktopFramePublicationV1::default()),
    )?;
    Ok(PreparedDesktopRun {
        preparation_id,
        core,
    })
}

impl PreparedDesktopRun {
    /// Runs the prepared event/render loop exactly once.
    pub fn run_measured(&mut self) -> Result<DesktopRunMeasurement, DesktopAdapterError> {
        let mut frame_source =
            |_: &[PlatformEventV1], _: Duration, _: &mut audio_output::DesktopAudioOutputV1| {
                Ok(DesktopFramePublicationV1::default())
            };
        self.core.run_loop(&mut frame_source)?;
        Ok(DesktopRunMeasurement {
            preparation_id: self.preparation_id,
            _private: (),
        })
    }

    /// Finalizes a completed run outside the caller's measurement window.
    ///
    /// Passing the error returned by [`Self::run_measured`] preserves the
    /// original diagnostic while still finalizing before native teardown.
    pub fn finish(
        self,
        measurement: Result<DesktopRunMeasurement, DesktopAdapterError>,
    ) -> Result<DesktopRunReport, DesktopAdapterError> {
        let measurement = measurement.and_then(|measurement| {
            validate_measurement_provenance(
                self.preparation_id,
                self.core.run_started,
                &measurement,
            )
        });
        self.core.finish(measurement)
    }
}

pub(super) fn run_interactive_with_shared_timed_frame_source_and_finalize(
    snapshot: Arc<PresentationSnapshotV3>,
    render_content_catalog: &RenderContentCatalogV1,
    options: &DesktopRunOptions,
    mut frame_source: impl FnMut(
        &[PlatformEventV1],
        Duration,
    )
        -> Result<Option<Arc<PresentationSnapshotV3>>, DesktopAdapterError>,
    finalize_application: impl FnMut() -> DesktopApplicationFinalization,
) -> Result<DesktopRunReport, DesktopAdapterError> {
    run_interactive_with_shared_frame_publication_and_finalize(
        snapshot,
        render_content_catalog,
        options,
        |events, elapsed, _audio| {
            frame_source(events, elapsed).map(DesktopFramePublicationV1::snapshot_only)
        },
        finalize_application,
    )
}

/// Publication-aware variant: the frame source receives the bounded audio
/// sink owned by this adapter on every pump (SPEC-08 AUDIO-P1) and returns
/// the snapshot replacement plus declared dynamic surface updates.
#[allow(
    clippy::too_many_arguments,
    reason = "the publication entry point keeps the shared snapshot, surface and sink boundary explicit"
)]
pub(super) fn run_interactive_with_shared_frame_publication_and_finalize(
    snapshot: Arc<PresentationSnapshotV3>,
    render_content_catalog: &RenderContentCatalogV1,
    options: &DesktopRunOptions,
    mut frame_source: impl FnMut(
        &[PlatformEventV1],
        Duration,
        &mut audio_output::DesktopAudioOutputV1,
    ) -> Result<DesktopFramePublicationV1, DesktopAdapterError>,
    finalize_application: impl FnMut() -> DesktopApplicationFinalization,
) -> Result<DesktopRunReport, DesktopAdapterError> {
    let mut core = InteractiveRunCore::prepare(
        snapshot,
        render_content_catalog.clone(),
        options.clone(),
        finalize_application,
        &mut frame_source,
    )?;
    let measurement = core.run_loop(&mut frame_source);
    core.finish(measurement)
}

fn complete_application_finalization() -> DesktopApplicationFinalization {
    DesktopApplicationFinalization::Complete
}

fn next_preparation_id() -> Result<u64, DesktopAdapterError> {
    NEXT_PREPARATION_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| DesktopAdapterError::CounterOverflow)
}

fn validate_measurement_provenance(
    preparation_id: u64,
    run_started: bool,
    measurement: &DesktopRunMeasurement,
) -> Result<(), DesktopAdapterError> {
    if !run_started || measurement.preparation_id != preparation_id {
        return Err(invalid_prepared_run(
            "measurement did not originate from this completed preparation",
        ));
    }
    Ok(())
}

fn invalid_prepared_run(message: &'static str) -> DesktopAdapterError {
    DesktopAdapterError::client(PREPARED_RUN_INVALID_CODE, message)
}

struct InteractiveRunCore<F: FnMut() -> DesktopApplicationFinalization> {
    current_snapshot: RefCell<Arc<PresentationSnapshotV3>>,
    dynamic_surfaces: RefCell<DynamicSurfaceState>,
    render_content_catalog: RenderContentCatalogV1,
    options: DesktopRunOptions,
    normalizer: Option<lifecycle::DesktopEventNormalizer>,
    event_stats: Option<DesktopEventStats>,
    pacing_clock: RefCell<InteractivePacingClock>,
    completion: Option<InteractiveRunCompletion>,
    run_started: bool,
    // Field order is intentional: application finalization must run before
    // GraphicsContext and every SDL owner are released.
    finalizer: AdapterFinalizer<F>,
    graphics: Option<GraphicsContext>,
    audio: RefCell<audio_output::DesktopAudioOutputV1>,
    events: sdl3::EventPump,
    window: Window,
    _video: sdl3::VideoSubsystem,
    _sdl: sdl3::Sdl,
}

struct InteractiveRunCompletion {
    normalizer: lifecycle::DesktopEventNormalizer,
    event_stats: DesktopEventStats,
    rendered_frames: u64,
    rendered_objects: u64,
    indexed_draws: u64,
    fallback_material_draws: u64,
    last_frame_plan_hash: Option<ContentHash>,
    last_drawable_extent: Option<[u32; 2]>,
    last_target_revision: Option<u64>,
    close_requested: bool,
    device_recoveries: u64,
    software_paced_iterations: u64,
    software_pacing_sleep_microseconds: u64,
    dynamic_surface_uploads: u64,
    dynamic_surface_upload_bytes: u64,
    dynamic_surface_draws: u64,
    particle_surface_uploads: u64,
    particle_surface_upload_bytes: u64,
    particle_surface_frames: u64,
}

impl<F: FnMut() -> DesktopApplicationFinalization> InteractiveRunCore<F> {
    fn prepare(
        snapshot: Arc<PresentationSnapshotV3>,
        render_content_catalog: RenderContentCatalogV1,
        options: DesktopRunOptions,
        finalize_application: F,
        frame_source: &mut impl FnMut(
            &[PlatformEventV1],
            Duration,
            &mut audio_output::DesktopAudioOutputV1,
        ) -> Result<DesktopFramePublicationV1, DesktopAdapterError>,
    ) -> Result<Self, DesktopAdapterError> {
        // Locals are declared before the finalizer so every partial
        // initialization failure finalizes before reverse-order native drops.
        let sdl;
        let video;
        let mut window;
        let mut events;
        let graphics;
        let finalizer = AdapterFinalizer::new(finalize_application);

        snapshot.validate()?;
        let current_snapshot = RefCell::new(snapshot);
        let dynamic_surfaces = RefCell::new(DynamicSurfaceState::new(
            &options.dynamic_surfaces,
            &render_content_catalog,
            options.particle_surface,
        )?);
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
        if options.prefer_borderless_fullscreen_when_display_matches
            && display_bounds_match_extent(&window, options.initial_extent)
        {
            let event_subsystem = sdl.event().map_err(sdl_error)?;
            enter_stable_borderless_fullscreen(
                &mut window,
                &mut events,
                &event_subsystem,
                options.initial_extent,
            )?;
        }
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
            &render_content_catalog,
            &options,
        )?);
        let mut audio_output =
            audio_output::DesktopAudioOutputV1::open(&sdl, options.audio_output_enabled);
        if options.inject_audio_device_loss_after_open {
            audio_output.note_device_removed();
            audio_output.note_device_added();
        }
        let mut normalizer = lifecycle::DesktopEventNormalizer::new(options.host_instance_id)?;
        let mut event_stats = DesktopEventStats::default();
        {
            let mut resume_sink = |events: &[PlatformEventV1], elapsed: Duration| {
                apply_frame_source_result(
                    &current_snapshot,
                    &dynamic_surfaces,
                    frame_source,
                    events,
                    elapsed,
                    &mut audio_output,
                )
            };
            publish_fresh_host_resume_if_requested(
                options.resume_suspended_application,
                &mut normalizer,
                &mut resume_sink,
                &mut event_stats,
            )?;
        }

        Ok(Self {
            current_snapshot,
            dynamic_surfaces,
            render_content_catalog,
            options,
            normalizer: Some(normalizer),
            event_stats: Some(event_stats),
            pacing_clock: RefCell::new(InteractivePacingClock::default()),
            completion: None,
            run_started: false,
            finalizer,
            graphics,
            audio: RefCell::new(audio_output),
            events,
            window,
            _video: video,
            _sdl: sdl,
        })
    }

    fn run_loop(
        &mut self,
        frame_source: &mut impl FnMut(
            &[PlatformEventV1],
            Duration,
            &mut audio_output::DesktopAudioOutputV1,
        ) -> Result<DesktopFramePublicationV1, DesktopAdapterError>,
    ) -> Result<(), DesktopAdapterError> {
        if self.run_started {
            return Err(invalid_prepared_run(
                "prepared desktop loop was already started",
            ));
        }
        self.run_started = true;
        let mut normalizer = self
            .normalizer
            .take()
            .ok_or_else(|| invalid_prepared_run("prepared normalizer is missing"))?;
        let mut event_stats = self
            .event_stats
            .take()
            .ok_or_else(|| invalid_prepared_run("prepared event state is missing"))?;

        let completion = {
            let current_snapshot = &self.current_snapshot;
            let dynamic_surfaces = &self.dynamic_surfaces;
            let render_content_catalog = &self.render_content_catalog;
            let options = &self.options;
            let pacing_clock = &self.pacing_clock;
            let graphics = &mut self.graphics;
            let events = &mut self.events;
            let window = &mut self.window;
            let audio = &self.audio;
            let mut event_sink = |events: &[PlatformEventV1]| {
                let elapsed = pacing_clock.borrow_mut().elapsed_for_pump(Instant::now());
                apply_frame_source_result(
                    current_snapshot,
                    dynamic_surfaces,
                    frame_source,
                    events,
                    elapsed,
                    &mut audio.borrow_mut(),
                )
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
            let mut dynamic_surface_uploads = 0_u64;
            let mut dynamic_surface_upload_bytes = 0_u64;
            let mut dynamic_surface_draws = 0_u64;
            let mut particle_surface_uploads = 0_u64;
            let mut particle_surface_upload_bytes = 0_u64;
            let mut particle_surface_frames = 0_u64;

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
                        Event::AudioDeviceRemoved { .. } => {
                            audio.borrow_mut().note_device_removed();
                        }
                        Event::AudioDeviceAdded { .. } => {
                            audio.borrow_mut().note_device_added();
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
                                let width = u32::try_from(width)
                                    .map_err(|_| DesktopAdapterError::InvalidExtent)?;
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
                            WindowEvent::Hidden
                            | WindowEvent::Minimized
                            | WindowEvent::Occluded
                                if !rendering_suspended =>
                            {
                                rendering_suspended = true;
                                observations.push(suspend_observation(
                                    platform_sample_tick,
                                    "nextengine.platform.reason.window-minimized",
                                ));
                            }
                            WindowEvent::Exposed | WindowEvent::Restored | WindowEvent::Shown
                                if rendering_suspended =>
                            {
                                rendering_suspended = false;
                                swapchain_dirty = true;
                                observations.push(resume_observation(
                                    platform_sample_tick,
                                    "nextengine.platform.reason.window-restored",
                                ));
                            }
                            _ => {}
                        },
                        Event::AppWillEnterBackground { .. }
                        | Event::AppDidEnterBackground { .. } => {
                            if !rendering_suspended {
                                rendering_suspended = true;
                                observations.push(suspend_observation(
                                    platform_sample_tick,
                                    "nextengine.platform.reason.application-backgrounded",
                                ));
                            }
                        }
                        Event::AppWillEnterForeground { .. }
                        | Event::AppDidEnterForeground { .. } => {
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
                        .recreate_swapchain(window);
                    if let Err(error) = recreate {
                        if error.is_recoverable_presentation_loss() {
                            recover_graphics(
                                graphics,
                                window,
                                render_content_catalog,
                                options,
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

                if !injected_device_loss
                    && options.inject_device_loss_after_frames == Some(rendered_frames)
                {
                    recover_graphics(
                        graphics,
                        window,
                        render_content_catalog,
                        options,
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
                let current_dynamic_surfaces = dynamic_surfaces.borrow();
                let render_result = graphics
                    .as_mut()
                    .ok_or(DesktopAdapterError::GraphicsContextMissing)?
                    .render(
                        current_snapshot.as_ref(),
                        current_dynamic_surfaces.current(),
                        current_dynamic_surfaces.current_particles(),
                        window,
                        event_and_frame_source_update_microseconds,
                        rendered_frames,
                    );
                drop(current_dynamic_surfaces);
                drop(current_snapshot);
                let submitted = match render_result {
                    Ok(submitted) => submitted,
                    Err(error) if error.is_recoverable_presentation_loss() => {
                        recover_graphics(
                            graphics,
                            window,
                            render_content_catalog,
                            options,
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
                    dynamic_surface_uploads = dynamic_surface_uploads
                        .checked_add(submitted.dynamic_surface_uploads)
                        .ok_or(DesktopAdapterError::CounterOverflow)?;
                    dynamic_surface_upload_bytes = dynamic_surface_upload_bytes
                        .checked_add(submitted.dynamic_surface_upload_bytes)
                        .ok_or(DesktopAdapterError::CounterOverflow)?;
                    dynamic_surface_draws = submitted.dynamic_surface_draws;
                    particle_surface_uploads = particle_surface_uploads
                        .checked_add(submitted.particle_surface_uploads)
                        .ok_or(DesktopAdapterError::CounterOverflow)?;
                    particle_surface_upload_bytes = particle_surface_upload_bytes
                        .checked_add(submitted.particle_surface_upload_bytes)
                        .ok_or(DesktopAdapterError::CounterOverflow)?;
                    particle_surface_frames = particle_surface_frames
                        .checked_add(u64::from(submitted.particle_surface_recorded))
                        .ok_or(DesktopAdapterError::CounterOverflow)?;
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

            InteractiveRunCompletion {
                normalizer,
                event_stats,
                rendered_frames,
                rendered_objects,
                indexed_draws,
                fallback_material_draws,
                last_frame_plan_hash,
                last_drawable_extent,
                last_target_revision,
                close_requested,
                device_recoveries,
                software_paced_iterations,
                software_pacing_sleep_microseconds,
                dynamic_surface_uploads,
                dynamic_surface_upload_bytes,
                dynamic_surface_draws,
                particle_surface_uploads,
                particle_surface_upload_bytes,
                particle_surface_frames,
            }
        };
        self.completion = Some(completion);
        Ok(())
    }

    fn finish(
        mut self,
        measurement: Result<(), DesktopAdapterError>,
    ) -> Result<DesktopRunReport, DesktopAdapterError> {
        measurement?;
        let completion = self
            .completion
            .take()
            .ok_or_else(|| invalid_prepared_run("completed desktop state is missing"))?;
        let (
            frame_profiling,
            device_allocation_bytes,
            device_allocation_count,
            frame_plan_metrics,
            ui_overlay_counters,
            captured_frames,
            particle_surface_available,
        ) = {
            let graphics = self
                .graphics
                .as_mut()
                .ok_or(DesktopAdapterError::GraphicsContextMissing)?;
            graphics.wait_idle()?;
            let (bytes, allocations) = graphics.device_allocation_stats()?;
            let captured_frames = graphics.take_captured_frames()?;
            (
                graphics.take_frame_profiling(),
                bytes,
                allocations,
                graphics.frame_plan_metrics(),
                graphics.ui_overlay_counters(),
                captured_frames,
                graphics.particle_surface_available(),
            )
        };
        let report = DesktopRunReport {
            rendered_frames: completion.rendered_frames,
            rendered_objects: completion.rendered_objects,
            indexed_draws: completion.indexed_draws,
            fallback_material_draws: completion.fallback_material_draws,
            last_frame_plan_hash: completion.last_frame_plan_hash,
            last_drawable_extent: completion.last_drawable_extent,
            last_target_revision: completion.last_target_revision,
            normalized_events: completion.event_stats.normalized_events,
            control_events: completion.event_stats.control_events,
            lifecycle_events: completion.event_stats.lifecycle_events,
            resize_events: completion.event_stats.resize_events,
            focus_events: completion.event_stats.focus_events,
            fullscreen_events: completion.event_stats.fullscreen_events,
            device_loss_events: completion.event_stats.device_loss_events,
            device_recoveries: completion.device_recoveries,
            close_requested: completion.close_requested,
            api_version: vk::API_VERSION_1_3,
            b0_capabilities_verified: true,
            capability_set_hash: completion.normalizer.capability_set_hash(),
            timebase_hash: completion.normalizer.timebase_hash(),
            last_platform_event_id: completion.event_stats.last_platform_event_id,
            frame_timings: frame_profiling.samples,
            vulkan_timestamp_queries: frame_profiling.timestamp_query_count,
            dropped_frame_timing_samples: frame_profiling.dropped_samples,
            software_paced_iterations: completion.software_paced_iterations,
            software_pacing_sleep_microseconds: completion.software_pacing_sleep_microseconds,
            frame_plan_cache_hits: frame_plan_metrics.cache_hits,
            frame_plan_cache_misses: frame_plan_metrics.cache_misses,
            frame_plan_build_failures: frame_plan_metrics.build_failures,
            frame_plan_explicit_invalidations: frame_plan_metrics.explicit_invalidations,
            device_allocation_bytes,
            device_allocation_count,
            ui_overlay_frames: ui_overlay_counters.0,
            ui_overlay_updates: ui_overlay_counters.1,
            ui_overlay_failures: ui_overlay_counters.2,
            audio_queued_samples: self.audio.borrow().queued_samples(),
            audio_dropped_samples: self.audio.borrow().dropped_samples(),
            audio_callback_underruns: self.audio.borrow().callback_underruns(),
            audio_device_faults: self.audio.borrow().device_faults(),
            audio_device_reopens: self.audio.borrow().reopens(),
            audio_output_active: self.audio.borrow().output_active(),
            dynamic_surface_publications: self.dynamic_surfaces.borrow().publications(),
            dynamic_surface_uploads: completion.dynamic_surface_uploads,
            dynamic_surface_upload_bytes: completion.dynamic_surface_upload_bytes,
            dynamic_surface_draws: completion.dynamic_surface_draws,
            dynamic_surface_hashes: self.dynamic_surfaces.borrow().current_hashes(),
            captured_frames,
            particle_surface_available,
            particle_surface_publications: self.dynamic_surfaces.borrow().particle_publications(),
            particle_surface_uploads: completion.particle_surface_uploads,
            particle_surface_upload_bytes: completion.particle_surface_upload_bytes,
            particle_surface_frames: completion.particle_surface_frames,
        };
        self.finalizer.finish();
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DropMarker<'a>(&'a RefCell<Vec<&'static str>>);

    impl Drop for DropMarker<'_> {
        fn drop(&mut self) {
            self.0.borrow_mut().push("adapter-drop");
        }
    }

    #[test]
    fn prepared_measurement_provenance_fails_closed() {
        let measurement = DesktopRunMeasurement {
            preparation_id: 7,
            _private: (),
        };
        for (preparation_id, run_started) in [(8, true), (7, false)] {
            let error = validate_measurement_provenance(preparation_id, run_started, &measurement)
                .expect_err("invalid provenance");
            assert_eq!(error.diagnostic_code(), PREPARED_RUN_INVALID_CODE);
        }
        validate_measurement_provenance(7, true, &measurement)
            .expect("matching completed preparation");
    }

    #[test]
    fn prepared_owner_keeps_native_state_bounded_on_the_stack() {
        let owner_size =
            std::mem::size_of::<InteractiveRunCore<fn() -> DesktopApplicationFinalization>>();
        assert!(
            owner_size <= 64 * 1_024,
            "prepared owner is {owner_size} bytes"
        );
    }

    #[test]
    fn prepared_owner_field_order_finalizes_before_native_drop() {
        struct OrderedOwners<'a, F: FnMut() -> DesktopApplicationFinalization> {
            _finalizer: AdapterFinalizer<F>,
            _adapter: DropMarker<'a>,
        }

        let order = RefCell::new(Vec::new());
        {
            let _owners = OrderedOwners {
                _finalizer: AdapterFinalizer::new(|| {
                    order.borrow_mut().push("finalize");
                    DesktopApplicationFinalization::Complete
                }),
                _adapter: DropMarker(&order),
            };
        }
        assert_eq!(order.into_inner(), ["finalize", "adapter-drop"]);
    }

    #[test]
    fn borderless_fullscreen_start_is_opt_in() {
        assert!(!DesktopRunOptions::default().prefer_borderless_fullscreen_when_display_matches);
    }

    #[test]
    fn fullscreen_start_extent_mismatch_has_typed_code_and_message() {
        let error = DesktopAdapterError::FullscreenStartExtentUnavailable {
            requested: [1_920, 1_080],
            observed: [1_853, 1_011],
        };
        assert_eq!(
            error.diagnostic_code(),
            "PLATFORM_FULLSCREEN_START_EXTENT_UNAVAILABLE"
        );
        let message = error.to_string();
        assert!(message.contains("1853x1011") || message.contains("[1853, 1011]"));
        assert!(message.contains("1920"));
    }

    #[test]
    fn extent_pixel_pair_comparison_is_exact() {
        assert!(extent_matches_pixel_pair([1_280, 720], 1_280, 720));
        assert!(!extent_matches_pixel_pair([1_920, 1_080], 1_920, 1_011));
    }
}
