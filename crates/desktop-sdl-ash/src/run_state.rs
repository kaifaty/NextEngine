use super::*;

pub(super) const INTERACTIVE_FRAME_INTERVAL: Duration = Duration::from_nanos(16_666_667);
const APPLICATION_FINALIZATION_RETRY_INTERVAL: Duration = Duration::from_millis(10);
pub const MAX_FRAME_PROFILING_SAMPLES: u32 = 65_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopApplicationFinalization {
    Complete,
    Retry,
}

pub(super) struct AdapterFinalizer<F: FnMut() -> DesktopApplicationFinalization> {
    callback: Option<F>,
}

impl<F: FnMut() -> DesktopApplicationFinalization> AdapterFinalizer<F> {
    pub(super) fn new(callback: F) -> Self {
        Self {
            callback: Some(callback),
        }
    }

    pub(super) fn finish(&mut self) {
        while let Some(callback) = self.callback.as_mut() {
            if callback() == DesktopApplicationFinalization::Complete {
                self.callback = None;
            } else {
                std::thread::sleep(APPLICATION_FINALIZATION_RETRY_INTERVAL);
            }
        }
    }
}

impl<F: FnMut() -> DesktopApplicationFinalization> Drop for AdapterFinalizer<F> {
    fn drop(&mut self) {
        self.finish();
    }
}

#[derive(Clone, Debug)]
pub struct DesktopRunOptions {
    pub title: String,
    pub initial_extent: [u32; 2],
    pub maximum_frames: Option<u64>,
    pub maximum_event_loop_iterations: Option<u64>,
    pub maximum_device_recoveries: u16,
    pub inject_device_loss_after_frames: Option<u64>,
    /// Test/acceptance probe: after the first real audio stream open, exercise
    /// the same bounded unavailable/reopen path used by native device events.
    pub inject_audio_device_loss_after_open: bool,
    pub inject_startup_lifecycle_probe: bool,
    pub host_instance_id: PersistentId,
    pub resume_suspended_application: bool,
    /// Cooked text catalogs for the optional semantic UI overlay. The default
    /// empty set disables the overlay entirely (no GPU objects, no draws).
    pub ui_text_catalogs: Vec<TextCatalogV1>,
    /// Requested text locale for the overlay; ignored when catalogs are empty.
    pub ui_locale: String,
    /// Local `PresentationOnly` text scale in milli (bounded 500..=2000 by the
    /// preference contract); the overlay rasterizer clamps defensively.
    pub ui_text_scale_milli: u32,
    /// Zero disables CPU/GPU frame timing. A non-zero value enables a bounded
    /// Vulkan timestamp buffer in the same release binary.
    pub frame_profiling_sample_capacity: u32,
    /// Baseline audio device output (A4): opens the SDL playback stream with
    /// bounded unavailable/silent fallback. Disable for audio-free runs.
    pub audio_output_enabled: bool,
    /// Local `PresentationOnly` subtitle visibility (A5): when false, the
    /// overlay drops `Subtitle`-role semantic elements. Voice-absent subtitle
    /// fallback stays enabled by default (SPEC-08/SPEC-18).
    pub ui_subtitles_enabled: bool,
    /// Presentation-start stabilization for prepared runs: when enabled and
    /// the declared extent equals the target display bounds, the window starts
    /// borderless fullscreen and the adapter absorbs the initial compositor
    /// configure before swapchain creation, so presentation begins from one
    /// stable declared extent instead of rebuilding mid-run. The final pixel
    /// extent is still enforced strictly; a compositor that cannot deliver it
    /// fails closed with a typed error. Extents are compared against display
    /// bounds, so hosts with a fractional logical/pixel scale mismatch fail
    /// closed rather than presenting at an undeclared size.
    pub prefer_borderless_fullscreen_when_display_matches: bool,
    /// Presentation-only dynamic surfaces declared for the whole run. Each
    /// entry names one exact catalog mesh revision and a fixed vertex/index
    /// capacity; the adapter allocates one host-visible ring per frame slot
    /// once and never rebuilds the render-content catalog to refresh it.
    pub dynamic_surfaces: Vec<DynamicSurfaceProfileV1>,
    /// ADR-102: at most one presentation-only particle surface.
    pub particle_surface: Option<crate::particle_surface::ParticleSurfaceProfileV1>,
    /// Bounded developer capture of a short burst of rendered frames (SPEC-04
    /// diagnostics only). The swapchain is created with transfer-source usage
    /// when set; a surface without that usage fails closed before the first
    /// frame.
    pub frame_capture: Option<DesktopFrameCaptureRequestV1>,
    /// Developer-scripted input pushed into SDL's event queue by run time
    /// (sorted by time on use); empty for ordinary runs.
    pub scripted_input: Vec<DesktopScriptedInputV1>,
}

/// A developer-scripted key for [`DesktopScriptedActionV1`] (the movement
/// keys of the reference game).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopScriptedKeyV1 {
    W,
    A,
    S,
    D,
}

