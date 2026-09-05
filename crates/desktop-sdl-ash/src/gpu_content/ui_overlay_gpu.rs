//! Device-owned resources and per-frame orchestration for the optional
//! semantic UI overlay (ADR-044 minimal widget adapter).
//!
//! `UiOverlayGpu` is fully self-contained: its own checked-in position/UV
//! shader suite and pipeline, its own descriptor layouts, pool,
//! sampler, identity frame uniform and a fixed NDC fullscreen quad. The CPU
//! raster produced by `next_presentation::rasterize_semantic_ui` is uploaded
//! as one RGBA8 texture sampled with straight-alpha blending after the B0
//! scene. `UiOverlayState` owns the optional text resolver, the upload-cache
//! key and the failure counters so an overlay failure degrades to "no
//! overlay" with a stable diagnostic instead of failing the frame.

use ash::vk;
use next_contracts::ids::ContentHash;
use next_contracts::localization::TextCatalogV1;
use next_contracts::presentation::{PresentationSnapshotV3, QuantizedPresentationTransformV1};
use next_contracts::project::domain_hash;
use next_presentation::{TextCatalogResolverV1, UiOverlayImageV1, rasterize_semantic_ui};

use super::B0GpuContentError;
use super::pipeline::{PipelineState, draw_push_constant_bytes, identity_matrix_bytes};
use super::resources::{BufferAllocation, TextureResource};

const UI_OVERLAY_VERTEX_COUNT: u32 = 6;
const UI_OVERLAY_VERTEX_BUFFER_BYTES: usize = 120;
const UI_OVERLAY_FRAME_UNIFORM_SIZE: vk::DeviceSize = 208;

/// NDC fullscreen triangle pair (position xyz + uv) for the private UI shader
/// interface. With a positive-height viewport NDC y=-1 is the
/// top framebuffer row, and texel row 0 of the CPU raster is the top row, so
/// `(-1,-1) -> uv (0,0)` keeps panel margins visually at the top-left.
const UI_OVERLAY_QUAD: [f32; 30] = [
    -1.0, -1.0, 0.5, 0.0, 0.0, //
    1.0, -1.0, 0.5, 1.0, 0.0, //
    1.0, 1.0, 0.5, 1.0, 1.0, //
    -1.0, -1.0, 0.5, 0.0, 0.0, //
    1.0, 1.0, 0.5, 1.0, 1.0, //
    -1.0, 1.0, 0.5, 0.0, 1.0,
];

fn overlay_quad_bytes() -> [u8; UI_OVERLAY_VERTEX_BUFFER_BYTES] {
    let mut bytes = [0_u8; UI_OVERLAY_VERTEX_BUFFER_BYTES];
    for (destination, value) in bytes.chunks_exact_mut(4).zip(UI_OVERLAY_QUAD) {
        destination.copy_from_slice(&value.to_le_bytes());
    }
    bytes
}

/// Per-frame overlay orchestration owned by the graphics context.
///
/// The overlay is strictly optional: a missing or invalid catalog set, a GPU
/// object creation failure, or an upload failure disables drawing (or keeps
/// the last good texture) and increments `failures` instead of failing the
/// frame, per SPEC-18's bounded-fallback rule for optional presentation.
pub(crate) struct UiOverlayState {
    resolver: Option<TextCatalogResolverV1>,
    gpu: Option<UiOverlayGpu>,
    overlay_key: Option<ContentHash>,
    draw_ready: bool,
    text_scale: u32,
    subtitles_enabled: bool,
    frames: u64,
    updates: u64,
    failures: u64,
}

