use super::*;
use crate::gpu_content::{B0GpuContent, DepthAttachment};
use next_render::{RenderTargetV1, build_b0_frame_plan};

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
    swapchain: Option<SwapchainState>,
    render_content_catalog: RenderContentCatalogV1,
    b0_content: Option<B0GpuContent>,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    image_available: vk::Semaphore,
    frame_fence: vk::Fence,
}

struct SwapchainState {
    device: ash::Device,
    loader: ash::khr::swapchain::Device,
    handle: vk::SwapchainKHR,
    format: vk::Format,
    depth_format: vk::Format,
    extent: vk::Extent2D,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    depth_attachments: Vec<DepthAttachment>,
    render_finished: Vec<vk::Semaphore>,
    initialized: Vec<bool>,
    depth_initialized: Vec<bool>,
}

impl Drop for SwapchainState {
    fn drop(&mut self) {
        // SAFETY: swapchain states are dropped only while their device is
        // idle, or before a newly created state has escaped construction.
        // Children are destroyed before the parent swapchain.
        unsafe {
            for semaphore in self.render_finished.drain(..) {
                self.device.destroy_semaphore(semaphore, None);
            }
            self.depth_attachments.clear();
            for view in self.image_views.drain(..) {
                self.device.destroy_image_view(view, None);
            }
            self.loader.destroy_swapchain(self.handle, None);
        }
    }
}

struct GraphicsInitializationGuard {
    armed: bool,
    instance: ash::Instance,
    surface_loader: Option<ash::khr::surface::Instance>,
    surface: vk::SurfaceKHR,
    device: Option<ash::Device>,
    swapchain: Option<SwapchainState>,
    command_pool: vk::CommandPool,
    image_available: vk::Semaphore,
    frame_fence: vk::Fence,
}

impl GraphicsInitializationGuard {
    fn new(instance: ash::Instance) -> Self {
        Self {
            armed: true,
            instance,
            surface_loader: None,
            surface: vk::SurfaceKHR::null(),
            device: None,
            swapchain: None,
            command_pool: vk::CommandPool::null(),
            image_available: vk::Semaphore::null(),
            frame_fence: vk::Fence::null(),
        }
    }

    fn finish(mut self) -> Option<SwapchainState> {
        self.armed = false;
        self.swapchain.take()
    }
}

