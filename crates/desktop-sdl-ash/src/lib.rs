#![allow(
    unsafe_code,
    reason = "ADR-003 confines graphics API calls and native surface ownership to this crate"
)]

use std::error::Error;
use std::ffi::CString;
use std::fmt::{Display, Formatter};
use std::time::Duration;

use ash::vk;
use next_contracts::ids::{ContentHash, PersistentId};
use next_contracts::platform::{NormalizedControlPhaseV1, PlatformEventKindV1, PlatformEventV1};
use next_contracts::presentation::{
    PresentationPrimitiveV1, PresentationSnapshotV2, ScenePresentationRecordV2,
};
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::{Mod, Scancode};
use sdl3::video::Window;

#[derive(Clone, Debug)]
pub struct DesktopRunOptions {
    pub title: String,
    pub initial_extent: [u32; 2],
    pub maximum_frames: Option<u64>,
    pub maximum_device_recoveries: u16,
    pub inject_device_loss_after_frames: Option<u64>,
    pub inject_startup_lifecycle_probe: bool,
    pub host_instance_id: PersistentId,
}

impl Default for DesktopRunOptions {
    fn default() -> Self {
        Self {
            title: "Next Engine — Cooked Offline RPG Slice".to_owned(),
            initial_extent: [960, 540],
            maximum_frames: None,
            maximum_device_recoveries: 2,
            inject_device_loss_after_frames: None,
            inject_startup_lifecycle_probe: false,
            host_instance_id: PersistentId::from_bytes([0x64; 16]),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesktopRunReport {
    pub rendered_frames: u64,
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
}

pub fn run_interactive(
    snapshot: &PresentationSnapshotV2,
    options: &DesktopRunOptions,
) -> Result<DesktopRunReport, DesktopAdapterError> {
    run_interactive_with_event_sink(snapshot, options, |_| Ok(()))
}

pub fn run_interactive_with_event_sink(
    snapshot: &PresentationSnapshotV2,
    options: &DesktopRunOptions,
    mut event_sink: impl FnMut(&[PlatformEventV1]) -> Result<(), DesktopAdapterError>,
) -> Result<DesktopRunReport, DesktopAdapterError> {
    snapshot.validate()?;
    if options.initial_extent[0] == 0 || options.initial_extent[1] == 0 {
        return Err(DesktopAdapterError::InvalidExtent);
    }
    let sdl = sdl3::init().map_err(sdl_error)?;
    let video = sdl.video().map_err(sdl_error)?;
    let mut window = video
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
    let mut events = sdl.event_pump().map_err(sdl_error)?;
    if options.inject_startup_lifecycle_probe {
        native_events::inject_startup_lifecycle_probe(
            &sdl.event().map_err(sdl_error)?,
            window.id(),
            options.initial_extent,
        )?;
    }
    let mut graphics = Some(GraphicsContext::new(&window)?);
    let mut normalizer = lifecycle::DesktopEventNormalizer::new(options.host_instance_id)?;
    let mut event_stats = DesktopEventStats::default();
    let mut rendered_frames = 0_u64;
    let mut close_requested = false;
    let mut rendering_suspended = false;
    let mut fullscreen = false;
    let mut device_recoveries = 0_u64;
    let mut injected_device_loss = false;

    'application: loop {
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
                        swapchain_dirty = true;
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
                _ => {}
            }
        }

        publish_observations(
            &mut normalizer,
            observations,
            &mut event_sink,
            &mut event_stats,
        )?;

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
            std::thread::sleep(Duration::from_millis(16));
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

        let render_result = graphics
            .as_mut()
            .ok_or(DesktopAdapterError::GraphicsContextMissing)?
            .render(snapshot, &window);
        let rendered = match render_result {
            Ok(rendered) => rendered,
            Err(error) if error.is_recoverable_presentation_loss() => {
                recover_graphics(
                    &mut graphics,
                    &window,
                    &mut normalizer,
                    &mut event_sink,
                    &mut event_stats,
                    &mut device_recoveries,
                    options.maximum_device_recoveries,
                    event_timestamp_fallback(rendered_frames),
                    error.presentation_loss_reason(),
                )?;
                false
            }
            Err(error) => return Err(error),
        };
        if rendered {
            rendered_frames = rendered_frames
                .checked_add(1)
                .ok_or(DesktopAdapterError::CounterOverflow)?;
        }
        if options
            .maximum_frames
            .is_some_and(|maximum| rendered_frames >= maximum)
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(16));
    }
    graphics
        .as_ref()
        .ok_or(DesktopAdapterError::GraphicsContextMissing)?
        .wait_idle()?;
    Ok(DesktopRunReport {
        rendered_frames,
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
    })
}

mod graphics;
mod lifecycle;
mod native_events;

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
) -> Result<(), DesktopAdapterError> {
    let events = normalizer.normalize(observations)?;
    if events.is_empty() {
        return Ok(());
    }
    event_sink(&events)?;
    event_stats.observe(&events)
}

