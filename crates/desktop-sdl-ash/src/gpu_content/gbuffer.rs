//! Water look L8 (plan `continuum-water/18`): the DLSS-ready outputs of the
//! desktop adapter. The scene passes render into an offscreen colour target
//! (HUD-less colour) that the frame copies to the swapchain before the UI
//! overlay; a separate `gbuffer` suite then draws the frame plan again into
//! four `32`-bit targets: albedo + group mask, normal + roughness, screen
//! motion vectors and linear depth. Motion vectors come from the previous
//! rendered frame's model matrix of the same draw key and the previous
//! jittered view-projection. Everything here is renderer-local: no frame
//! plan field, no B0 shader interface and no gameplay root changes.

use std::collections::BTreeMap;

use ash::vk;
use next_contracts::project::AssetRevisionRefV1;

use super::B0GpuContentError;
use super::pipeline::{PipelineState, RasterFixedStateV1};
use super::resources::{BufferAllocation, ImageAllocation};

/// Albedo (rgb) and the group mask (`group / 255` in alpha).
pub(crate) const GBUFFER_ALBEDO_MASK_FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;
/// World normal encoded `0.5 n + 0.5` (rgb) and roughness (alpha).
pub(crate) const GBUFFER_NORMAL_ROUGHNESS_FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;
/// Screen motion in pixels from the previous frame to this one, `+x` right
/// and `+y` down.
pub(crate) const GBUFFER_MOTION_FORMAT: vk::Format = vk::Format::R16G16_SFLOAT;
/// View-space distance along the camera axis in metres.
pub(crate) const GBUFFER_LINEAR_DEPTH_FORMAT: vk::Format = vk::Format::R32_SFLOAT;
/// The four G-buffer attachment formats in attachment order.
pub(crate) const GBUFFER_COLOR_FORMATS: [vk::Format; 4] = [
    GBUFFER_ALBEDO_MASK_FORMAT,
    GBUFFER_NORMAL_ROUGHNESS_FORMAT,
    GBUFFER_MOTION_FORMAT,
    GBUFFER_LINEAR_DEPTH_FORMAT,
];
/// Draws with a previous-model entry per frame; later draws are skipped by
/// the G-buffer pass and counted.
pub(crate) const GBUFFER_MAX_DRAWS: u32 = 4_096;
/// `view_projection`, `previous_view_projection`, `viewport`, `jitter`.
pub(crate) const GBUFFER_UNIFORM_SIZE: vk::DeviceSize = 160;
/// `model`, `base_color_factor`, `meta` (draw index, group, roughness, 0).
pub(crate) const GBUFFER_PUSH_CONSTANT_SIZE: u32 = 96;
const MODEL_MATRIX_BYTES: vk::DeviceSize = 64;

/// The engine-level object groups of the mask channel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(crate) enum GBufferGroupV1 {
    Environment = 1,
    Character = 2,
    WaterSurface = 3,
    DynamicSurface = 4,
}

impl GBufferGroupV1 {
    /// A fixed roughness per group; the catalog carries no roughness.
    pub(crate) const fn roughness(self) -> f32 {
        match self {
            Self::Environment => 0.8,
            Self::Character => 0.6,
            Self::WaterSurface => 0.05,
            Self::DynamicSurface => 0.5,
        }
    }
}

/// What identifies a draw across frames: its mesh, material and texture
/// revisions (the scene record hash changes with the pose, so it cannot be
/// the key).
pub(crate) type DrawIdentity = (AssetRevisionRefV1, AssetRevisionRefV1, AssetRevisionRefV1);

/// The key of one draw across frames: its identity and its occurrence
/// among the plan's draws with the same identity (the planner's draw order
/// is deterministic for a snapshot, so instances keep their occurrence).
pub(crate) type DrawHistoryKey = (DrawIdentity, u32);