impl UiOverlayState {
    #[allow(
        clippy::too_many_arguments,
        reason = "the private adapter boundary keeps Vulkan ownership inputs explicit"
    )]
    pub(crate) fn new(
        text_catalogs: &[TextCatalogV1],
        locale: &str,
        text_scale_milli: u32,
        subtitles_enabled: bool,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        queue: vk::Queue,
        queue_family_index: u32,
        color_format: vk::Format,
        depth_format: vk::Format,
    ) -> Self {
        let mut state = Self {
            resolver: None,
            gpu: None,
            overlay_key: None,
            draw_ready: false,
            text_scale: next_contracts::preferences::text_scale_from_milli(text_scale_milli),
            subtitles_enabled,
            frames: 0,
            updates: 0,
            failures: 0,
        };
        if text_catalogs.is_empty() {
            return state;
        }
        state.resolver = match TextCatalogResolverV1::new(text_catalogs.to_vec(), locale) {
            Ok(resolver) => Some(resolver),
            Err(_) => {
                state.failures = state.failures.saturating_add(1);
                None
            }
        };
        if state.resolver.is_some() {
            match UiOverlayGpu::new(
                instance,
                physical_device,
                device,
                queue,
                queue_family_index,
                color_format,
                depth_format,
            ) {
                Ok(gpu) => state.gpu = Some(gpu),
                Err(_) => state.failures = state.failures.saturating_add(1),
            }
        }
        state
    }

    /// Recreates device objects after a swapchain format change. The caller
    /// has already idled the device; the cached raster key is reset so the
    /// next frame re-rasterizes and re-uploads.
    #[allow(
        clippy::too_many_arguments,
        reason = "the private adapter boundary keeps Vulkan ownership inputs explicit"
    )]
    pub(crate) fn recreate(
        &mut self,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        queue: vk::Queue,
        queue_family_index: u32,
        color_format: vk::Format,
        depth_format: vk::Format,
    ) {
        if self.resolver.is_none() {
            return;
        }
        match UiOverlayGpu::new(
            instance,
            physical_device,
            device,
            queue,
            queue_family_index,
            color_format,
            depth_format,
        ) {
            Ok(gpu) => {
                self.gpu = Some(gpu);
                self.overlay_key = None;
                self.draw_ready = false;
            }
            Err(_) => {
                self.gpu = None;
                self.draw_ready = false;
                self.failures = self.failures.saturating_add(1);
            }
        }
    }

    #[must_use]
    pub(crate) const fn enabled(&self) -> bool {
        self.resolver.is_some()
    }

    /// Re-rasterizes and re-uploads the overlay only when the record set or
    /// the target extent changed since the last accepted upload.
    pub(crate) fn update(
        &mut self,
        snapshot: &PresentationSnapshotV3,
        extent: vk::Extent2D,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        queue: vk::Queue,
        queue_family_index: u32,
    ) {
        let Some(resolver) = self.resolver.as_ref() else {
            return;
        };
        let mut preimage = Vec::with_capacity(8_usize.saturating_add(32));
        preimage.extend_from_slice(&extent.width.to_le_bytes());
        preimage.extend_from_slice(&extent.height.to_le_bytes());
        let mut record_count = 0_usize;
        for record in snapshot.semantic_ui_records() {
            preimage.extend_from_slice(record.canonical_hash.as_bytes());
            record_count = record_count.saturating_add(1);
        }
        let key = domain_hash("nextengine.ui-overlay-key.v1", &preimage);
        if self.overlay_key == Some(key) {
            return;
        }
        // Content changed: the new key is accepted exactly once, so a
        // persistent failure degrades to a bounded "no overlay" fallback
        // instead of stalling every frame on a re-upload attempt.
        self.overlay_key = Some(key);
        self.draw_ready = false;
        if record_count == 0 {
            return;
        }
        let records = snapshot.semantic_ui_records().cloned().collect::<Vec<_>>();
        // Local PresentationOnly subtitle preference (A5): subtitle elements
        // are dropped from the raster when disabled; the semantic snapshot
        // itself stays deterministic and unchanged.
        let records = if self.subtitles_enabled {
            records
        } else {
            records
                .into_iter()
                .filter(|record| {
                    record.element.role != next_contracts::presentation::UiElementRoleV1::Subtitle
                })
                .collect()
        };
        let Some(image) = rasterize_semantic_ui(
            &records,
            resolver,
            extent.width,
            extent.height,
            self.text_scale,
        ) else {
            return;
        };
        let Some(gpu) = self.gpu.as_mut() else {
            return;
        };
        match gpu.update(instance, physical_device, queue, queue_family_index, &image) {
            Ok(()) => {
                self.updates = self.updates.saturating_add(1);
                self.draw_ready = true;
            }
            Err(_) => {
                self.failures = self.failures.saturating_add(1);
            }
        }
    }

    /// Records the overlay draw into the active dynamic rendering instance.
    pub(crate) fn record(&mut self, command_buffer: vk::CommandBuffer, extent: vk::Extent2D) {
        if !self.draw_ready {
            return;
        }
        let Some(gpu) = self.gpu.as_ref() else {
            self.draw_ready = false;
            return;
        };
        match gpu.record(command_buffer, extent) {
            Ok(()) => self.frames = self.frames.saturating_add(1),
            Err(_) => {
                self.failures = self.failures.saturating_add(1);
                self.draw_ready = false;
            }
        }
    }

    #[must_use]
    pub(crate) fn allocation_stats(&self) -> (u64, u64) {
        let Some(gpu) = self.gpu.as_ref() else {
            return (0, 0);
        };
        gpu.allocation_stats()
    }

    /// Returns `(frames drawn, successful uploads, bounded failures)`.
    #[must_use]
    pub(crate) const fn counters(&self) -> (u64, u64, u64) {
        (self.frames, self.updates, self.failures)
    }

    /// Drops GPU children while the device is still alive. The graphics
    /// context calls this before explicit device destruction.
    pub(crate) fn teardown(&mut self) {
        drop(self.gpu.take());
    }
}

