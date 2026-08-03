use super::*;
use crate::gpu_content::{B0GpuContent, DepthAttachment, UiOverlayState};
use next_render::{B0FramePlannerMetricsV1, B0FramePlannerV1, RenderTargetV1};

mod capabilities;
mod overlay;
mod profiling;
mod setup;

use capabilities::select_physical_device;
use profiling::{CpuFramePhaseTimings, FrameProfilingReport, VulkanFrameProfiler};
use setup::*;

const FRAME_SLOT_COUNT: usize = 2;

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
    frame_planner: B0FramePlannerV1,
    b0_content: Option<B0GpuContent>,
    ui_overlay: UiOverlayState,
    command_pool: vk::CommandPool,
    frame_slots: Vec<FrameSlot>,
    next_frame_slot: usize,
    frame_profiler: Option<VulkanFrameProfiler>,
}

#[derive(Clone, Copy)]
struct FrameSlot {
    command_buffer: vk::CommandBuffer,
    image_available: vk::Semaphore,
    fence: vk::Fence,
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
    images_in_flight: Vec<vk::Fence>,
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
    image_available: Vec<vk::Semaphore>,
    frame_fences: Vec<vk::Fence>,
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
            image_available: Vec::new(),
            frame_fences: Vec::new(),
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
                for fence in self.frame_fences.drain(..) {
                    device.destroy_fence(fence, None);
                }
                for semaphore in self.image_available.drain(..) {
                    device.destroy_semaphore(semaphore, None);
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

impl GraphicsContext {
    pub(super) fn new(
        window: &Window,
        render_content_catalog: &RenderContentCatalogV1,
        options: &DesktopRunOptions,
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
        // SAFETY: both capability queries return value data for the selected
        // physical device and do not retain host pointers.
        let physical_device_properties =
            unsafe { instance.get_physical_device_properties(physical_device) };
        // SAFETY: physical device belongs to this live instance.
        let queue_family_properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let timestamp_valid_bits = queue_family_properties
            .get(
                usize::try_from(queue_family_index)
                    .map_err(|_| DesktopAdapterError::CounterOverflow)?,
            )
            .ok_or(DesktopAdapterError::GpuUnsupported)?
            .timestamp_valid_bits;

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
            .command_buffer_count(
                u32::try_from(FRAME_SLOT_COUNT)
                    .map_err(|_| DesktopAdapterError::CounterOverflow)?,
            );
        // SAFETY: command pool is live and owned by this device.
        let command_buffers = unsafe { device.allocate_command_buffers(&command_buffer_info) }?;
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let mut frame_slots = Vec::with_capacity(FRAME_SLOT_COUNT);
        for command_buffer in command_buffers {
            // SAFETY: device is live and synchronization objects use no
            // callbacks. Handles are recorded in the guard immediately.
            let image_available = unsafe { device.create_semaphore(&semaphore_info, None) }?;
            initialization.image_available.push(image_available);
            // SAFETY: device is live and fence uses no callbacks.
            let fence = unsafe { device.create_fence(&fence_info, None) }?;
            initialization.frame_fences.push(fence);
            frame_slots.push(FrameSlot {
                command_buffer,
                image_available,
                fence,
            });
        }
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
                    frame_slots.len(),
                )
            })
            .transpose()?;
        let ui_overlay = initialization.swapchain.as_ref().map_or_else(
            || {
                UiOverlayState::new(
                    &[],
                    &options.ui_locale,
                    &instance,
                    physical_device,
                    &device,
                    queue,
                    queue_family_index,
                    vk::Format::UNDEFINED,
                    vk::Format::UNDEFINED,
                )
            },
            |swapchain| {
                UiOverlayState::new(
                    &options.ui_text_catalogs,
                    &options.ui_locale,
                    &instance,
                    physical_device,
                    &device,
                    queue,
                    queue_family_index,
                    swapchain.format,
                    swapchain.depth_format,
                )
            },
        );
        let frame_profiler = (options.frame_profiling_sample_capacity > 0)
            .then(|| {
                VulkanFrameProfiler::new(
                    &device,
                    options.frame_profiling_sample_capacity,
                    physical_device_properties.limits.timestamp_period,
                    timestamp_valid_bits,
                    frame_slots.len(),
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
            frame_planner: B0FramePlannerV1::new(),
            b0_content,
            ui_overlay,
            command_pool,
            frame_slots,
            next_frame_slot: 0,
            frame_profiler,
        })
    }

