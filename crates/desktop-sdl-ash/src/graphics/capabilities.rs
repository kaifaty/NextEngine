use super::*;

pub(super) fn select_physical_device(
    instance: &ash::Instance,
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, u32), DesktopAdapterError> {
    // SAFETY: instance is live and enumeration writes owned handles.
    let physical_devices = unsafe { instance.enumerate_physical_devices() }
        .map_err(classify_physical_device_enumeration_error)?;
    if physical_devices.is_empty() {
        return Err(DesktopAdapterError::IcdUnavailable { error: None });
    }
    for physical_device in physical_devices {
        // SAFETY: physical device belongs to the instance.
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        if properties.api_version < vk::API_VERSION_1_3 {
            continue;
        }
        let mut features_12 = vk::PhysicalDeviceVulkan12Features::default();
        let mut features_13 = vk::PhysicalDeviceVulkan13Features::default();
        let mut features = vk::PhysicalDeviceFeatures2::default()
            .push_next(&mut features_12)
            .push_next(&mut features_13);
        // SAFETY: feature output chain is valid and stack-owned for the call.
        unsafe {
            instance.get_physical_device_features2(physical_device, &mut features);
        }
        if features_12.timeline_semaphore == 0
            || features_12.buffer_device_address == 0
            || features_13.dynamic_rendering == 0
            || features_13.synchronization2 == 0
        {
            continue;
        }
        // SAFETY: physical device belongs to this live instance and the
        // returned extension properties are copied into Rust-owned storage.
        let extensions =
            unsafe { instance.enumerate_device_extension_properties(physical_device) }?;
        if !supports_required_device_extension(&extensions, ash::khr::swapchain::NAME) {
            continue;
        }
        // SAFETY: physical device belongs to the instance.
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        for (index, family) in queue_families.iter().enumerate() {
            let index = u32::try_from(index).map_err(|_| DesktopAdapterError::CounterOverflow)?;
            // SAFETY: surface and physical device share the same instance.
            let present = unsafe {
                surface_loader.get_physical_device_surface_support(
                    physical_device,
                    index,
                    surface,
                )?
            };
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) && present {
                return Ok((physical_device, index));
            }
        }
    }
    Err(DesktopAdapterError::GpuUnsupported)
}
