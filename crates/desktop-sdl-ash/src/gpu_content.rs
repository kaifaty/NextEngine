pub(crate) mod ao;
pub(crate) mod fluid;
pub(crate) mod gbuffer;
mod pipeline;
pub(crate) mod post;
pub(crate) mod taa;
pub(crate) mod water;
pub(crate) use pipeline::B0_SUN_DIRECTION_INTENSITY;
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
    PipelineState, SHADOW_CASCADE_EXTENTS_METRES, draw_push_constant_bytes,
    frame_raster_state_jittered, identity_matrix_bytes, micrometres_to_metres_f32,
    mirrored_frame_raster_state, model_matrix, shadow_cascade_matrices,
};
pub(crate) use self::pipeline::{ProjectionJitterV1, projection_jitter};
pub(crate) use self::resources::{BufferAllocation, DepthAttachment};
use self::resources::{
    DescriptorState, MaterialMapsV1, MaterialPlaceholdersV1, ShadowMap, TextureResource,
    WhiteTexture, upload_content,
};
use self::shadow::{ShadowPipelineState, initialize_shadow_map};
pub(crate) use self::ui_overlay_gpu::UiOverlayState;
use crate::dynamic_surface::DynamicSurfaceShadingV1;
use crate::dynamic_surface::{
    DynamicSurfaceProfileV1, DynamicSurfaceResidencyV1, DynamicSurfaceUpdateV1, validate_profiles,
};

const VERTEX_STRIDE: u32 = 28;
const INDIRECT_COMMAND_STRIDE: u32 = 20;
const FRAME_UNIFORM_SIZE: vk::DeviceSize = 208;
const DRAW_PUSH_CONSTANT_SIZE: u32 = 96;
/// Scene look L1: the fog's density per metre (the colour comes from the sky).
pub(super) const B0_FOG_DENSITY: f32 = 0.006;
/// Scene look L1: the material lane of a draw whose material record is
/// unknown (fallback material): dielectric, matte.
const DEFAULT_MATERIAL_PARAMS: [f32; 4] = [0.0, 0.8, 0.0, 1.0];
/// Scene look L1: the material lane of a `WaterSurface` ring draw (the
/// water programs do not read it).
const WATER_RING_MATERIAL_PARAMS: [f32; 4] = [0.0, 0.05, 0.0, 1.0];
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
    /// Plan `continuum-water/12`: the material suite of `WaterSurface` rings.
    water_pipeline: PipelineState,
    pipeline: PipelineState,
    shadow_pipeline: Option<ShadowPipelineState>,
    descriptors: DescriptorState,
    textures: BTreeMap<AssetRevisionRefV1, TextureResource>,
    shadow_map: Option<ShadowMap>,
    indirect: BufferAllocation,
    frame_uniforms: Vec<BufferAllocation>,
    /// Scene look L1: the `LightingUniforms` block per frame slot.
    lighting_uniforms: Vec<BufferAllocation>,
    /// Scene look L1: the analytic sky the lighting block is built from.
    sky: crate::sky::SkyModel,
    /// Scene look L7: the post chain owns the fog (the block carries a
    /// zero density for the world programs).
    volumetric_fog: bool,
    /// Scene look L1: metallic, roughness and emissive intensity per
    /// material revision, read from the catalog's material records.
    materials: BTreeMap<AssetRevisionRefV1, [f32; 4]>,
    /// Scene look L3: the `1 x 1` white image bound at set 2 binding 1
    /// until the occlusion pass binds its target.
    white: WhiteTexture,
    /// Scene look L5: the flat placeholders of the material bindings, held
    /// for the descriptor sets that reference their views.
    #[allow(
        dead_code,
        reason = "owned for the lifetime of the sets that sample it"
    )]
    placeholders: Box<MaterialPlaceholdersV1>,
    dynamic_vertices: Vec<BufferAllocation>,
    geometry: BufferAllocation,
    index_buffer_offset: vk::DeviceSize,
    draw_offsets: BTreeMap<DrawKey, vk::DeviceSize>,
    draw_commands: BTreeMap<DrawKey, PreparedDrawCommand>,
    vertex_templates: BTreeMap<AssetRevisionRefV1, Vec<[u8; 16]>>,
    dynamic_vertex_scratch: Vec<u8>,
    dynamic_vertex_offsets: Vec<i32>,
    dynamic_surfaces: BTreeMap<AssetRevisionRefV1, DynamicSurfaceRing>,
    catalog_hash: ContentHash,
}

/// Per-frame-slot vertex/index storage for one declared presentation-only
/// dynamic surface. Capacity is fixed at creation.
struct DynamicSurfaceRing {
    profile: DynamicSurfaceProfileV1,
    /// The catalog mesh's horizontal extent (`x` times `z`, square
    /// micrometres): plan 15 mirrors about the largest water surface.
    plan_area_square_micrometres: i128,
    /// Plan 33: the catalog mesh's `x z` bounds (micrometres, relative to
    /// the draw translation) for the submersion test.
    plan_bounds_micrometres: ([i64; 2], [i64; 2]),
    slots: Vec<DynamicSurfaceSlot>,
}

/// `vertices`/`indices` are what draws bind. For `HostVisible` they are the
/// host-coherent allocations written directly; for `DeviceLocal` they are
/// device-local and `staging` holds the host-coherent pair whose pending
/// byte counts are copied by `record_dynamic_surface_uploads`.
struct DynamicSurfaceSlot {
    vertices: BufferAllocation,
    indices: BufferAllocation,
    staging: Option<DynamicSurfaceStaging>,
    uploaded: Option<UploadedDynamicSurface>,
}

struct DynamicSurfaceStaging {
    vertices: BufferAllocation,
    indices: BufferAllocation,
    pending_vertex_bytes: vk::DeviceSize,
    pending_index_bytes: vk::DeviceSize,
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

/// How the plan's `WaterSurface` rings are treated by a draw recording.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WaterRingModeV1 {
    /// No water pass: the rings draw through the WL1 suite in the world pass.
    InWorldPass,
    /// The water pass draws them later (plan 13) and the reflection pass
    /// never draws them (plan 15).
    Skip,
}