#[allow(
    clippy::too_many_arguments,
    reason = "the private recovery boundary keeps the bounded policy, event publication and graphics ownership explicit"
)]
fn recover_graphics(
    graphics: &mut Option<GraphicsContext>,
    window: &Window,
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
    publish_observations(
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
    *graphics = Some(GraphicsContext::new(window)?);
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

fn primitive_color(primitive: PresentationPrimitiveV1) -> vk::ClearValue {
    let color = match primitive {
        PresentationPrimitiveV1::Floor => [0.10, 0.12, 0.16, 1.0],
        PresentationPrimitiveV1::Capsule => [0.22, 0.66, 1.0, 1.0],
        PresentationPrimitiveV1::Switch => [1.0, 0.38, 0.18, 1.0],
        PresentationPrimitiveV1::Item => [1.0, 0.82, 0.22, 1.0],
        PresentationPrimitiveV1::Character => [0.68, 0.34, 0.92, 1.0],
    };
    vk::ClearValue {
        color: vk::ClearColorValue { float32: color },
    }
}

fn record_rectangle(record: &ScenePresentationRecordV2, extent: vk::Extent2D) -> vk::ClearRect {
    let [x, _, z] = record.current_transform.translation_micrometres;
    let width_i64 = i64::from(extent.width);
    let height_i64 = i64::from(extent.height);
    let center_x =
        ((x.saturating_add(1_000_000)).clamp(0, 2_000_000) * width_i64 / 2_000_000) as i32;
    let center_y =
        ((z.saturating_add(1_000_000)).clamp(0, 2_000_000) * height_i64 / 2_000_000) as i32;
    let [width, height] = match record.primitive {
        PresentationPrimitiveV1::Floor => [extent.width, extent.height],
        PresentationPrimitiveV1::Capsule => [28, 52],
        PresentationPrimitiveV1::Switch => [34, 34],
        PresentationPrimitiveV1::Item => [18, 18],
        PresentationPrimitiveV1::Character => [30, 54],
    };
    let max_x = i32::try_from(extent.width.saturating_sub(width)).unwrap_or(i32::MAX);
    let max_y = i32::try_from(extent.height.saturating_sub(height)).unwrap_or(i32::MAX);
    let offset_x = center_x
        .saturating_sub(i32::try_from(width / 2).unwrap_or(0))
        .clamp(0, max_x);
    let offset_y = center_y
        .saturating_sub(i32::try_from(height / 2).unwrap_or(0))
        .clamp(0, max_y);
    vk::ClearRect {
        rect: vk::Rect2D {
            offset: vk::Offset2D {
                x: offset_x,
                y: offset_y,
            },
            extent: vk::Extent2D { width, height },
        },
        base_array_layer: 0,
        layer_count: 1,
    }
}

fn sdl_error(error: impl Display) -> DesktopAdapterError {
    DesktopAdapterError::Sdl(error.to_string())
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DesktopAdapterError {
    Presentation(next_contracts::presentation::PresentationContractError),
    Platform(next_contracts::platform::PlatformContractError),
    Identifier(next_contracts::ids::IdentifierError),
    Sdl(String),
    Loader(String),
    Graphics(vk::Result),
    InvalidName,
    InvalidExtent,
    GpuUnsupported,
    CounterOverflow,
    EventBatchLimitExceeded,
    TimebaseRegression { previous: u64, actual: u64 },
    GraphicsContextMissing,
    DeviceRecoveryLimitExceeded { maximum: u16 },
}

impl DesktopAdapterError {
    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::Presentation(_) => "PRESENTATION_SNAPSHOT_INVALID",
            Self::Platform(_) | Self::Identifier(_) => "PLATFORM_EVENT_SCHEMA_INVALID",
            Self::Sdl(_) => "PLATFORM_DESKTOP_RUNTIME_UNAVAILABLE",
            Self::Loader(_) => "PLATFORM_GRAPHICS_LOADER_UNAVAILABLE",
            Self::Graphics(vk::Result::ERROR_DEVICE_LOST) => "PRESENTATION_DEVICE_LOST",
            Self::Graphics(vk::Result::ERROR_SURFACE_LOST_KHR) => "PRESENTATION_SURFACE_LOST",
            Self::Graphics(_) => "PRESENTATION_GRAPHICS_FAILED",
            Self::InvalidName => "PLATFORM_NATIVE_NAME_INVALID",
            Self::InvalidExtent => "PLATFORM_DRAWABLE_EXTENT_INVALID",
            Self::GpuUnsupported => "GPU_UNSUPPORTED",
            Self::CounterOverflow => "PLATFORM_COUNTER_OVERFLOW",
            Self::EventBatchLimitExceeded => "PLATFORM_EVENT_BATCH_LIMIT_EXCEEDED",
            Self::TimebaseRegression { .. } => "PLATFORM_TIMEBASE_INVALID",
            Self::GraphicsContextMissing => "PRESENTATION_GRAPHICS_CONTEXT_MISSING",
            Self::DeviceRecoveryLimitExceeded { .. } => "PRESENTATION_DEVICE_RECOVERY_EXHAUSTED",
        }
    }

    const fn is_recoverable_presentation_loss(&self) -> bool {
        matches!(
            self,
            Self::Graphics(vk::Result::ERROR_DEVICE_LOST | vk::Result::ERROR_SURFACE_LOST_KHR)
        )
    }

    const fn presentation_loss_reason(&self) -> &'static str {
        match self {
            Self::Graphics(vk::Result::ERROR_SURFACE_LOST_KHR) => {
                "nextengine.platform.reason.surface-lost"
            }
            _ => "nextengine.platform.reason.device-lost",
        }
    }
}

