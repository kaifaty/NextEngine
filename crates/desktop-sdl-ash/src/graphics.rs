use std::collections::BTreeMap;

use super::*;
use crate::dynamic_surface::{DynamicSurfaceProfileV1, DynamicSurfaceUpdateV1};
use crate::gpu_content::fluid::{FluidPassState, FluidUploadStats};
use crate::gpu_content::gbuffer::{GBufferCaptureImageV1, GBufferPassState};
use crate::gpu_content::water::WaterPassState;
use crate::gpu_content::{
    B0_SUN_DIRECTION_INTENSITY, B0GpuContent, BufferAllocation, DepthAttachment, UiOverlayState,
    projection_jitter,
};
use crate::particle_surface::{ParticleSurfaceProfileV1, ParticleSurfaceUpdateV1};
use crate::run_state::{
    DesktopCaptureSourceV1, DesktopCapturedFrameV1, DesktopFrameCaptureRequestV1,
};
use next_render::{B0FramePlannerMetricsV1, B0FramePlannerV1, RenderTargetV1};
mod capabilities;
mod overlay;
mod profiling;
mod setup;
use capabilities::select_physical_device;
use profiling::{CpuFramePhaseTimings, FrameProfilingReport, VulkanFrameProfiler};
use setup::*;
/// Frames that may be in flight at once. Every per-slot ring (uniforms,
/// skinned vertices, dynamic surfaces, timestamps) has exactly this depth.
pub const FRAME_SLOT_COUNT: usize = 2;
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
    dynamic_surface_profiles: Vec<DynamicSurfaceProfileV1>,
    frame_planner: B0FramePlannerV1,
    b0_content: Option<B0GpuContent>,
    ui_overlay: UiOverlayState,
    command_pool: vk::CommandPool,
    frame_slots: Vec<FrameSlot>,
    next_frame_slot: usize,
    frame_profiler: Option<VulkanFrameProfiler>,
    capture: Option<FrameCaptureState>,
    /// ADR-102 presentation-only particle surface pass, when declared and
    /// supported by the device and surface.
    particle_surface_profile: Option<ParticleSurfaceProfileV1>,
    fluid: Option<FluidPassState>,
    /// Plan 13: the water pass of `WaterSurface` rings; `None` falls back to
    /// the WL1 material inside the world pass.
    water: Option<WaterPassState>,
    /// Plan 18: the DLSS-ready outputs (HUD-less scene colour target and
    /// the G-buffer pass); `None` renders straight into the swapchain.
    gbuffer: Option<GBufferPassState>,
    /// Plan 18: sub-pixel projection jitter per rendered frame.
    projection_jitter: bool,
    particle_surface_available: bool,
}

/// Bounded developer capture of a short frame burst: one host-visible
/// destination per captured frame and the pending copy recorded in that
/// frame's command buffer. Buffers are read back only after the device idles.
struct FrameCaptureState {
    request: DesktopFrameCaptureRequestV1,
    pending: Vec<(PendingFrameCapture, BufferAllocation)>,
}