#[derive(Clone, Copy)]
struct DynamicDrawBinding {
    vertex_buffer: vk::Buffer,
    index_buffer: vk::Buffer,
    index_count: u32,
    shading: DynamicSurfaceShadingV1,
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
        let mut lighting_uniforms = Vec::with_capacity(frame_slot_count);
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
            lighting_uniforms.push(BufferAllocation::new(
                instance,
                physical_device,
                device,
                crate::sky::LIGHTING_UNIFORM_SIZE,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?);
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
            let host =
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
            let mut slots = Vec::with_capacity(frame_slot_count);
            for _ in 0..frame_slot_count {
                let allocate = |size, usage, memory| {
                    BufferAllocation::new(instance, physical_device, device, size, usage, memory)
                };
                slots.push(match profile.residency {
                    DynamicSurfaceResidencyV1::HostVisible => DynamicSurfaceSlot {
                        vertices: allocate(
                            vertex_bytes,
                            vk::BufferUsageFlags::VERTEX_BUFFER,
                            host,
                        )?,
                        indices: allocate(index_bytes, vk::BufferUsageFlags::INDEX_BUFFER, host)?,
                        staging: None,
                        uploaded: None,
                    },
                    DynamicSurfaceResidencyV1::DeviceLocal => DynamicSurfaceSlot {
                        vertices: allocate(
                            vertex_bytes,
                            vk::BufferUsageFlags::VERTEX_BUFFER
                                | vk::BufferUsageFlags::TRANSFER_DST,
                            vk::MemoryPropertyFlags::DEVICE_LOCAL,
                        )?,
                        indices: allocate(
                            index_bytes,
                            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
                            vk::MemoryPropertyFlags::DEVICE_LOCAL,
                        )?,
                        staging: Some(DynamicSurfaceStaging {
                            vertices: allocate(
                                vertex_bytes,
                                vk::BufferUsageFlags::TRANSFER_SRC,
                                host,
                            )?,
                            indices: allocate(
                                index_bytes,
                                vk::BufferUsageFlags::TRANSFER_SRC,
                                host,
                            )?,
                            pending_vertex_bytes: 0,
                            pending_index_bytes: 0,
                        }),
                        uploaded: None,
                    },
                });
            }
            if dynamic_surfaces
                .insert(
                    profile.mesh_revision,
                    DynamicSurfaceRing {
                        profile: *profile,
                        plan_area_square_micrometres: catalog
                            .meshes()
                            .iter()
                            .find(|mesh| mesh.asset_revision().ok() == Some(profile.mesh_revision))
                            .map_or(0, |mesh| {
                                let bounds = mesh.bounds();
                                i128::from(bounds.max()[0] - bounds.min()[0])
                                    * i128::from(bounds.max()[2] - bounds.min()[2])
                            }),
                        plan_bounds_micrometres: catalog
                            .meshes()
                            .iter()
                            .find(|mesh| mesh.asset_revision().ok() == Some(profile.mesh_revision))
                            .map_or(([0; 2], [0; 2]), |mesh| {
                                let bounds = mesh.bounds();
                                (
                                    [bounds.min()[0], bounds.min()[2]],
                                    [bounds.max()[0], bounds.max()[2]],
                                )
                            }),
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
            let resource = TextureResource::new_array(
                instance,
                physical_device,
                device,
                texture.mips[0].extent,
                texture.format,
                u32::try_from(texture.mips.len()).map_err(|_| B0GpuContentError::CountOverflow)?,
                texture.layers,
            )?;
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
            eprintln!(
                "next_game: SHADOW_CASCADES active layers={} extents={}/{}/{}",
                super::gpu_content::resources::SHADOW_CASCADES,
                SHADOW_CASCADE_EXTENTS_METRES[0],
                SHADOW_CASCADE_EXTENTS_METRES[1],
                SHADOW_CASCADE_EXTENTS_METRES[2]
            );
        } else {
            eprintln!(
                "next_game: SHADOW_MAP_FALLBACK: sampled depth format or 2048x2048 allocation unavailable"
            );
        }
        // Scene look L3: the white placeholder of the occlusion binding.
        let white = WhiteTexture::new(instance, physical_device, device)?;
        resources::initialize_white_texture(device, queue, queue_family_index, &white)?;
        // Scene look L5: the flat placeholders of the material bindings
        // (white base, roughness one and metallic zero, a flat normal).
        let placeholders = Box::new(MaterialPlaceholdersV1 {
            white: WhiteTexture::solid(
                instance,
                physical_device,
                device,
                vk::Format::R8G8B8A8_SRGB,
                [1.0, 1.0, 1.0, 1.0],
            )?,
            metallic_roughness: WhiteTexture::solid(
                instance,
                physical_device,
                device,
                vk::Format::R8G8B8A8_UNORM,
                [0.0, 1.0, 0.0, 1.0],
            )?,
            normal: WhiteTexture::solid(
                instance,
                physical_device,
                device,
                vk::Format::R8G8B8A8_UNORM,
                [0.5, 0.5, 1.0, 1.0],
            )?,
            splat_control: WhiteTexture::solid(
                instance,
                physical_device,
                device,
                vk::Format::R8G8B8A8_UNORM,
                [1.0, 0.0, 0.0, 0.0],
            )?,
        });
        for placeholder in [
            &placeholders.white,
            &placeholders.metallic_roughness,
            &placeholders.normal,
            &placeholders.splat_control,
        ] {
            resources::initialize_white_texture(device, queue, queue_family_index, placeholder)?;
        }
        let descriptors = DescriptorState::new(
            device,
            &frame_uniforms,
            &lighting_uniforms,
            &textures,
            shadow_map.as_ref(),
            &white,
            &prepared.material_maps,
            &placeholders,
        )?;
        let with_normal = prepared
            .material_maps
            .values()
            .filter(|maps| maps.normal.is_some())
            .count();
        let with_metallic_roughness = prepared
            .material_maps
            .values()
            .filter(|maps| maps.metallic_roughness.is_some())
            .count();
        let mip_levels: usize = prepared
            .textures
            .iter()
            .map(|texture| texture.mips.len())
            .sum();
        eprintln!(
            "next_game: MATERIAL_MAPS active materials={} normal={with_normal} metallic_roughness={with_metallic_roughness} mips={mip_levels}",
            prepared.material_maps.len()
        );
        // Scene look L6a: the splat materials and their layer count.
        let splat_materials = prepared
            .material_maps
            .values()
            .filter(|maps| maps.splat_control.is_some())
            .count();
        if splat_materials > 0 {
            let layers = prepared
                .textures
                .iter()
                .map(|texture| texture.layers)
                .max()
                .unwrap_or(1);
            eprintln!("next_game: TERRAIN active materials={splat_materials} layers={layers}");
        }
        // Scene look L1: the sky for the B0 sun; the lighting block is
        // written per frame from it.
        let sun = B0_SUN_DIRECTION_INTENSITY;
        let sky = crate::sky::SkyModel::new([-sun[0], -sun[1], -sun[2]], crate::sky::SKY_TURBIDITY);
        let pipeline = PipelineState::new(
            device,
            color_format,
            depth_format,
            descriptors.frame_layout,
            descriptors.texture_layout,
            descriptors.shadow_layout,
            shadow_map.is_some(),
        )?;
        let sky_pipeline =
            PipelineState::new_sky(device, color_format, depth_format, descriptors.frame_layout)?;
        let water_pipeline = PipelineState::new_water_surface(
            device,
            color_format,
            depth_format,
            descriptors.frame_layout,
            descriptors.texture_layout,
            descriptors.shadow_layout,
        )?;
        let shadow_pipeline = shadow_map
            .as_ref()
            .map(|shadow| {
                ShadowPipelineState::new(device, shadow.format(), descriptors.frame_layout)
            })
            .transpose()?;

        Ok(Self {
            sky_pipeline,
            water_pipeline,
            pipeline,
            shadow_pipeline,
            descriptors,
            textures,
            shadow_map,
            indirect,
            frame_uniforms,
            lighting_uniforms,
            sky,
            volumetric_fog: false,
            materials: prepared.materials,
            white,
            placeholders,
            dynamic_vertices,
            geometry,
            index_buffer_offset: prepared.index_buffer_offset,
            draw_offsets: prepared.draw_offsets,
            draw_commands: prepared.draw_commands,
            vertex_templates: prepared.vertex_templates,
            dynamic_vertex_scratch: Vec::new(),
            dynamic_vertex_offsets: Vec::new(),
            dynamic_surfaces,
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
            let vertex_bytes = u64::from(update.vertex_count())
                .checked_mul(u64::from(VERTEX_STRIDE))
                .ok_or(B0GpuContentError::CountOverflow)?;
            let index_bytes = u64::from(update.index_count())
                .checked_mul(size_of::<u32>() as u64)
                .ok_or(B0GpuContentError::CountOverflow)?;
            // The payload was packed on the producer thread; this is a copy
            // into mapped memory only.
            match slot.staging.as_mut() {
                Some(staging) => {
                    staging.vertices.write(0, update.packed_b0_vertices())?;
                    staging.indices.write(0, update.packed_b0_indices())?;
                    staging.pending_vertex_bytes = vertex_bytes;
                    staging.pending_index_bytes = index_bytes;
                }
                None => {
                    slot.vertices.write(0, update.packed_b0_vertices())?;
                    slot.indices.write(0, update.packed_b0_indices())?;
                }
            }
            slot.uploaded = Some(UploadedDynamicSurface {
                canonical_hash: update.canonical_hash(),
                index_count: update.index_count(),
            });
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

    /// Records the staging-to-device copies queued by
    /// [`Self::prepare_dynamic_surfaces`] for device-local rings. Must run
    /// outside any rendering instance and before the first pass that reads
    /// the ring (the shadow pass).
    pub(super) fn record_dynamic_surface_uploads(
        &mut self,
        command_buffer: vk::CommandBuffer,
        frame_slot_index: usize,
    ) -> Result<(), B0GpuContentError> {
        let mut barriers = Vec::new();
        for ring in self.dynamic_surfaces.values_mut() {
            let slot =
                ring.slots
                    .get_mut(frame_slot_index)
                    .ok_or(B0GpuContentError::InvalidFramePlan(
                        "frame slot index is outside the dynamic surface ring",
                    ))?;
            let Some(staging) = slot.staging.as_mut() else {
                continue;
            };
            if staging.pending_vertex_bytes == 0 || staging.pending_index_bytes == 0 {
                continue;
            }
            let copies = [
                (
                    staging.vertices.buffer,
                    slot.vertices.buffer,
                    staging.pending_vertex_bytes,
                ),
                (
                    staging.indices.buffer,
                    slot.indices.buffer,
                    staging.pending_index_bytes,
                ),
            ];
            for (source, destination, size) in copies {
                let regions = [vk::BufferCopy::default()
                    .src_offset(0)
                    .dst_offset(0)
                    .size(size)];
                // SAFETY: both buffers belong to this device, the slot fence
                // completed so neither is read by pending work, and the size
                // was bounded by the declared capacity in prepare.
                unsafe {
                    self.geometry.device.cmd_copy_buffer(
                        command_buffer,
                        source,
                        destination,
                        &regions,
                    );
                }
                barriers.push(
                    vk::BufferMemoryBarrier2::default()
                        .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                        .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                        .dst_stage_mask(vk::PipelineStageFlags2::VERTEX_INPUT)
                        .dst_access_mask(
                            vk::AccessFlags2::VERTEX_ATTRIBUTE_READ | vk::AccessFlags2::INDEX_READ,
                        )
                        .buffer(destination)
                        .offset(0)
                        .size(size),
                );
            }
            staging.pending_vertex_bytes = 0;
            staging.pending_index_bytes = 0;
        }
        if !barriers.is_empty() {
            let dependency = vk::DependencyInfo::default().buffer_memory_barriers(&barriers);
            // SAFETY: the barriers reference buffers copied above in this
            // same command buffer and make them visible to vertex input.
            unsafe {
                self.geometry
                    .device
                    .cmd_pipeline_barrier2(command_buffer, &dependency);
            }
        }
        Ok(())
    }

    /// Returns the dynamic ring binding for a draw whose mesh revision is a
    /// declared surface with an uploaded update in this frame slot.
    fn dynamic_draw_binding(
        &self,
        mesh_revision: AssetRevisionRefV1,
        frame_slot_index: usize,
    ) -> Option<DynamicDrawBinding> {
        let ring = self.dynamic_surfaces.get(&mesh_revision)?;
        let slot = ring.slots.get(frame_slot_index)?;
        let uploaded = slot.uploaded?;
        Some(DynamicDrawBinding {
            vertex_buffer: slot.vertices.buffer,
            index_buffer: slot.indices.buffer,
            index_count: uploaded.index_count,
            shading: ring.profile.shading,
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
        skip_water_surfaces: bool,
        jitter: Option<ProjectionJitterV1>,
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
        // The skinned vertex stream of this frame slot was prepared by
        // `prepare_dynamic_vertices` before the first pass of the frame.
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
        let raster_state = frame_raster_state_jittered(plan.camera.as_ref(), extent, jitter)?;
        frame_uniform.write(0, &raster_state.view_projection_bytes)?;
        // Scene look L1: the lighting block of the frame (the sky's static
        // part and the inverse of this frame's view-projection).
        let lighting_uniform = self.lighting_uniforms.get(frame_slot_index).ok_or(
            B0GpuContentError::InvalidFramePlan(
                "frame slot index is outside the allocated lighting ring",
            ),
        )?;
        let cascades = shadow_cascade_matrices(raster_state.camera_position)?;
        lighting_uniform.write(
            0,
            &self.sky.lighting_uniform_bytes(
                crate::sky::invert_matrix(raster_state.view_projection),
                if self.volumetric_fog {
                    0.0
                } else {
                    B0_FOG_DENSITY
                },
                &cascades,
                SHADOW_CASCADE_EXTENTS_METRES.map(|extent| extent as f32),
            ),
        )?;
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

        self.record_plan_draws(
            command_buffer,
            plan,
            frame_slot_index,
            self.pipeline.layout,
            if skip_water_surfaces {
                WaterRingModeV1::Skip
            } else {
                WaterRingModeV1::InWorldPass
            },
            None,
        )
    }

    /// Plan 18: the B0 texture set layout, reused as set 1 of the G-buffer
    /// suite.
    pub(super) fn texture_layout(&self) -> vk::DescriptorSetLayout {
        self.descriptors.texture_layout
    }

    /// Plan 18: records the plan's draws into the G-buffer pass with the
    /// frame's jittered camera; a G-buffer rendering instance must be
    /// active. Water rings draw here with the G-buffer suite like every
    /// other draw.
    pub(super) fn record_gbuffer(
        &mut self,
        command_buffer: vk::CommandBuffer,
        plan: &B0FramePlanV1,
        extent: vk::Extent2D,
        frame_slot_index: usize,
        gbuffer: &mut gbuffer::GBufferPassState,
        jitter: Option<ProjectionJitterV1>,
    ) -> Result<(), B0GpuContentError> {
        let raster_state = frame_raster_state_jittered(plan.camera.as_ref(), extent, jitter)?;
        let jitter_pixels = jitter.map_or([0.0; 2], |jitter| jitter.pixels);
        let set = gbuffer.prepare(
            frame_slot_index,
            raster_state.view_projection,
            jitter_pixels,
        )?;
        let viewports = [raster_state.viewport];
        let scissors = [raster_state.scissor];
        let vertex_buffers = [self.geometry.buffer];
        let vertex_offsets = [0];
        let layout = gbuffer.layout();
        // SAFETY: the G-buffer pipeline owns its layout; the set and
        // buffers are live on this device inside the G-buffer rendering
        // instance.
        unsafe {
            self.geometry.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                gbuffer.pipeline(),
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
                layout,
                0,
                &[set],
                &[],
            );
        }
        self.record_plan_draws(
            command_buffer,
            plan,
            frame_slot_index,
            layout,
            WaterRingModeV1::InWorldPass,
            Some(gbuffer),
        )?;
        gbuffer.end_frame_draws();
        Ok(())
    }

    /// Records the plan's draws for the currently bound pipeline and frame
    /// set (the world pass or the mirrored reflection pass of plan 15).
    fn record_plan_draws(
        &mut self,
        command_buffer: vk::CommandBuffer,
        plan: &B0FramePlanV1,
        frame_slot_index: usize,
        layout: vk::PipelineLayout,
        water_rings: WaterRingModeV1,
        mut gbuffer: Option<&mut gbuffer::GBufferPassState>,
    ) -> Result<u64, B0GpuContentError> {
        let mut dynamic_surface_draws = 0_u64;
        for (draw_index, draw) in plan.draws.iter().enumerate() {
            let dynamic_binding = self.dynamic_draw_binding(draw.mesh_revision, frame_slot_index);
            // Plan 13/15: with the water pass active, water rings draw there
            // and never in the world or reflection passes.
            if water_rings == WaterRingModeV1::Skip
                && dynamic_binding
                    .is_some_and(|binding| binding.shading == DynamicSurfaceShadingV1::WaterSurface)
            {
                continue;
            }
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
            // Scene look L5: the material's set (its maps), else the base
            // texture's set with the flat placeholders.
            let texture_set = self
                .descriptors
                .material_sets
                .get(&draw.material_revision)
                .or_else(|| self.descriptors.texture_sets.get(&draw.texture_revision))
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
            let draw_bytes = draw_push_constant_bytes(
                draw.transform,
                draw.base_color_rgba_unorm16,
                self.material_params(draw),
            );
            // Plan 18: the G-buffer pass pushes the draw bytes plus `meta`
            // and stores the draw's previous model; draws beyond its bound
            // are skipped there.
            let push_constants: Vec<u8> = match gbuffer.as_deref_mut() {
                Some(gbuffer) => {
                    let group = if dynamic_binding.is_some_and(|binding| {
                        binding.shading == DynamicSurfaceShadingV1::WaterSurface
                    }) {
                        gbuffer::GBufferGroupV1::WaterSurface
                    } else if dynamic_binding.is_some() {
                        gbuffer::GBufferGroupV1::DynamicSurface
                    } else if draw.skinning_vertex_stream_index.is_some() {
                        gbuffer::GBufferGroupV1::Character
                    } else {
                        gbuffer::GBufferGroupV1::Environment
                    };
                    let Some(bytes) = gbuffer.push_bytes(
                        frame_slot_index,
                        u32::try_from(draw_index).map_err(|_| B0GpuContentError::CountOverflow)?,
                        (
                            draw.mesh_revision,
                            draw.material_revision,
                            draw.texture_revision,
                        ),
                        model_matrix(draw.transform),
                        draw_bytes,
                        group,
                    )?
                    else {
                        continue;
                    };
                    bytes.to_vec()
                }
                None => draw_bytes.to_vec(),
            };
            let water_suite_switch = gbuffer.is_none();

            // SAFETY: descriptor set one was allocated from the pipeline's
            // texture layout, push bytes exactly cover its declared 80-byte
            // range, and the indirect offset selects one initialized command.
            unsafe {
                self.geometry.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    layout,
                    1,
                    &texture_sets,
                    &[],
                );
                self.geometry.device.cmd_push_constants(
                    command_buffer,
                    layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    &push_constants,
                );
                if let Some(binding) = dynamic_binding {
                    // Plan 12: a water ring draws through the water suite on
                    // the same layout, so the bound sets and push constants
                    // stay valid; the world suite is rebound afterwards.
                    if water_suite_switch
                        && binding.shading == DynamicSurfaceShadingV1::WaterSurface
                    {
                        self.geometry.device.cmd_bind_pipeline(
                            command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            self.water_pipeline.pipeline,
                        );
                    }
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
                    // stream and the world suite again.
                    self.geometry.device.cmd_bind_index_buffer(
                        command_buffer,
                        self.geometry.buffer,
                        self.index_buffer_offset,
                        vk::IndexType::UINT32,
                    );
                    if water_suite_switch
                        && binding.shading == DynamicSurfaceShadingV1::WaterSurface
                    {
                        self.geometry.device.cmd_bind_pipeline(
                            command_buffer,
                            vk::PipelineBindPoint::GRAPHICS,
                            self.pipeline.pipeline,
                        );
                    }
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

    /// Uploads the plan's skinned vertex streams into this frame slot's
    /// dynamic vertex ring and records their base offsets. Runs once per
    /// frame before any pass that draws the plan (the shadow and reflection
    /// passes draw skinned meshes before the main pass; plan 29 found the
    /// reflection of the first frame after a device recovery reading the
    /// offsets of a context that had prepared none).
    pub(super) fn prepare_dynamic_vertices(
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
    /// The B0 descriptor layouts (frame, texture, shadow) the water pass
    /// pipeline shares; `None` without a shadow map, which the pass needs.
    pub(super) fn water_pass_layouts(
        &self,
    ) -> Option<(
        vk::DescriptorSetLayout,
        vk::DescriptorSetLayout,
        vk::DescriptorSetLayout,
    )> {
        self.descriptors.shadow_set?;
        Some((
            self.descriptors.frame_layout,
            self.descriptors.texture_layout,
            self.descriptors.shadow_layout,
        ))
    }

    /// Scene look L1: the material lane of a draw from its material record.
    fn material_params(&self, draw: &next_render::B0IndexedDrawV1) -> [f32; 4] {
        self.materials
            .get(&draw.material_revision)
            .copied()
            .unwrap_or(DEFAULT_MATERIAL_PARAMS)
    }

    /// Scene look L1: the exposure the tone map applies (the sky model's).
    pub(super) fn exposure(&self) -> f32 {
        self.sky.exposure()
    }

    /// Scene look L3: the B0 frame set of a slot (set 0), for passes that
    /// read the frame and lighting blocks (the occlusion pass).
    pub(super) fn frame_set(
        &self,
        frame_slot_index: usize,
    ) -> Result<vk::DescriptorSet, B0GpuContentError> {
        self.descriptors
            .frame_sets
            .get(frame_slot_index)
            .copied()
            .ok_or(B0GpuContentError::InvalidFramePlan(
                "frame slot index is outside the descriptor ring",
            ))
    }

    pub(super) fn frame_layout(&self) -> vk::DescriptorSetLayout {
        self.descriptors.frame_layout
    }

    /// Scene look L7 (plan `look/07`): the shadow set (set 2) and its
    /// layout, for the post chain's shafts; `None` without a shadow map.
    pub(super) fn shadow_set_and_layout(
        &self,
    ) -> Option<(vk::DescriptorSet, vk::DescriptorSetLayout)> {
        self.descriptors
            .shadow_set
            .map(|set| (set, self.descriptors.shadow_layout))
    }

    /// Scene look L7: whether the post chain owns the fog; the world
    /// programs' per-pixel fog then yields (a zero density in the block).
    pub(super) fn set_volumetric_fog(&mut self, owned_by_post_chain: bool) {
        self.volumetric_fog = owned_by_post_chain;
    }

    /// Scene look L7: the fog density the post chain marches (the plan 01
    /// ground-level density).
    pub(super) const fn fog_density() -> f32 {
        B0_FOG_DENSITY
    }

    /// Scene look L3: binds the occlusion target at set 2 binding 1 (the
    /// device must be idle: no recorded frame may reference the old view).
    pub(super) fn bind_ambient_occlusion(&self, view: vk::ImageView, sampler: vk::Sampler) {
        self.descriptors.write_ambient_occlusion(view, sampler);
    }

    /// Scene look L3: the white placeholder back at set 2 binding 1, before
    /// an occlusion pass is dropped.
    pub(super) fn bind_ambient_occlusion_placeholder(&self) {
        self.descriptors
            .write_ambient_occlusion(self.white.view(), self.descriptors.sampler());
    }

    /// Scene look L1: the lighting block buffer of every frame slot, for
    /// passes that build their own B0 frame sets (the reflection pass).
    pub(super) fn lighting_buffers(&self) -> Vec<vk::Buffer> {
        self.lighting_uniforms
            .iter()
            .map(|buffer| buffer.buffer)
            .collect()
    }

    /// Whether any declared ring uses the water surface shading.
    pub(super) fn has_water_surface_rings(&self) -> bool {
        self.dynamic_surfaces
            .values()
            .any(|ring| ring.profile.shading == DynamicSurfaceShadingV1::WaterSurface)
    }

    /// Plan 13: draws the uploaded `WaterSurface` rings of the plan through
    /// the water pass pipeline. A water rendering instance with the colour
    /// attachment loaded and the depth attachment read-only must be active.
    /// Returns the number of water draws.
    pub(super) fn record_water_surfaces(
        &self,
        command_buffer: vk::CommandBuffer,
        plan: &B0FramePlanV1,
        frame_slot_index: usize,
        water: &water::WaterPassState,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
    ) -> Result<u64, B0GpuContentError> {
        let frame_set = *self.descriptors.frame_sets.get(frame_slot_index).ok_or(
            B0GpuContentError::InvalidFramePlan("frame slot index is outside the descriptor ring"),
        )?;
        let shadow_set = self
            .descriptors
            .shadow_set
            .ok_or(B0GpuContentError::ResourceMissing(
                "shadow descriptor set for the water pass",
            ))?;
        let water_set = water.set(frame_slot_index)?;
        let viewports = [viewport];
        let scissors = [scissor];
        // SAFETY: all bound objects belong to the same live device, the
        // command buffer is recording inside the water rendering instance,
        // and the layouts of sets 0-2 equal the B0 layouts.
        unsafe {
            self.geometry.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                water.pipeline(),
            );
            self.geometry
                .device
                .cmd_set_viewport(command_buffer, 0, &viewports);
            self.geometry
                .device
                .cmd_set_scissor(command_buffer, 0, &scissors);
            self.geometry.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                water.layout(),
                0,
                &[frame_set],
                &[],
            );
            self.geometry.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                water.layout(),
                2,
                &[shadow_set, water_set],
                &[],
            );
        }
        let mut draws = 0_u64;
        for draw in &plan.draws {
            let Some(binding) = self.dynamic_draw_binding(draw.mesh_revision, frame_slot_index)
            else {
                continue;
            };
            if binding.shading != DynamicSurfaceShadingV1::WaterSurface {
                continue;
            }
            let texture_set = self
                .descriptors
                .texture_sets
                .get(&draw.texture_revision)
                .copied()
                .ok_or(B0GpuContentError::ResourceMissing(
                    "exact base-color texture descriptor",
                ))?;
            let push_constants = draw_push_constant_bytes(
                draw.transform,
                draw.base_color_rgba_unorm16,
                WATER_RING_MATERIAL_PARAMS,
            );
            // SAFETY: the texture set matches set layout 1, the push bytes
            // cover the declared 80-byte range, and the ring buffers hold
            // this slot's uploaded update.
            unsafe {
                self.geometry.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    water.layout(),
                    1,
                    &[texture_set],
                    &[],
                );
                self.geometry.device.cmd_push_constants(
                    command_buffer,
                    water.layout(),
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    &push_constants,
                );
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
            }
            draws = draws
                .checked_add(1)
                .ok_or(B0GpuContentError::CountOverflow)?;
        }
        Ok(draws)
    }

    /// The mirror plane of plan 15: the translation `y` (metres) of the
    /// `WaterSurface` ring draw (with an uploaded update) whose catalog mesh
    /// has the largest horizontal extent (the basin of the reference scene).
    pub(super) fn water_plane_height_metres(
        &self,
        plan: &B0FramePlanV1,
        frame_slot_index: usize,
    ) -> Option<f32> {
        plan.draws
            .iter()
            .filter_map(|draw| {
                let ring = self.dynamic_surfaces.get(&draw.mesh_revision)?;
                let binding = self.dynamic_draw_binding(draw.mesh_revision, frame_slot_index)?;
                (binding.shading == DynamicSurfaceShadingV1::WaterSurface)
                    .then_some((ring.plan_area_square_micrometres, draw))
            })
            .max_by_key(|(area, _)| *area)
            .map(|(_, draw)| draw.transform.translation_micrometres[1] as f32 / 1_000_000.0)
    }

    /// Plan 33: the level of the water ring the camera is submerged in,
    /// from the frame's rings and the eye of the plan's camera.
    pub(super) fn water_submersion_level_metres(
        &self,
        plan: &B0FramePlanV1,
        rings: &[water::WaterRingPlanV1],
    ) -> Result<Option<f32>, B0GpuContentError> {
        let Some(camera) = plan.camera.as_ref() else {
            return Ok(None);
        };
        let eye =
            micrometres_to_metres_f32(camera.current_result_sample.pose.translation_micrometres)?;
        Ok(water::water_submersion_level(eye, rings))
    }

    /// Plans 33 and 35: the water rings of the frame (the catalog mesh's
    /// `x z` bounds plus the draw translation, the level from the
    /// translation), for the submersion test and the wet band.
    pub(super) fn water_ring_plans(
        &self,
        plan: &B0FramePlanV1,
        frame_slot_index: usize,
    ) -> Result<Vec<water::WaterRingPlanV1>, B0GpuContentError> {
        let mut rings = Vec::new();
        for draw in &plan.draws {
            let Some(ring) = self.dynamic_surfaces.get(&draw.mesh_revision) else {
                continue;
            };
            if ring.profile.shading != DynamicSurfaceShadingV1::WaterSurface
                || self
                    .dynamic_draw_binding(draw.mesh_revision, frame_slot_index)
                    .is_none()
            {
                continue;
            }
            let translation = micrometres_to_metres_f32(draw.transform.translation_micrometres)?;
            let (minimum, maximum) = ring.plan_bounds_micrometres;
            rings.push(water::WaterRingPlanV1 {
                minimum_metres: [
                    minimum[0] as f32 / 1_000_000.0 + translation[0],
                    minimum[1] as f32 / 1_000_000.0 + translation[2],
                ],
                maximum_metres: [
                    maximum[0] as f32 / 1_000_000.0 + translation[0],
                    maximum[1] as f32 / 1_000_000.0 + translation[2],
                ],
                level_metres: translation[1],
            });
        }
        Ok(rings)
    }

    /// Plan 33: the fullscreen water-between pass, inside the water
    /// rendering instance before the rings.
    pub(super) fn record_water_under(
        &self,
        command_buffer: vk::CommandBuffer,
        frame_slot_index: usize,
        water: &water::WaterPassState,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
    ) -> Result<(), B0GpuContentError> {
        self.record_water_fullscreen(
            command_buffer,
            frame_slot_index,
            water,
            water.under_pipeline(),
            viewport,
            scissor,
        )
    }

    /// Plan 35: the fullscreen wet band pass, inside the water rendering
    /// instance before the rings.
    pub(super) fn record_water_wet(
        &self,
        command_buffer: vk::CommandBuffer,
        frame_slot_index: usize,
        water: &water::WaterPassState,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
    ) -> Result<(), B0GpuContentError> {
        self.record_water_fullscreen(
            command_buffer,
            frame_slot_index,
            water,
            water.wet_pipeline(),
            viewport,
            scissor,
        )
    }

    /// One fullscreen triangle with the water pass layout (sets 0 and 3).
    fn record_water_fullscreen(
        &self,
        command_buffer: vk::CommandBuffer,
        frame_slot_index: usize,
        water: &water::WaterPassState,
        pipeline: vk::Pipeline,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
    ) -> Result<(), B0GpuContentError> {
        let frame_set = *self.descriptors.frame_sets.get(frame_slot_index).ok_or(
            B0GpuContentError::InvalidFramePlan("frame slot index is outside the descriptor ring"),
        )?;
        let water_set = water.set(frame_slot_index)?;
        let viewports = [viewport];
        let scissors = [scissor];
        // SAFETY: the pipeline shares the water pass layout; sets 0 and 3
        // are the only sets the suite reads.
        unsafe {
            self.geometry.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline,
            );
            self.geometry
                .device
                .cmd_set_viewport(command_buffer, 0, &viewports);
            self.geometry
                .device
                .cmd_set_scissor(command_buffer, 0, &scissors);
            self.geometry.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                water.layout(),
                0,
                &[frame_set],
                &[],
            );
            self.geometry.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                water.layout(),
                3,
                &[water_set],
                &[],
            );
            self.geometry.device.cmd_draw(command_buffer, 3, 1, 0, 0);
        }
        Ok(())
    }

    /// Plan 15: records the plan's draws with the mirrored camera into the
    /// reflection pass; a reflection rendering instance must be active.
    #[allow(
        clippy::too_many_arguments,
        reason = "the pass inputs of the private adapter boundary stay explicit"
    )]
    pub(super) fn record_reflection(
        &mut self,
        command_buffer: vk::CommandBuffer,
        plan: &B0FramePlanV1,
        extent: vk::Extent2D,
        frame_slot_index: usize,
        water: &mut water::WaterPassState,
        plane_height_metres: f32,
        jitter: Option<ProjectionJitterV1>,
    ) -> Result<(), B0GpuContentError> {
        let Some(camera) = plan.camera.as_ref() else {
            return Ok(());
        };
        let raster_state =
            mirrored_frame_raster_state(camera, extent, plane_height_metres, jitter)?;
        let frame_set = water.prepare_reflection(frame_slot_index, &raster_state)?;
        let shadow_set = self
            .descriptors
            .shadow_set
            .ok_or(B0GpuContentError::ResourceMissing(
                "shadow descriptor set for the reflection pass",
            ))?;
        let viewports = [raster_state.viewport];
        let scissors = [raster_state.scissor];
        let vertex_buffers = [self.geometry.buffer];
        let vertex_offsets = [0];
        let layout = water.reflection_layout();
        // SAFETY: the reflection pipeline shares the B0 layouts; the sets
        // and buffers are live on this device inside the reflection
        // rendering instance.
        unsafe {
            self.geometry.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                water.reflection_pipeline(),
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
                layout,
                0,
                &[frame_set],
                &[],
            );
            self.geometry.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                layout,
                2,
                &[shadow_set],
                &[],
            );
        }
        self.record_plan_draws(
            command_buffer,
            plan,
            frame_slot_index,
            layout,
            WaterRingModeV1::Skip,
            None,
        )?;
        Ok(())
    }

    pub(super) fn record_sky(
        &self,
        command_buffer: vk::CommandBuffer,
        extent: vk::Extent2D,
        frame_slot_index: usize,
    ) -> Result<(), B0GpuContentError> {
        if extent.width == 0 || extent.height == 0 {
            return Err(B0GpuContentError::InvalidFramePlan(
                "sky extent must be non-zero",
            ));
        }
        let frame_set = *self.descriptors.frame_sets.get(frame_slot_index).ok_or(
            B0GpuContentError::InvalidFramePlan("frame slot index is outside the descriptor ring"),
        )?;
        let frame_sets = [frame_set];
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
            self.geometry.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.sky_pipeline.layout,
                0,
                &frame_sets,
                &[],
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
                if let Some(staging) = slot.staging.as_ref() {
                    bytes = bytes
                        .checked_add(staging.vertices.allocation_size())
                        .and_then(|value| value.checked_add(staging.indices.allocation_size()))
                        .ok_or(B0GpuContentError::CountOverflow)?;
                    dynamic_surface_allocations = dynamic_surface_allocations
                        .checked_add(2)
                        .ok_or(B0GpuContentError::CountOverflow)?;
                }
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

/// Scene look L5: one uploaded mip level.
struct PreparedMip {
    extent: vk::Extent3D,
    staging_offset: vk::DeviceSize,
}

struct PreparedTexture {
    revision: AssetRevisionRefV1,
    format: vk::Format,
    /// Scene look L6a: the array layers (one for a plain texture).
    layers: u32,
    /// Level 0 first.
    mips: Vec<PreparedMip>,
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
    /// Scene look L1: metallic, roughness, emissive intensity per material.
    materials: BTreeMap<AssetRevisionRefV1, [f32; 4]>,
    /// Scene look L5: the maps each material binds.
    material_maps: BTreeMap<AssetRevisionRefV1, MaterialMapsV1>,
}

impl PreparedContent {
    fn from_catalog(catalog: &RenderContentCatalogV1) -> Result<Self, B0GpuContentError> {
        let mut vertex_bytes = Vec::new();
        let mut index_bytes = Vec::new();
        let mut indirect_bytes = Vec::new();
        let mut draw_offsets = BTreeMap::new();
        let mut draw_commands = BTreeMap::new();
        let mut vertex_templates = BTreeMap::new();
        let mut materials = BTreeMap::new();
        let mut material_maps = BTreeMap::new();
        for material in catalog.materials() {
            let maps = material_maps_of(material.texture_bindings())?;
            materials.insert(
                material.asset_revision()?,
                [
                    f32::from(material.metallic_unorm16()) / f32::from(u16::MAX),
                    f32::from(material.roughness_unorm16()) / f32::from(u16::MAX),
                    material.emissive_intensity_q16_16() as f32 / 65_536.0,
                    // Scene look L6a: a negative scale marks a splat material.
                    if maps.splat_control.is_some() {
                        -maps.uv_scale
                    } else {
                        maps.uv_scale
                    },
                ],
            );
            material_maps.insert(material.asset_revision()?, maps);
        }

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
        // Scene look L5: 8-bit 2D textures in their own format with every
        // mip level.
        for texture in catalog.textures() {
            let format = texture_format(texture.texel_encoding(), texture.color_space()).ok_or(
                B0GpuContentError::InvalidCatalog("texture is outside the B0 profile"),
            )?;
            if texture.dimension() != NeutralTextureDimensionV1::D2
                || texture.array_layers() > next_contracts::render_content::B0_MAX_SPLAT_LAYERS
                || texture.mip_levels().is_empty()
                || texture.mip_levels()[0].extent() != texture.extent()
                || texture.extent()[2] != 1
            {
                return Err(B0GpuContentError::InvalidCatalog(
                    "texture mip extent does not match the 2D image",
                ));
            }
            let mut mips = Vec::with_capacity(texture.mip_levels().len());
            for mip in texture.mip_levels() {
                let extent = mip.extent();
                let staging_offset = append_aligned(&mut staging_bytes, mip.texels(), 4)?;
                mips.push(PreparedMip {
                    extent: vk::Extent3D {
                        width: extent[0],
                        height: extent[1],
                        depth: 1,
                    },
                    staging_offset,
                });
            }
            textures.push(PreparedTexture {
                revision: texture.asset_revision()?,
                format,
                layers: texture.array_layers(),
                mips,
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
            materials,
            material_maps,
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

/// Packs a dynamic surface payload into the locked 28-byte B0 vertex layout:
/// metre position, planar metre UV and snorm16x4 normal. The planar UV keeps
/// the closed B0 shader interface unchanged for a surface whose topology may
/// differ from the catalog placeholder every frame. Runs on the producer
/// thread inside `DynamicSurfaceUpdateV1::new`, never on the render thread.
pub(crate) fn pack_b0_vertices(
    positions_micrometres: &[[i64; 3]],
    normals_snorm16: &[[i16; 3]],
) -> Option<Vec<u8>> {
    let bytes = positions_micrometres
        .len()
        .checked_mul(VERTEX_STRIDE as usize)?;
    let mut packed = Vec::new();
    packed.try_reserve_exact(bytes).ok()?;
    for (position, normal) in positions_micrometres.iter().zip(normals_snorm16) {
        let metres = position.map(|component| component as f32 / 1_000_000.0);
        for component in metres {
            push_f32(&mut packed, component);
        }
        push_f32(&mut packed, metres[0]);
        push_f32(&mut packed, metres[2]);
        push_normal_snorm16(&mut packed, *normal);
    }
    debug_assert_eq!(packed.len(), bytes);
    Some(packed)
}

pub(crate) fn pack_b0_indices(indices: &[u32]) -> Option<Vec<u8>> {
    let bytes = indices.len().checked_mul(size_of::<u32>())?;
    let mut packed = Vec::new();
    packed.try_reserve_exact(bytes).ok()?;
    for index in indices {
        packed.extend_from_slice(&index.to_le_bytes());
    }
    Some(packed)
}

fn push_normal_snorm16(bytes: &mut Vec<u8>, normal: [i16; 3]) {
    for component in normal {
        bytes.extend_from_slice(&component.to_le_bytes());
    }
    bytes.extend_from_slice(&0_i16.to_le_bytes());
}

/// Scene look L5: the Vulkan format of a B0 texture from its encoding and
/// colour space; `None` outside the profile.
fn texture_format(
    encoding: NeutralTexelEncodingV1,
    color_space: NeutralTextureColorSpaceV1,
) -> Option<vk::Format> {
    match (encoding, color_space) {
        (NeutralTexelEncodingV1::Rgba8Unorm, NeutralTextureColorSpaceV1::Srgb) => {
            Some(vk::Format::R8G8B8A8_SRGB)
        }
        (NeutralTexelEncodingV1::Rgba8Unorm, NeutralTextureColorSpaceV1::Linear) => {
            Some(vk::Format::R8G8B8A8_UNORM)
        }
        (NeutralTexelEncodingV1::Rg8Unorm, NeutralTextureColorSpaceV1::Linear) => {
            Some(vk::Format::R8G8_UNORM)
        }
        (NeutralTexelEncodingV1::R8Unorm, NeutralTextureColorSpaceV1::Linear) => {
            Some(vk::Format::R8_UNORM)
        }
        _ => None,
    }
}

/// Scene look L5: the maps of a material from its bindings (the profile
/// guarantees the base colour first and one uniform UV scale).
fn material_maps_of(
    bindings: &[next_contracts::render_content::NeutralMaterialTextureBindingV1],
) -> Result<MaterialMapsV1, B0GpuContentError> {
    use next_contracts::render_content::MaterialTextureSlotV1;
    let base = bindings
        .first()
        .filter(|binding| binding.slot() == MaterialTextureSlotV1::BaseColor)
        .ok_or(B0GpuContentError::InvalidCatalog(
            "material binds no base colour texture",
        ))?;
    let uv_scale = base
        .uv_transform()
        .uniform_scale_q16_16()
        .map_or(1.0, |scale| scale as f32 / 65_536.0);
    let mut maps = MaterialMapsV1 {
        base_color: base.texture(),
        metallic_roughness: None,
        normal: None,
        splat_control: None,
        uv_scale,
    };
    for binding in &bindings[1..] {
        match binding.slot() {
            MaterialTextureSlotV1::MetallicRoughness => {
                maps.metallic_roughness = Some(binding.texture());
            }
            MaterialTextureSlotV1::Normal => maps.normal = Some(binding.texture()),
            MaterialTextureSlotV1::SplatControl => maps.splat_control = Some(binding.texture()),
            _ => {}
        }
    }
    Ok(maps)
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
        let scratch = update.packed_b0_vertices();
        assert_eq!(scratch.len(), 3 * VERTEX_STRIDE as usize);
        assert_eq!(&scratch[0..4], &1.0_f32.to_le_bytes());
        assert_eq!(&scratch[4..8], &0.25_f32.to_le_bytes());
        assert_eq!(&scratch[8..12], &(-0.5_f32).to_le_bytes());
        assert_eq!(&scratch[12..16], &1.0_f32.to_le_bytes());
        assert_eq!(&scratch[16..20], &(-0.5_f32).to_le_bytes());
        assert_eq!(&scratch[22..24], &i16::MAX.to_le_bytes());
        assert_eq!(&scratch[26..28], &0_i16.to_le_bytes());
        assert_eq!(
            update.packed_b0_indices(),
            [0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0]
        );
    }
}
