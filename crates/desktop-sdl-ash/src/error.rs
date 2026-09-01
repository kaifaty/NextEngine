use std::error::Error;
use std::fmt::{Display, Formatter};

use ash::vk;

#[derive(Debug)]
#[non_exhaustive]
pub enum DesktopAdapterError {
    Client {
        code: &'static str,
        message: String,
    },
    Presentation(next_contracts::presentation::PresentationContractError),
    Render(next_render::RenderDeviceError),
    RenderContent(String),
    Platform(next_contracts::platform::PlatformContractError),
    Identifier(next_contracts::ids::IdentifierError),
    Sdl(String),
    Loader(String),
    LoaderVersionUnsupported {
        required: u32,
        actual: u32,
    },
    IcdUnavailable {
        error: Option<vk::Result>,
    },
    Graphics(vk::Result),
    InvalidName,
    InvalidExtent,
    GpuUnsupported,
    GpuTimestampsUnsupported,
    GpuProfilingSampleCapacityExceeded {
        maximum: u32,
    },
    GpuTimestampStateInvalid,
    CounterOverflow,
    EventBatchLimitExceeded,
    EventLoopIterationLimitExceeded {
        maximum: u64,
    },
    SnapshotTransitionInvalid,
    TimebaseRegression {
        previous: u64,
        actual: u64,
    },
    GraphicsContextMissing,
    DeviceRecoveryLimitExceeded {
        maximum: u16,
    },
    FullscreenStartExtentUnavailable {
        requested: [u32; 2],
        observed: [u32; 2],
    },
    DynamicSurfaceInvalid {
        reason: &'static str,
    },
    DynamicSurfaceUndeclared,
    DynamicSurfaceCapacityExceeded {
        requested: u32,
        limit: u32,
    },
    DynamicSurfaceSequenceRegressed {
        previous: u64,
        actual: u64,
    },
}

impl DesktopAdapterError {
    #[must_use]
    pub fn client(code: &'static str, message: impl Into<String>) -> Self {
        Self::Client {
            code,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::Client { code, .. } => code,
            Self::Presentation(_) => "PRESENTATION_SNAPSHOT_INVALID",
            Self::Render(_) => "PRESENTATION_RENDER_PLAN_INVALID",
            Self::RenderContent(_) => "PRESENTATION_RENDER_CONTENT_FAILED",
            Self::Platform(_) | Self::Identifier(_) => "PLATFORM_EVENT_SCHEMA_INVALID",
            Self::Sdl(_) => "PLATFORM_DESKTOP_RUNTIME_UNAVAILABLE",
            Self::Loader(_) => "PLATFORM_GRAPHICS_LOADER_UNAVAILABLE",
            Self::LoaderVersionUnsupported { .. } => "PLATFORM_GRAPHICS_LOADER_VERSION_UNSUPPORTED",
            Self::IcdUnavailable { .. } => "PLATFORM_GRAPHICS_ICD_UNAVAILABLE",
            Self::Graphics(vk::Result::ERROR_DEVICE_LOST) => "PRESENTATION_DEVICE_LOST",
            Self::Graphics(vk::Result::ERROR_SURFACE_LOST_KHR) => "PRESENTATION_SURFACE_LOST",
            Self::Graphics(_) => "PRESENTATION_GRAPHICS_FAILED",
            Self::InvalidName => "PLATFORM_NATIVE_NAME_INVALID",
            Self::InvalidExtent => "PLATFORM_DRAWABLE_EXTENT_INVALID",
            Self::GpuUnsupported => "GPU_UNSUPPORTED",
            Self::GpuTimestampsUnsupported => "PERF_GPU_TIMESTAMPS_UNSUPPORTED",
            Self::GpuProfilingSampleCapacityExceeded { .. } => "PERF_GPU_SAMPLE_CAPACITY_EXCEEDED",
            Self::GpuTimestampStateInvalid => "PERF_GPU_TIMESTAMP_STATE_INVALID",
            Self::CounterOverflow => "PLATFORM_COUNTER_OVERFLOW",
            Self::EventBatchLimitExceeded => "PLATFORM_EVENT_BATCH_LIMIT_EXCEEDED",
            Self::EventLoopIterationLimitExceeded { .. } => {
                "PLATFORM_EVENT_LOOP_ITERATION_LIMIT_EXCEEDED"
            }
            Self::SnapshotTransitionInvalid => "PRESENTATION_SNAPSHOT_TRANSITION_INVALID",
            Self::TimebaseRegression { .. } => "PLATFORM_TIMEBASE_INVALID",
            Self::GraphicsContextMissing => "PRESENTATION_GRAPHICS_CONTEXT_MISSING",
            Self::DeviceRecoveryLimitExceeded { .. } => "PRESENTATION_DEVICE_RECOVERY_EXHAUSTED",
            Self::FullscreenStartExtentUnavailable { .. } => {
                "PLATFORM_FULLSCREEN_START_EXTENT_UNAVAILABLE"
            }
            Self::DynamicSurfaceInvalid { .. } => "PRESENTATION_DYNAMIC_SURFACE_INVALID",
            Self::DynamicSurfaceUndeclared => "PRESENTATION_DYNAMIC_SURFACE_UNDECLARED",
            Self::DynamicSurfaceCapacityExceeded { .. } => {
                "PRESENTATION_DYNAMIC_SURFACE_CAPACITY_EXCEEDED"
            }
            Self::DynamicSurfaceSequenceRegressed { .. } => {
                "PRESENTATION_DYNAMIC_SURFACE_SEQUENCE_INVALID"
            }
        }
    }