/// Device-owned overlay resources. Field declaration order keeps the pipeline
/// ahead of descriptor objects and backing storage for child-before-parent
/// teardown.
struct UiOverlayGpu {
    pipeline: PipelineState,
    pool: vk::DescriptorPool,
    sampler: vk::Sampler,
    texture_layout: vk::DescriptorSetLayout,
    frame_layout: vk::DescriptorSetLayout,
    texture_set: vk::DescriptorSet,
    frame_set: vk::DescriptorSet,
    texture: Option<TextureResource>,
    vertices: BufferAllocation,
    frame_uniform: BufferAllocation,
    device: ash::Device,
}

impl UiOverlayGpu {
    #[allow(
        clippy::too_many_arguments,
        reason = "the private adapter boundary keeps Vulkan ownership inputs explicit"
    )]
    fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        queue: vk::Queue,
        queue_family_index: u32,
        color_format: vk::Format,
        depth_format: vk::Format,
    ) -> Result<Self, B0GpuContentError> {
        let _ = queue;
        let _ = queue_family_index;
        let mut guard = OverlayObjectGuard::new(device);
        let frame_bindings = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)];
        let texture_bindings = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)];
        let frame_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&frame_bindings);
        let texture_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&texture_bindings);
        // SAFETY: bindings are closed overlay values and no pointer is retained.
        guard.frame_layout =
            unsafe { device.create_descriptor_set_layout(&frame_layout_info, None) }?;
        // SAFETY: same ownership conditions as the frame layout.
        guard.texture_layout =
            unsafe { device.create_descriptor_set_layout(&texture_layout_info, None) }?;
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::NEAREST)
            .min_filter(vk::Filter::NEAREST)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .min_lod(0.0)
            .max_lod(0.0);
        // SAFETY: the sampler uses only core, non-anisotropic features.
        guard.sampler = unsafe { device.create_sampler(&sampler_info, None) }?;
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
            },
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(2)
            .pool_sizes(&pool_sizes);
        // SAFETY: pool sizes exactly cover the two immutable overlay sets.
        guard.pool = unsafe { device.create_descriptor_pool(&pool_info, None) }?;
        let layouts = [guard.frame_layout, guard.texture_layout];
        let allocation_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(guard.pool)
            .set_layouts(&layouts);
        // SAFETY: pool and both layouts are live and owned by this device.
        let sets = unsafe { device.allocate_descriptor_sets(&allocation_info) }?;
        let (frame_set, texture_set) = (sets[0], sets[1]);

        let frame_uniform = BufferAllocation::new(
            instance,
            physical_device,
            device,
            UI_OVERLAY_FRAME_UNIFORM_SIZE,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        frame_uniform.write(0, &identity_matrix_bytes())?;
        let vertices = BufferAllocation::new(
            instance,
            physical_device,
            device,
            UI_OVERLAY_VERTEX_BUFFER_BYTES as vk::DeviceSize,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        vertices.write(0, &overlay_quad_bytes())?;

        let frame_info = [vk::DescriptorBufferInfo::default()
            .buffer(frame_uniform.buffer)
            .offset(0)
            .range(UI_OVERLAY_FRAME_UNIFORM_SIZE)];
        let frame_writes = [vk::WriteDescriptorSet::default()
            .dst_set(frame_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(&frame_info)];
        // SAFETY: destination set and uniform buffer are live; Vulkan copies
        // descriptor values during this call.
        unsafe { device.update_descriptor_sets(&frame_writes, &[]) };

        let pipeline = PipelineState::new_ui_overlay(
            device,
            color_format,
            depth_format,
            guard.frame_layout,
            guard.texture_layout,
        )?;
        let gpu = Self {
            pipeline,
            pool: guard.pool,
            sampler: guard.sampler,
            texture_layout: guard.texture_layout,
            frame_layout: guard.frame_layout,
            texture_set,
            frame_set,
            texture: None,
            vertices,
            frame_uniform,
            device: device.clone(),
        };
        guard.disarm();
        Ok(gpu)
    }

    /// Replaces the sampled overlay texture with one freshly rasterized image.
    ///
    /// The device is idled first: an in-flight frame may still reference the
    /// previous texture, and UI content changes are rare enough that a
    /// bounded idle is the simplest safe replacement point.
    fn update(
        &mut self,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        queue: vk::Queue,
        queue_family_index: u32,
        image: &UiOverlayImageV1,
    ) -> Result<(), B0GpuContentError> {
        if image.width == 0 || image.height == 0 {
            return Err(B0GpuContentError::InvalidFramePlan(
                "ui overlay image extent must be non-zero",
            ));
        }
        // SAFETY: device is live; idling proves no in-flight command buffer
        // samples the texture about to be replaced.
        unsafe { self.device.device_wait_idle()? };
        let extent = vk::Extent3D {
            width: image.width,
            height: image.height,
            depth: 1,
        };
        let texture = TextureResource::new(instance, physical_device, &self.device, extent)?;
        let staging = BufferAllocation::new(
            instance,
            physical_device,
            &self.device,
            image.rgba.len() as vk::DeviceSize,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        staging.write(0, &image.rgba)?;
        let upload = upload_overlay_texture(
            &self.device,
            queue,
            queue_family_index,
            &staging,
            &texture,
            extent,
        );
        if let Err(error) = upload {
            if matches!(&error, B0GpuContentError::UploadCompletionUnknown(_)) {
                // Mirror the B0 upload rule: when submission completion cannot
                // be established, leak the referenced allocations until device
                // teardown instead of risking use-after-free.
                std::mem::forget(staging);
                std::mem::forget(texture);
            }
            return Err(error);
        }
        drop(staging);
        let image_info = [vk::DescriptorImageInfo::default()
            .sampler(self.sampler)
            .image_view(texture.view())
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
        let writes = [vk::WriteDescriptorSet::default()
            .dst_set(self.texture_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_info)];
        // SAFETY: set, sampler and view are live and the device is idle;
        // descriptor payload is copied synchronously.
        unsafe { self.device.update_descriptor_sets(&writes, &[]) };
        self.texture = Some(texture);
        Ok(())
    }

    /// Records the fullscreen overlay draw. A dynamic rendering instance must
    /// already be active on this command buffer.
    fn record(
        &self,
        command_buffer: vk::CommandBuffer,
        extent: vk::Extent2D,
    ) -> Result<(), B0GpuContentError> {
        if extent.width == 0 || extent.height == 0 {
            return Err(B0GpuContentError::InvalidFramePlan(
                "ui overlay extent must be non-zero",
            ));
        }
        if self.texture.is_none() {
            return Err(B0GpuContentError::ResourceMissing("ui overlay texture"));
        }
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        }];
        let vertex_buffers = [self.vertices.buffer];
        let vertex_offsets = [0];
        let frame_sets = [self.frame_set];
        let texture_sets = [self.texture_set];
        let push_constants = draw_push_constant_bytes(
            QuantizedPresentationTransformV1::default(),
            [u16::MAX, u16::MAX, u16::MAX, u16::MAX],
            [0.0; 4],
        );
        // SAFETY: all bound objects belong to the same live device, the
        // command buffer is recording inside dynamic rendering, the quad
        // buffer exactly covers six fixed-stride vertices, and the push bytes
        // cover the declared 80-byte range.
        unsafe {
            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );
            self.device.cmd_set_viewport(command_buffer, 0, &viewports);
            self.device.cmd_set_scissor(command_buffer, 0, &scissors);
            self.device.cmd_bind_vertex_buffers(
                command_buffer,
                0,
                &vertex_buffers,
                &vertex_offsets,
            );
            self.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0,
                &frame_sets,
                &[],
            );
            self.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                1,
                &texture_sets,
                &[],
            );
            self.device.cmd_push_constants(
                command_buffer,
                self.pipeline.layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                &push_constants,
            );
            self.device
                .cmd_draw(command_buffer, UI_OVERLAY_VERTEX_COUNT, 1, 0, 0);
        }
        Ok(())
    }

    fn allocation_stats(&self) -> (u64, u64) {
        let mut bytes = self
            .frame_uniform
            .allocation_size()
            .saturating_add(self.vertices.allocation_size());
        let mut count = 2_u64;
        if let Some(texture) = self.texture.as_ref() {
            bytes = bytes.saturating_add(texture.allocation_size());
            count = count.saturating_add(1);
        }
        (bytes, count)
    }
}

