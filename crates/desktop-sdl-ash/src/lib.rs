#![allow(
    unsafe_code,
    reason = "ADR-003 confines graphics API calls and native surface ownership to this crate"
)]

use std::error::Error;
use std::ffi::CString;
use std::fmt::{Display, Formatter};
use std::time::Duration;

use ash::vk;
use next_contracts::presentation::{
    PresentationPrimitiveV1, PresentationSnapshotV2, ScenePresentationRecordV2,
};
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::video::Window;

#[derive(Clone, Debug)]
pub struct DesktopRunOptions {
    pub title: String,
    pub initial_extent: [u32; 2],
    pub maximum_frames: Option<u64>,
}

impl Default for DesktopRunOptions {
    fn default() -> Self {
        Self {
            title: "Next Engine — Cooked Offline RPG Slice".to_owned(),
            initial_extent: [960, 540],
            maximum_frames: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesktopRunReport {
    pub rendered_frames: u64,
    pub resize_events: u64,
    pub focus_events: u64,
    pub close_requested: bool,
    pub api_version: u32,
    pub b0_capabilities_verified: bool,
}

pub fn run_interactive(
    snapshot: &PresentationSnapshotV2,
    options: &DesktopRunOptions,
) -> Result<DesktopRunReport, DesktopAdapterError> {
    snapshot.validate()?;
    if options.initial_extent[0] == 0 || options.initial_extent[1] == 0 {
        return Err(DesktopAdapterError::InvalidExtent);
    }
    let sdl = sdl3::init().map_err(sdl_error)?;
    let video = sdl.video().map_err(sdl_error)?;
    let window = video
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
    let mut graphics = GraphicsContext::new(&window)?;
    let mut rendered_frames = 0_u64;
    let mut resize_events = 0_u64;
    let mut focus_events = 0_u64;
    let mut close_requested = false;

    'application: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Window {
                    win_event: WindowEvent::CloseRequested,
                    ..
                } => {
                    close_requested = true;
                    break 'application;
                }
                Event::Window {
                    win_event: WindowEvent::Resized(_, _) | WindowEvent::PixelSizeChanged(_, _),
                    ..
                } => {
                    resize_events = resize_events
                        .checked_add(1)
                        .ok_or(DesktopAdapterError::CounterOverflow)?;
                    graphics.recreate_swapchain(&window)?;
                }
                Event::Window {
                    win_event: WindowEvent::FocusGained | WindowEvent::FocusLost,
                    ..
                } => {
                    focus_events = focus_events
                        .checked_add(1)
                        .ok_or(DesktopAdapterError::CounterOverflow)?;
                }
                _ => {}
            }
        }
        if graphics.render(snapshot, &window)? {
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
    graphics.wait_idle()?;
    Ok(DesktopRunReport {
        rendered_frames,
        resize_events,
        focus_events,
        close_requested,
        api_version: vk::API_VERSION_1_3,
        b0_capabilities_verified: true,
    })
}

mod graphics;

use graphics::GraphicsContext;

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
    Sdl(String),
    Loader(String),
    Graphics(vk::Result),
    InvalidName,
    InvalidExtent,
    GpuUnsupported,
    CounterOverflow,
}

impl Display for DesktopAdapterError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Presentation(error) => write!(formatter, "{error}"),
            Self::Sdl(error) => write!(formatter, "desktop host failed: {error}"),
            Self::Loader(error) => write!(formatter, "graphics loader failed: {error}"),
            Self::Graphics(error) => write!(formatter, "graphics API failed: {error:?}"),
            Self::InvalidName => formatter.write_str("desktop application name invalid"),
            Self::InvalidExtent => formatter.write_str("desktop surface extent invalid"),
            Self::GpuUnsupported => {
                formatter.write_str("GPU_UNSUPPORTED: required B0 capabilities unavailable")
            }
            Self::CounterOverflow => formatter.write_str("desktop adapter counter overflow"),
        }
    }
}

impl Error for DesktopAdapterError {}

impl From<next_contracts::presentation::PresentationContractError> for DesktopAdapterError {
    fn from(error: next_contracts::presentation::PresentationContractError) -> Self {
        Self::Presentation(error)
    }
}

impl From<vk::Result> for DesktopAdapterError {
    fn from(error: vk::Result) -> Self {
        Self::Graphics(error)
    }
}