    pub(super) const fn is_recoverable_presentation_loss(&self) -> bool {
        matches!(
            self,
            Self::Graphics(vk::Result::ERROR_DEVICE_LOST | vk::Result::ERROR_SURFACE_LOST_KHR)
        )
    }

    pub(super) const fn presentation_loss_reason(&self) -> &'static str {
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
            Self::Client { code, message } => write!(formatter, "{code}: {message}"),
            Self::Presentation(error) => write!(formatter, "{error}"),
            Self::Render(error) => write!(formatter, "B0 frame planning failed: {error}"),
            Self::RenderContent(error) => {
                write!(formatter, "B0 GPU content failed: {error}")
            }
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
            Self::LoaderVersionUnsupported { required, actual } => write!(
                formatter,
                "PLATFORM_GRAPHICS_LOADER_VERSION_UNSUPPORTED: required API version {required:#010x}, got {actual:#010x}"
            ),
            Self::IcdUnavailable { error: Some(error) } => write!(
                formatter,
                "PLATFORM_GRAPHICS_ICD_UNAVAILABLE: graphics driver initialization failed: {error:?}"
            ),
            Self::IcdUnavailable { error: None } => formatter.write_str(
                "PLATFORM_GRAPHICS_ICD_UNAVAILABLE: no graphics devices were enumerated",
            ),
            Self::Graphics(error) => write!(formatter, "graphics API failed: {error:?}"),
            Self::InvalidName => formatter.write_str("desktop application name invalid"),
            Self::InvalidExtent => formatter.write_str("desktop surface extent invalid"),
            Self::GpuUnsupported => {
                formatter.write_str("GPU_UNSUPPORTED: required B0 capabilities unavailable")
            }
            Self::GpuTimestampsUnsupported => formatter.write_str(
                "PERF_GPU_TIMESTAMPS_UNSUPPORTED: selected graphics queue has no usable timestamp support",
            ),
            Self::GpuProfilingSampleCapacityExceeded { maximum } => write!(
                formatter,
                "PERF_GPU_SAMPLE_CAPACITY_EXCEEDED: maximum samples {maximum}"
            ),
            Self::GpuTimestampStateInvalid => formatter.write_str(
                "PERF_GPU_TIMESTAMP_STATE_INVALID: a submitted frame was not collected before query reuse",
            ),
            Self::CounterOverflow => formatter.write_str("desktop adapter counter overflow"),
            Self::EventBatchLimitExceeded => {
                formatter.write_str("platform event batch limit exceeded")
            }
            Self::EventLoopIterationLimitExceeded { maximum } => write!(
                formatter,
                "PLATFORM_EVENT_LOOP_ITERATION_LIMIT_EXCEEDED: maximum iterations {maximum}"
            ),
            Self::SnapshotTransitionInvalid => formatter.write_str(
                "presentation snapshot regressed or crossed an unvalidated publication boundary",
            ),
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
            Self::FullscreenStartExtentUnavailable { requested, observed } => write!(
                formatter,
                "PLATFORM_FULLSCREEN_START_EXTENT_UNAVAILABLE: borderless fullscreen start settled at {:?} instead of the declared extent {:?}",
                observed, requested
            ),
            Self::DynamicSurfaceInvalid { reason } => write!(
                formatter,
                "PRESENTATION_DYNAMIC_SURFACE_INVALID: {reason}"
            ),
            Self::DynamicSurfaceUndeclared => formatter.write_str(
                "PRESENTATION_DYNAMIC_SURFACE_UNDECLARED: mesh revision is not a declared dynamic surface in the exact catalog",
            ),
            Self::DynamicSurfaceCapacityExceeded { requested, limit } => write!(
                formatter,
                "PRESENTATION_DYNAMIC_SURFACE_CAPACITY_EXCEEDED: requested {requested} exceeds declared capacity {limit}"
            ),
            Self::DynamicSurfaceSequenceRegressed { previous, actual } => write!(
                formatter,
                "PRESENTATION_DYNAMIC_SURFACE_SEQUENCE_INVALID: previous {previous}, got {actual}"
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

impl From<next_render::RenderDeviceError> for DesktopAdapterError {
    fn from(error: next_render::RenderDeviceError) -> Self {
        Self::Render(error)
    }
}

impl From<crate::gpu_content::B0GpuContentError> for DesktopAdapterError {
    fn from(error: crate::gpu_content::B0GpuContentError) -> Self {
        match error {
            crate::gpu_content::B0GpuContentError::Graphics(error) => Self::Graphics(error),
            error => Self::RenderContent(error.to_string()),
        }
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
