mod pipeline;
mod resources;
mod shadow;
mod ui_overlay_gpu;

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::mem::size_of;
use std::sync::Arc;

use ash::vk;
use next_contracts::ids::ContentHash;
use next_contracts::project::AssetRevisionRefV1;
use next_contracts::render_content::{
    NeutralTexelEncodingV1, NeutralTextureColorSpaceV1, NeutralTextureDimensionV1,
    RenderContentCatalogV1, RenderContentContractError,
};
use next_render::B0FramePlanV1;

use self::pipeline::{
    PipelineState, draw_push_constant_bytes, frame_raster_state, identity_matrix_bytes,
};
pub(crate) use self::resources::DepthAttachment;
use self::resources::{
    BufferAllocation, DescriptorState, ShadowMap, TextureResource, upload_content,
};
use self::shadow::{ShadowPipelineState, initialize_shadow_map};
pub(crate) use self::ui_overlay_gpu::UiOverlayState;
use crate::dynamic_surface::{DynamicSurfaceProfileV1, DynamicSurfaceUpdateV1, validate_profiles};

const VERTEX_STRIDE: u32 = 28;
const INDIRECT_COMMAND_STRIDE: u32 = 20;
const FRAME_UNIFORM_SIZE: vk::DeviceSize = 208;
const DRAW_PUSH_CONSTANT_SIZE: u32 = 80;
const MINIMUM_BUFFER_SIZE: vk::DeviceSize = 4;

#[derive(Debug)]
pub(super) enum B0GpuContentError {
    Graphics(vk::Result),
    Contract(RenderContentContractError),
    ShaderAsset(&'static str),
    MemoryTypeUnavailable,
    UploadCompletionUnknown(vk::Result),
    InvalidCatalog(&'static str),
    InvalidFramePlan(&'static str),
    ResourceMissing(&'static str),
    CountOverflow,
}

impl Display for B0GpuContentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Graphics(error) => write!(formatter, "graphics API failed: {error:?}"),
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::ShaderAsset(error) => write!(formatter, "offline shader asset invalid: {error}"),
            Self::MemoryTypeUnavailable => {
                formatter.write_str("required Vulkan memory type unavailable")
            }
            Self::UploadCompletionUnknown(error) => write!(
                formatter,
                "upload submission completion could not be established: {error:?}"
            ),
            Self::InvalidCatalog(reason) => {
                write!(formatter, "B0 render catalog invalid: {reason}")
            }
            Self::InvalidFramePlan(reason) => write!(formatter, "B0 frame plan invalid: {reason}"),
            Self::ResourceMissing(resource) => {
                write!(formatter, "B0 GPU resource missing: {resource}")
            }
            Self::CountOverflow => formatter.write_str("B0 GPU resource count overflow"),
        }
    }
}

impl Error for B0GpuContentError {}

impl From<vk::Result> for B0GpuContentError {
    fn from(error: vk::Result) -> Self {
        Self::Graphics(error)
    }
}

impl From<RenderContentContractError> for B0GpuContentError {
    fn from(error: RenderContentContractError) -> Self {
        Self::Contract(error)
    }
}

/// Device-owned, reconstructible B0 resources.
///
/// Fields are declared in dependency-drop order: the pipeline is destroyed
/// before descriptor layouts, descriptor sets before referenced images, and
/// all children before the logical device that owns this value.
pub(super) struct B0GpuContent {
    sky_pipeline: PipelineState,
    pipeline: PipelineState,
    shadow_pipeline: Option<ShadowPipelineState>,
    descriptors: DescriptorState,
    textures: BTreeMap<AssetRevisionRefV1, TextureResource>,
    shadow_map: Option<ShadowMap>,
    indirect: BufferAllocation,
    frame_uniforms: Vec<BufferAllocation>,
    dynamic_vertices: Vec<BufferAllocation>,
    geometry: BufferAllocation,
    index_buffer_offset: vk::DeviceSize,
    draw_offsets: BTreeMap<DrawKey, vk::DeviceSize>,
    draw_commands: BTreeMap<DrawKey, PreparedDrawCommand>,
    vertex_templates: BTreeMap<AssetRevisionRefV1, Vec<[u8; 16]>>,
    dynamic_vertex_scratch: Vec<u8>,
    dynamic_vertex_offsets: Vec<i32>,
    dynamic_surfaces: BTreeMap<AssetRevisionRefV1, DynamicSurfaceRing>,
    dynamic_surface_scratch: Vec<u8>,
    catalog_hash: ContentHash,
}

/// Per-frame-slot host-visible vertex/index storage for one declared
/// presentation-only dynamic surface. Capacity is fixed at creation.
struct DynamicSurfaceRing {
    profile: DynamicSurfaceProfileV1,
    slots: Vec<DynamicSurfaceSlot>,
}

struct DynamicSurfaceSlot {
    vertices: BufferAllocation,
    indices: BufferAllocation,
    uploaded: Option<UploadedDynamicSurface>,
}

#[derive(Clone, Copy)]
struct UploadedDynamicSurface {
    canonical_hash: ContentHash,
    index_count: u32,
}

/// Bytes and refresh count copied into dynamic surface rings by one frame.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct DynamicSurfaceUploadStats {
    pub(super) uploads: u64,
    pub(super) bytes: u64,
}

#[derive(Clone, Copy)]
struct DynamicDrawBinding {
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    index_count: u32,
}

