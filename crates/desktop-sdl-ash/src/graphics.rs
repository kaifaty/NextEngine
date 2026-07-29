use super::*;

pub(super) struct GraphicsContext {
    _entry: ash::Entry,
    instance: ash::Instance,
    surface_loader: ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    queue: vk::Queue,
    queue_family_index: u32,
    swapchain_loader: ash::khr::swapchain::Device,
    swapchain: SwapchainState,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    image_available: vk::Semaphore,
    render_finished: vk::Semaphore,
    frame_fence: vk::Fence,
}

struct SwapchainState {
    handle: vk::SwapchainKHR,
    extent: vk::Extent2D,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    initialized: Vec<bool>,
}

impl GraphicsContext {
    pub(super) fn new(window: &Window) -> Result<Self, DesktopAdapterError> {
        // SAFETY: loading the process graphics loader creates an owned entry;
        // all child objects are destroyed in reverse ownership order below.
        let entry = unsafe { ash::Entry::load() }
            .map_err(|error| DesktopAdapterError::Loader(error.to_string()))?;
        let application_name =
            CString::new("Next Engine").map_err(|_| DesktopAdapterError::InvalidName)?;
        let application_info = vk::ApplicationInfo::default()
            .application_name(&application_name)
            .application_version(1)
            .engine_name(&application_name)
            .engine_version(1)
            .api_version(vk::API_VERSION_1_3);
        let extension_names = window.vulkan_instance_extensions().map_err(sdl_error)?;
        let extension_names = extension_names
            .into_iter()
            .map(|name| CString::new(name).map_err(|_| DesktopAdapterError::InvalidName))
            .collect::<Result<Vec<_>, _>>()?;
        let extension_pointers = extension_names
            .iter()
            .map(|name| name.as_ptr())
            .collect::<Vec<_>>();
        let instance_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(&extension_pointers);
        // SAFETY: all pointer arrays in `instance_info` live for the call and
        // no custom allocator is retained.
        let instance = unsafe { entry.create_instance(&instance_info, None) }?;
        // SAFETY: the SDL window was created with its graphics-surface flag and
        // remains alive until after this context has been dropped.
        let surface =
            unsafe { window.vulkan_create_surface(instance.handle()) }.map_err(sdl_error)?;
        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);
        let (physical_device, queue_family_index) =
            select_physical_device(&instance, &surface_loader, surface)?;

