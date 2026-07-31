use super::*;

pub(super) const B0_TARGET_REVISION: u64 = 1;
pub(super) const B0_SURFACE_COLOR_SPACE: vk::ColorSpaceKHR = vk::ColorSpaceKHR::SRGB_NONLINEAR;
pub(super) const B0_SURFACE_FORMATS: [vk::Format; 2] =
    [vk::Format::B8G8R8A8_SRGB, vk::Format::R8G8B8A8_SRGB];
pub(super) const B0_DEPTH_FORMATS: [vk::Format; 2] =
    [vk::Format::D32_SFLOAT, vk::Format::D16_UNORM];
pub(super) const B0_PRESENT_MODE: vk::PresentModeKHR = vk::PresentModeKHR::FIFO;

pub(super) fn validate_loader_api_version(actual: u32) -> Result<(), DesktopAdapterError> {
    if actual < vk::API_VERSION_1_3 {
        return Err(DesktopAdapterError::LoaderVersionUnsupported {
            required: vk::API_VERSION_1_3,
            actual,
        });
    }
    Ok(())
}

pub(super) fn classify_instance_creation_error(error: vk::Result) -> DesktopAdapterError {
    if matches!(
        error,
        vk::Result::ERROR_INCOMPATIBLE_DRIVER | vk::Result::ERROR_INITIALIZATION_FAILED
    ) {
        DesktopAdapterError::IcdUnavailable { error: Some(error) }
    } else {
        DesktopAdapterError::Graphics(error)
    }
}

pub(super) fn classify_physical_device_enumeration_error(error: vk::Result) -> DesktopAdapterError {
    if error == vk::Result::ERROR_INITIALIZATION_FAILED {
        DesktopAdapterError::IcdUnavailable { error: Some(error) }
    } else {
        DesktopAdapterError::Graphics(error)
    }
}

pub(super) fn classify_device_creation_error(error: vk::Result) -> DesktopAdapterError {
    match error {
        vk::Result::ERROR_FEATURE_NOT_PRESENT | vk::Result::ERROR_EXTENSION_NOT_PRESENT => {
            DesktopAdapterError::GpuUnsupported
        }
        vk::Result::ERROR_INCOMPATIBLE_DRIVER | vk::Result::ERROR_INITIALIZATION_FAILED => {
            DesktopAdapterError::IcdUnavailable { error: Some(error) }
        }
        _ => DesktopAdapterError::Graphics(error),
    }
}

pub(super) fn supports_required_device_extension(
    properties: &[vk::ExtensionProperties],
    required: &std::ffi::CStr,
) -> bool {
    properties
        .iter()
        .any(|property| extension_property_matches(property, required))
}

pub(super) fn extension_property_matches(
    property: &vk::ExtensionProperties,
    required: &std::ffi::CStr,
) -> bool {
    let required = required.to_bytes_with_nul();
    property
        .extension_name
        .get(..required.len())
        .is_some_and(|candidate| {
            candidate
                .iter()
                .map(|byte| *byte as u8)
                .eq(required.iter().copied())
        })
}

pub(super) fn select_composite_alpha(
    supported: vk::CompositeAlphaFlagsKHR,
) -> Option<vk::CompositeAlphaFlagsKHR> {
    [
        vk::CompositeAlphaFlagsKHR::OPAQUE,
        vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED,
        vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED,
        vk::CompositeAlphaFlagsKHR::INHERIT,
    ]
    .into_iter()
    .find(|candidate| supported.contains(*candidate))
}

pub(super) fn select_b0_surface_format(
    formats: &[vk::SurfaceFormatKHR],
) -> Option<vk::SurfaceFormatKHR> {
    if formats.len() == 1
        && formats[0].format == vk::Format::UNDEFINED
        && formats[0].color_space == B0_SURFACE_COLOR_SPACE
    {
        return Some(vk::SurfaceFormatKHR {
            format: B0_SURFACE_FORMATS[0],
            color_space: formats[0].color_space,
        });
    }
    B0_SURFACE_FORMATS.iter().find_map(|required_format| {
        formats.iter().copied().find(|candidate| {
            candidate.format == *required_format && candidate.color_space == B0_SURFACE_COLOR_SPACE
        })
    })
}

pub(super) fn select_b0_depth_format(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> Option<vk::Format> {
    select_b0_depth_format_with(|format| {
        // SAFETY: the physical-device handle belongs to this live instance and
        // the query returns format capability value data only.
        let properties =
            unsafe { instance.get_physical_device_format_properties(physical_device, format) };
        properties
            .optimal_tiling_features
            .contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT)
    })
}

pub(super) fn select_b0_depth_format_with(
    mut supports_depth_attachment: impl FnMut(vk::Format) -> bool,
) -> Option<vk::Format> {
    B0_DEPTH_FORMATS
        .into_iter()
        .find(|format| supports_depth_attachment(*format))
}

pub(super) fn defer_out_of_date<T>(
    result: Result<T, vk::Result>,
) -> Result<Option<T>, DesktopAdapterError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn next_frame_slot(current: usize, frame_slot_count: usize) -> Option<usize> {
    (frame_slot_count > 0).then(|| (current + 1) % frame_slot_count)
}

pub(super) fn image_fence_to_wait(
    image_fence: vk::Fence,
    current_frame_fence: vk::Fence,
) -> Option<vk::Fence> {
    (image_fence != vk::Fence::null() && image_fence != current_frame_fence).then_some(image_fence)
}

pub(super) fn retire_completed_image_fence_mappings(
    images_in_flight: &mut [vk::Fence],
    completed_fence: vk::Fence,
) -> usize {
    if completed_fence == vk::Fence::null() {
        return 0;
    }
    let mut retired = 0;
    for image_fence in images_in_flight {
        if *image_fence == completed_fence {
            *image_fence = vk::Fence::null();
            retired += 1;
        }
    }
    retired
}

pub(super) fn elapsed_microseconds(started: Option<Instant>) -> Result<u64, DesktopAdapterError> {
    started.map_or(Ok(0), |started| {
        u64::try_from(started.elapsed().as_micros())
            .map_err(|_| DesktopAdapterError::CounterOverflow)
    })
}

pub(super) fn select_surface_extent(
    capabilities: &vk::SurfaceCapabilitiesKHR,
    window_extent: [u32; 2],
) -> Option<vk::Extent2D> {
    if window_extent[0] == 0 || window_extent[1] == 0 {
        return None;
    }
    let extent = if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        vk::Extent2D {
            width: window_extent[0].clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: window_extent[1].clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    };
    (extent.width > 0 && extent.height > 0).then_some(extent)
}