/// The previous-frame model matrices of the plan's draws, keyed by draw
/// history key. A key absent in the previous frame yields the current
/// matrix (zero object motion); keys not seen in a frame are forgotten.
#[derive(Debug, Default)]
pub(crate) struct MotionHistoryV1 {
    previous: BTreeMap<DrawHistoryKey, [f32; 16]>,
    current: BTreeMap<DrawHistoryKey, [f32; 16]>,
    occurrences: BTreeMap<DrawIdentity, u32>,
    previous_view_projection: Option<[f32; 16]>,
    previous_jitter_pixels: [f32; 2],
}

impl MotionHistoryV1 {
    pub(crate) fn begin_frame(&mut self) {
        self.current.clear();
        self.occurrences.clear();
    }

    /// Records the draw's current matrix and returns its previous one.
    pub(crate) fn resolve(&mut self, identity: DrawIdentity, current: [f32; 16]) -> [f32; 16] {
        let occurrence = self.occurrences.entry(identity).or_insert(0);
        let key = (identity, *occurrence);
        *occurrence = occurrence.saturating_add(1);
        self.current.insert(key, current);
        self.previous.get(&key).copied().unwrap_or(current)
    }

    /// Commits the frame: its draws, view-projection and jitter become the
    /// previous ones of the next frame.
    pub(crate) fn end_frame(&mut self, view_projection: [f32; 16], jitter_pixels: [f32; 2]) {
        std::mem::swap(&mut self.previous, &mut self.current);
        self.current.clear();
        self.previous_view_projection = Some(view_projection);
        self.previous_jitter_pixels = jitter_pixels;
    }

    /// The previous frame's view-projection, or the given one on the first
    /// frame (camera motion zero).
    pub(crate) fn previous_view_projection(&self, current: [f32; 16]) -> [f32; 16] {
        self.previous_view_projection.unwrap_or(current)
    }

    pub(crate) const fn previous_jitter_pixels(&self) -> [f32; 2] {
        self.previous_jitter_pixels
    }
}

/// The G-buffer uniform block bytes (`GBUFFER_UNIFORM_SIZE`).
pub(crate) fn gbuffer_uniform_bytes(
    view_projection: [f32; 16],
    previous_view_projection: [f32; 16],
    extent: vk::Extent2D,
    jitter_pixels: [f32; 2],
    previous_jitter_pixels: [f32; 2],
) -> [u8; GBUFFER_UNIFORM_SIZE as usize] {
    let mut bytes = [0_u8; GBUFFER_UNIFORM_SIZE as usize];
    let mut offset = 0;
    let mut push = |value: f32| {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        offset += 4;
    };
    for value in view_projection {
        push(value);
    }
    for value in previous_view_projection {
        push(value);
    }
    let width = extent.width as f32;
    let height = extent.height as f32;
    push(width);
    push(height);
    push(1.0 / width.max(1.0));
    push(1.0 / height.max(1.0));
    push(jitter_pixels[0]);
    push(jitter_pixels[1]);
    push(previous_jitter_pixels[0]);
    push(previous_jitter_pixels[1]);
    bytes
}

/// The G-buffer push block: the B0 draw bytes followed by `meta`.
pub(crate) fn gbuffer_push_constant_bytes(
    draw_bytes: [u8; super::DRAW_PUSH_CONSTANT_SIZE as usize],
    draw_index: u32,
    group: GBufferGroupV1,
) -> [u8; GBUFFER_PUSH_CONSTANT_SIZE as usize] {
    let mut bytes = [0_u8; GBUFFER_PUSH_CONSTANT_SIZE as usize];
    bytes[..draw_bytes.len()].copy_from_slice(&draw_bytes);
    let meta = [
        draw_index as f32,
        f32::from(group as u8),
        group.roughness(),
        0.0,
    ];
    for (lane, value) in meta.iter().enumerate() {
        let start = draw_bytes.len() + lane * 4;
        bytes[start..start + 4].copy_from_slice(&value.to_le_bytes());
    }
    bytes
}

/// Which image a developer capture reads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GBufferCaptureImageV1 {
    Scene,
    AlbedoMask,
    NormalRoughness,
    Motion,
    LinearDepth,
}

struct Target {
    image: ImageAllocation,
    view: vk::ImageView,
    format: vk::Format,
}

struct GBufferSlot {
    uniform: BufferAllocation,
    previous_models: BufferAllocation,
    set: vk::DescriptorSet,
}

