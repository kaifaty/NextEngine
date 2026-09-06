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
/// `model`, `base_color_factor`, `material_params`, `meta` (draw index,
/// group, roughness, 0).
pub(crate) const GBUFFER_PUSH_CONSTANT_SIZE: u32 = 112;
/// Scene look L1 (plan `look/01`): the HDR scene target format; the scene
/// passes render into it and the `tonemap` suite resolves it to the swapchain.
pub(crate) const HDR_SCENE_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
/// `exposure` and three spare lanes.
const TONEMAP_PUSH_CONSTANT_SIZE: u32 = 16;
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

/// Scene look L1: the fullscreen resolve of the HDR scene target into the
/// swapchain (exposure and the ACES curve). Present only when the scene
/// format differs from the swapchain's.
struct TonemapPass {
    sampler: vk::Sampler,
    set_layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    set: vk::DescriptorSet,
    layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
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
    tonemap: Option<TonemapPass>,
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
        scene_format: vk::Format,
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
            samplers: Vec::new(),
            pools: Vec::new(),
            pipeline_layouts: Vec::new(),
            pipelines: Vec::new(),
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
        // Scene look L1: the scene target in the scene format, sampled by
        // the tone-map resolve when that format is the HDR one.
        let scene_color = target(
            scene_format,
            vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::SAMPLED,
            &mut guard,
        )?;
        // Scene look L3: the normal and linear-depth targets feed the
        // occlusion pass.
        let gbuffer_usage = vk::ImageUsageFlags::COLOR_ATTACHMENT
            | vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::SAMPLED;
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
        let tonemap = if scene_format == color_format {
            None
        } else {
            Some(create_tonemap_pass(
                device,
                color_format,
                scene_color.view,
                &mut guard,
            )?)
        };
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
            tonemap,
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

    pub(crate) fn linear_depth_image(&self) -> vk::Image {
        self.linear_depth.image.image()
    }

    pub(crate) fn motion_image(&self) -> vk::Image {
        self.motion.image.image()
    }

    /// Scene look L4: the jittered view-projection of the frame being
    /// recorded (set by [`Self::prepare`]).
    pub(crate) const fn frame_view_projection(&self) -> [f32; 16] {
        self.frame_view_projection
    }

    pub(crate) fn normal_roughness_image(&self) -> vk::Image {
        self.normal_roughness.image.image()
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

    /// Resolves the HUD-less scene colour to the swapchain image (which is
    /// in attachment layout before and after): the tone-map draw when the
    /// scene target is the HDR one, a copy otherwise. Leaves the scene
    /// target in transfer-source layout for a capture. No rendering
    /// instance may be active.
    pub(crate) fn record_scene_to_swapchain(
        &self,
        command_buffer: vk::CommandBuffer,
        swapchain_image: vk::Image,
        swapchain_view: vk::ImageView,
        exposure: f32,
    ) {
        if let Some(tonemap) = self.tonemap.as_ref() {
            self.record_tonemap(command_buffer, tonemap, swapchain_view, exposure);
            return;
        }
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

    /// Scene look L1: the tone-map resolve. The scene target goes to
    /// shader-read layout, the fullscreen triangle writes the swapchain
    /// image inside its own rendering instance, then the scene target goes
    /// to transfer-source layout for a capture and the swapchain image is
    /// made ready for the overlay's load.
    fn record_tonemap(
        &self,
        command_buffer: vk::CommandBuffer,
        tonemap: &TonemapPass,
        swapchain_view: vk::ImageView,
        exposure: f32,
    ) {
        let to_sampled = [image_barrier(
            self.scene_color.image.image(),
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            (
                vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            ),
            (
                vk::PipelineStageFlags2::FRAGMENT_SHADER,
                vk::AccessFlags2::SHADER_READ,
            ),
        )];
        let to_transfer = [image_barrier(
            self.scene_color.image.image(),
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            (
                vk::PipelineStageFlags2::FRAGMENT_SHADER,
                vk::AccessFlags2::SHADER_READ,
            ),
            (
                vk::PipelineStageFlags2::TRANSFER,
                vk::AccessFlags2::TRANSFER_READ,
            ),
        )];
        let color_attachments = [vk::RenderingAttachmentInfo::default()
            .image_view(swapchain_view)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)];
        let rendering_info = vk::RenderingInfo::default()
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.extent,
            })
            .layer_count(1)
            .color_attachments(&color_attachments);
        let viewport = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.extent.width as f32,
            height: self.extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissor = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.extent,
        }];
        let mut push = [0_u8; TONEMAP_PUSH_CONSTANT_SIZE as usize];
        push[..4].copy_from_slice(&exposure.to_le_bytes());
        // SAFETY: the scene target and the swapchain image are in the
        // declared layouts, no rendering instance is active, and every
        // handle belongs to this pass or the live swapchain.
        unsafe {
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&to_sampled),
            );
            self.device
                .cmd_begin_rendering(command_buffer, &rendering_info);
            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                tonemap.pipeline,
            );
            self.device.cmd_set_viewport(command_buffer, 0, &viewport);
            self.device.cmd_set_scissor(command_buffer, 0, &scissor);
            self.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                tonemap.layout,
                0,
                &[tonemap.set],
                &[],
            );
            self.device.cmd_push_constants(
                command_buffer,
                tonemap.layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                &push,
            );
            self.device.cmd_draw(command_buffer, 3, 1, 0, 0);
            self.device.cmd_end_rendering(command_buffer);
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&to_transfer),
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
            if let Some(tonemap) = self.tonemap.take() {
                self.device.destroy_pipeline(tonemap.pipeline, None);
                self.device.destroy_pipeline_layout(tonemap.layout, None);
                self.device.destroy_descriptor_pool(tonemap.pool, None);
                self.device
                    .destroy_descriptor_set_layout(tonemap.set_layout, None);
                self.device.destroy_sampler(tonemap.sampler, None);
            }
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

