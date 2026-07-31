use super::*;
use ash::vk::Handle;

fn extension_property(name: &std::ffi::CStr) -> vk::ExtensionProperties {
    let mut property = vk::ExtensionProperties::default();
    for (target, source) in property
        .extension_name
        .iter_mut()
        .zip(name.to_bytes_with_nul())
    {
        *target = i8::try_from(*source).expect("extension names are ASCII");
    }
    property
}

#[test]
fn loader_version_requires_vulkan_1_3() {
    validate_loader_api_version(vk::API_VERSION_1_3).expect("Vulkan 1.3 loader");
    let error =
        validate_loader_api_version(vk::API_VERSION_1_2).expect_err("Vulkan 1.2 is below B0");
    assert_eq!(
        error.diagnostic_code(),
        "PLATFORM_GRAPHICS_LOADER_VERSION_UNSUPPORTED"
    );
}

#[test]
fn startup_error_classifiers_separate_icd_from_gpu_capability_failures() {
    assert_eq!(
        classify_instance_creation_error(vk::Result::ERROR_INCOMPATIBLE_DRIVER).diagnostic_code(),
        "PLATFORM_GRAPHICS_ICD_UNAVAILABLE"
    );
    assert_eq!(
        classify_physical_device_enumeration_error(vk::Result::ERROR_INITIALIZATION_FAILED)
            .diagnostic_code(),
        "PLATFORM_GRAPHICS_ICD_UNAVAILABLE"
    );
    assert_eq!(
        classify_device_creation_error(vk::Result::ERROR_FEATURE_NOT_PRESENT).diagnostic_code(),
        "GPU_UNSUPPORTED"
    );
    assert_eq!(
        classify_device_creation_error(vk::Result::ERROR_EXTENSION_NOT_PRESENT).diagnostic_code(),
        "GPU_UNSUPPORTED"
    );
    assert_eq!(
        classify_instance_creation_error(vk::Result::ERROR_OUT_OF_HOST_MEMORY).diagnostic_code(),
        "PRESENTATION_GRAPHICS_FAILED"
    );
}

#[test]
fn required_swapchain_extension_is_matched_by_exact_name() {
    let swapchain = extension_property(ash::khr::swapchain::NAME);
    let near_match = std::ffi::CString::new("VK_KHR_swapchain_extra").expect("extension name");
    let properties = [swapchain, extension_property(&near_match)];

    assert!(supports_required_device_extension(
        &properties,
        ash::khr::swapchain::NAME
    ));
    assert!(!supports_required_device_extension(
        &properties[1..],
        ash::khr::swapchain::NAME
    ));
    let missing = std::ffi::CString::new("VK_EXT_missing").expect("extension name");
    assert!(!supports_required_device_extension(&properties, &missing));
}

#[test]
fn composite_alpha_uses_fixed_preference_order() {
    let supported =
        vk::CompositeAlphaFlagsKHR::INHERIT | vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED;
    assert!(select_composite_alpha(supported) == Some(vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED));
    let supported =
        vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED | vk::CompositeAlphaFlagsKHR::OPAQUE;
    assert!(select_composite_alpha(supported) == Some(vk::CompositeAlphaFlagsKHR::OPAQUE));
    assert!(select_composite_alpha(vk::CompositeAlphaFlagsKHR::empty()).is_none());
}

#[test]
fn b0_surface_format_requires_srgb_storage_and_fails_closed() {
    let bgra_srgb = vk::SurfaceFormatKHR {
        format: vk::Format::B8G8R8A8_SRGB,
        color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
    };
    let rgba_srgb = vk::SurfaceFormatKHR {
        format: vk::Format::R8G8B8A8_SRGB,
        color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
    };
    let bgra_unorm = vk::SurfaceFormatKHR {
        format: vk::Format::B8G8R8A8_UNORM,
        color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
    };
    assert!(select_b0_surface_format(&[rgba_srgb, bgra_srgb, bgra_unorm]) == Some(bgra_srgb));
    assert!(select_b0_surface_format(&[rgba_srgb]) == Some(rgba_srgb));
    assert!(select_b0_surface_format(&[bgra_unorm]).is_none());
    assert!(
        select_b0_surface_format(&[vk::SurfaceFormatKHR {
            format: vk::Format::UNDEFINED,
            color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
        }]) == Some(bgra_srgb)
    );
    assert!(
        select_b0_surface_format(&[vk::SurfaceFormatKHR {
            format: vk::Format::UNDEFINED,
            color_space: vk::ColorSpaceKHR::DISPLAY_P3_NONLINEAR_EXT,
        }])
        .is_none()
    );
}