impl Display for DesktopAdapterError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Presentation(error) => write!(formatter, "{error}"),
            Self::Platform(error) => write!(formatter, "{error}"),
            Self::Identifier(error) => write!(formatter, "{error}"),
            Self::Sdl(error) => write!(
                formatter,
                "PLATFORM_DESKTOP_RUNTIME_UNAVAILABLE: desktop host failed: {error}"
            ),
            Self::Loader(error) => write!(
                formatter,
                "PLATFORM_GRAPHICS_LOADER_UNAVAILABLE: graphics loader failed: {error}"
            ),
            Self::Graphics(error) => write!(formatter, "graphics API failed: {error:?}"),
            Self::InvalidName => formatter.write_str("desktop application name invalid"),
            Self::InvalidExtent => formatter.write_str("desktop surface extent invalid"),
            Self::GpuUnsupported => {
                formatter.write_str("GPU_UNSUPPORTED: required B0 capabilities unavailable")
            }
            Self::CounterOverflow => formatter.write_str("desktop adapter counter overflow"),
            Self::EventBatchLimitExceeded => {
                formatter.write_str("platform event batch limit exceeded")
            }
            Self::TimebaseRegression { previous, actual } => write!(
                formatter,
                "platform timebase regressed: previous {previous}, got {actual}"
            ),
            Self::GraphicsContextMissing => {
                formatter.write_str("presentation graphics context missing")
            }
            Self::DeviceRecoveryLimitExceeded { maximum } => write!(
                formatter,
                "PRESENTATION_DEVICE_RECOVERY_EXHAUSTED: maximum recoveries {maximum}"
            ),
        }
    }
}

impl Error for DesktopAdapterError {}

impl From<next_contracts::presentation::PresentationContractError> for DesktopAdapterError {
    fn from(error: next_contracts::presentation::PresentationContractError) -> Self {
        Self::Presentation(error)
    }
}

impl From<next_contracts::platform::PlatformContractError> for DesktopAdapterError {
    fn from(error: next_contracts::platform::PlatformContractError) -> Self {
        Self::Platform(error)
    }
}

impl From<next_contracts::ids::IdentifierError> for DesktopAdapterError {
    fn from(error: next_contracts::ids::IdentifierError) -> Self {
        Self::Identifier(error)
    }
}

impl From<vk::Result> for DesktopAdapterError {
    fn from(error: vk::Result) -> Self {
        Self::Graphics(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_controls_use_engine_owned_paths_and_modifiers() {
        assert_eq!(
            keyboard_control_path(Scancode::W),
            Some("nextengine.input.keyboard.w")
        );
        assert_eq!(keyboard_control_path(Scancode::F1), None);
        assert_eq!(
            normalized_modifiers(Mod::LSHIFTMOD | Mod::RCTRLMOD),
            vec![
                "nextengine.input.modifier.shift",
                "nextengine.input.modifier.control"
            ]
        );
    }

    #[test]
    fn fullscreen_shortcuts_are_shell_requests_not_close_requests() {
        assert!(is_fullscreen_shortcut(Some(Scancode::F11), Mod::NOMOD));
        assert!(is_fullscreen_shortcut(Some(Scancode::Return), Mod::LALTMOD));
        assert!(!is_fullscreen_shortcut(Some(Scancode::Escape), Mod::NOMOD));
    }

    #[test]
    fn graphics_loss_has_stable_recovery_diagnostics() {
        let device = DesktopAdapterError::Graphics(vk::Result::ERROR_DEVICE_LOST);
        let surface = DesktopAdapterError::Graphics(vk::Result::ERROR_SURFACE_LOST_KHR);
        assert!(device.is_recoverable_presentation_loss());
        assert!(surface.is_recoverable_presentation_loss());
        assert_eq!(device.diagnostic_code(), "PRESENTATION_DEVICE_LOST");
        assert_eq!(surface.diagnostic_code(), "PRESENTATION_SURFACE_LOST");

        let exhausted = DesktopAdapterError::DeviceRecoveryLimitExceeded { maximum: 2 };
        assert_eq!(
            exhausted.diagnostic_code(),
            "PRESENTATION_DEVICE_RECOVERY_EXHAUSTED"
        );
    }
}