/// Scene look L1: builds the tone-map resolve objects (sampler, one
/// descriptor set over the scene view, the pipeline over the swapchain
/// format); every handle is recorded in the guard until construction ends.
fn create_tonemap_pass(
    device: &ash::Device,
    swapchain_format: vk::Format,
    scene_view: vk::ImageView,
    guard: &mut Teardown,
) -> Result<TonemapPass, B0GpuContentError> {
    let modules = super::super::shader_assets::tonemap_shader_modules()
        .map_err(B0GpuContentError::ShaderAsset)?;
    let sampler_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::NEAREST)
        .min_filter(vk::Filter::NEAREST)
        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .min_lod(0.0)
        .max_lod(0.0);
    // SAFETY: plain sampler creation on the live device.
    let sampler = unsafe { device.create_sampler(&sampler_info, None) }?;
    guard.samplers.push(sampler);
    let bindings = [vk::DescriptorSetLayoutBinding::default()
        .binding(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)];
    let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    // SAFETY: plain layout creation on the live device.
    let set_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None) }?;
    guard.layouts.push(set_layout);
    let pool_sizes = [vk::DescriptorPoolSize {
        ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: 1,
    }];
    let pool_info = vk::DescriptorPoolCreateInfo::default()
        .max_sets(1)
        .pool_sizes(&pool_sizes);
    // SAFETY: the pool exactly covers one set.
    let pool = unsafe { device.create_descriptor_pool(&pool_info, None) }?;
    guard.pools.push(pool);
    let layouts = [set_layout];
    let allocation_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    // SAFETY: pool and layout are live on this device.
    let set = unsafe { device.allocate_descriptor_sets(&allocation_info) }?[0];
    let image_info = [vk::DescriptorImageInfo::default()
        .sampler(sampler)
        .image_view(scene_view)
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
    let writes = [vk::WriteDescriptorSet::default()
        .dst_set(set)
        .dst_binding(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(&image_info)];
    // SAFETY: the set, sampler and view are live; the descriptor is copied now.
    unsafe { device.update_descriptor_sets(&writes, &[]) };
    let push_ranges = [vk::PushConstantRange {
        stage_flags: vk::ShaderStageFlags::FRAGMENT,
        offset: 0,
        size: TONEMAP_PUSH_CONSTANT_SIZE,
    }];
    let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(&layouts)
        .push_constant_ranges(&push_ranges);
    // SAFETY: the set layout is live.
    let layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }?;
    guard.pipeline_layouts.push(layout);
    let pipeline = create_fullscreen_pipeline(
        device,
        layout,
        swapchain_format,
        &modules.vertex,
        &modules.fragment,
    )?;
    guard.pipelines.push(pipeline);
    Ok(TonemapPass {
        sampler,
        set_layout,
        pool,
        set,
        layout,
        pipeline,
    })
}