    pub(super) fn render(
        &mut self,
        snapshot: &PresentationSnapshotV2,
        window: &Window,
        event_and_frame_source_update_microseconds: u64,
    ) -> Result<Option<SubmittedB0Frame>, DesktopAdapterError> {
        if self.swapchain.is_none() {
            self.recreate_swapchain(window)?;
            if self.swapchain.is_none() {
                return Ok(None);
            }
        }
        let frame_slot_index = self.next_frame_slot;
        let frame_slot = self
            .frame_slots
            .get(frame_slot_index)
            .copied()
            .ok_or(DesktopAdapterError::GraphicsContextMissing)?;
        let profiling_enabled = self.frame_profiler.is_some();
        let frame_slot_wait_started = profiling_enabled.then(Instant::now);
        // SAFETY: this slot fence guards its reusable command buffer, acquire
        // semaphore and uniform/descriptor pair.
        unsafe {
            self.device
                .wait_for_fences(&[frame_slot.fence], true, u64::MAX)?;
        }
        // The fence is about to be reused for another submission. Retire every
        // image alias proven complete by the wait first, otherwise an old
        // image could later appear to depend on the slot's unrelated new work.
        retire_completed_image_fence_mappings(
            &mut self
                .swapchain
                .as_mut()
                .ok_or(DesktopAdapterError::GraphicsContextMissing)?
                .images_in_flight,
            frame_slot.fence,
        );
        // The slot fence proves this slot's prior overlay sample completed;
        // the optional overlay re-rasterizes and re-uploads only on a content
        // or extent change and never fails the frame on its own.
        self.update_ui_overlay(snapshot);
        let mut cpu_phases = CpuFramePhaseTimings {
            event_and_frame_source_update_microseconds,
            frame_slot_wait_microseconds: elapsed_microseconds(frame_slot_wait_started)?,
            ..CpuFramePhaseTimings::default()
        };
        if let Some(profiler) = self.frame_profiler.as_mut() {
            profiler.collect_pending(frame_slot_index)?;
        }
        let swapchain_handle = self
            .swapchain
            .as_ref()
            .ok_or(DesktopAdapterError::GraphicsContextMissing)?
            .handle;
        let image_acquire_wait_started = profiling_enabled.then(Instant::now);
        // SAFETY: swapchain and the slot's acquire semaphore are live; the slot
        // fence established that the semaphore's prior wait was consumed.
        let acquired = unsafe {
            self.swapchain_loader.acquire_next_image(
                swapchain_handle,
                u64::MAX,
                frame_slot.image_available,
                vk::Fence::null(),
            )
        };
        cpu_phases.image_acquire_wait_microseconds =
            elapsed_microseconds(image_acquire_wait_started)?;
        let Some((image_index, acquisition_suboptimal)) = defer_out_of_date(acquired)? else {
            self.recreate_swapchain(window)?;
            return Ok(None);
        };
        let image_usize =
            usize::try_from(image_index).map_err(|_| DesktopAdapterError::CounterOverflow)?;
        let prior_image_fence = self
            .swapchain
            .as_ref()
            .and_then(|swapchain| swapchain.images_in_flight.get(image_usize))
            .copied()
            .ok_or(DesktopAdapterError::GraphicsContextMissing)?;
        let swapchain_image_wait_started = profiling_enabled.then(Instant::now);
        if let Some(prior_image_fence) = image_fence_to_wait(prior_image_fence, frame_slot.fence) {
            // SAFETY: the fence is owned by another live frame slot and the
            // acquired image cannot be reused until that submission completes.
            unsafe {
                self.device
                    .wait_for_fences(&[prior_image_fence], true, u64::MAX)?;
            }
        }
        cpu_phases.swapchain_image_wait_microseconds =
            elapsed_microseconds(swapchain_image_wait_started)?;

        // SAFETY: the slot fence completed, so its command buffer is no longer
        // pending. The fence remains signaled until immediately before submit,
        // preventing a recording failure from leaving a dead slot.
        unsafe {
            self.device.reset_command_buffer(
                frame_slot.command_buffer,
                vk::CommandBufferResetFlags::empty(),
            )?;
            self.device.begin_command_buffer(
                frame_slot.command_buffer,
                &vk::CommandBufferBeginInfo::default(),
            )?;
        }
        let cpu_profile_started = profiling_enabled.then(Instant::now);
        if let Some(profiler) = self.frame_profiler.as_ref() {
            profiler.write_start(frame_slot.command_buffer, frame_slot_index)?;
        }
        let swapchain = self
            .swapchain
            .as_ref()
            .ok_or(DesktopAdapterError::GraphicsContextMissing)?;
        let target = RenderTargetV1 {
            extent: [swapchain.extent.width, swapchain.extent.height],
            target_revision: B0_TARGET_REVISION,
        };
        let frame_plan_started = profiling_enabled.then(Instant::now);
        let frame_plan =
            self.frame_planner
                .build_or_reuse(snapshot, &self.render_content_catalog, target)?;
        cpu_phases.frame_plan_microseconds = elapsed_microseconds(frame_plan_started)?;
        let command_record_started = profiling_enabled.then(Instant::now);
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
                .cmd_pipeline_barrier2(frame_slot.command_buffer, &attachment_dependency);
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
                .cmd_begin_rendering(frame_slot.command_buffer, &rendering_info);
        }
        self.b0_content
            .as_ref()
            .ok_or(DesktopAdapterError::GraphicsContextMissing)?
            .record(
                frame_slot.command_buffer,
                frame_plan,
                swapchain.extent,
                frame_slot_index,
            )?;
        self.ui_overlay
            .record(frame_slot.command_buffer, swapchain.extent);
        // SAFETY: a dynamic rendering instance is active on this command
        // buffer and is ended exactly once.
        unsafe {
            self.device.cmd_end_rendering(frame_slot.command_buffer);
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
                .cmd_pipeline_barrier2(frame_slot.command_buffer, &to_present_dependency);
        }
        if let Some(profiler) = self.frame_profiler.as_ref() {
            profiler.write_end(frame_slot.command_buffer, frame_slot_index)?;
        }
        // SAFETY: all render and profiling commands have been recorded and the
        // primary command buffer is still in the recording state.
        unsafe {
            self.device.end_command_buffer(frame_slot.command_buffer)?;
        }
        cpu_phases.command_record_microseconds = elapsed_microseconds(command_record_started)?;
        let wait_semaphores = [frame_slot.image_available];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [frame_slot.command_buffer];
        let signal_semaphores = [swapchain.render_finished[image_usize]];
        let submit_info = [vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)];
        let queue_submit_started = profiling_enabled.then(Instant::now);
        // SAFETY: synchronization objects and command buffer are live. The
        // completed slot fence is reset immediately before this one submission.
        unsafe {
            self.device.reset_fences(&[frame_slot.fence])?;
            self.device
                .queue_submit(self.queue, &submit_info, frame_slot.fence)?;
        }
        cpu_phases.queue_submit_microseconds = elapsed_microseconds(queue_submit_started)?;
        if let Some(started) = cpu_profile_started {
            let cpu_extract_and_submit_microseconds = elapsed_microseconds(Some(started))?;
            self.frame_profiler
                .as_mut()
                .ok_or(DesktopAdapterError::GpuTimestampStateInvalid)?
                .mark_submitted(
                    frame_slot_index,
                    cpu_extract_and_submit_microseconds,
                    cpu_phases,
                )?;
        }
        let submitted_frame = SubmittedB0Frame {
            rendered_objects: u64::from(frame_plan.visible_object_count),
            indexed_draws: u64::from(frame_plan.indexed_draw_count),
            fallback_material_draws: u64::from(frame_plan.fallback_material_draw_count),
            frame_plan_hash: frame_plan.frame_plan_hash,
            drawable_extent: target.extent,
            target_revision: target.target_revision,
        };
        self.swapchain
            .as_mut()
            .and_then(|swapchain| swapchain.images_in_flight.get_mut(image_usize))
            .map(|image_fence| *image_fence = frame_slot.fence)
            .ok_or(DesktopAdapterError::GraphicsContextMissing)?;
        self.next_frame_slot = next_frame_slot(frame_slot_index, self.frame_slots.len())
            .ok_or(DesktopAdapterError::GraphicsContextMissing)?;
        let swapchains = [swapchain_handle];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);
        let present_wait_started = profiling_enabled.then(Instant::now);
        // SAFETY: image index was acquired from this swapchain and rendering
        // completion is signaled by the semaphore owned by this acquired
        // swapchain image. Reacquiring the image proves the presentation
        // engine consumed its prior wait before that semaphore is reused.
        let presented = unsafe {
            self.swapchain_loader
                .queue_present(self.queue, &present_info)
        };
        let present_wait_microseconds = elapsed_microseconds(present_wait_started)?;
        if let Some(profiler) = self.frame_profiler.as_mut() {
            profiler.mark_presented(frame_slot_index, present_wait_microseconds)?;
        }
        let presented = defer_out_of_date(presented)?;
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
        Ok(Some(submitted_frame))
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
                        self.frame_slots.len(),
                    )
                })
                .transpose();
            if replacement_content.is_err() && old_swapchain_retired {
                drop(self.swapchain.take());
            }
            self.b0_content = replacement_content?;
        }
        // The overlay pipeline commits to the swapchain color/depth formats;
        // a format change recreates it (the device was idled above) and the
        // next frame re-rasterizes into fresh resources.
        if replacement_formats != current_formats && self.ui_overlay.enabled() {
            if let Some((color_format, depth_format)) = replacement_formats {
                self.ui_overlay.recreate(
                    &self.instance,
                    self.physical_device,
                    &self.device,
                    self.queue,
                    self.queue_family_index,
                    color_format,
                    depth_format,
                );
            }
        }
        self.swapchain = replacement;
        Ok(())
    }

    pub(super) fn wait_idle(&mut self) -> Result<(), DesktopAdapterError> {
        // SAFETY: device remains live throughout the context lifetime.
        unsafe { self.device.device_wait_idle()? };
        if let Some(profiler) = self.frame_profiler.as_mut() {
            profiler.collect_all_pending()?;
        }
        Ok(())
    }

    pub(super) fn take_frame_profiling(&mut self) -> FrameProfilingReport {
        self.frame_profiler.as_mut().map_or_else(
            FrameProfilingReport::default,
            VulkanFrameProfiler::take_report,
        )
    }

    pub(super) fn device_allocation_stats(&self) -> Result<(u64, u64), DesktopAdapterError> {
        let (mut bytes, mut allocations) = self
            .b0_content
            .as_ref()
            .map(B0GpuContent::device_allocation_stats)
            .transpose()?
            .unwrap_or((0, 0));
        let (overlay_bytes, overlay_allocations) = self.ui_overlay.allocation_stats();
        bytes = bytes
            .checked_add(overlay_bytes)
            .ok_or(DesktopAdapterError::CounterOverflow)?;
        allocations = allocations
            .checked_add(overlay_allocations)
            .ok_or(DesktopAdapterError::CounterOverflow)?;
        if let Some(swapchain) = &self.swapchain {
            for attachment in &swapchain.depth_attachments {
                bytes = bytes
                    .checked_add(attachment.allocation_size())
                    .ok_or(DesktopAdapterError::CounterOverflow)?;
                allocations = allocations
                    .checked_add(1)
                    .ok_or(DesktopAdapterError::CounterOverflow)?;
            }
        }
        Ok((bytes, allocations))
    }

    pub(super) const fn frame_plan_metrics(&self) -> B0FramePlannerMetricsV1 {
        self.frame_planner.metrics()
    }
}

impl Drop for GraphicsContext {
    fn drop(&mut self) {
        // SAFETY: all handles were created by this context. Destruction follows
        // child-before-parent ownership and ignores only shutdown-time errors.
        unsafe {
            let _ = self.device.device_wait_idle();
            drop(self.frame_profiler.take());
            self.ui_overlay.teardown();
            drop(self.b0_content.take());
            for frame_slot in &self.frame_slots {
                self.device.destroy_fence(frame_slot.fence, None);
                self.device
                    .destroy_semaphore(frame_slot.image_available, None);
            }
            self.device.destroy_command_pool(self.command_pool, None);
            drop(self.swapchain.take());
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
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
        .present_mode(B0_PRESENT_MODE)
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
        images_in_flight: Vec::new(),
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
    state.images_in_flight = vec![vk::Fence::null(); state.images.len()];
    Ok(Some(state))
}

#[cfg(test)]
mod tests;