impl Drop for UiOverlayGpu {
    fn drop(&mut self) {
        // SAFETY: the graphics context drops this value only while the device
        // is idle and alive; the pipeline field destroys itself afterwards.
        unsafe {
            self.device.destroy_descriptor_pool(self.pool, None);
            self.device.destroy_sampler(self.sampler, None);
            self.device
                .destroy_descriptor_set_layout(self.texture_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.frame_layout, None);
        }
    }
}

/// Destroys partially created overlay objects on any construction error.
struct OverlayObjectGuard {
    device: ash::Device,
    pool: vk::DescriptorPool,
    sampler: vk::Sampler,
    texture_layout: vk::DescriptorSetLayout,
    frame_layout: vk::DescriptorSetLayout,
    armed: bool,
}

impl OverlayObjectGuard {
    fn new(device: &ash::Device) -> Self {
        Self {
            device: device.clone(),
            pool: vk::DescriptorPool::null(),
            sampler: vk::Sampler::null(),
            texture_layout: vk::DescriptorSetLayout::null(),
            frame_layout: vk::DescriptorSetLayout::null(),
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for OverlayObjectGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        // SAFETY: every non-null handle was created by this device during the
        // failed construction and has no external dependants.
        unsafe {
            if self.pool != vk::DescriptorPool::null() {
                self.device.destroy_descriptor_pool(self.pool, None);
            }
            if self.sampler != vk::Sampler::null() {
                self.device.destroy_sampler(self.sampler, None);
            }
            if self.texture_layout != vk::DescriptorSetLayout::null() {
                self.device
                    .destroy_descriptor_set_layout(self.texture_layout, None);
            }
            if self.frame_layout != vk::DescriptorSetLayout::null() {
                self.device
                    .destroy_descriptor_set_layout(self.frame_layout, None);
            }
        }
    }
}

/// One-shot staging upload of a single RGBA8 overlay texture, mirroring the
/// B0 upload completion rules on a smaller surface.
fn upload_overlay_texture(
    device: &ash::Device,
    queue: vk::Queue,
    queue_family_index: u32,
    staging: &BufferAllocation,
    texture: &TextureResource,
    extent: vk::Extent3D,
) -> Result<(), B0GpuContentError> {
    let pool_info = vk::CommandPoolCreateInfo::default()
        .queue_family_index(queue_family_index)
        .flags(vk::CommandPoolCreateFlags::TRANSIENT);
    // SAFETY: the queue family belongs to this live logical device.
    let command_pool = unsafe { device.create_command_pool(&pool_info, None) }?;
    let mut upload_completion_known = true;
    let result = (|| {
        let allocation_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        // SAFETY: the transient pool is live and owned for the whole upload.
        let command_buffer = unsafe { device.allocate_command_buffers(&allocation_info) }?[0];
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        // SAFETY: the new primary command buffer is recorded exactly once.
        unsafe { device.begin_command_buffer(command_buffer, &begin_info) }?;
        let subresource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let to_transfer = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::NONE)
            .src_access_mask(vk::AccessFlags2::NONE)
            .dst_stage_mask(vk::PipelineStageFlags2::TRANSFER)
            .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .image(texture.image())
            .subresource_range(subresource_range)];
        let dependency = vk::DependencyInfo::default().image_memory_barriers(&to_transfer);
        // SAFETY: the freshly created image starts in UNDEFINED and the
        // barrier targets its sole color mip and layer.
        unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency) };
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
            .image_extent(extent)];
        // SAFETY: the staging payload is exactly extent.width * height * 4
        // bytes and the image is in TRANSFER_DST layout.
        unsafe {
            device.cmd_copy_buffer_to_image(
                command_buffer,
                staging.buffer,
                texture.image(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &region,
            );
        }
        let to_shader = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
            .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::FRAGMENT_SHADER)
            .dst_access_mask(vk::AccessFlags2::SHADER_SAMPLED_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image(texture.image())
            .subresource_range(subresource_range)];
        let dependency = vk::DependencyInfo::default().image_memory_barriers(&to_shader);
        // SAFETY: the copy above populated the image earlier in this command
        // buffer and the image stays alive through queue completion.
        unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency) };
        // SAFETY: recording is active and all commands reference live objects.
        unsafe { device.end_command_buffer(command_buffer) }?;

        let command_buffers = [command_buffer];
        let submit_infos = [vk::SubmitInfo::default().command_buffers(&command_buffers)];
        let fence_info = vk::FenceCreateInfo::default();
        // SAFETY: fence creation retains no host pointer.
        let fence = unsafe { device.create_fence(&fence_info, None) }?;
        // SAFETY: the command buffer is executable and the fence is used for
        // this one submission only.
        let submit_result = unsafe { device.queue_submit(queue, &submit_infos, fence) };
        let wait_result: Result<(), B0GpuContentError> = match submit_result {
            Err(error) => Err(error.into()),
            Ok(()) => {
                // SAFETY: the fence stays live for this unbounded setup wait.
                match unsafe { device.wait_for_fences(&[fence], true, u64::MAX) } {
                    Ok(()) => Ok(()),
                    Err(vk::Result::ERROR_DEVICE_LOST) => Err(vk::Result::ERROR_DEVICE_LOST.into()),
                    Err(wait_error) => {
                        // SAFETY: fall back to idling the queue before any
                        // upload object is destroyed.
                        match unsafe { device.queue_wait_idle(queue) } {
                            Ok(()) => Err(wait_error.into()),
                            Err(vk::Result::ERROR_DEVICE_LOST) => {
                                Err(vk::Result::ERROR_DEVICE_LOST.into())
                            }
                            Err(idle_error) => {
                                upload_completion_known = false;
                                Err(B0GpuContentError::UploadCompletionUnknown(idle_error))
                            }
                        }
                    }
                }
            }
        };
        if upload_completion_known {
            // SAFETY: submission never started, has completed, or the device
            // was lost; the fence is no longer in use.
            unsafe { device.destroy_fence(fence, None) };
        }
        wait_result?;
        Ok(())
    })();
    if upload_completion_known {
        // SAFETY: successful upload waited for completion; failure paths made
        // the work idle, were lost, or deliberately leak the pool.
        unsafe { device.destroy_command_pool(command_pool, None) };
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_quad_matches_the_fixed_b0_vertex_interface() {
        let bytes = overlay_quad_bytes();
        assert_eq!(bytes.len() as u32 % 20, 0);
        assert_eq!(bytes.len() / 20, UI_OVERLAY_VERTEX_COUNT as usize);
        // First vertex: NDC (-1,-1, 0.5) with uv (0,0) — framebuffer top-left.
        let first: Vec<f32> = bytes[..20]
            .chunks_exact(4)
            .map(|word| f32::from_le_bytes(word.try_into().expect("one component")))
            .collect();
        assert_eq!(first, vec![-1.0, -1.0, 0.5, 0.0, 0.0]);
    }

    #[test]
    fn overlay_key_commits_to_extent_and_record_hashes() {
        // The key preimage layout is extent.le ++ record canonical hashes; a
        // stable domain tag keeps it separate from other engine hashes.
        let mut preimage = Vec::new();
        preimage.extend_from_slice(&960_u32.to_le_bytes());
        preimage.extend_from_slice(&540_u32.to_le_bytes());
        preimage.extend_from_slice(ContentHash::from_bytes([0x42; 32]).as_bytes());
        let key = domain_hash("nextengine.ui-overlay-key.v1", &preimage);
        assert_ne!(key, ContentHash::default());
        let mut changed = preimage.clone();
        *changed.last_mut().expect("non-empty preimage") ^= 0x01;
        assert_ne!(key, domain_hash("nextengine.ui-overlay-key.v1", &changed));
    }
}