        let priorities = [1.0_f32];
        let queue_info = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities)];
        let device_extensions = [ash::khr::swapchain::NAME.as_ptr()];
        let mut features_12 = vk::PhysicalDeviceVulkan12Features::default()
            .timeline_semaphore(true)
            .buffer_device_address(true);
        let mut features_13 = vk::PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(true)
            .synchronization2(true);
        let device_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_info)
            .enabled_extension_names(&device_extensions)
            .push_next(&mut features_12)
            .push_next(&mut features_13);
        // SAFETY: the selected device and queue family were queried from this
        // instance and the feature chain contains only stack-owned call data.
        let device = unsafe { instance.create_device(physical_device, &device_info, None) }?;
        // SAFETY: queue zero exists because one priority was requested.
        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };
        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);
        let swapchain = create_swapchain(
            window,
            physical_device,
            queue_family_index,
            &surface_loader,
            surface,
            &device,
            &swapchain_loader,
            vk::SwapchainKHR::null(),
        )?;
        let command_pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        // SAFETY: queue family belongs to the selected logical device.
        let command_pool = unsafe { device.create_command_pool(&command_pool_info, None) }?;
        let command_buffer_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        // SAFETY: command pool is live and owned by this device.
        let command_buffer = unsafe { device.allocate_command_buffers(&command_buffer_info) }?[0];
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        // SAFETY: device is live and synchronization objects use no callbacks.
        let image_available = unsafe { device.create_semaphore(&semaphore_info, None) }?;
        // SAFETY: same ownership conditions as `image_available`.
        let render_finished = unsafe { device.create_semaphore(&semaphore_info, None) }?;
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        // SAFETY: device is live and fence uses no callbacks.
        let frame_fence = unsafe { device.create_fence(&fence_info, None) }?;
        Ok(Self {
            _entry: entry,
            instance,
            surface_loader,
            surface,
            physical_device,
            device,
            queue,
            queue_family_index,
            swapchain_loader,
            swapchain,
            command_pool,
            command_buffer,
            image_available,
            render_finished,
            frame_fence,
        })
    }

    pub(super) fn render(
        &mut self,
        snapshot: &PresentationSnapshotV2,
        window: &Window,
    ) -> Result<bool, DesktopAdapterError> {
        // SAFETY: the fence belongs to this device and guards the one reusable
        // command buffer and frame synchronization set.
        unsafe {
            self.device
                .wait_for_fences(&[self.frame_fence], true, u64::MAX)?;
        }
        // SAFETY: swapchain and semaphore are live; no fence is needed for
        // acquisition because the frame fence guards prior use.
        let acquired = unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain.handle,
                u64::MAX,
                self.image_available,
                vk::Fence::null(),
            )
        };
        let (image_index, acquisition_suboptimal) = match acquired {
            Ok(value) => value,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.recreate_swapchain(window)?;
                return Ok(false);
            }
            Err(error) => return Err(error.into()),
        };
        // SAFETY: the frame fence has completed and the command buffer is not
        // pending. Reset operations target objects owned by this context.
        unsafe {
            self.device.reset_fences(&[self.frame_fence])?;
            self.device
                .reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty())?;
            self.device.begin_command_buffer(
                self.command_buffer,
                &vk::CommandBufferBeginInfo::default(),
            )?;
        }
        let image_usize =
            usize::try_from(image_index).map_err(|_| DesktopAdapterError::CounterOverflow)?;
        let old_layout = if self.swapchain.initialized[image_usize] {
            vk::ImageLayout::PRESENT_SRC_KHR
        } else {
            vk::ImageLayout::UNDEFINED
        };
        let subresource = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let to_color = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(if old_layout == vk::ImageLayout::UNDEFINED {
                vk::PipelineStageFlags2::NONE
            } else {
                vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT
            })
            .src_access_mask(vk::AccessFlags2::NONE)
            .dst_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
            .old_layout(old_layout)
            .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .image(self.swapchain.images[image_usize])
            .subresource_range(subresource)];
        let to_color_dependency = vk::DependencyInfo::default().image_memory_barriers(&to_color);
        // SAFETY: the image belongs to the acquired swapchain index and the
        // barrier is recorded into the reset primary command buffer.
        unsafe {
            self.device
                .cmd_pipeline_barrier2(self.command_buffer, &to_color_dependency);
        }
        let clear = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.035, 0.045, 0.065, 1.0],
            },
        };
        let color_attachments = [vk::RenderingAttachmentInfo::default()
            .image_view(self.swapchain.image_views[image_usize])
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(clear)];
        let render_area = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain.extent,
        };
        let rendering_info = vk::RenderingInfo::default()
            .render_area(render_area)
            .layer_count(1)
            .color_attachments(&color_attachments);
        // SAFETY: dynamic rendering was checked and enabled; the attachment
        // image is in color-attachment layout for the acquired index.
        unsafe {
            self.device
                .cmd_begin_rendering(self.command_buffer, &rendering_info);
        }
        for record in snapshot.scene_records().filter(|record| record.visible) {
            if record.primitive == PresentationPrimitiveV1::Floor {
                continue;
            }
            let attachment = [vk::ClearAttachment {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                color_attachment: 0,
                clear_value: primitive_color(record.primitive),
            }];
            let rectangles = [record_rectangle(record, self.swapchain.extent)];
            // SAFETY: clear rectangles lie within the render area and target
            // color attachment zero of the active dynamic rendering instance.
            unsafe {
                self.device
                    .cmd_clear_attachments(self.command_buffer, &attachment, &rectangles);
            }
        }
        // SAFETY: a dynamic rendering instance is active on this command
        // buffer and is ended exactly once.
        unsafe {
            self.device.cmd_end_rendering(self.command_buffer);
        }
        let to_present = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::NONE)
            .dst_access_mask(vk::AccessFlags2::NONE)
            .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .image(self.swapchain.images[image_usize])
            .subresource_range(subresource)];
        let to_present_dependency =
            vk::DependencyInfo::default().image_memory_barriers(&to_present);
        // SAFETY: the rendering instance has ended and the image remains the
        // acquired swapchain image.
        unsafe {
            self.device
                .cmd_pipeline_barrier2(self.command_buffer, &to_present_dependency);
            self.device.end_command_buffer(self.command_buffer)?;
        }
        let wait_semaphores = [self.image_available];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [self.command_buffer];
        let signal_semaphores = [self.render_finished];
        let submit_info = [vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)];
        // SAFETY: synchronization objects and command buffer are live and the
        // frame fence is unsignaled for this one submission.
        unsafe {
            self.device
                .queue_submit(self.queue, &submit_info, self.frame_fence)?;
        }
        let swapchains = [self.swapchain.handle];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);
        // SAFETY: image index was acquired from this swapchain and rendering
        // completion is signaled by `render_finished`.
        let present_suboptimal = match unsafe {
            self.swapchain_loader
                .queue_present(self.queue, &present_info)
        } {
            Ok(value) => value,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => true,
            Err(error) => return Err(error.into()),
        };
        self.swapchain.initialized[image_usize] = true;
        if acquisition_suboptimal || present_suboptimal {
            self.recreate_swapchain(window)?;
        }
        Ok(true)
    }

    pub(super) fn recreate_swapchain(
        &mut self,
        window: &Window,
    ) -> Result<(), DesktopAdapterError> {
        self.wait_idle()?;
        let old = self.swapchain.handle;
        let replacement = create_swapchain(
            window,
            self.physical_device,
            self.queue_family_index,
            &self.surface_loader,
            self.surface,
            &self.device,
            &self.swapchain_loader,
            old,
        )?;
        destroy_swapchain_state(&self.device, &self.swapchain_loader, &mut self.swapchain);
        self.swapchain = replacement;
        Ok(())
    }

    pub(super) fn wait_idle(&self) -> Result<(), DesktopAdapterError> {
        // SAFETY: device remains live throughout the context lifetime.
        unsafe { self.device.device_wait_idle()? };
        Ok(())
    }
}