#[test]
fn b0_depth_format_uses_fixed_non_stencil_preference_and_fails_closed() {
    let selected = select_b0_depth_format_with(|format| {
        format == vk::Format::D32_SFLOAT || format == vk::Format::D16_UNORM
    });
    assert!(selected == Some(vk::Format::D32_SFLOAT));

    let fallback = select_b0_depth_format_with(|format| format == vk::Format::D16_UNORM);
    assert!(fallback == Some(vk::Format::D16_UNORM));
    assert!(select_b0_depth_format_with(|_| false).is_none());
}

#[test]
fn zero_extent_and_out_of_date_defer_presentation() {
    let variable_capabilities = vk::SurfaceCapabilitiesKHR {
        current_extent: vk::Extent2D {
            width: u32::MAX,
            height: u32::MAX,
        },
        min_image_extent: vk::Extent2D {
            width: 1,
            height: 1,
        },
        max_image_extent: vk::Extent2D {
            width: 1_920,
            height: 1_080,
        },
        ..vk::SurfaceCapabilitiesKHR::default()
    };
    assert!(select_surface_extent(&variable_capabilities, [0, 720]).is_none());
    let selected = select_surface_extent(&variable_capabilities, [1_280, 720])
        .expect("positive variable extent");
    assert_eq!([selected.width, selected.height], [1_280, 720]);
    let fixed_zero_capabilities = vk::SurfaceCapabilitiesKHR {
        current_extent: vk::Extent2D {
            width: 0,
            height: 0,
        },
        ..variable_capabilities
    };
    assert!(select_surface_extent(&fixed_zero_capabilities, [1_280, 720]).is_none());

    let deferred = defer_out_of_date::<bool>(Err(vk::Result::ERROR_OUT_OF_DATE_KHR))
        .expect("out-of-date is a deferred presentation result");
    assert_eq!(deferred, None);
    let device_loss = defer_out_of_date::<bool>(Err(vk::Result::ERROR_DEVICE_LOST))
        .expect_err("device loss remains a recoverable presentation error");
    assert_eq!(device_loss.diagnostic_code(), "PRESENTATION_DEVICE_LOST");
}

#[test]
fn frame_slot_ring_is_two_deep_and_wraps_without_aliasing_adjacent_frames() {
    assert_eq!(FRAME_SLOT_COUNT, 2);
    assert_eq!(next_frame_slot(0, FRAME_SLOT_COUNT), Some(1));
    assert_eq!(next_frame_slot(1, FRAME_SLOT_COUNT), Some(0));
    assert_eq!(next_frame_slot(0, 0), None);
}

#[test]
fn acquired_image_waits_only_for_a_different_live_frame_slot() {
    let current = vk::Fence::from_raw(11);
    let prior = vk::Fence::from_raw(22);
    assert_eq!(image_fence_to_wait(vk::Fence::null(), current), None);
    assert_eq!(image_fence_to_wait(current, current), None);
    assert_eq!(image_fence_to_wait(prior, current), Some(prior));
}

#[test]
fn completed_slot_fence_aliases_are_retired_before_fence_reuse() {
    let completed = vk::Fence::from_raw(11);
    let still_pending = vk::Fence::from_raw(22);
    let mut images_in_flight = [completed, still_pending, completed, vk::Fence::null()];

    assert_eq!(
        retire_completed_image_fence_mappings(&mut images_in_flight, completed),
        2
    );
    assert_eq!(
        images_in_flight,
        [
            vk::Fence::null(),
            still_pending,
            vk::Fence::null(),
            vk::Fence::null(),
        ]
    );
    assert_eq!(
        retire_completed_image_fence_mappings(&mut images_in_flight, vk::Fence::null()),
        0,
        "a null sentinel is not a submitted fence and must not retire empty slots"
    );
}

#[test]
fn b0_swapchain_uses_fifo_as_its_only_presentation_pacing_source() {
    assert!(B0_PRESENT_MODE == vk::PresentModeKHR::FIFO);
}