/// Owner of every device object of the pass. Fields drop in dependency
/// order: pipeline, descriptors, views, images, buffers.
pub(crate) struct GBufferPassState {
    device: ash::Device,
    extent: vk::Extent2D,
    pipeline: PipelineState,
    descriptor_pool: vk::DescriptorPool,
    set_layout: vk::DescriptorSetLayout,
    scene_color: Target,
    albedo_mask: Target,
    normal_roughness: Target,
    motion: Target,
    linear_depth: Target,
    slots: Vec<GBufferSlot>,
    history: MotionHistoryV1,
    /// Per-frame state: the view-projection and jitter of the frame being
    /// recorded, committed by `end_frame_draws`.
    frame_view_projection: [f32; 16],
    frame_jitter_pixels: [f32; 2],
}

impl GBufferPassState {
    /// Builds the pass; `Ok(Err(reason))` is the declared fallback (the
    /// frame renders straight into the swapchain without G-buffer), `Err`
    /// a real device failure.
    #[allow(
        clippy::too_many_arguments,
        reason = "Vulkan ownership inputs are explicit at the private adapter boundary"
    )]
    pub(crate) fn try_new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        color_format: vk::Format,
        depth_format: vk::Format,
        extent: vk::Extent2D,
        frame_slot_count: usize,
        texture_layout: vk::DescriptorSetLayout,
        transfer_source: bool,
    ) -> Result<Result<Self, &'static str>, B0GpuContentError> {
        if !transfer_source {
            return Ok(Err("swapchain images carry no transfer-source usage"));
        }
        if extent.width == 0 || extent.height == 0 {
            return Ok(Err("render extent is empty"));
        }
        if frame_slot_count == 0 {
            return Ok(Err("no frame slots"));
        }
        let mut guard = Teardown {
            device: device.clone(),
            layouts: Vec::new(),
            pool: vk::DescriptorPool::null(),
            views: Vec::new(),
            armed: true,
        };
        let target = |format: vk::Format, usage: vk::ImageUsageFlags, guard: &mut Teardown| {
            let image = ImageAllocation::new(
                instance,
                physical_device,
                device,
                vk::Extent3D {
                    width: extent.width,
                    height: extent.height,
                    depth: 1,
                },
                format,
                usage,
            )?;
            let aspect = vk::ImageAspectFlags::COLOR;
            let view_info = vk::ImageViewCreateInfo::default()
                .image(image.image())
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .subresource_range(subresource(aspect));
            // SAFETY: the image is live and uses this exact format.
            let view = unsafe { device.create_image_view(&view_info, None) }?;
            guard.views.push(view);
            Ok::<Target, B0GpuContentError>(Target {
                image,
                view,
                format,
            })
        };
        let scene_color = target(
            color_format,
            vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST,
            &mut guard,
        )?;
        let gbuffer_usage =
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC;
        let albedo_mask = target(GBUFFER_ALBEDO_MASK_FORMAT, gbuffer_usage, &mut guard)?;
        let normal_roughness = target(GBUFFER_NORMAL_ROUGHNESS_FORMAT, gbuffer_usage, &mut guard)?;
        let motion = target(GBUFFER_MOTION_FORMAT, gbuffer_usage, &mut guard)?;
        let linear_depth = target(GBUFFER_LINEAR_DEPTH_FORMAT, gbuffer_usage, &mut guard)?;

        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        // SAFETY: plain layout creation on the live device.
        let set_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None) }?;
        guard.layouts.push(set_layout);

        let slot_count =
            u32::try_from(frame_slot_count).map_err(|_| B0GpuContentError::CountOverflow)?;
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: slot_count,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: slot_count,
            },
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(slot_count)
            .pool_sizes(&pool_sizes);
        // SAFETY: pool sizes exactly cover one set per frame slot.
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None) }?;
        guard.pool = descriptor_pool;
        let layouts = vec![set_layout; frame_slot_count];
        let allocation_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);
        // SAFETY: pool and layouts are live on this device.
        let sets = unsafe { device.allocate_descriptor_sets(&allocation_info) }?;
        let host_visible =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let mut slots = Vec::with_capacity(frame_slot_count);
        for set in sets {
            let uniform = BufferAllocation::new(
                instance,
                physical_device,
                device,
                GBUFFER_UNIFORM_SIZE,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                host_visible,
            )?;
            let previous_models = BufferAllocation::new(
                instance,
                physical_device,
                device,
                vk::DeviceSize::from(GBUFFER_MAX_DRAWS) * MODEL_MATRIX_BYTES,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                host_visible,
            )?;
            let uniform_info = [vk::DescriptorBufferInfo::default()
                .buffer(uniform.buffer)
                .offset(0)
                .range(GBUFFER_UNIFORM_SIZE)];
            let storage_info = [vk::DescriptorBufferInfo::default()
                .buffer(previous_models.buffer)
                .offset(0)
                .range(vk::DeviceSize::from(GBUFFER_MAX_DRAWS) * MODEL_MATRIX_BYTES)];
            let writes = [
                vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&uniform_info),
                vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(&storage_info),
            ];
            // SAFETY: sets and buffers are live; descriptors are copied now.
            unsafe { device.update_descriptor_sets(&writes, &[]) };
            slots.push(GBufferSlot {
                uniform,
                previous_models,
                set,
            });
        }
        let pipeline = PipelineState::new_gbuffer(
            device,
            &GBUFFER_COLOR_FORMATS,
            depth_format,
            set_layout,
            texture_layout,
            GBUFFER_RASTER_FIXED_STATE,
        )?;
        guard.armed = false;
        Ok(Ok(Self {
            device: device.clone(),
            extent,
            pipeline,
            descriptor_pool,
            set_layout,
            scene_color,
            albedo_mask,
            normal_roughness,
            motion,
            linear_depth,
            slots,
            history: MotionHistoryV1::default(),
            frame_view_projection: IDENTITY,
            frame_jitter_pixels: [0.0; 2],
        }))
    }

    pub(crate) fn scene_color_view(&self) -> vk::ImageView {
        self.scene_color.view
    }

    pub(crate) fn scene_color_image(&self) -> vk::Image {
        self.scene_color.image.image()
    }

    pub(crate) fn albedo_mask_view(&self) -> vk::ImageView {
        self.albedo_mask.view
    }

    pub(crate) fn normal_roughness_view(&self) -> vk::ImageView {
        self.normal_roughness.view
    }

    pub(crate) fn motion_view(&self) -> vk::ImageView {
        self.motion.view
    }

    pub(crate) fn linear_depth_view(&self) -> vk::ImageView {
        self.linear_depth.view
    }

    pub(crate) fn pipeline(&self) -> vk::Pipeline {
        self.pipeline.pipeline
    }

    pub(crate) fn layout(&self) -> vk::PipelineLayout {
        self.pipeline.layout
    }

    /// The image and format a capture of `source` reads; every returned
    /// image is in `TRANSFER_SRC_OPTIMAL` after the frame's G-buffer pass.
    pub(crate) fn capture_image(&self, source: GBufferCaptureImageV1) -> (vk::Image, vk::Format) {
        let target = match source {
            GBufferCaptureImageV1::Scene => &self.scene_color,
            GBufferCaptureImageV1::AlbedoMask => &self.albedo_mask,
            GBufferCaptureImageV1::NormalRoughness => &self.normal_roughness,
            GBufferCaptureImageV1::Motion => &self.motion,
            GBufferCaptureImageV1::LinearDepth => &self.linear_depth,
        };
        (target.image.image(), target.format)
    }

    /// Writes the frame's uniform for `frame_slot_index` and opens the
    /// draw history of the frame; returns the descriptor set to bind.
    pub(crate) fn prepare(
        &mut self,
        frame_slot_index: usize,
        view_projection: [f32; 16],
        jitter_pixels: [f32; 2],
    ) -> Result<vk::DescriptorSet, B0GpuContentError> {
        let previous = self.history.previous_view_projection(view_projection);
        let previous_jitter = self.history.previous_jitter_pixels();
        let slot = self
            .slots
            .get(frame_slot_index)
            .ok_or(B0GpuContentError::InvalidFramePlan(
                "frame slot index is outside the G-buffer ring",
            ))?;
        slot.uniform.write(
            0,
            &gbuffer_uniform_bytes(
                view_projection,
                previous,
                self.extent,
                jitter_pixels,
                previous_jitter,
            ),
        )?;
        self.frame_view_projection = view_projection;
        self.frame_jitter_pixels = jitter_pixels;
        self.history.begin_frame();
        Ok(slot.set)
    }

    /// Resolves the previous model of a draw, stores it in the slot's
    /// storage entry `draw_index` and returns the push bytes; `None` when
    /// the draw is beyond the bound (the pass skips it).
    pub(crate) fn push_bytes(
        &mut self,
        frame_slot_index: usize,
        draw_index: u32,
        identity: DrawIdentity,
        model: [f32; 16],
        draw_bytes: [u8; super::DRAW_PUSH_CONSTANT_SIZE as usize],
        group: GBufferGroupV1,
    ) -> Result<Option<[u8; GBUFFER_PUSH_CONSTANT_SIZE as usize]>, B0GpuContentError> {
        if draw_index >= GBUFFER_MAX_DRAWS {
            return Ok(None);
        }
        let previous = self.history.resolve(identity, model);
        let slot = self
            .slots
            .get(frame_slot_index)
            .ok_or(B0GpuContentError::InvalidFramePlan(
                "frame slot index is outside the G-buffer ring",
            ))?;
        let mut bytes = [0_u8; MODEL_MATRIX_BYTES as usize];
        for (lane, value) in previous.iter().enumerate() {
            bytes[lane * 4..lane * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        slot.previous_models.write(
            vk::DeviceSize::from(draw_index) * MODEL_MATRIX_BYTES,
            &bytes,
        )?;
        Ok(Some(gbuffer_push_constant_bytes(
            draw_bytes, draw_index, group,
        )))
    }

    /// Commits the frame's draw history.
    pub(crate) fn end_frame_draws(&mut self) {
        self.history
            .end_frame(self.frame_view_projection, self.frame_jitter_pixels);
    }

    /// Puts the scene colour target into attachment layout at frame start.
    pub(crate) fn record_scene_begin(&self, command_buffer: vk::CommandBuffer) {
        let barriers = [image_barrier(
            self.scene_color.image.image(),
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            (
                vk::PipelineStageFlags2::TRANSFER,
                vk::AccessFlags2::TRANSFER_READ,
            ),
            (
                vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            ),
        )];
        // SAFETY: the image belongs to this pass; no rendering instance is
        // active on the command buffer.
        unsafe {
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&barriers),
            );
        }
    }

    /// Puts the four G-buffer targets into attachment layout.
    pub(crate) fn record_gbuffer_begin(&self, command_buffer: vk::CommandBuffer) {
        let barriers = [
            &self.albedo_mask,
            &self.normal_roughness,
            &self.motion,
            &self.linear_depth,
        ]
        .map(|target| {
            image_barrier(
                target.image.image(),
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                (
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_READ,
                ),
                (
                    vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                    vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                ),
            )
        });
        // SAFETY: the images belong to this pass; no rendering instance is
        // active on the command buffer.
        unsafe {
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&barriers),
            );
        }
    }

    /// Makes the four G-buffer targets readable by a transfer (capture or a
    /// later consumer) after the G-buffer rendering instance ended.
    pub(crate) fn record_gbuffer_end(&self, command_buffer: vk::CommandBuffer) {
        let barriers = [
            &self.albedo_mask,
            &self.normal_roughness,
            &self.motion,
            &self.linear_depth,
        ]
        .map(|target| {
            image_barrier(
                target.image.image(),
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                (
                    vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                    vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                ),
                (
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_READ,
                ),
            )
        });
        // SAFETY: the G-buffer rendering instance has ended.
        unsafe {
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&barriers),
            );
        }
    }

    /// Copies the HUD-less scene colour to the swapchain image (which is in
    /// attachment layout before and after) and leaves the scene target in
    /// transfer-source layout for a capture. No rendering instance may be
    /// active.
    pub(crate) fn record_scene_to_swapchain(
        &self,
        command_buffer: vk::CommandBuffer,
        swapchain_image: vk::Image,
    ) {
        let before = [
            image_barrier(
                self.scene_color.image.image(),
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                (
                    vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                    vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                ),
                (
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_READ,
                ),
            ),
            image_barrier(
                swapchain_image,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                (
                    vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                    vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                ),
                (
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_WRITE,
                ),
            ),
        ];
        let layers = vk::ImageSubresourceLayers::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1);
        let region = [vk::ImageCopy::default()
            .src_subresource(layers)
            .dst_subresource(layers)
            .extent(vk::Extent3D {
                width: self.extent.width,
                height: self.extent.height,
                depth: 1,
            })];
        let after = [image_barrier(
            swapchain_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            (
                vk::PipelineStageFlags2::TRANSFER,
                vk::AccessFlags2::TRANSFER_WRITE,
            ),
            (
                vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                vk::AccessFlags2::COLOR_ATTACHMENT_READ | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            ),
        )];
        // SAFETY: both images share the swapchain format and extent, are in
        // the declared layouts, and no rendering instance is active.
        unsafe {
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&before),
            );
            self.device.cmd_copy_image(
                command_buffer,
                self.scene_color.image.image(),
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                swapchain_image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &region,
            );
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&after),
            );
        }
    }

    pub(crate) fn allocation_bytes(&self) -> vk::DeviceSize {
        self.scene_color.image.allocation_size()
            + self.albedo_mask.image.allocation_size()
            + self.normal_roughness.image.allocation_size()
            + self.motion.image.allocation_size()
            + self.linear_depth.image.allocation_size()
            + self
                .slots
                .iter()
                .map(|slot| slot.uniform.allocation_size() + slot.previous_models.allocation_size())
                .sum::<vk::DeviceSize>()
    }
}