/// The fullscreen tone-map pipeline over the swapchain colour format: no
/// vertex input, no depth, no blending, dynamic viewport and scissor.
pub(super) fn create_fullscreen_pipeline(
    device: &ash::Device,
    layout: vk::PipelineLayout,
    color_format: vk::Format,
    vertex_words: &[u32],
    fragment_words: &[u32],
) -> Result<vk::Pipeline, B0GpuContentError> {
    let vertex_info = vk::ShaderModuleCreateInfo::default().code(vertex_words);
    let fragment_info = vk::ShaderModuleCreateInfo::default().code(fragment_words);
    // SAFETY: checked-in SPIR-V passed structural validation.
    let vertex_module = unsafe { device.create_shader_module(&vertex_info, None) }?;
    // SAFETY: same conditions as the vertex module.
    let fragment_module = match unsafe { device.create_shader_module(&fragment_info, None) } {
        Ok(module) => module,
        Err(error) => {
            // SAFETY: the vertex module has no dependants.
            unsafe { device.destroy_shader_module(vertex_module, None) };
            return Err(error.into());
        }
    };
    let result = {
        let stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_module)
                .name(c"main"),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_module)
                .name(c"main"),
        ];
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .line_width(1.0);
        let multisample = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(false)
            .depth_write_enable(false);
        let blend_attachments = [vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(false)
            .color_write_mask(vk::ColorComponentFlags::RGBA)];
        let color_blend =
            vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachments);
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        let color_formats = [color_format];
        let mut rendering =
            vk::PipelineRenderingCreateInfo::default().color_attachment_formats(&color_formats);
        let info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization)
            .multisample_state(&multisample)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blend)
            .dynamic_state(&dynamic)
            .layout(layout)
            .push_next(&mut rendering);
        // SAFETY: every referenced create-info structure and module stays
        // live for the call; dynamic rendering declares the exact format.
        unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &[info], None) }
            .map(|pipelines| pipelines[0])
            .map_err(|(_, error)| B0GpuContentError::from(error))
    };
    // SAFETY: pipeline creation copied all module state.
    unsafe {
        device.destroy_shader_module(fragment_module, None);
        device.destroy_shader_module(vertex_module, None);
    }
    result
}

/// Destroys partially constructed objects when construction fails midway.
struct Teardown {
    device: ash::Device,
    layouts: Vec<vk::DescriptorSetLayout>,
    pool: vk::DescriptorPool,
    views: Vec<vk::ImageView>,
    samplers: Vec<vk::Sampler>,
    pools: Vec<vk::DescriptorPool>,
    pipeline_layouts: Vec<vk::PipelineLayout>,
    pipelines: Vec<vk::Pipeline>,
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
            for pipeline in self.pipelines.drain(..) {
                self.device.destroy_pipeline(pipeline, None);
            }
            for layout in self.pipeline_layouts.drain(..) {
                self.device.destroy_pipeline_layout(layout, None);
            }
            for pool in self.pools.drain(..) {
                self.device.destroy_descriptor_pool(pool, None);
            }
            for sampler in self.samplers.drain(..) {
                self.device.destroy_sampler(sampler, None);
            }
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
        // Scene look L1: the draw bytes carry the material lane (96 bytes).
        assert_eq!(push.len(), 112);
        assert_eq!(&push[..96], &draw[..]);
        assert_eq!(&push[96..100], &41.0_f32.to_le_bytes());
        assert_eq!(&push[100..104], &3.0_f32.to_le_bytes());
        assert_eq!(&push[104..108], &0.05_f32.to_le_bytes());
        assert_eq!(&push[108..112], &0.0_f32.to_le_bytes());
    }
}
