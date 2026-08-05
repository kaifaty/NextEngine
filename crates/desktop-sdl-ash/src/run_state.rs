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
            inject_startup_lifecycle_probe: false,
            host_instance_id: PersistentId::from_bytes([0x64; 16]),
            resume_suspended_application: false,
            ui_text_catalogs: Vec::new(),
            ui_locale: "en".to_owned(),
            ui_text_scale_milli:
                next_contracts::preferences::PLAYER_PREFERENCE_TEXT_SCALE_MILLI_DEFAULT,
            frame_profiling_sample_capacity: 0,
            audio_output_enabled: true,
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
    pub command_record_microseconds: u64,
    pub queue_submit_microseconds: u64,
    pub present_wait_microseconds: u64,
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