/// One scripted input action, injected through SDL's own event queue like
/// the startup lifecycle probe, so it flows through the ordinary
/// normalization path with the same identities as real input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopScriptedActionV1 {
    KeyDown(DesktopScriptedKeyV1),
    KeyUp(DesktopScriptedKeyV1),
    /// Relative mouse motion in pixels (camera orbit).
    MouseMotion {
        x_relative: i32,
        y_relative: i32,
    },
}

/// A scripted action at a run time in milliseconds since the first
/// pumped frame (developer diagnostic; never part of a root).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopScriptedInputV1 {
    pub at_milliseconds: u64,
    pub action: DesktopScriptedActionV1,
}

/// Which rendered frames to copy back to host memory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopFrameCaptureRequestV1 {
    /// Zero-based index among successfully submitted frames of the first
    /// captured frame.
    pub rendered_frame_index: u64,
    /// Consecutive rendered frames to capture starting at that index;
    /// clamped to `1..=MAX_FRAME_CAPTURE_BURST`.
    pub frame_count: u32,
}

/// Upper bound on consecutive captured frames per run (host memory bound).
pub const MAX_FRAME_CAPTURE_BURST: u32 = 8;

impl DesktopFrameCaptureRequestV1 {
    #[must_use]
    pub const fn burst_length(&self) -> u32 {
        if self.frame_count == 0 {
            1
        } else if self.frame_count > MAX_FRAME_CAPTURE_BURST {
            MAX_FRAME_CAPTURE_BURST
        } else {
            self.frame_count
        }
    }

    #[must_use]
    pub const fn covers(&self, rendered_frame_index: u64) -> bool {
        rendered_frame_index >= self.rendered_frame_index
            && rendered_frame_index
                < self
                    .rendered_frame_index
                    .saturating_add(self.burst_length() as u64)
    }
}

/// One rendered frame in tightly packed sRGB-encoded RGBA8, top row first.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesktopCapturedFrameV1 {
    pub rendered_frame_index: u64,
    pub extent: [u32; 2],
    pub rgba8: Vec<u8>,
}