impl Drop for GraphicsContext {
    fn drop(&mut self) {
        // SAFETY: all handles were created by this context. Destruction follows
        // child-before-parent ownership and ignores only shutdown-time errors.
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_fence(self.frame_fence, None);
            self.device.destroy_semaphore(self.render_finished, None);
            self.device.destroy_semaphore(self.image_available, None);
            self.device.destroy_command_pool(self.command_pool, None);
            destroy_swapchain_state(&self.device, &self.swapchain_loader, &mut self.swapchain);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}

fn select_physical_device(
    instance: &ash::Instance,
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, u32), DesktopAdapterError> {
    // SAFETY: instance is live and enumeration writes owned handles.
    let physical_devices = unsafe { instance.enumerate_physical_devices() }?;
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

#[allow(
    clippy::too_many_arguments,
    reason = "the private adapter creation boundary keeps all ownership inputs explicit"
)]
fn create_swapchain(
    window: &Window,
    physical_device: vk::PhysicalDevice,
    queue_family_index: u32,
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    device: &ash::Device,
    swapchain_loader: &ash::khr::swapchain::Device,
    old_swapchain: vk::SwapchainKHR,
) -> Result<SwapchainState, DesktopAdapterError> {
    // SAFETY: physical device and surface share a live instance.
    let capabilities = unsafe {
        surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?
    };
    // SAFETY: same ownership as the capability query.
    let formats =
        unsafe { surface_loader.get_physical_device_surface_formats(physical_device, surface)? };
    let selected_format = formats
        .iter()
        .copied()
        .find(|format| {
            format.format == vk::Format::B8G8R8A8_UNORM
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .or_else(|| formats.first().copied())
        .ok_or(DesktopAdapterError::GpuUnsupported)?;
    let (window_width, window_height) = window.size_in_pixels();
    let extent = if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        vk::Extent2D {
            width: window_width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: window_height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    };
    if extent.width == 0 || extent.height == 0 {
        return Err(DesktopAdapterError::InvalidExtent);
    }
    let desired = capabilities.min_image_count.saturating_add(1);
    let image_count = if capabilities.max_image_count == 0 {
        desired
    } else {
        desired.min(capabilities.max_image_count)
    };
    let queue_families = [queue_family_index];
    let create_info = vk::SwapchainCreateInfoKHR::default()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(selected_format.format)
        .image_color_space(selected_format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .queue_family_indices(&queue_families)
        .pre_transform(capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(vk::PresentModeKHR::FIFO)
        .clipped(true)
        .old_swapchain(old_swapchain);
    // SAFETY: all references in create info live for the call and the surface
    // belongs to the same instance/device pair.
    let handle = unsafe { swapchain_loader.create_swapchain(&create_info, None) }?;
    // SAFETY: handle is the live swapchain just created.
    let images = unsafe { swapchain_loader.get_swapchain_images(handle) }?;
    let mut image_views = Vec::with_capacity(images.len());
    for image in &images {
        let view_info = vk::ImageViewCreateInfo::default()
            .image(*image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(selected_format.format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );
        // SAFETY: image belongs to the swapchain and view metadata matches its
        // selected format.
        match unsafe { device.create_image_view(&view_info, None) } {
            Ok(view) => image_views.push(view),
            Err(error) => {
                // SAFETY: these views and swapchain were created in this
                // function and have not escaped.
                unsafe {
                    for view in image_views {
                        device.destroy_image_view(view, None);
                    }
                    swapchain_loader.destroy_swapchain(handle, None);
                }
                return Err(error.into());
            }
        }
    }
    Ok(SwapchainState {
        handle,
        extent,
        initialized: vec![false; images.len()],
        images,
        image_views,
    })
}

fn destroy_swapchain_state(
    device: &ash::Device,
    loader: &ash::khr::swapchain::Device,
    state: &mut SwapchainState,
) {
    for view in state.image_views.drain(..) {
        // SAFETY: caller guarantees the device is idle and each view belongs
        // to this device and is destroyed once.
        unsafe {
            device.destroy_image_view(view, None);
        }
    }
    if state.handle != vk::SwapchainKHR::null() {
        // SAFETY: caller guarantees the swapchain is no longer in use.
        unsafe {
            loader.destroy_swapchain(state.handle, None);
        }
        state.handle = vk::SwapchainKHR::null();
    }
}