#[derive(Clone, Copy)]
struct PendingFrameCapture {
    rendered_frame_index: u64,
    extent: [u32; 2],
    format: vk::Format,
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
    /// Whether the presentation engine ignores the alpha channel, which lets
    /// the particle surface pass use alpha as its coverage diagnostic.
    opaque_composite: bool,
    /// Whether the images carry transfer-source usage (capture, scene copy).
    transfer_source: bool,
    /// Plan 18: whether the images accept a transfer write (the scene copy).
    transfer_destination: bool,
    /// Whether the depth attachments can be sampled (the water pass).
    depth_sampled: bool,
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
    pub(super) dynamic_surface_uploads: u64,
    pub(super) dynamic_surface_upload_bytes: u64,
    pub(super) dynamic_surface_draws: u64,
    pub(super) particle_surface_uploads: u64,
    pub(super) particle_surface_upload_bytes: u64,
    pub(super) particle_surface_recorded: bool,
    /// Plan 33: the eye was under a water ring's level this frame.
    pub(super) submerged: bool,
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
            options.frame_capture.is_some(),
            options.particle_surface.is_some(),
            water_surface_requested(&options.dynamic_surfaces),
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
                    &options.dynamic_surfaces,
                )
            })
            .transpose()?;
        let ui_overlay = initialization.swapchain.as_ref().map_or_else(
            || {
                UiOverlayState::new(
                    &[],
                    &options.ui_locale,
                    options.ui_text_scale_milli,
                    options.ui_subtitles_enabled,
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
                    options.ui_text_scale_milli,
                    options.ui_subtitles_enabled,
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
        let (fluid, particle_surface_available) = create_fluid_pass(
            &instance,
            physical_device,
            &device,
            options.particle_surface,
            initialization.swapchain.as_ref(),
            frame_slots.len(),
        )?;
        let water = create_water_pass(
            &instance,
            physical_device,
            &device,
            b0_content.as_ref(),
            initialization.swapchain.as_ref(),
            frame_slots.len(),
        )?;
        let gbuffer = create_gbuffer_pass(
            &instance,
            physical_device,
            &device,
            b0_content.as_ref(),
            initialization.swapchain.as_ref(),
            frame_slots.len(),
        )?;
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
            dynamic_surface_profiles: options.dynamic_surfaces.clone(),
            frame_planner: B0FramePlannerV1::new(),
            b0_content,
            ui_overlay,
            command_pool,
            frame_slots,
            next_frame_slot: 0,
            frame_profiler,
            capture: options.frame_capture.map(|request| FrameCaptureState {
                request,
                pending: Vec::new(),
            }),
            particle_surface_profile: options.particle_surface,
            fluid,
            water,
            gbuffer,
            projection_jitter: options.projection_jitter,
            particle_surface_available,
        })
    }

    pub(super) fn render(
        &mut self,
        snapshot: &PresentationSnapshotV3,
        dynamic_surfaces: &BTreeMap<AssetRevisionRefV1, Arc<DynamicSurfaceUpdateV1>>,
        particles: Option<&Arc<ParticleSurfaceUpdateV1>>,
        window: &Window,
        event_and_frame_source_update_microseconds: u64,
        rendered_frame_index: u64,
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
        let b0_content = self
            .b0_content
            .as_mut()
            .ok_or(DesktopAdapterError::GraphicsContextMissing)?;
        // The slot fence completed above, so this slot's dynamic surface ring
        // is not read by any pending submission and may be refreshed now.
        let dynamic_surface_upload_started = profiling_enabled.then(Instant::now);
        let dynamic_surface_uploads =
            b0_content.prepare_dynamic_surfaces(dynamic_surfaces, frame_slot_index)?;
        cpu_phases.dynamic_surface_upload_microseconds =
            elapsed_microseconds(dynamic_surface_upload_started)?;
        cpu_phases.dynamic_surface_uploads = dynamic_surface_uploads.uploads;
        // The skinned vertex stream is prepared once per frame, before the
        // shadow, reflection and main passes that draw it.
        b0_content.prepare_dynamic_vertices(frame_plan, frame_slot_index)?;
        let particle_uploads = match self.fluid.as_mut() {
            Some(fluid) => fluid.prepare(particles, frame_slot_index)?,
            None => FluidUploadStats::default(),
        };
        let command_record_started = profiling_enabled.then(Instant::now);
        b0_content.record_dynamic_surface_uploads(frame_slot.command_buffer, frame_slot_index)?;
        b0_content.record_shadow(
            frame_slot.command_buffer,
            frame_plan,
            swapchain.extent,
            frame_slot_index,
        )?;
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
        // Plan 33: the eye under a ring's level; the droplet layer belongs
        // to the air side and is skipped while submerged.
        let submerged_level = if self.water.is_some() {
            b0_content.water_submersion_level_metres(frame_plan, frame_slot_index)?
        } else {
            None
        };
        let particle_pass_this_frame = self.fluid.is_some()
            && particle_uploads.particle_count > 0
            && submerged_level.is_none();
        // Plan 13: the water pass draws the water rings after the opaque
        // scene when the pass exists and the plan has a camera.
        let water_pass_this_frame = self.water.is_some() && frame_plan.camera.is_some();
        // Plan 18: the scene passes render into the HUD-less scene target and
        // the G-buffer pass follows them when the pass exists and the plan
        // has a camera; the UI overlay then draws on the swapchain.
        let gbuffer_this_frame = self.gbuffer.is_some() && frame_plan.camera.is_some();
        let jitter = self
            .projection_jitter
            .then(|| projection_jitter(rendered_frame_index, swapchain.extent));
        let (scene_image, scene_view) = match self.gbuffer.as_ref() {
            Some(gbuffer) if gbuffer_this_frame => {
                gbuffer.record_scene_begin(frame_slot.command_buffer);
                (gbuffer.scene_color_image(), gbuffer.scene_color_view())
            }
            _ => (
                swapchain.images[image_usize],
                swapchain.image_views[image_usize],
            ),
        };
        let color_attachments = [vk::RenderingAttachmentInfo::default()
            .image_view(scene_view)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(color_clear)];
        let depth_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(swapchain.depth_attachments[image_usize].view())
            .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            // The particle surface pass depth-tests against the opaque scene.
            .store_op(
                if particle_pass_this_frame || water_pass_this_frame || gbuffer_this_frame {
                    vk::AttachmentStoreOp::STORE
                } else {
                    vk::AttachmentStoreOp::DONT_CARE
                },
            )
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
        // Plan 15: the mirrored reflection pass renders the plan into the
        // water pass's targets before the world pass.
        if water_pass_this_frame
            && let Some(water) = self.water.as_mut()
            && let Some(plane_height) =
                b0_content.water_plane_height_metres(frame_plan, frame_slot_index)
        {
            water.record_reflection_begin(frame_slot.command_buffer);
            let reflection_colors = [vk::RenderingAttachmentInfo::default()
                .image_view(water.reflection_color_view())
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 0.0],
                    },
                })];
            let reflection_depth = vk::RenderingAttachmentInfo::default()
                .image_view(water.reflection_depth_view())
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .clear_value(depth_clear);
            let reflection_info = vk::RenderingInfo::default()
                .render_area(render_area)
                .layer_count(1)
                .color_attachments(&reflection_colors)
                .depth_attachment(&reflection_depth);
            // SAFETY: the reflection targets are in attachment layouts and
            // no rendering instance is active.
            unsafe {
                self.device
                    .cmd_begin_rendering(frame_slot.command_buffer, &reflection_info);
            }
            b0_content.record_reflection(
                frame_slot.command_buffer,
                frame_plan,
                swapchain.extent,
                frame_slot_index,
                water,
                plane_height,
                jitter,
            )?;
            // SAFETY: the reflection rendering instance is ended exactly once.
            unsafe {
                self.device.cmd_end_rendering(frame_slot.command_buffer);
            }
            water.record_reflection_end(frame_slot.command_buffer);
        }
        // SAFETY: dynamic rendering was checked and enabled; the attachment
        // images are in their declared attachment layouts for this index.
        unsafe {
            self.device
                .cmd_begin_rendering(frame_slot.command_buffer, &rendering_info);
        }
        b0_content.record_sky(frame_slot.command_buffer, swapchain.extent)?;
        let mut dynamic_surface_draws = b0_content.record(
            frame_slot.command_buffer,
            frame_plan,
            swapchain.extent,
            frame_slot_index,
            water_pass_this_frame,
            jitter,
        )?;
        if water_pass_this_frame
            && let (Some(water), Some(camera)) = (self.water.as_mut(), frame_plan.camera.as_ref())
        {
            let depth = &swapchain.depth_attachments[image_usize];
            let (viewport, scissor) = water.prepare(
                frame_slot_index,
                camera,
                depth.view(),
                rendered_frame_index,
                jitter,
                submerged_level,
            )?;
            // SAFETY: the opaque world rendering instance ends before the
            // pass copies the swapchain colour and samples the scene depth.
            unsafe {
                self.device.cmd_end_rendering(frame_slot.command_buffer);
            }
            water.record_begin(frame_slot.command_buffer, scene_image, depth.image());
            let water_colors = [vk::RenderingAttachmentInfo::default()
                .image_view(scene_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE)];
            let water_depth = vk::RenderingAttachmentInfo::default()
                .image_view(depth.view())
                .image_layout(vk::ImageLayout::DEPTH_READ_ONLY_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE);
            let water_info = vk::RenderingInfo::default()
                .render_area(render_area)
                .layer_count(1)
                .color_attachments(&water_colors)
                .depth_attachment(&water_depth);
            // SAFETY: the swapchain image is back in attachment layout and the
            // depth image is read-only for this instance.
            unsafe {
                self.device
                    .cmd_begin_rendering(frame_slot.command_buffer, &water_info);
            }
            if submerged_level.is_some() {
                b0_content.record_water_under(
                    frame_slot.command_buffer,
                    frame_slot_index,
                    water,
                    viewport,
                    scissor,
                )?;
            }
            dynamic_surface_draws = dynamic_surface_draws
                .checked_add(b0_content.record_water_surfaces(
                    frame_slot.command_buffer,
                    frame_plan,
                    frame_slot_index,
                    water,
                    viewport,
                    scissor,
                )?)
                .ok_or(DesktopAdapterError::CounterOverflow)?;
            // SAFETY: the water rendering instance is ended exactly once and
            // the depth image returns to attachment layout for later passes.
            unsafe {
                self.device.cmd_end_rendering(frame_slot.command_buffer);
            }
            water.record_end(frame_slot.command_buffer, depth.image());
            let resume_colors = [vk::RenderingAttachmentInfo::default()
                .image_view(scene_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE)];
            let resume_depth = vk::RenderingAttachmentInfo::default()
                .image_view(depth.view())
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(if particle_pass_this_frame || gbuffer_this_frame {
                    vk::AttachmentStoreOp::STORE
                } else {
                    vk::AttachmentStoreOp::DONT_CARE
                });
            let resume_info = vk::RenderingInfo::default()
                .render_area(render_area)
                .layer_count(1)
                .color_attachments(&resume_colors)
                .depth_attachment(&resume_depth);
            // SAFETY: both attachments are in their declared layouts; the
            // later passes expect an active rendering instance.
            unsafe {
                self.device
                    .cmd_begin_rendering(frame_slot.command_buffer, &resume_info);
            }
        }
        let mut particle_surface_recorded = false;
        if let Some(profiler) = self.frame_profiler.as_ref() {
            profiler.write_particle_surface(frame_slot.command_buffer, frame_slot_index, false)?;
        }
        if particle_pass_this_frame && let Some(fluid) = self.fluid.as_mut() {
            // SAFETY: the opaque world rendering instance ends before the
            // pass copies the swapchain colour and samples the scene depth.
            unsafe {
                self.device.cmd_end_rendering(frame_slot.command_buffer);
            }
            particle_surface_recorded = fluid.record(
                frame_slot.command_buffer,
                frame_slot_index,
                frame_plan.camera.as_ref(),
                scene_image,
                scene_view,
                swapchain.depth_attachments[image_usize].view(),
                B0_SUN_DIRECTION_INTENSITY,
                jitter,
            )?;
            let overlay_colors = [vk::RenderingAttachmentInfo::default()
                .image_view(scene_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE)];
            let overlay_depth = vk::RenderingAttachmentInfo::default()
                .image_view(swapchain.depth_attachments[image_usize].view())
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(if gbuffer_this_frame {
                    vk::AttachmentStoreOp::STORE
                } else {
                    vk::AttachmentStoreOp::DONT_CARE
                });
            let overlay_info = vk::RenderingInfo::default()
                .render_area(render_area)
                .layer_count(1)
                .color_attachments(&overlay_colors)
                .depth_attachment(&overlay_depth);
            // SAFETY: the pass left the swapchain image in attachment layout
            // and the depth attachment untouched; the overlay reopens both.
            unsafe {
                self.device
                    .cmd_begin_rendering(frame_slot.command_buffer, &overlay_info);
            }
        }
        if let Some(profiler) = self.frame_profiler.as_ref() {
            profiler.write_particle_surface(frame_slot.command_buffer, frame_slot_index, true)?;
        }
        if gbuffer_this_frame && let Some(gbuffer) = self.gbuffer.as_mut() {
            // SAFETY: the scene rendering instance ends before the G-buffer
            // pass; the scene target keeps its contents.
            unsafe {
                self.device.cmd_end_rendering(frame_slot.command_buffer);
            }
            gbuffer.record_gbuffer_begin(frame_slot.command_buffer);
            let gbuffer_clear = vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
            };
            let gbuffer_colors = [
                gbuffer.albedo_mask_view(),
                gbuffer.normal_roughness_view(),
                gbuffer.motion_view(),
                gbuffer.linear_depth_view(),
            ]
            .map(|view| {
                vk::RenderingAttachmentInfo::default()
                    .image_view(view)
                    .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .clear_value(gbuffer_clear)
            });
            let gbuffer_depth = vk::RenderingAttachmentInfo::default()
                .image_view(swapchain.depth_attachments[image_usize].view())
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::DONT_CARE);
            let gbuffer_info = vk::RenderingInfo::default()
                .render_area(render_area)
                .layer_count(1)
                .color_attachments(&gbuffer_colors)
                .depth_attachment(&gbuffer_depth);
            // SAFETY: the four targets are in attachment layout and the
            // depth attachment holds the scene depth.
            unsafe {
                self.device
                    .cmd_begin_rendering(frame_slot.command_buffer, &gbuffer_info);
            }
            b0_content.record_gbuffer(
                frame_slot.command_buffer,
                frame_plan,
                swapchain.extent,
                frame_slot_index,
                gbuffer,
                jitter,
            )?;
            // SAFETY: the G-buffer rendering instance is ended exactly once.
            unsafe {
                self.device.cmd_end_rendering(frame_slot.command_buffer);
            }
            gbuffer.record_gbuffer_end(frame_slot.command_buffer);
            gbuffer.record_scene_to_swapchain(
                frame_slot.command_buffer,
                swapchain.images[image_usize],
            );
            let hud_colors = [vk::RenderingAttachmentInfo::default()
                .image_view(swapchain.image_views[image_usize])
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE)];
            let hud_depth = vk::RenderingAttachmentInfo::default()
                .image_view(swapchain.depth_attachments[image_usize].view())
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::DONT_CARE);
            let hud_info = vk::RenderingInfo::default()
                .render_area(render_area)
                .layer_count(1)
                .color_attachments(&hud_colors)
                .depth_attachment(&hud_depth);
            // SAFETY: the swapchain image holds the copied scene in
            // attachment layout; the UI overlay draws on top of it.
            unsafe {
                self.device
                    .cmd_begin_rendering(frame_slot.command_buffer, &hud_info);
            }
        }
        self.ui_overlay
            .record(frame_slot.command_buffer, swapchain.extent);
        // SAFETY: a dynamic rendering instance is active on this command
        // buffer and is ended exactly once.
        unsafe {
            self.device.cmd_end_rendering(frame_slot.command_buffer);
        }
        let capture_this_frame = self.capture.as_ref().is_some_and(|capture| {
            capture.request.covers(rendered_frame_index)
                && capture.pending.len() < capture.request.burst_length() as usize
        });
        let mut presentable_layout = vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL;
        // Plan 18: a capture reads the swapchain (HUD included) or one of
        // the G-buffer pass images, which are already in transfer-source
        // layout after the pass.
        let capture_source = self
            .capture
            .as_ref()
            .map_or(DesktopCaptureSourceV1::Color, |capture| {
                capture.request.source
            });
        let gbuffer_capture = match (capture_source, self.gbuffer.as_ref()) {
            (DesktopCaptureSourceV1::Color, _) | (_, None) => None,
            (source, Some(gbuffer)) if gbuffer_this_frame => {
                let image = match source {
                    DesktopCaptureSourceV1::Color => unreachable!("handled above"),
                    DesktopCaptureSourceV1::Scene => GBufferCaptureImageV1::Scene,
                    DesktopCaptureSourceV1::AlbedoMask => GBufferCaptureImageV1::AlbedoMask,
                    DesktopCaptureSourceV1::NormalRoughness => {
                        GBufferCaptureImageV1::NormalRoughness
                    }
                    DesktopCaptureSourceV1::Motion => GBufferCaptureImageV1::Motion,
                    DesktopCaptureSourceV1::LinearDepth => GBufferCaptureImageV1::LinearDepth,
                };
                Some(gbuffer.capture_image(image))
            }
            _ => None,
        };
        if capture_this_frame {
            let byte_count = u64::from(swapchain.extent.width)
                .checked_mul(u64::from(swapchain.extent.height))
                .and_then(|pixels| pixels.checked_mul(4))
                .ok_or(DesktopAdapterError::CounterOverflow)?;
            let buffer = BufferAllocation::new(
                &self.instance,
                self.physical_device,
                &self.device,
                byte_count,
                vk::BufferUsageFlags::TRANSFER_DST,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            let (capture_image, capture_format) =
                gbuffer_capture.unwrap_or((swapchain.images[image_usize], swapchain.format));
            let to_transfer = [vk::ImageMemoryBarrier2::default()
                .src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
                .dst_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                .dst_access_mask(vk::AccessFlags2::TRANSFER_READ)
                .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                .image(swapchain.images[image_usize])
                .subresource_range(subresource)];
            let to_transfer_dependency =
                vk::DependencyInfo::default().image_memory_barriers(&to_transfer);
            let region = [vk::BufferImageCopy::default()
                .buffer_offset(0)
                .buffer_row_length(0)
                .buffer_image_height(0)
                .image_subresource(
                    vk::ImageSubresourceLayers::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .mip_level(0)
                        .base_array_layer(0)
                        .layer_count(1),
                )
                .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(vk::Extent3D {
                    width: swapchain.extent.width,
                    height: swapchain.extent.height,
                    depth: 1,
                })];
            // SAFETY: the swapchain was created with transfer-source usage for
            // this run, rendering has ended, the destination buffer covers the
            // tightly packed image, and it outlives the submission.
            unsafe {
                if gbuffer_capture.is_none() {
                    self.device
                        .cmd_pipeline_barrier2(frame_slot.command_buffer, &to_transfer_dependency);
                }
                self.device.cmd_copy_image_to_buffer(
                    frame_slot.command_buffer,
                    capture_image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    buffer.buffer,
                    &region,
                );
            }
            if gbuffer_capture.is_none() {
                presentable_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            }
            if let Some(capture) = self.capture.as_mut() {
                capture.pending.push((
                    PendingFrameCapture {
                        rendered_frame_index,
                        extent: [swapchain.extent.width, swapchain.extent.height],
                        format: capture_format,
                    },
                    buffer,
                ));
            }
        }
        let to_present = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(if capture_this_frame {
                vk::PipelineStageFlags2::TRANSFER
            } else {
                vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT
            })
            .src_access_mask(if capture_this_frame {
                vk::AccessFlags2::TRANSFER_READ
            } else {
                vk::AccessFlags2::COLOR_ATTACHMENT_WRITE
            })
            .dst_stage_mask(vk::PipelineStageFlags2::NONE)
            .dst_access_mask(vk::AccessFlags2::NONE)
            .old_layout(presentable_layout)
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
            dynamic_surface_uploads: dynamic_surface_uploads.uploads,
            dynamic_surface_upload_bytes: dynamic_surface_uploads.bytes,
            dynamic_surface_draws,
            particle_surface_uploads: particle_uploads.uploads,
            particle_surface_upload_bytes: particle_uploads.bytes,
            particle_surface_recorded,
            submerged: submerged_level.is_some(),
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
            // The command buffer and profiling queries were already submitted
            // before presentation reported an out-of-date swapchain. Count
            // that bounded frame exactly once; returning `None` here made the
            // caller submit a replacement frame while cache, UI and profiler
            // counters retained the completed submission.
            return Ok(Some(submitted_frame));
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
            self.capture.is_some(),
            self.particle_surface_profile.is_some(),
            water_surface_requested(&self.dynamic_surface_profiles),
        )
        .inspect_err(|_| {
            if old_swapchain_retired {
                drop(self.swapchain.take());
            }
        })?;
        // Screen-sized pass targets follow the swapchain extent and format.
        drop(self.fluid.take());
        drop(self.water.take());
        drop(self.gbuffer.take());
        let (fluid, particle_surface_available) = create_fluid_pass(
            &self.instance,
            self.physical_device,
            &self.device,
            self.particle_surface_profile,
            replacement.as_ref(),
            self.frame_slots.len(),
        )?;
        self.fluid = fluid;
        self.water = create_water_pass(
            &self.instance,
            self.physical_device,
            &self.device,
            self.b0_content.as_ref(),
            replacement.as_ref(),
            self.frame_slots.len(),
        )?;
        self.gbuffer = create_gbuffer_pass(
            &self.instance,
            self.physical_device,
            &self.device,
            self.b0_content.as_ref(),
            replacement.as_ref(),
            self.frame_slots.len(),
        )?;
        self.particle_surface_available =
            self.particle_surface_available || particle_surface_available;
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
                        &self.dynamic_surface_profiles,
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
        if replacement_formats != current_formats
            && self.ui_overlay.enabled()
            && let Some((color_format, depth_format)) = replacement_formats
        {
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
        self.swapchain = replacement;
        Ok(())
    }

    /// Reads the completed developer captures in rendered-frame order. Must
    /// be called after [`Self::wait_idle`] so every copy has completed.
    pub(super) fn take_captured_frames(
        &mut self,
    ) -> Result<Vec<DesktopCapturedFrameV1>, DesktopAdapterError> {
        let Some(capture) = self.capture.as_mut() else {
            return Ok(Vec::new());
        };
        let mut frames = Vec::with_capacity(capture.pending.len());
        for (pending, buffer) in capture.pending.drain(..) {
            let byte_count = usize::try_from(
                u64::from(pending.extent[0])
                    .checked_mul(u64::from(pending.extent[1]))
                    .and_then(|pixels| pixels.checked_mul(4))
                    .ok_or(DesktopAdapterError::CounterOverflow)?,
            )
            .map_err(|_| DesktopAdapterError::CounterOverflow)?;
            let mut rgba8 = vec![0_u8; byte_count];
            buffer.read(0, &mut rgba8)?;
            if pending.format == vk::Format::B8G8R8A8_SRGB {
                for pixel in rgba8.chunks_exact_mut(4) {
                    pixel.swap(0, 2);
                }
            }
            frames.push(DesktopCapturedFrameV1 {
                rendered_frame_index: pending.rendered_frame_index,
                extent: pending.extent,
                rgba8,
            });
        }
        Ok(frames)
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
        if let Some(water) = self.water.as_ref() {
            bytes = bytes
                .checked_add(water.allocation_bytes())
                .ok_or(DesktopAdapterError::CounterOverflow)?;
            allocations = allocations
                .checked_add(1)
                .ok_or(DesktopAdapterError::CounterOverflow)?;
        }
        if let Some(gbuffer) = self.gbuffer.as_ref() {
            bytes = bytes
                .checked_add(gbuffer.allocation_bytes())
                .ok_or(DesktopAdapterError::CounterOverflow)?;
            allocations = allocations
                .checked_add(1)
                .ok_or(DesktopAdapterError::CounterOverflow)?;
        }
        if let Some(fluid) = self.fluid.as_ref() {
            let (fluid_bytes, fluid_allocations) = fluid.allocation_stats()?;
            bytes = bytes
                .checked_add(fluid_bytes)
                .ok_or(DesktopAdapterError::CounterOverflow)?;
            allocations = allocations
                .checked_add(fluid_allocations)
                .ok_or(DesktopAdapterError::CounterOverflow)?;
        }
        Ok((bytes, allocations))
    }

    pub(super) const fn particle_surface_available(&self) -> bool {
        self.particle_surface_available
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
            drop(self.fluid.take());
            drop(self.water.take());
            drop(self.gbuffer.take());
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

/// Builds the ADR-102 pass for the current swapchain, or reports it
/// unavailable (no profile, no swapchain, no transfer-source images, or an
/// unsupported target format). Device failures still propagate.
fn create_fluid_pass(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
    profile: Option<ParticleSurfaceProfileV1>,
    swapchain: Option<&SwapchainState>,
    frame_slot_count: usize,
) -> Result<(Option<FluidPassState>, bool), DesktopAdapterError> {
    let (Some(profile), Some(swapchain)) = (profile, swapchain) else {
        return Ok((None, false));
    };
    if !swapchain.transfer_source {
        eprintln!(
            "next_game: PARTICLE_SURFACE_FALLBACK: swapchain images cannot be transfer sources"
        );
        return Ok((None, false));
    }
    if !swapchain.opaque_composite {
        // The composite stage writes alpha 0 on fluid pixels as a capture-only
        // coverage channel; a compositor that reads alpha would show through.
        eprintln!("next_game: PARTICLE_SURFACE_FALLBACK: swapchain composite alpha is not opaque");
        return Ok((None, false));
    }
    match FluidPassState::try_new(
        instance,
        physical_device,
        device,
        profile,
        swapchain.format,
        swapchain.depth_format,
        swapchain.extent,
        frame_slot_count,
    )? {
        Ok(fluid) => Ok((Some(fluid), true)),
        Err(reason) => {
            eprintln!("next_game: PARTICLE_SURFACE_FALLBACK: {reason:?}");
            Ok((None, false))
        }
    }
}

fn water_surface_requested(profiles: &[DynamicSurfaceProfileV1]) -> bool {
    profiles
        .iter()
        .any(|profile| profile.shading == DynamicSurfaceShadingV1::WaterSurface)
}

/// Plan 13: builds the water pass when a `WaterSurface` ring is declared
/// and the swapchain supports it; otherwise the WL1 material draws inside
/// the world pass and the fallback is printed once.
fn create_water_pass(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
    b0_content: Option<&B0GpuContent>,
    swapchain: Option<&SwapchainState>,
    frame_slot_count: usize,
) -> Result<Option<WaterPassState>, DesktopAdapterError> {
    let (Some(content), Some(swapchain)) = (b0_content, swapchain) else {
        return Ok(None);
    };
    if !content.has_water_surface_rings() {
        return Ok(None);
    }
    let Some((frame_layout, texture_layout, shadow_layout)) = content.water_pass_layouts() else {
        eprintln!("next_game: WATER_PASS_FALLBACK: no shadow map for the water pass");
        return Ok(None);
    };
    match WaterPassState::try_new(
        instance,
        physical_device,
        device,
        swapchain.format,
        swapchain.depth_format,
        swapchain.extent,
        frame_slot_count,
        frame_layout,
        texture_layout,
        shadow_layout,
        swapchain.transfer_source,
        swapchain.depth_sampled,
    )? {
        Ok(water) => Ok(Some(water)),
        Err(reason) => {
            eprintln!("next_game: WATER_PASS_FALLBACK: {reason}");
            Ok(None)
        }
    }
}

/// Plan 18: the DLSS-ready outputs pass over the current swapchain; the
/// declared fallback prints `GBUFFER_PASS_FALLBACK` and renders straight
/// into the swapchain.
fn create_gbuffer_pass(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
    b0_content: Option<&B0GpuContent>,
    swapchain: Option<&SwapchainState>,
    frame_slot_count: usize,
) -> Result<Option<GBufferPassState>, DesktopAdapterError> {
    let (Some(content), Some(swapchain)) = (b0_content, swapchain) else {
        return Ok(None);
    };
    if !swapchain.transfer_destination {
        eprintln!(
            "next_game: GBUFFER_PASS_FALLBACK: swapchain images carry no transfer-destination usage"
        );
        return Ok(None);
    }
    match GBufferPassState::try_new(
        instance,
        physical_device,
        device,
        swapchain.format,
        swapchain.depth_format,
        swapchain.extent,
        frame_slot_count,
        content.texture_layout(),
        swapchain.transfer_source,
    )? {
        Ok(gbuffer) => Ok(Some(gbuffer)),
        Err(reason) => {
            eprintln!("next_game: GBUFFER_PASS_FALLBACK: {reason}");
            Ok(None)
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
    capture_requested: bool,
    particle_surface_requested: bool,
    water_surface_requested: bool,
) -> Result<Option<SwapchainState>, DesktopAdapterError> {
    debug_assert!(!*old_swapchain_retired);
    // SAFETY: physical device and surface share a live instance.
    let capabilities = unsafe {
        surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?
    };
    let mut image_usage = vk::ImageUsageFlags::COLOR_ATTACHMENT;
    let transfer_source_supported = capabilities
        .supported_usage_flags
        .contains(vk::ImageUsageFlags::TRANSFER_SRC);
    if capture_requested && !transfer_source_supported {
        return Err(DesktopAdapterError::FrameCaptureUnsupported);
    }
    // The particle surface pass copies the opaque scene colour for
    // refraction; without transfer-source images it reports itself
    // unavailable instead of failing the run (ADR-102 fallback).
    if (capture_requested || particle_surface_requested || water_surface_requested)
        && transfer_source_supported
    {
        image_usage |= vk::ImageUsageFlags::TRANSFER_SRC;
    }
    // Plan 18: the HUD-less scene target is copied into the swapchain image
    // before the UI overlay; without transfer-destination images the frame
    // renders straight into the swapchain (declared fallback).
    if capabilities
        .supported_usage_flags
        .contains(vk::ImageUsageFlags::TRANSFER_DST)
    {
        image_usage |= vk::ImageUsageFlags::TRANSFER_DST;
    }
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
        .image_usage(image_usage)
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
        opaque_composite: composite_alpha == vk::CompositeAlphaFlagsKHR::OPAQUE,
        transfer_source: image_usage.contains(vk::ImageUsageFlags::TRANSFER_SRC),
        transfer_destination: image_usage.contains(vk::ImageUsageFlags::TRANSFER_DST),
        depth_sampled: false,
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
        // SAFETY: image belongs to the swapchain and view metadata matches its format.
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
    state.depth_sampled = state.depth_attachments.iter().all(DepthAttachment::sampled);
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