impl Default for DesktopRunOptions {
    fn default() -> Self {
        Self {
            title: "Next Engine — Cooked Offline RPG Slice".to_owned(),
            initial_extent: [960, 540],
            maximum_frames: None,
            maximum_event_loop_iterations: None,
            maximum_device_recoveries: 2,
            inject_device_loss_after_frames: None,
            inject_audio_device_loss_after_open: false,
            inject_startup_lifecycle_probe: false,
            host_instance_id: PersistentId::from_bytes([0x64; 16]),
            resume_suspended_application: false,
            ui_text_catalogs: Vec::new(),
            ui_locale: "en".to_owned(),
            ui_text_scale_milli:
                next_contracts::preferences::PLAYER_PREFERENCE_TEXT_SCALE_MILLI_DEFAULT,
            frame_profiling_sample_capacity: 0,
            audio_output_enabled: true,
            ui_subtitles_enabled: true,
            prefer_borderless_fullscreen_when_display_matches: false,
            dynamic_surfaces: Vec::new(),
            frame_capture: None,
            particle_surface: None,
            scripted_input: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopFrameTimingSample {
    /// ADR-036 critical-path CPU span: frame-plan extraction through queue
    /// submission. FIFO/acquire and presentation waits are intentionally
    /// reported separately.
    pub cpu_extract_and_submit_microseconds: u64,
    pub gpu_duration_microseconds: u64,
    pub event_and_frame_source_update_microseconds: u64,
    pub frame_slot_wait_microseconds: u64,
    pub image_acquire_wait_microseconds: u64,
    pub swapchain_image_wait_microseconds: u64,
    pub frame_plan_microseconds: u64,
    /// Host-visible dynamic surface ring refresh for this frame slot,
    /// including the per-surface hash check that skips an unchanged ring.
    pub dynamic_surface_upload_microseconds: u64,
    /// Declared surfaces whose ring was actually rewritten in this frame.
    pub dynamic_surface_uploads: u64,
    pub command_record_microseconds: u64,
    pub queue_submit_microseconds: u64,
    pub present_wait_microseconds: u64,
    /// GPU time of the ADR-102 particle surface pass (zero when not recorded).
    pub particle_surface_gpu_microseconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesktopRunReport {
    pub rendered_frames: u64,
    pub rendered_objects: u64,
    pub indexed_draws: u64,
    pub fallback_material_draws: u64,
    pub last_frame_plan_hash: Option<ContentHash>,
    pub last_drawable_extent: Option<[u32; 2]>,
    pub last_target_revision: Option<u64>,
    pub normalized_events: u64,
    pub control_events: u64,
    pub lifecycle_events: u64,
    pub resize_events: u64,
    pub focus_events: u64,
    pub fullscreen_events: u64,
    pub device_loss_events: u64,
    pub device_recoveries: u64,
    pub close_requested: bool,
    pub api_version: u32,
    pub b0_capabilities_verified: bool,
    pub capability_set_hash: ContentHash,
    pub timebase_hash: ContentHash,
    pub last_platform_event_id: Option<ContentHash>,
    pub frame_timings: Vec<DesktopFrameTimingSample>,
    pub vulkan_timestamp_queries: u64,
    pub dropped_frame_timing_samples: u64,
    /// Deferred/suspended iterations paced in software. Successfully submitted
    /// FIFO frames are paced only by Vulkan acquire/present backpressure.
    pub software_paced_iterations: u64,
    pub software_pacing_sleep_microseconds: u64,
    pub frame_plan_cache_hits: u64,
    pub frame_plan_cache_misses: u64,
    pub frame_plan_build_failures: u64,
    pub frame_plan_explicit_invalidations: u64,
    /// Engine-owned, currently bound Vulkan memory. This is a conservative
    /// residency ceiling and excludes presentation-engine swapchain storage.
    pub device_allocation_bytes: u64,
    pub device_allocation_count: u64,
    /// Frames in which the optional semantic UI overlay was composited.
    pub ui_overlay_frames: u64,
    /// Successful overlay texture uploads (one per accepted content change).
    pub ui_overlay_updates: u64,
    /// Bounded overlay failures absorbed without failing the frame (SPEC-18).
    pub ui_overlay_failures: u64,
    /// Canonical PCM samples accepted by the audio sink (A4).
    pub audio_queued_samples: u64,
    /// Oldest samples dropped past the bounded audio ring.
    pub audio_dropped_samples: u64,
    /// Callback windows that ran out of queued audio (silence emitted).
    pub audio_callback_underruns: u64,
    /// Audio device loss/open-failure facts (typed, presentation-only).
    pub audio_device_faults: u64,
    /// Successful audio stream (re)opens.
    pub audio_device_reopens: u64,
    /// Whether a live audio stream existed at report time.
    pub audio_output_active: bool,
    /// Frame-source publications accepted for declared dynamic surfaces.
    pub dynamic_surface_publications: u64,
    /// Host-visible ring refreshes performed across all frame slots.
    pub dynamic_surface_uploads: u64,
    /// Bytes copied into dynamic surface rings across the whole run.
    pub dynamic_surface_upload_bytes: u64,
    /// Draws in the last submitted frame that consumed a dynamic ring instead
    /// of the immutable catalog geometry.
    pub dynamic_surface_draws: u64,
    /// Canonical hash of the current update per declared surface at exit.
    pub dynamic_surface_hashes: Vec<(AssetRevisionRefV1, ContentHash)>,
    /// The requested developer captures in rendered-frame order; a frame is
    /// present only when it was submitted and its copy completed.
    pub captured_frames: Vec<DesktopCapturedFrameV1>,
    /// ADR-102: whether the declared particle surface pass was constructed.
    pub particle_surface_available: bool,
    /// Frame-source publications accepted for the particle surface.
    pub particle_surface_publications: u64,
    /// Particle buffer refreshes across all frame slots.
    pub particle_surface_uploads: u64,
    /// Bytes copied into particle buffers across the whole run.
    pub particle_surface_upload_bytes: u64,
    /// Frames in which the particle surface pass was recorded.
    pub particle_surface_frames: u64,
}

#[derive(Debug, Default)]
pub(super) struct InteractivePacingClock {
    prior_pump_time: Option<Instant>,
    first_frame_submitted: bool,
}

impl InteractivePacingClock {
    pub(super) fn elapsed_for_pump(&mut self, now: Instant) -> Duration {
        let elapsed = if self.first_frame_submitted {
            self.prior_pump_time
                .map_or(Duration::ZERO, |prior| now.duration_since(prior))
        } else {
            Duration::ZERO
        };
        self.prior_pump_time = Some(now);
        elapsed
    }

    pub(super) fn observe_frame_submission(&mut self, now: Instant) {
        if !self.first_frame_submitted {
            self.first_frame_submitted = true;
            self.prior_pump_time = Some(now);
        }
    }
}

pub(super) fn remaining_frame_budget(elapsed: Duration) -> Duration {
    INTERACTIVE_FRAME_INTERVAL.saturating_sub(elapsed)
}

pub(super) fn software_pacing_delay(elapsed: Duration, frame_was_submitted: bool) -> Duration {
    if frame_was_submitted {
        Duration::ZERO
    } else {
        remaining_frame_budget(elapsed)
    }
}

pub(super) fn apply_software_pacing(
    elapsed: Duration,
    frame_was_submitted: bool,
    paced_iterations: &mut u64,
    pacing_sleep_microseconds: &mut u64,
) -> Result<(), DesktopAdapterError> {
    let delay = software_pacing_delay(elapsed, frame_was_submitted);
    if delay.is_zero() {
        return Ok(());
    }
    std::thread::sleep(delay);
    *paced_iterations = paced_iterations
        .checked_add(1)
        .ok_or(DesktopAdapterError::CounterOverflow)?;
    *pacing_sleep_microseconds = pacing_sleep_microseconds
        .checked_add(
            u64::try_from(delay.as_micros()).map_err(|_| DesktopAdapterError::CounterOverflow)?,
        )
        .ok_or(DesktopAdapterError::CounterOverflow)?;
    Ok(())
}
