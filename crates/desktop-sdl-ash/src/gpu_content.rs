mod pipeline;
mod resources;
mod ui_overlay_gpu;

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::mem::size_of;

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
use self::resources::{BufferAllocation, DescriptorState, TextureResource, upload_content};
pub(crate) use self::ui_overlay_gpu::UiOverlayState;

const VERTEX_STRIDE: u32 = 20;
const INDIRECT_COMMAND_STRIDE: u32 = 20;
const FRAME_UNIFORM_SIZE: vk::DeviceSize = 64;
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
    pipeline: PipelineState,
    descriptors: DescriptorState,
    textures: BTreeMap<AssetRevisionRefV1, TextureResource>,
    indirect: BufferAllocation,
    frame_uniforms: Vec<BufferAllocation>,
    geometry: BufferAllocation,
    index_buffer_offset: vk::DeviceSize,
    draw_offsets: BTreeMap<DrawKey, vk::DeviceSize>,
    catalog_hash: ContentHash,
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
    ) -> Result<Self, B0GpuContentError> {
        if frame_slot_count == 0 {
            return Err(B0GpuContentError::InvalidCatalog(
                "frame slot count must be non-zero",
            ));
        }
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

        let descriptors = DescriptorState::new(device, &frame_uniforms, &textures)?;
        let pipeline = PipelineState::new(
            device,
            color_format,
            depth_format,
            descriptors.frame_layout,
            descriptors.texture_layout,
        )?;

        Ok(Self {
            pipeline,
            descriptors,
            textures,
            indirect,
            frame_uniforms,
            geometry,
            index_buffer_offset: prepared.index_buffer_offset,
            draw_offsets: prepared.draw_offsets,
            catalog_hash: catalog.catalog_sha256(),
        })
    }

    /// Records the CPU-selected visible list as one indexed-indirect command
    /// per draw. Dynamic rendering must already be active for `color_format`.
    pub(super) fn record(
        &self,
        command_buffer: vk::CommandBuffer,
        plan: &B0FramePlanV1,
        extent: vk::Extent2D,
        frame_slot_index: usize,
    ) -> Result<(), B0GpuContentError> {
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
        }

        for draw in &plan.draws {
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
                self.geometry.device.cmd_draw_indexed_indirect(
                    command_buffer,
                    self.indirect.buffer,
                    indirect_offset,
                    1,
                    INDIRECT_COMMAND_STRIDE,
                );
            }
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
        for texture in self.textures.values() {
            bytes = bytes
                .checked_add(texture.allocation_size())
                .ok_or(B0GpuContentError::CountOverflow)?;
        }
        let texture_count =
            u64::try_from(self.textures.len()).map_err(|_| B0GpuContentError::CountOverflow)?;
        let frame_uniform_count = u64::try_from(self.frame_uniforms.len())
            .map_err(|_| B0GpuContentError::CountOverflow)?;
        let allocation_count = 2_u64
            .checked_add(frame_uniform_count)
            .and_then(|value| value.checked_add(texture_count))
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
    textures: Vec<PreparedTexture>,
}

impl PreparedContent {
    fn from_catalog(catalog: &RenderContentCatalogV1) -> Result<Self, B0GpuContentError> {
        let mut vertex_bytes = Vec::new();
        let mut index_bytes = Vec::new();
        let mut indirect_bytes = Vec::new();
        let mut draw_offsets = BTreeMap::new();

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
            for (position, uv) in mesh.positions_micrometres().iter().zip(uv0) {
                for component in position {
                    push_f32(&mut vertex_bytes, *component as f32 / 1_000_000.0);
                }
                for component in uv {
                    push_f32(&mut vertex_bytes, *component as f32 / 65_536.0);
                }
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
            }
        }

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
            textures,
        })
    }

    fn geometry_size(&self) -> vk::DeviceSize {
        self.geometry_payload_size.max(MINIMUM_BUFFER_SIZE)
    }

    fn indirect_size(&self) -> vk::DeviceSize {
        self.indirect_payload_size.max(MINIMUM_BUFFER_SIZE)
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
}