impl Drop for GBufferPassState {
    fn drop(&mut self) {
        // SAFETY: the owner waits for device idle before dropping; children
        // are destroyed before parents (the pipeline state drops itself).
        unsafe {
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device
                .destroy_descriptor_set_layout(self.set_layout, None);
            for target in [
                &self.scene_color,
                &self.albedo_mask,
                &self.normal_roughness,
                &self.motion,
                &self.linear_depth,
            ] {
                self.device.destroy_image_view(target.view, None);
            }
        }
    }
}

/// G-buffer pass: the B0 fixed state (depth tested and written, back-face
/// culling), no blending on any attachment.
const GBUFFER_RASTER_FIXED_STATE: RasterFixedStateV1 = RasterFixedStateV1 {
    blend_enable: false,
    depth_test_enable: true,
    depth_write_enable: true,
    cull_mode: vk::CullModeFlags::BACK,
};

const IDENTITY: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

/// Destroys partially constructed objects when construction fails midway.
struct Teardown {
    device: ash::Device,
    layouts: Vec<vk::DescriptorSetLayout>,
    pool: vk::DescriptorPool,
    views: Vec<vk::ImageView>,
    armed: bool,
}

impl Drop for Teardown {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        // SAFETY: every handle was recorded right after creation and none
        // has been submitted; children go before parents.
        unsafe {
            if self.pool != vk::DescriptorPool::null() {
                self.device.destroy_descriptor_pool(self.pool, None);
            }
            for layout in self.layouts.drain(..) {
                self.device.destroy_descriptor_set_layout(layout, None);
            }
            for view in self.views.drain(..) {
                self.device.destroy_image_view(view, None);
            }
        }
    }
}