impl B0GpuContent {
    #[allow(
        clippy::too_many_arguments,
        reason = "Vulkan ownership inputs are explicit at the private adapter boundary"
    )]
    pub(super) fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        queue: vk::Queue,
        queue_family_index: u32,
        color_format: vk::Format,
        depth_format: vk::Format,
        catalog: &RenderContentCatalogV1,
        frame_slot_count: usize,
        dynamic_surface_profiles: &[DynamicSurfaceProfileV1],
    ) -> Result<Self, B0GpuContentError> {
        if frame_slot_count == 0 {
            return Err(B0GpuContentError::InvalidCatalog(
                "frame slot count must be non-zero",
            ));
        }
        validate_profiles(dynamic_surface_profiles, catalog).map_err(|_| {
            B0GpuContentError::InvalidCatalog("dynamic surface profile does not match the catalog")
        })?;
        let prepared = PreparedContent::from_catalog(catalog)?;
        let geometry = BufferAllocation::new(
            instance,
            physical_device,
            device,
            prepared.geometry_size(),
            vk::BufferUsageFlags::VERTEX_BUFFER
                | vk::BufferUsageFlags::INDEX_BUFFER
                | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        let indirect = BufferAllocation::new(
            instance,
            physical_device,
            device,
            prepared.indirect_size(),
            vk::BufferUsageFlags::INDIRECT_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        let mut frame_uniforms = Vec::with_capacity(frame_slot_count);
        let mut dynamic_vertices = Vec::with_capacity(frame_slot_count);
        for _ in 0..frame_slot_count {
            let frame_uniform = BufferAllocation::new(
                instance,
                physical_device,
                device,
                FRAME_UNIFORM_SIZE,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            frame_uniform.write(0, &identity_matrix_bytes())?;
            frame_uniforms.push(frame_uniform);
            dynamic_vertices.push(BufferAllocation::new(
                instance,
                physical_device,
                device,
                prepared.dynamic_vertex_capacity_size(),
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?);
        }

        let mut dynamic_surfaces = BTreeMap::new();
        for profile in dynamic_surface_profiles {
            let vertex_bytes = vk::DeviceSize::from(profile.vertex_capacity)
                .checked_mul(vk::DeviceSize::from(VERTEX_STRIDE))
                .ok_or(B0GpuContentError::CountOverflow)?;
            let index_bytes = vk::DeviceSize::from(profile.index_capacity)
                .checked_mul(size_of::<u32>() as vk::DeviceSize)
                .ok_or(B0GpuContentError::CountOverflow)?;
            let mut slots = Vec::with_capacity(frame_slot_count);
            for _ in 0..frame_slot_count {
                slots.push(DynamicSurfaceSlot {
                    vertices: BufferAllocation::new(
                        instance,
                        physical_device,
                        device,
                        vertex_bytes,
                        vk::BufferUsageFlags::VERTEX_BUFFER,
                        vk::MemoryPropertyFlags::HOST_VISIBLE
                            | vk::MemoryPropertyFlags::HOST_COHERENT,
                    )?,
                    indices: BufferAllocation::new(
                        instance,
                        physical_device,
                        device,
                        index_bytes,
                        vk::BufferUsageFlags::INDEX_BUFFER,
                        vk::MemoryPropertyFlags::HOST_VISIBLE
                            | vk::MemoryPropertyFlags::HOST_COHERENT,
                    )?,
                    uploaded: None,
                });
            }
            if dynamic_surfaces
                .insert(
                    profile.mesh_revision,
                    DynamicSurfaceRing {
                        profile: *profile,
                        slots,
                    },
                )
                .is_some()
            {
                return Err(B0GpuContentError::InvalidCatalog(
                    "duplicate dynamic surface mesh revision",
                ));
            }
        }

        let mut textures = BTreeMap::new();
        for texture in &prepared.textures {
            let resource = TextureResource::new(instance, physical_device, device, texture.extent)?;
            if textures.insert(texture.revision, resource).is_some() {
                return Err(B0GpuContentError::InvalidCatalog(
                    "duplicate exact texture revision",
                ));
            }
        }

        let staging = BufferAllocation::new(
            instance,
            physical_device,
            device,
            prepared.staging_bytes.len() as vk::DeviceSize,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        staging.write(0, &prepared.staging_bytes)?;
        let upload_result = upload_content(
            device,
            queue,
            queue_family_index,
            &staging,
            &geometry,
            &indirect,
            &textures,
            &prepared,
        );
        if let Err(error) = upload_result {
            if matches!(&error, B0GpuContentError::UploadCompletionUnknown(_)) {
                // The queue may still reference each upload allocation.
                // Leaking them until device teardown is the only safe cleanup
                // when the driver cannot establish submission completion.
                std::mem::forget(staging);
                std::mem::forget(textures);
                std::mem::forget(indirect);
                std::mem::forget(geometry);
            }
            return Err(error);
        }
        drop(staging);

        let shadow_map = ShadowMap::try_new(instance, physical_device, device)?;
        if let Some(shadow) = shadow_map.as_ref() {
            initialize_shadow_map(device, queue, queue_family_index, shadow)?;
        } else {
            eprintln!(
                "next_game: SHADOW_MAP_FALLBACK: sampled depth format or 2048x2048 allocation unavailable"
            );
        }
        let descriptors =
            DescriptorState::new(device, &frame_uniforms, &textures, shadow_map.as_ref())?;
        let pipeline = PipelineState::new(
            device,
            color_format,
            depth_format,
            descriptors.frame_layout,
            descriptors.texture_layout,
            descriptors.shadow_layout,
            shadow_map.is_some(),
        )?;
        let sky_pipeline = PipelineState::new_sky(device, color_format, depth_format)?;
        let shadow_pipeline = shadow_map
            .as_ref()
            .map(|shadow| {
                ShadowPipelineState::new(device, shadow.format(), descriptors.frame_layout)
            })
            .transpose()?;

        Ok(Self {
            sky_pipeline,
            pipeline,
            shadow_pipeline,
            descriptors,
            textures,
            shadow_map,
            indirect,
            frame_uniforms,
            dynamic_vertices,
            geometry,
            index_buffer_offset: prepared.index_buffer_offset,
            draw_offsets: prepared.draw_offsets,
            draw_commands: prepared.draw_commands,
            vertex_templates: prepared.vertex_templates,
            dynamic_vertex_scratch: Vec::new(),
            dynamic_vertex_offsets: Vec::new(),
            dynamic_surfaces,
            dynamic_surface_scratch: Vec::new(),
            catalog_hash: catalog.catalog_sha256(),
        })
    }

    /// Refreshes this frame slot's dynamic surface rings from the current
    /// immutable updates. A slot that already holds an update's exact
    /// canonical hash is left untouched, so a repeated frame costs no copy.
    ///
    /// The caller has already waited for the slot fence, so no submitted
    /// command buffer still reads these host-visible allocations.
    pub(super) fn prepare_dynamic_surfaces(
        &mut self,
        updates: &BTreeMap<AssetRevisionRefV1, Arc<DynamicSurfaceUpdateV1>>,
        frame_slot_index: usize,
    ) -> Result<DynamicSurfaceUploadStats, B0GpuContentError> {
        let mut stats = DynamicSurfaceUploadStats::default();
        for (revision, update) in updates {
            let ring = self.dynamic_surfaces.get_mut(revision).ok_or(
                B0GpuContentError::ResourceMissing("declared dynamic surface ring"),
            )?;
            if update.mesh_revision() != *revision {
                return Err(B0GpuContentError::InvalidFramePlan(
                    "dynamic surface update is keyed by a different mesh revision",
                ));
            }
            if update.vertex_count() > ring.profile.vertex_capacity
                || update.index_count() > ring.profile.index_capacity
            {
                return Err(B0GpuContentError::InvalidFramePlan(
                    "dynamic surface update exceeds the declared ring capacity",
                ));
            }
            let slot =
                ring.slots
                    .get_mut(frame_slot_index)
                    .ok_or(B0GpuContentError::InvalidFramePlan(
                        "frame slot index is outside the dynamic surface ring",
                    ))?;
            if slot
                .uploaded
                .is_some_and(|uploaded| uploaded.canonical_hash == update.canonical_hash())
            {
                continue;
            }
            pack_dynamic_surface_vertices(update, &mut self.dynamic_surface_scratch)?;
            slot.vertices.write(0, &self.dynamic_surface_scratch)?;
            pack_dynamic_surface_indices(update, &mut self.dynamic_surface_scratch)?;
            slot.indices.write(0, &self.dynamic_surface_scratch)?;
            slot.uploaded = Some(UploadedDynamicSurface {
                canonical_hash: update.canonical_hash(),
                index_count: update.index_count(),
            });
            let vertex_bytes = u64::from(update.vertex_count())
                .checked_mul(u64::from(VERTEX_STRIDE))
                .ok_or(B0GpuContentError::CountOverflow)?;
            let index_bytes = u64::from(update.index_count())
                .checked_mul(size_of::<u32>() as u64)
                .ok_or(B0GpuContentError::CountOverflow)?;
            stats.uploads = stats
                .uploads
                .checked_add(1)
                .ok_or(B0GpuContentError::CountOverflow)?;
            stats.bytes = stats
                .bytes
                .checked_add(vertex_bytes)
                .and_then(|value| value.checked_add(index_bytes))
                .ok_or(B0GpuContentError::CountOverflow)?;
        }
        Ok(stats)
    }

    /// Returns the dynamic ring binding for a draw whose mesh revision is a
    /// declared surface with an uploaded update in this frame slot.
    fn dynamic_draw_binding(
        &self,
        mesh_revision: AssetRevisionRefV1,
        frame_slot_index: usize,
    ) -> Option<DynamicDrawBinding> {
        let slot = self
            .dynamic_surfaces
            .get(&mesh_revision)?
            .slots
            .get(frame_slot_index)?;
        let uploaded = slot.uploaded?;
        Some(DynamicDrawBinding {
            vertex_buffer: slot.vertices.buffer,
            index_buffer: slot.indices.buffer,
            index_count: uploaded.index_count,
        })
    }

    /// Records the CPU-selected visible list. Static draws use the immutable
    /// indexed-indirect stream; skinned draws use the frame-slot vertex ring;
    /// declared dynamic surfaces with an uploaded update use their own
    /// frame-slot vertex/index ring. Dynamic rendering must already be active
    /// for `color_format`. Returns the number of dynamic surface draws.
    pub(super) fn record(
        &mut self,
        command_buffer: vk::CommandBuffer,
        plan: &B0FramePlanV1,
        extent: vk::Extent2D,
        frame_slot_index: usize,
    ) -> Result<u64, B0GpuContentError> {
        if extent.width == 0 || extent.height == 0 {
            return Err(B0GpuContentError::InvalidFramePlan(
                "render extent must be non-zero",
            ));
        }
        if plan.catalog_hash != self.catalog_hash {
            return Err(B0GpuContentError::InvalidFramePlan(
                "catalog hash does not match uploaded resources",
            ));
        }
        if plan.target.extent != [extent.width, extent.height] {
            return Err(B0GpuContentError::InvalidFramePlan(
                "frame-plan extent does not match the active target",
            ));
        }
        if usize::try_from(plan.indexed_draw_count).map_err(|_| B0GpuContentError::CountOverflow)?
            != plan.draws.len()
        {
            return Err(B0GpuContentError::InvalidFramePlan(
                "indexed draw count does not match ordered draws",
            ));
        }
        self.prepare_dynamic_vertices(plan, frame_slot_index)?;

        let frame_uniform = self.frame_uniforms.get(frame_slot_index).ok_or(
            B0GpuContentError::InvalidFramePlan(
                "frame slot index is outside the allocated uniform ring",
            ),
        )?;
        let frame_set = self
            .descriptors
            .frame_sets
            .get(frame_slot_index)
            .copied()
            .ok_or(B0GpuContentError::InvalidFramePlan(
                "frame slot index is outside the descriptor ring",
            ))?;
        let raster_state = frame_raster_state(plan.camera.as_ref(), extent)?;
        frame_uniform.write(0, &raster_state.view_projection_bytes)?;
        let viewports = [raster_state.viewport];
        let scissors = [raster_state.scissor];
        let vertex_buffers = [self.geometry.buffer];
        let vertex_offsets = [0];
        let frame_sets = [frame_set];

        // SAFETY: all bound objects belong to the same live device, the
        // command buffer is recording inside dynamic rendering, and ranges
        // match the pipeline's fixed B0 interface.
        unsafe {
            self.geometry.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );
            self.geometry
                .device
                .cmd_set_viewport(command_buffer, 0, &viewports);
            self.geometry
                .device
                .cmd_set_scissor(command_buffer, 0, &scissors);
            self.geometry.device.cmd_bind_vertex_buffers(
                command_buffer,
                0,
                &vertex_buffers,
                &vertex_offsets,
            );
            self.geometry.device.cmd_bind_index_buffer(
                command_buffer,
                self.geometry.buffer,
                self.index_buffer_offset,
                vk::IndexType::UINT32,
            );
            self.geometry.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0,
                &frame_sets,
                &[],
            );
            if let Some(shadow_set) = self.descriptors.shadow_set {
                self.geometry.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.pipeline.layout,
                    2,
                    &[shadow_set],
                    &[],
                );
            }
        }

        let mut dynamic_surface_draws = 0_u64;
        for draw in &plan.draws {
            let dynamic_binding = self.dynamic_draw_binding(draw.mesh_revision, frame_slot_index);
            if dynamic_binding.is_some() && draw.skinning_vertex_stream_index.is_some() {
                return Err(B0GpuContentError::InvalidFramePlan(
                    "dynamic surface draw cannot also be skinned",
                ));
            }
            match (
                draw.skinning_vertex_stream_index,
                draw.skinning_vertex_stream_hash,
            ) {
                (Some(stream_index), Some(stream_hash)) => {
                    let stream = plan
                        .skinned_vertex_streams
                        .get(
                            usize::try_from(stream_index)
                                .map_err(|_| B0GpuContentError::CountOverflow)?,
                        )
                        .ok_or(B0GpuContentError::InvalidFramePlan(
                            "skinning vertex-stream index is outside the frame plan",
                        ))?;
                    if stream.mesh_revision != draw.mesh_revision
                        || stream.vertex_stream_hash != stream_hash
                        || stream.used_bind_pose_fallback != draw.base_skinning_fallback
                    {
                        return Err(B0GpuContentError::InvalidFramePlan(
                            "skinned draw does not match its exact vertex stream",
                        ));
                    }
                }
                (None, None) if !draw.base_skinning_fallback => {}
                _ => {
                    return Err(B0GpuContentError::InvalidFramePlan(
                        "skinned draw binding is incomplete",
                    ));
                }
            }
            let texture_set = self
                .descriptors
                .texture_sets
                .get(&draw.texture_revision)
                .copied()
                .ok_or(B0GpuContentError::ResourceMissing(
                    "exact base-color texture descriptor",
                ))?;
            if !self.textures.contains_key(&draw.texture_revision) {
                return Err(B0GpuContentError::ResourceMissing(
                    "exact base-color texture image",
                ));
            }
            let draw_key = DrawKey {
                mesh_revision: draw.mesh_revision,
                first_index: draw.first_index,
                index_count: draw.index_count,
            };
            let indirect_offset = self.draw_offsets.get(&draw_key).copied().ok_or(
                B0GpuContentError::ResourceMissing("exact indexed-indirect command"),
            )?;
            let draw_command = self.draw_commands.get(&draw_key).copied().ok_or(
                B0GpuContentError::ResourceMissing("exact indexed draw command"),
            )?;
            let texture_sets = [texture_set];
            let push_constants =
                draw_push_constant_bytes(draw.transform, draw.base_color_rgba_unorm16);

            // SAFETY: descriptor set one was allocated from the pipeline's
            // texture layout, push bytes exactly cover its declared 80-byte
            // range, and the indirect offset selects one initialized command.
            unsafe {
                self.geometry.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.pipeline.layout,
                    1,
                    &texture_sets,
                    &[],
                );
                self.geometry.device.cmd_push_constants(
                    command_buffer,
                    self.pipeline.layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    &push_constants,
                );
                if let Some(binding) = dynamic_binding {
                    self.geometry.device.cmd_bind_vertex_buffers(
                        command_buffer,
                        0,
                        &[binding.vertex_buffer],
                        &[0],
                    );
                    self.geometry.device.cmd_bind_index_buffer(
                        command_buffer,
                        binding.index_buffer,
                        0,
                        vk::IndexType::UINT32,
                    );
                    self.geometry.device.cmd_draw_indexed(
                        command_buffer,
                        binding.index_count,
                        1,
                        0,
                        0,
                        0,
                    );
                    // Later static/skinned draws expect the immutable index
                    // stream again.
                    self.geometry.device.cmd_bind_index_buffer(
                        command_buffer,
                        self.geometry.buffer,
                        self.index_buffer_offset,
                        vk::IndexType::UINT32,
                    );
                    dynamic_surface_draws = dynamic_surface_draws
                        .checked_add(1)
                        .ok_or(B0GpuContentError::CountOverflow)?;
                } else if let Some(stream_index) = draw.skinning_vertex_stream_index {
                    let stream_index = usize::try_from(stream_index)
                        .map_err(|_| B0GpuContentError::CountOverflow)?;
                    let vertex_offset = *self.dynamic_vertex_offsets.get(stream_index).ok_or(
                        B0GpuContentError::InvalidFramePlan(
                            "skinning vertex-stream index is outside the uploaded frame stream",
                        ),
                    )?;
                    let dynamic_buffer = self
                        .dynamic_vertices
                        .get(frame_slot_index)
                        .ok_or(B0GpuContentError::InvalidFramePlan(
                            "frame slot index is outside the dynamic vertex ring",
                        ))?
                        .buffer;
                    self.geometry.device.cmd_bind_vertex_buffers(
                        command_buffer,
                        0,
                        &[dynamic_buffer],
                        &[0],
                    );
                    self.geometry.device.cmd_draw_indexed(
                        command_buffer,
                        draw_command.index_count,
                        1,
                        draw_command.first_index,
                        vertex_offset,
                        0,
                    );
                } else {
                    self.geometry.device.cmd_bind_vertex_buffers(
                        command_buffer,
                        0,
                        &[self.geometry.buffer],
                        &[0],
                    );
                    self.geometry.device.cmd_draw_indexed_indirect(
                        command_buffer,
                        self.indirect.buffer,
                        indirect_offset,
                        1,
                        INDIRECT_COMMAND_STRIDE,
                    );
                }
            }
        }
        Ok(dynamic_surface_draws)
    }

    fn prepare_dynamic_vertices(
        &mut self,
        plan: &B0FramePlanV1,
        frame_slot_index: usize,
    ) -> Result<(), B0GpuContentError> {
        self.dynamic_vertex_scratch.clear();
        self.dynamic_vertex_offsets.clear();
        self.dynamic_vertex_offsets
            .try_reserve(plan.skinned_vertex_streams.len())
            .map_err(|_| B0GpuContentError::CountOverflow)?;
        for stream in &plan.skinned_vertex_streams {
            let template = self.vertex_templates.get(&stream.mesh_revision).ok_or(
                B0GpuContentError::ResourceMissing("exact skinned mesh vertex template"),
            )?;
            if template.len() != stream.positions_micrometres.len() {
                return Err(B0GpuContentError::InvalidFramePlan(
                    "skinned position count does not match the exact mesh",
                ));
            }
            let base_vertex =
                i32::try_from(self.dynamic_vertex_scratch.len() / VERTEX_STRIDE as usize)
                    .map_err(|_| B0GpuContentError::CountOverflow)?;
            self.dynamic_vertex_offsets.push(base_vertex);
            let added_bytes = template
                .len()
                .checked_mul(VERTEX_STRIDE as usize)
                .ok_or(B0GpuContentError::CountOverflow)?;
            self.dynamic_vertex_scratch
                .try_reserve(added_bytes)
                .map_err(|_| B0GpuContentError::CountOverflow)?;
            for (position, attributes) in stream.positions_micrometres.iter().zip(template) {
                for component in position {
                    push_f32(
                        &mut self.dynamic_vertex_scratch,
                        *component as f32 / 1_000_000.0,
                    );
                }
                self.dynamic_vertex_scratch.extend_from_slice(attributes);
            }
        }
        self.dynamic_vertices
            .get(frame_slot_index)
            .ok_or(B0GpuContentError::InvalidFramePlan(
                "frame slot index is outside the dynamic vertex ring",
            ))?
            .write(0, &self.dynamic_vertex_scratch)
    }

    /// Records a presentation-only fullscreen sky before opaque world draws.
    pub(super) fn record_sky(
        &self,
        command_buffer: vk::CommandBuffer,
        extent: vk::Extent2D,
    ) -> Result<(), B0GpuContentError> {
        if extent.width == 0 || extent.height == 0 {
            return Err(B0GpuContentError::InvalidFramePlan(
                "sky extent must be non-zero",
            ));
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
        // SAFETY: the pipeline belongs to the recording device, dynamic
        // rendering is active, and the sky vertex shader synthesizes exactly
        // three vertices from gl_VertexIndex without buffer access.
        unsafe {
            self.geometry.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.sky_pipeline.pipeline,
            );
            self.geometry
                .device
                .cmd_set_viewport(command_buffer, 0, &viewports);
            self.geometry
                .device
                .cmd_set_scissor(command_buffer, 0, &scissors);
            self.geometry.device.cmd_draw(command_buffer, 3, 1, 0, 0);
        }
        Ok(())
    }

    pub(super) fn device_allocation_stats(&self) -> Result<(u64, u64), B0GpuContentError> {
        let mut bytes = self
            .geometry
            .allocation_size()
            .checked_add(self.indirect.allocation_size())
            .ok_or(B0GpuContentError::CountOverflow)?;
        for frame_uniform in &self.frame_uniforms {
            bytes = bytes
                .checked_add(frame_uniform.allocation_size())
                .ok_or(B0GpuContentError::CountOverflow)?;
        }
        for dynamic_vertices in &self.dynamic_vertices {
            bytes = bytes
                .checked_add(dynamic_vertices.allocation_size())
                .ok_or(B0GpuContentError::CountOverflow)?;
        }
        let mut dynamic_surface_allocations = 0_u64;
        for ring in self.dynamic_surfaces.values() {
            for slot in &ring.slots {
                bytes = bytes
                    .checked_add(slot.vertices.allocation_size())
                    .and_then(|value| value.checked_add(slot.indices.allocation_size()))
                    .ok_or(B0GpuContentError::CountOverflow)?;
                dynamic_surface_allocations = dynamic_surface_allocations
                    .checked_add(2)
                    .ok_or(B0GpuContentError::CountOverflow)?;
            }
        }
        for texture in self.textures.values() {
            bytes = bytes
                .checked_add(texture.allocation_size())
                .ok_or(B0GpuContentError::CountOverflow)?;
        }
        if let Some(shadow) = self.shadow_map.as_ref() {
            bytes = bytes
                .checked_add(shadow.allocation_size())
                .ok_or(B0GpuContentError::CountOverflow)?;
        }
        let texture_count =
            u64::try_from(self.textures.len()).map_err(|_| B0GpuContentError::CountOverflow)?;
        let frame_uniform_count = u64::try_from(self.frame_uniforms.len())
            .map_err(|_| B0GpuContentError::CountOverflow)?;
        let dynamic_vertex_count = u64::try_from(self.dynamic_vertices.len())
            .map_err(|_| B0GpuContentError::CountOverflow)?;
        let allocation_count = 2_u64
            .checked_add(frame_uniform_count)
            .and_then(|value| value.checked_add(dynamic_vertex_count))
            .and_then(|value| value.checked_add(dynamic_surface_allocations))
            .and_then(|value| value.checked_add(texture_count))
            .and_then(|value| value.checked_add(u64::from(self.shadow_map.is_some())))
            .ok_or(B0GpuContentError::CountOverflow)?;
        Ok((bytes, allocation_count))
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct DrawKey {
    mesh_revision: AssetRevisionRefV1,
    first_index: u32,
    index_count: u32,
}

#[derive(Clone, Copy)]
struct PreparedDrawCommand {
    index_count: u32,
    first_index: u32,
}

struct PreparedTexture {
    revision: AssetRevisionRefV1,
    extent: vk::Extent3D,
    staging_offset: vk::DeviceSize,
}

struct PreparedContent {
    staging_bytes: Vec<u8>,
    geometry_staging_offset: vk::DeviceSize,
    geometry_payload_size: vk::DeviceSize,
    indirect_staging_offset: vk::DeviceSize,
    indirect_payload_size: vk::DeviceSize,
    index_buffer_offset: vk::DeviceSize,
    draw_offsets: BTreeMap<DrawKey, vk::DeviceSize>,
    draw_commands: BTreeMap<DrawKey, PreparedDrawCommand>,
    vertex_templates: BTreeMap<AssetRevisionRefV1, Vec<[u8; 16]>>,
    dynamic_vertex_capacity: vk::DeviceSize,
    textures: Vec<PreparedTexture>,
}

impl PreparedContent {
    fn from_catalog(catalog: &RenderContentCatalogV1) -> Result<Self, B0GpuContentError> {
        let mut vertex_bytes = Vec::new();
        let mut index_bytes = Vec::new();
        let mut indirect_bytes = Vec::new();
        let mut draw_offsets = BTreeMap::new();
        let mut draw_commands = BTreeMap::new();
        let mut vertex_templates = BTreeMap::new();

        for mesh in catalog.meshes() {
            let revision = mesh.asset_revision()?;
            let base_vertex = i32::try_from(vertex_bytes.len() / VERTEX_STRIDE as usize)
                .map_err(|_| B0GpuContentError::CountOverflow)?;
            let first_index = u32::try_from(index_bytes.len() / size_of::<u32>())
                .map_err(|_| B0GpuContentError::CountOverflow)?;
            let uv0 = mesh
                .texcoords_q16_16()
                .first()
                .ok_or(B0GpuContentError::InvalidCatalog(
                    "mesh does not contain UV set zero",
                ))?;
            if uv0.len() != mesh.positions_micrometres().len() {
                return Err(B0GpuContentError::InvalidCatalog(
                    "position and UV counts differ",
                ));
            }
            let normals = mesh.normals_snorm16();
            if normals.is_some_and(|values| values.len() != mesh.positions_micrometres().len()) {
                return Err(B0GpuContentError::InvalidCatalog(
                    "position and normal counts differ",
                ));
            }
            let mut mesh_vertex_templates = Vec::with_capacity(mesh.positions_micrometres().len());
            for (vertex_index, (position, uv)) in
                mesh.positions_micrometres().iter().zip(uv0).enumerate()
            {
                for component in position {
                    push_f32(&mut vertex_bytes, *component as f32 / 1_000_000.0);
                }
                let mut attributes = Vec::with_capacity(16);
                for component in uv {
                    let value = *component as f32 / 65_536.0;
                    push_f32(&mut vertex_bytes, value);
                    push_f32(&mut attributes, value);
                }
                let normal = normals
                    .and_then(|values| values.get(vertex_index))
                    .copied()
                    .unwrap_or([0, 0, 0]);
                push_normal_snorm16(&mut vertex_bytes, normal);
                push_normal_snorm16(&mut attributes, normal);
                mesh_vertex_templates.push(
                    attributes
                        .try_into()
                        .map_err(|_| B0GpuContentError::InvalidCatalog("vertex stride mismatch"))?,
                );
            }
            if vertex_templates
                .insert(revision, mesh_vertex_templates)
                .is_some()
            {
                return Err(B0GpuContentError::InvalidCatalog(
                    "duplicate exact mesh revision",
                ));
            }
            for index in mesh.indices() {
                index_bytes.extend_from_slice(&index.to_le_bytes());
            }

            for primitive in mesh.primitives() {
                let global_first_index = first_index
                    .checked_add(primitive.first_index())
                    .ok_or(B0GpuContentError::CountOverflow)?;
                let offset = indirect_bytes.len() as vk::DeviceSize;
                indirect_bytes.extend_from_slice(&primitive.index_count().to_le_bytes());
                indirect_bytes.extend_from_slice(&1_u32.to_le_bytes());
                indirect_bytes.extend_from_slice(&global_first_index.to_le_bytes());
                indirect_bytes.extend_from_slice(&base_vertex.to_le_bytes());
                indirect_bytes.extend_from_slice(&0_u32.to_le_bytes());
                let key = DrawKey {
                    mesh_revision: revision,
                    first_index: primitive.first_index(),
                    index_count: primitive.index_count(),
                };
                if draw_offsets.insert(key, offset).is_some() {
                    return Err(B0GpuContentError::InvalidCatalog(
                        "duplicate mesh primitive draw key",
                    ));
                }
                if draw_commands
                    .insert(
                        key,
                        PreparedDrawCommand {
                            index_count: primitive.index_count(),
                            first_index: global_first_index,
                        },
                    )
                    .is_some()
                {
                    return Err(B0GpuContentError::InvalidCatalog(
                        "duplicate mesh primitive draw command",
                    ));
                }
            }
        }

        let dynamic_vertex_capacity =
            catalog
                .base_skinning_profiles()
                .iter()
                .try_fold(0_u64, |total, profile| {
                    let vertex_count = u64::try_from(profile.vertices().len())
                        .map_err(|_| B0GpuContentError::CountOverflow)?;
                    let profile_bytes = vertex_count
                        .checked_mul(u64::from(profile.max_instances_per_frame()))
                        .and_then(|value| value.checked_mul(u64::from(VERTEX_STRIDE)))
                        .ok_or(B0GpuContentError::CountOverflow)?;
                    total
                        .checked_add(profile_bytes)
                        .ok_or(B0GpuContentError::CountOverflow)
                })?;

        let index_buffer_offset = vertex_bytes.len() as vk::DeviceSize;
        let mut geometry_bytes = vertex_bytes;
        geometry_bytes.extend_from_slice(&index_bytes);

        let mut staging_bytes = Vec::new();
        let geometry_staging_offset = append_aligned(&mut staging_bytes, &geometry_bytes, 4)?;
        let indirect_staging_offset = append_aligned(&mut staging_bytes, &indirect_bytes, 4)?;
        let mut textures = Vec::with_capacity(catalog.textures().len());
        for texture in catalog.textures() {
            if texture.dimension() != NeutralTextureDimensionV1::D2
                || texture.color_space() != NeutralTextureColorSpaceV1::Srgb
                || texture.texel_encoding() != NeutralTexelEncodingV1::Rgba8Unorm
                || texture.array_layers() != 1
                || texture.mip_levels().len() != 1
            {
                return Err(B0GpuContentError::InvalidCatalog(
                    "texture is outside the closed B0 RGBA8-sRGB profile",
                ));
            }
            let mip = &texture.mip_levels()[0];
            let extent = mip.extent();
            if extent != texture.extent() || extent[2] != 1 {
                return Err(B0GpuContentError::InvalidCatalog(
                    "texture mip extent does not match the 2D image",
                ));
            }
            let staging_offset = append_aligned(&mut staging_bytes, mip.texels(), 4)?;
            textures.push(PreparedTexture {
                revision: texture.asset_revision()?,
                extent: vk::Extent3D {
                    width: extent[0],
                    height: extent[1],
                    depth: 1,
                },
                staging_offset,
            });
        }
        if staging_bytes.is_empty() {
            staging_bytes.resize(MINIMUM_BUFFER_SIZE as usize, 0);
        }

        Ok(Self {
            staging_bytes,
            geometry_staging_offset,
            geometry_payload_size: geometry_bytes.len() as vk::DeviceSize,
            indirect_staging_offset,
            indirect_payload_size: indirect_bytes.len() as vk::DeviceSize,
            index_buffer_offset,
            draw_offsets,
            draw_commands,
            vertex_templates,
            dynamic_vertex_capacity,
            textures,
        })
    }

    fn geometry_size(&self) -> vk::DeviceSize {
        self.geometry_payload_size.max(MINIMUM_BUFFER_SIZE)
    }

    fn indirect_size(&self) -> vk::DeviceSize {
        self.indirect_payload_size.max(MINIMUM_BUFFER_SIZE)
    }

    fn dynamic_vertex_capacity_size(&self) -> vk::DeviceSize {
        self.dynamic_vertex_capacity.max(MINIMUM_BUFFER_SIZE)
    }
}

fn append_aligned(
    destination: &mut Vec<u8>,
    bytes: &[u8],
    alignment: usize,
) -> Result<vk::DeviceSize, B0GpuContentError> {
    let padding = (alignment - destination.len() % alignment) % alignment;
    let padded_len = destination
        .len()
        .checked_add(padding)
        .ok_or(B0GpuContentError::CountOverflow)?;
    destination.resize(padded_len, 0);
    let offset = destination.len() as vk::DeviceSize;
    let final_len = destination
        .len()
        .checked_add(bytes.len())
        .ok_or(B0GpuContentError::CountOverflow)?;
    destination.reserve(bytes.len());
    destination.extend_from_slice(bytes);
    debug_assert_eq!(destination.len(), final_len);
    Ok(offset)
}

fn push_f32(bytes: &mut Vec<u8>, value: f32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

/// Packs one dynamic surface update into the locked 28-byte B0 vertex layout:
/// metre position, planar metre UV and snorm16x4 normal. The planar UV keeps
/// the closed B0 shader interface unchanged for a surface whose topology may
/// differ from the catalog placeholder every frame.
fn pack_dynamic_surface_vertices(
    update: &DynamicSurfaceUpdateV1,
    scratch: &mut Vec<u8>,
) -> Result<(), B0GpuContentError> {
    scratch.clear();
    let bytes = update
        .positions_micrometres()
        .len()
        .checked_mul(VERTEX_STRIDE as usize)
        .ok_or(B0GpuContentError::CountOverflow)?;
    scratch
        .try_reserve(bytes)
        .map_err(|_| B0GpuContentError::CountOverflow)?;
    for (position, normal) in update
        .positions_micrometres()
        .iter()
        .zip(update.normals_snorm16())
    {
        let metres = position.map(|component| component as f32 / 1_000_000.0);
        for component in metres {
            push_f32(scratch, component);
        }
        push_f32(scratch, metres[0]);
        push_f32(scratch, metres[2]);
        push_normal_snorm16(scratch, *normal);
    }
    debug_assert_eq!(scratch.len(), bytes);
    Ok(())
}

fn pack_dynamic_surface_indices(
    update: &DynamicSurfaceUpdateV1,
    scratch: &mut Vec<u8>,
) -> Result<(), B0GpuContentError> {
    scratch.clear();
    let bytes = update
        .indices()
        .len()
        .checked_mul(size_of::<u32>())
        .ok_or(B0GpuContentError::CountOverflow)?;
    scratch
        .try_reserve(bytes)
        .map_err(|_| B0GpuContentError::CountOverflow)?;
    for index in update.indices() {
        scratch.extend_from_slice(&index.to_le_bytes());
    }
    Ok(())
}

fn push_normal_snorm16(bytes: &mut Vec<u8>, normal: [i16; 3]) {
    for component in normal {
        bytes.extend_from_slice(&component.to_le_bytes());
    }
    bytes.extend_from_slice(&0_i16.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upload_payload_offsets_are_four_byte_aligned() {
        let mut bytes = Vec::new();
        assert_eq!(
            append_aligned(&mut bytes, &[1, 2, 3], 4).expect("first payload"),
            0
        );
        assert_eq!(
            append_aligned(&mut bytes, &[4, 5], 4).expect("second payload"),
            4
        );
        assert_eq!(bytes, vec![1, 2, 3, 0, 4, 5]);
    }

    #[test]
    fn authored_and_fallback_normals_pack_into_exact_snorm16x4_vertices() {
        let mut authored = Vec::new();
        push_normal_snorm16(&mut authored, [i16::MIN + 1, i16::MAX, 17]);
        assert_eq!(authored.len(), 8);
        assert_eq!(&authored[0..2], &(i16::MIN + 1).to_le_bytes());
        assert_eq!(&authored[2..4], &i16::MAX.to_le_bytes());
        assert_eq!(&authored[4..6], &17_i16.to_le_bytes());
        assert_eq!(&authored[6..8], &0_i16.to_le_bytes());

        let mut fallback = Vec::new();
        push_normal_snorm16(&mut fallback, [0; 3]);
        assert_eq!(fallback, [0; 8]);
    }

    #[test]
    fn dynamic_surface_update_packs_into_the_locked_b0_vertex_layout() {
        use next_contracts::ids::AssetId;
        use next_contracts::project::domain_hash;

        let update = DynamicSurfaceUpdateV1::new(
            AssetRevisionRefV1 {
                asset_id: AssetId::from_bytes([0xd1; 16]),
                record_sha256: domain_hash("test.dynamic-surface.mesh", b"mesh"),
            },
            1,
            vec![[1_000_000, 250_000, -500_000], [0, 0, 0], [2_000_000, 0, 0]],
            vec![[0, i16::MAX, 0], [i16::MAX, 0, 0], [0, 0, i16::MIN + 1]],
            vec![0, 2, 1],
        )
        .expect("valid update");
        let mut scratch = Vec::new();
        pack_dynamic_surface_vertices(&update, &mut scratch).expect("packs vertices");
        assert_eq!(scratch.len(), 3 * VERTEX_STRIDE as usize);
        assert_eq!(&scratch[0..4], &1.0_f32.to_le_bytes());
        assert_eq!(&scratch[4..8], &0.25_f32.to_le_bytes());
        assert_eq!(&scratch[8..12], &(-0.5_f32).to_le_bytes());
        assert_eq!(&scratch[12..16], &1.0_f32.to_le_bytes());
        assert_eq!(&scratch[16..20], &(-0.5_f32).to_le_bytes());
        assert_eq!(&scratch[22..24], &i16::MAX.to_le_bytes());
        assert_eq!(&scratch[26..28], &0_i16.to_le_bytes());
        pack_dynamic_surface_indices(&update, &mut scratch).expect("packs indices");
        assert_eq!(scratch, [0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0]);
    }
}