impl Drop for GraphicsInitializationGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        // SAFETY: every non-null handle was recorded immediately after its
        // successful creation and has not escaped. The device is made idle
        // before child-first teardown; device loss still permits teardown.
        unsafe {
            if let Some(device) = self.device.as_ref() {
                let _ = device.device_wait_idle();
                if self.frame_fence != vk::Fence::null() {
                    device.destroy_fence(self.frame_fence, None);
                }
                if self.image_available != vk::Semaphore::null() {
                    device.destroy_semaphore(self.image_available, None);
                }
                if self.command_pool != vk::CommandPool::null() {
                    device.destroy_command_pool(self.command_pool, None);
                }
                drop(self.swapchain.take());
                device.destroy_device(None);
            }
            if self.surface != vk::SurfaceKHR::null()
                && let Some(surface_loader) = self.surface_loader.as_ref()
            {
                surface_loader.destroy_surface(self.surface, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SubmittedB0Frame {
    pub(super) rendered_objects: u64,
    pub(super) indexed_draws: u64,
    pub(super) fallback_material_draws: u64,
    pub(super) frame_plan_hash: ContentHash,
    pub(super) drawable_extent: [u32; 2],
    pub(super) target_revision: u64,
}

const B0_TARGET_REVISION: u64 = 1;
const B0_SURFACE_COLOR_SPACE: vk::ColorSpaceKHR = vk::ColorSpaceKHR::SRGB_NONLINEAR;
const B0_SURFACE_FORMATS: [vk::Format; 2] = [vk::Format::B8G8R8A8_SRGB, vk::Format::R8G8B8A8_SRGB];
const B0_DEPTH_FORMATS: [vk::Format; 2] = [vk::Format::D32_SFLOAT, vk::Format::D16_UNORM];

fn validate_loader_api_version(actual: u32) -> Result<(), DesktopAdapterError> {
    if actual < vk::API_VERSION_1_3 {
        return Err(DesktopAdapterError::LoaderVersionUnsupported {
            required: vk::API_VERSION_1_3,
            actual,
        });
    }
    Ok(())
}

fn classify_instance_creation_error(error: vk::Result) -> DesktopAdapterError {
    if matches!(
        error,
        vk::Result::ERROR_INCOMPATIBLE_DRIVER | vk::Result::ERROR_INITIALIZATION_FAILED
    ) {
        DesktopAdapterError::IcdUnavailable { error: Some(error) }
    } else {
        DesktopAdapterError::Graphics(error)
    }
}

fn classify_physical_device_enumeration_error(error: vk::Result) -> DesktopAdapterError {
    if error == vk::Result::ERROR_INITIALIZATION_FAILED {
        DesktopAdapterError::IcdUnavailable { error: Some(error) }
    } else {
        DesktopAdapterError::Graphics(error)
    }
}

fn classify_device_creation_error(error: vk::Result) -> DesktopAdapterError {
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

fn supports_required_device_extension(
    properties: &[vk::ExtensionProperties],
    required: &std::ffi::CStr,
) -> bool {
    properties
        .iter()
        .any(|property| extension_property_matches(property, required))
}

fn extension_property_matches(
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

fn select_composite_alpha(
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

fn select_b0_surface_format(formats: &[vk::SurfaceFormatKHR]) -> Option<vk::SurfaceFormatKHR> {
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

fn select_b0_depth_format(
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

fn select_b0_depth_format_with(
    mut supports_depth_attachment: impl FnMut(vk::Format) -> bool,
) -> Option<vk::Format> {
    B0_DEPTH_FORMATS
        .into_iter()
        .find(|format| supports_depth_attachment(*format))
}

fn defer_out_of_date<T>(result: Result<T, vk::Result>) -> Result<Option<T>, DesktopAdapterError> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn select_surface_extent(
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

impl GraphicsContext {
    pub(super) fn new(
        window: &Window,
        render_content_catalog: &RenderContentCatalogV1,
    ) -> Result<Self, DesktopAdapterError> {
        // SAFETY: loading the process graphics loader creates an owned entry;
        // all child objects are destroyed in reverse ownership order below.
        let entry = unsafe { ash::Entry::load() }
            .map_err(|error| DesktopAdapterError::Loader(error.to_string()))?;
        // SAFETY: the loaded entry owns the Vulkan loader function table and
        // the query writes no caller-provided memory.
        let loader_api_version = unsafe { entry.try_enumerate_instance_version() }
            .map_err(|error| DesktopAdapterError::Loader(error.to_string()))?
            .unwrap_or(vk::API_VERSION_1_0);
        validate_loader_api_version(loader_api_version)?;
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
        let instance = unsafe { entry.create_instance(&instance_info, None) }
            .map_err(classify_instance_creation_error)?;
        let mut initialization = GraphicsInitializationGuard::new(instance.clone());
        // SAFETY: the SDL window was created with its graphics-surface flag and
        // remains alive until after this context has been dropped.
        let surface =
            unsafe { window.vulkan_create_surface(instance.handle()) }.map_err(sdl_error)?;
        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);
        initialization.surface = surface;
        initialization.surface_loader = Some(surface_loader.clone());
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
        let device = unsafe { instance.create_device(physical_device, &device_info, None) }
            .map_err(classify_device_creation_error)?;
        initialization.device = Some(device.clone());
        // SAFETY: queue zero exists because one priority was requested.
        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };
        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);
        let mut old_swapchain_retired = false;
        let swapchain = create_swapchain(
            window,
            &instance,
            physical_device,
            queue_family_index,
            &surface_loader,
            surface,
            &device,
            &swapchain_loader,
            vk::SwapchainKHR::null(),
            &mut old_swapchain_retired,
        )?;
        debug_assert!(!old_swapchain_retired);
        initialization.swapchain = swapchain;
        let command_pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        // SAFETY: queue family belongs to the selected logical device.
        let command_pool = unsafe { device.create_command_pool(&command_pool_info, None) }?;
        initialization.command_pool = command_pool;
        let command_buffer_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        // SAFETY: command pool is live and owned by this device.
        let command_buffer = unsafe { device.allocate_command_buffers(&command_buffer_info) }?[0];
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        // SAFETY: device is live and synchronization objects use no callbacks.
        let image_available = unsafe { device.create_semaphore(&semaphore_info, None) }?;
        initialization.image_available = image_available;
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        // SAFETY: device is live and fence uses no callbacks.
        let frame_fence = unsafe { device.create_fence(&fence_info, None) }?;
        initialization.frame_fence = frame_fence;
        let b0_content = initialization
            .swapchain
            .as_ref()
            .map(|swapchain| {
                B0GpuContent::new(
                    &instance,
                    physical_device,
                    &device,
                    queue,
                    queue_family_index,
                    swapchain.format,
                    swapchain.depth_format,
                    render_content_catalog,
                )
            })
            .transpose()?;
        let swapchain = initialization.finish();
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
            render_content_catalog: render_content_catalog.clone(),
            b0_content,
            command_pool,
            command_buffer,
            image_available,
            frame_fence,
        })
    }

    pub(super) fn render(
        &mut self,
        snapshot: &PresentationSnapshotV2,
        window: &Window,
    ) -> Result<Option<SubmittedB0Frame>, DesktopAdapterError> {
        if self.swapchain.is_none() {
            self.recreate_swapchain(window)?;
            if self.swapchain.is_none() {
                return Ok(None);
            }
        }
        // SAFETY: the fence belongs to this device and guards the one reusable
        // command buffer and frame synchronization set.
        unsafe {
            self.device
                .wait_for_fences(&[self.frame_fence], true, u64::MAX)?;
        }
        let swapchain_handle = self
            .swapchain
            .as_ref()
            .ok_or(DesktopAdapterError::GraphicsContextMissing)?
            .handle;
        // SAFETY: swapchain and semaphore are live; no fence is needed for
        // acquisition because the frame fence guards prior use.
        let acquired = unsafe {
            self.swapchain_loader.acquire_next_image(
                swapchain_handle,
                u64::MAX,
                self.image_available,
                vk::Fence::null(),
            )
        };
        let Some((image_index, acquisition_suboptimal)) = defer_out_of_date(acquired)? else {
            self.recreate_swapchain(window)?;
            return Ok(None);
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
        let swapchain = self
            .swapchain
            .as_ref()
            .ok_or(DesktopAdapterError::GraphicsContextMissing)?;
        let target = RenderTargetV1 {
            extent: [swapchain.extent.width, swapchain.extent.height],
            target_revision: B0_TARGET_REVISION,
        };
        let frame_plan = build_b0_frame_plan(snapshot, &self.render_content_catalog, target)?;
        let old_layout = if swapchain.initialized[image_usize] {
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
        let depth_old_layout = if swapchain.depth_initialized[image_usize] {
            vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL
        } else {
            vk::ImageLayout::UNDEFINED
        };
        let depth_subresource = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::DEPTH)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let attachment_barriers = [
            vk::ImageMemoryBarrier2::default()
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
                .image(swapchain.images[image_usize])
                .subresource_range(subresource),
            vk::ImageMemoryBarrier2::default()
                .src_stage_mask(if depth_old_layout == vk::ImageLayout::UNDEFINED {
                    vk::PipelineStageFlags2::NONE
                } else {
                    vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
                        | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS
                })
                .src_access_mask(if depth_old_layout == vk::ImageLayout::UNDEFINED {
                    vk::AccessFlags2::NONE
                } else {
                    vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE
                })
                .dst_stage_mask(
                    vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
                        | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
                )
                .dst_access_mask(
                    vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ
                        | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
                )
                .old_layout(depth_old_layout)
                .new_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .image(swapchain.depth_attachments[image_usize].image())
                .subresource_range(depth_subresource),
        ];
        let attachment_dependency =
            vk::DependencyInfo::default().image_memory_barriers(&attachment_barriers);
        // SAFETY: both images belong to the acquired swapchain slot and the
        // barriers are recorded into the reset primary command buffer.
        unsafe {
            self.device
                .cmd_pipeline_barrier2(self.command_buffer, &attachment_dependency);
        }
        let color_clear = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.035, 0.045, 0.065, 1.0],
            },
        };
        let depth_clear = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        };
        let color_attachments = [vk::RenderingAttachmentInfo::default()
            .image_view(swapchain.image_views[image_usize])
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(color_clear)];
        let depth_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(swapchain.depth_attachments[image_usize].view())
            .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .clear_value(depth_clear);
        let render_area = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain.extent,
        };
        let rendering_info = vk::RenderingInfo::default()
            .render_area(render_area)
            .layer_count(1)
            .color_attachments(&color_attachments)
            .depth_attachment(&depth_attachment);
        // SAFETY: dynamic rendering was checked and enabled; the attachment
        // images are in their declared attachment layouts for this index.
        unsafe {
            self.device
                .cmd_begin_rendering(self.command_buffer, &rendering_info);
        }
        self.b0_content
            .as_ref()
            .ok_or(DesktopAdapterError::GraphicsContextMissing)?
            .record(self.command_buffer, &frame_plan, swapchain.extent)?;
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
            .image(swapchain.images[image_usize])
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
        let signal_semaphores = [swapchain.render_finished[image_usize]];
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
        let swapchains = [swapchain_handle];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);
        // SAFETY: image index was acquired from this swapchain and rendering
        // completion is signaled by the semaphore owned by this acquired
        // swapchain image. Reacquiring the image proves the presentation
        // engine consumed its prior wait before that semaphore is reused.
        let presented = defer_out_of_date(unsafe {
            self.swapchain_loader
                .queue_present(self.queue, &present_info)
        })?;
        let Some(present_suboptimal) = presented else {
            self.recreate_swapchain(window)?;
            return Ok(None);
        };
        let swapchain = self
            .swapchain
            .as_mut()
            .ok_or(DesktopAdapterError::GraphicsContextMissing)?;
        swapchain.initialized[image_usize] = true;
        swapchain.depth_initialized[image_usize] = true;
        if acquisition_suboptimal || present_suboptimal {
            self.recreate_swapchain(window)?;
        }
        Ok(Some(SubmittedB0Frame {
            rendered_objects: u64::from(frame_plan.visible_object_count),
            indexed_draws: u64::from(frame_plan.indexed_draw_count),
            fallback_material_draws: u64::from(frame_plan.fallback_material_draw_count),
            frame_plan_hash: frame_plan.frame_plan_hash,
            drawable_extent: target.extent,
            target_revision: target.target_revision,
        }))
    }

    pub(super) fn recreate_swapchain(
        &mut self,
        window: &Window,
    ) -> Result<(), DesktopAdapterError> {
        self.wait_idle()?;
        let old_swapchain = self
            .swapchain
            .as_ref()
            .map_or(vk::SwapchainKHR::null(), |swapchain| swapchain.handle);
        let mut old_swapchain_retired = false;
        let replacement = create_swapchain(
            window,
            &self.instance,
            self.physical_device,
            self.queue_family_index,
            &self.surface_loader,
            self.surface,
            &self.device,
            &self.swapchain_loader,
            old_swapchain,
            &mut old_swapchain_retired,
        )
        .inspect_err(|_| {
            if old_swapchain_retired {
                drop(self.swapchain.take());
            }
        })?;
        let replacement_formats = replacement
            .as_ref()
            .map(|swapchain| (swapchain.format, swapchain.depth_format));
        let current_formats = self
            .swapchain
            .as_ref()
            .map(|swapchain| (swapchain.format, swapchain.depth_format));
        if replacement_formats != current_formats || self.b0_content.is_none() {
            let replacement_content = replacement_formats
                .map(|(color_format, depth_format)| {
                    B0GpuContent::new(
                        &self.instance,
                        self.physical_device,
                        &self.device,
                        self.queue,
                        self.queue_family_index,
                        color_format,
                        depth_format,
                        &self.render_content_catalog,
                    )
                })
                .transpose();
            if replacement_content.is_err() && old_swapchain_retired {
                drop(self.swapchain.take());
            }
            self.b0_content = replacement_content?;
        }
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
            drop(self.b0_content.take());
            self.device.destroy_fence(self.frame_fence, None);
            self.device.destroy_semaphore(self.image_available, None);
            self.device.destroy_command_pool(self.command_pool, None);
            drop(self.swapchain.take());
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

#[allow(
    clippy::too_many_arguments,
    reason = "the private adapter creation boundary keeps all ownership inputs explicit"
)]
fn create_swapchain(
    window: &Window,
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    queue_family_index: u32,
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    device: &ash::Device,
    swapchain_loader: &ash::khr::swapchain::Device,
    old_swapchain: vk::SwapchainKHR,
    old_swapchain_retired: &mut bool,
) -> Result<Option<SwapchainState>, DesktopAdapterError> {
    debug_assert!(!*old_swapchain_retired);
    // SAFETY: physical device and surface share a live instance.
    let capabilities = unsafe {
        surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?
    };
    // SAFETY: same ownership as the capability query.
    let formats =
        unsafe { surface_loader.get_physical_device_surface_formats(physical_device, surface)? };
    let selected_format =
        select_b0_surface_format(&formats).ok_or(DesktopAdapterError::GpuUnsupported)?;
    let depth_format = select_b0_depth_format(instance, physical_device)
        .ok_or(DesktopAdapterError::GpuUnsupported)?;
    let (window_width, window_height) = window.size_in_pixels();
    let Some(extent) = select_surface_extent(&capabilities, [window_width, window_height]) else {
        return Ok(None);
    };
    let composite_alpha = select_composite_alpha(capabilities.supported_composite_alpha)
        .ok_or(DesktopAdapterError::GpuUnsupported)?;
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
        .composite_alpha(composite_alpha)
        .present_mode(vk::PresentModeKHR::FIFO)
        .clipped(true)
        .old_swapchain(old_swapchain);
    // SAFETY: all references in create info live for the call and the surface
    // belongs to the same instance/device pair.
    let handle = unsafe { swapchain_loader.create_swapchain(&create_info, None) }?;
    *old_swapchain_retired = old_swapchain != vk::SwapchainKHR::null();
    let mut state = SwapchainState {
        device: device.clone(),
        loader: swapchain_loader.clone(),
        handle,
        format: selected_format.format,
        depth_format,
        extent,
        images: Vec::new(),
        image_views: Vec::new(),
        depth_attachments: Vec::new(),
        render_finished: Vec::new(),
        initialized: Vec::new(),
        depth_initialized: Vec::new(),
    };
    // SAFETY: handle is the live swapchain just created.
    state.images = unsafe { swapchain_loader.get_swapchain_images(handle) }?;
    state.image_views.reserve(state.images.len());
    for image in &state.images {
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
        state
            .image_views
            .push(unsafe { device.create_image_view(&view_info, None) }?);
    }
    state.depth_attachments.reserve(state.images.len());
    for _ in &state.images {
        state.depth_attachments.push(DepthAttachment::new(
            instance,
            physical_device,
            device,
            depth_format,
            extent,
        )?);
    }
    state.render_finished.reserve(state.images.len());
    for _ in &state.images {
        // SAFETY: each binary semaphore is device-owned and has no retained
        // host pointers. It is dedicated to one swapchain image.
        state
            .render_finished
            .push(unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None) }?);
    }
    state.initialized = vec![false; state.images.len()];
    state.depth_initialized = vec![false; state.images.len()];
    Ok(Some(state))
}

#[cfg(test)]
mod tests;