fn subresource(aspect: vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::default()
        .aspect_mask(aspect)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1)
}

fn image_barrier(
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    source: (vk::PipelineStageFlags2, vk::AccessFlags2),
    destination: (vk::PipelineStageFlags2, vk::AccessFlags2),
) -> vk::ImageMemoryBarrier2<'static> {
    vk::ImageMemoryBarrier2::default()
        .src_stage_mask(source.0)
        .src_access_mask(source.1)
        .dst_stage_mask(destination.0)
        .dst_access_mask(destination.1)
        .old_layout(old_layout)
        .new_layout(new_layout)
        .image(image)
        .subresource_range(subresource(vk::ImageAspectFlags::COLOR))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matrix(scale: f32) -> [f32; 16] {
        let mut value = IDENTITY;
        value[0] = scale;
        value
    }

    fn identity(tag: u8) -> DrawIdentity {
        use next_contracts::ids::{AssetId, ContentHash};
        let revision = AssetRevisionRefV1 {
            asset_id: AssetId::from_bytes([tag; 16]),
            record_sha256: ContentHash::from_bytes([tag; 32]),
        };
        (revision, revision, revision)
    }

    #[test]
    fn motion_history_returns_current_then_previous_and_forgets_missing_keys() {
        let record_a = identity(0xa1);
        let record_b = identity(0xb2);
        let mut history = MotionHistoryV1::default();
        // Frame 1: nothing is known, every draw moves with itself.
        history.begin_frame();
        assert_eq!(history.resolve(record_a, matrix(1.0)), matrix(1.0));
        assert_eq!(history.resolve(record_a, matrix(2.0)), matrix(2.0));
        assert_eq!(history.resolve(record_b, matrix(3.0)), matrix(3.0));
        assert_eq!(history.previous_view_projection(matrix(9.0)), matrix(9.0));
        history.end_frame(matrix(9.0), [0.25, -0.25]);
        // Frame 2: the same keys read frame 1; `record_b` is absent.
        history.begin_frame();
        assert_eq!(history.resolve(record_a, matrix(1.5)), matrix(1.0));
        assert_eq!(history.resolve(record_a, matrix(2.5)), matrix(2.0));
        assert_eq!(history.previous_view_projection(matrix(10.0)), matrix(9.0));
        assert_eq!(history.previous_jitter_pixels(), [0.25, -0.25]);
        history.end_frame(matrix(10.0), [0.0, 0.0]);
        // Frame 3: `record_b` returns after one absent frame: no history.
        history.begin_frame();
        assert_eq!(history.resolve(record_b, matrix(4.0)), matrix(4.0));
        assert_eq!(history.resolve(record_a, matrix(1.75)), matrix(1.5));
        history.end_frame(matrix(11.0), [0.0, 0.0]);
    }

    #[test]
    fn uniform_and_push_blocks_have_the_declared_layouts() {
        let bytes = gbuffer_uniform_bytes(
            matrix(2.0),
            matrix(3.0),
            vk::Extent2D {
                width: 960,
                height: 540,
            },
            [0.25, -0.125],
            [-0.5, 0.5],
        );
        assert_eq!(bytes.len(), 160);
        assert_eq!(&bytes[0..4], &2.0_f32.to_le_bytes());
        assert_eq!(&bytes[64..68], &3.0_f32.to_le_bytes());
        assert_eq!(&bytes[128..132], &960.0_f32.to_le_bytes());
        assert_eq!(&bytes[132..136], &540.0_f32.to_le_bytes());
        assert_eq!(&bytes[136..140], &(1.0_f32 / 960.0).to_le_bytes());
        assert_eq!(&bytes[144..148], &0.25_f32.to_le_bytes());
        assert_eq!(&bytes[148..152], &(-0.125_f32).to_le_bytes());
        assert_eq!(&bytes[152..156], &(-0.5_f32).to_le_bytes());
        assert_eq!(&bytes[156..160], &0.5_f32.to_le_bytes());

        let draw = [7_u8; super::super::DRAW_PUSH_CONSTANT_SIZE as usize];
        let push = gbuffer_push_constant_bytes(draw, 41, GBufferGroupV1::WaterSurface);
        assert_eq!(push.len(), 96);
        assert_eq!(&push[..80], &draw[..]);
        assert_eq!(&push[80..84], &41.0_f32.to_le_bytes());
        assert_eq!(&push[84..88], &3.0_f32.to_le_bytes());
        assert_eq!(&push[88..92], &0.05_f32.to_le_bytes());
        assert_eq!(&push[92..96], &0.0_f32.to_le_bytes());
    }
}
