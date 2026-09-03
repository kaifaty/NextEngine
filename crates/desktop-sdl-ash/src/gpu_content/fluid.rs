//! ADR-102 presentation-only particle surface pass (screen-space fluid).
//!
//! Renderer-private resources for one declared particle set: a per-frame-slot
//! instance buffer and uniform, three screen-sized single-channel targets
//! (nearest depth, thickness, a smoothing ping target), a copy of the opaque
//! scene colour, and four pipelines (sphere splat with two colour outputs,
//! separable narrow-range smoothing, composite). Everything here is a
//! reconstructible cache outside every gameplay, snapshot and frame-plan
//! root; the closed B0 shader interface and its descriptor layouts are
//! untouched.

use std::sync::Arc;

use ash::vk;
use next_contracts::ids::ContentHash;
use next_render::B0CameraFrameV1;

use super::B0GpuContentError;
use super::pipeline::{ProjectionJitterV1, camera_matrices_jittered, camera_raster_region};
use super::resources::{BufferAllocation, ImageAllocation};
use crate::particle_surface::{
    PARTICLE_SURFACE_STRIDE, ParticleSurfaceProfileV1, ParticleSurfaceUpdateV1,
};

/// Fluid frame uniform: view, projection, viewport, params, absorption,
/// focal, sun, spray, spray2, filter (eight 16-byte-aligned rows plus two
/// matrices).
const FLUID_UNIFORM_SIZE: vk::DeviceSize = 64 + 64 + 16 * 8;
/// Depth target clear value; the shaders treat anything at or above
/// `EMPTY_DEPTH_METRES` as "no fluid".
const EMPTY_DEPTH_METRES: f32 = 1.0e30;
const DEPTH_FORMAT: vk::Format = vk::Format::R32_SFLOAT;
const THICKNESS_FORMAT: vk::Format = vk::Format::R16_SFLOAT;
const FILTER_PUSH_SIZE: u32 = 8;

/// Why the pass could not be constructed on this device; the run then keeps
/// the ADR-100 still surface and reports the pass as unavailable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FluidPassUnavailable {
    Depth,
    Thickness,
    SceneCopy,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct FluidUploadStats {
    pub(crate) uploads: u64,
    pub(crate) bytes: u64,
    pub(crate) particle_count: u32,
}

struct Target {
    image: ImageAllocation,
    view: vk::ImageView,
}

impl Target {
    fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        extent: vk::Extent2D,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
    ) -> Result<Self, B0GpuContentError> {
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
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image.image())
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(color_subresource());
        // SAFETY: the image is live and uses this exact colour format.
        let view = unsafe { device.create_image_view(&view_info, None) }?;
        Ok(Self { image, view })
    }
}

struct FluidSlot {
    particles: BufferAllocation,
    uniform: BufferAllocation,
    uploaded_hash: Option<ContentHash>,
    uploaded_count: u32,
}

struct FluidPipeline {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
}

/// Owner of every device object of the pass. Fields drop in dependency
/// order: pipelines, descriptors, samplers, views, images, buffers.
pub(crate) struct FluidPassState {
    device: ash::Device,
    profile: ParticleSurfaceProfileV1,
    extent: vk::Extent2D,
    splat: FluidPipeline,
    filter: FluidPipeline,
    thickness: FluidPipeline,
    composite: FluidPipeline,
    spray: FluidPipeline,
    descriptor_pool: vk::DescriptorPool,
    frame_layout: vk::DescriptorSetLayout,
    image_layout: vk::DescriptorSetLayout,
    frame_sets: Vec<vk::DescriptorSet>,
    filter_a_set: vk::DescriptorSet,
    filter_b_set: vk::DescriptorSet,
    thickness_a_set: vk::DescriptorSet,
    thickness_b_set: vk::DescriptorSet,
    composite_set: vk::DescriptorSet,
    nearest_sampler: vk::Sampler,
    linear_sampler: vk::Sampler,
    depth_target: Target,
    ping_target: Target,
    thickness_target: Target,
    thickness_ping: Target,
    scene_copy: Target,
    slots: Vec<FluidSlot>,
}

impl FluidPassState {
    /// Probes the formats the pass needs; `Ok(Err(reason))` is the declared
    /// fallback, `Err` a real device failure.
    #[allow(
        clippy::too_many_arguments,
        reason = "Vulkan ownership inputs are explicit at the private adapter boundary"
    )]
    pub(crate) fn try_new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        profile: ParticleSurfaceProfileV1,
        color_format: vk::Format,
        depth_format: vk::Format,
        extent: vk::Extent2D,
        frame_slot_count: usize,
    ) -> Result<Result<Self, FluidPassUnavailable>, B0GpuContentError> {
        let attachment_features = vk::FormatFeatureFlags::COLOR_ATTACHMENT
            | vk::FormatFeatureFlags::COLOR_ATTACHMENT_BLEND
            | vk::FormatFeatureFlags::SAMPLED_IMAGE;
        if !format_supports(instance, physical_device, DEPTH_FORMAT, attachment_features) {
            return Ok(Err(FluidPassUnavailable::Depth));
        }
        if !format_supports(
            instance,
            physical_device,
            THICKNESS_FORMAT,
            attachment_features,
        ) {
            return Ok(Err(FluidPassUnavailable::Thickness));
        }
        if !format_supports(
            instance,
            physical_device,
            color_format,
            vk::FormatFeatureFlags::SAMPLED_IMAGE | vk::FormatFeatureFlags::TRANSFER_DST,
        ) {
            return Ok(Err(FluidPassUnavailable::SceneCopy));
        }
        Self::new(
            instance,
            physical_device,
            device,
            profile,
            color_format,
            depth_format,
            extent,
            frame_slot_count,
        )
        .map(Ok)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Vulkan ownership inputs are explicit at the private adapter boundary"
    )]
    fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        profile: ParticleSurfaceProfileV1,
        color_format: vk::Format,
        depth_format: vk::Format,
        extent: vk::Extent2D,
        frame_slot_count: usize,
    ) -> Result<Self, B0GpuContentError> {
        if frame_slot_count == 0 || extent.width == 0 || extent.height == 0 {
            return Err(B0GpuContentError::InvalidCatalog(
                "fluid pass needs a non-zero extent and frame slot count",
            ));
        }
        let target_usage = vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED;
        let depth_target = Target::new(
            instance,
            physical_device,
            device,
            extent,
            DEPTH_FORMAT,
            target_usage,
        )?;
        let ping_target = Target::new(
            instance,
            physical_device,
            device,
            extent,
            DEPTH_FORMAT,
            target_usage,
        )?;
        let thickness_target = Target::new(
            instance,
            physical_device,
            device,
            extent,
            THICKNESS_FORMAT,
            target_usage,
        )?;
        let thickness_ping = Target::new(
            instance,
            physical_device,
            device,
            extent,
            THICKNESS_FORMAT,
            target_usage,
        )?;
        let scene_copy = Target::new(
            instance,
            physical_device,
            device,
            extent,
            color_format,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        )?;
        let host = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let particle_bytes = vk::DeviceSize::from(profile.particle_capacity)
            .checked_mul(vk::DeviceSize::from(PARTICLE_SURFACE_STRIDE))
            .ok_or(B0GpuContentError::CountOverflow)?;
        let mut slots = Vec::with_capacity(frame_slot_count);
        for _ in 0..frame_slot_count {
            slots.push(FluidSlot {
                particles: BufferAllocation::new(
                    instance,
                    physical_device,
                    device,
                    particle_bytes,
                    vk::BufferUsageFlags::VERTEX_BUFFER,
                    host,
                )?,
                uniform: BufferAllocation::new(
                    instance,
                    physical_device,
                    device,
                    FLUID_UNIFORM_SIZE,
                    vk::BufferUsageFlags::UNIFORM_BUFFER,
                    host,
                )?,
                uploaded_hash: None,
                uploaded_count: 0,
            });
        }

        let mut guard = Teardown {
            device: device.clone(),
            layouts: Vec::new(),
            samplers: Vec::new(),
            pool: vk::DescriptorPool::null(),
            pipelines: Vec::new(),
            views: vec![
                depth_target.view,
                ping_target.view,
                thickness_target.view,
                thickness_ping.view,
                scene_copy.view,
            ],
            armed: true,
        };

        let frame_bindings = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)];
        let image_bindings = [0_u32, 1, 2].map(|binding| {
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        });
        let frame_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&frame_bindings);
        let image_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&image_bindings);
        // SAFETY: bindings are fixed pass values and retain no host pointers.
        let frame_layout =
            unsafe { device.create_descriptor_set_layout(&frame_layout_info, None) }?;
        guard.layouts.push(frame_layout);
        // SAFETY: same ownership as the frame layout.
        let image_layout =
            unsafe { device.create_descriptor_set_layout(&image_layout_info, None) }?;
        guard.layouts.push(image_layout);

        let sampler = |filter: vk::Filter| {
            vk::SamplerCreateInfo::default()
                .mag_filter(filter)
                .min_filter(filter)
                .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .min_lod(0.0)
                .max_lod(0.0)
        };
        // SAFETY: core non-anisotropic samplers on the live device.
        let nearest_sampler =
            unsafe { device.create_sampler(&sampler(vk::Filter::NEAREST), None) }?;
        guard.samplers.push(nearest_sampler);
        // SAFETY: same as above.
        let linear_sampler = unsafe { device.create_sampler(&sampler(vk::Filter::LINEAR), None) }?;
        guard.samplers.push(linear_sampler);

        let slot_count =
            u32::try_from(frame_slot_count).map_err(|_| B0GpuContentError::CountOverflow)?;
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: slot_count,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 15,
            },
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(slot_count.saturating_add(5))
            .pool_sizes(&pool_sizes);
        // SAFETY: pool sizes exactly cover the fixed set of the pass.
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None) }?;
        guard.pool = descriptor_pool;
        let mut layouts = vec![frame_layout; frame_slot_count];
        layouts.extend([image_layout; 5]);
        let allocation_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);
        // SAFETY: pool and layouts are live on this device.
        let sets = unsafe { device.allocate_descriptor_sets(&allocation_info) }?;
        let frame_sets = sets[..frame_slot_count].to_vec();
        let filter_a_set = sets[frame_slot_count];
        let filter_b_set = sets[frame_slot_count + 1];
        let composite_set = sets[frame_slot_count + 2];
        let thickness_a_set = sets[frame_slot_count + 3];
        let thickness_b_set = sets[frame_slot_count + 4];
        for (slot, set) in slots.iter().zip(&frame_sets) {
            let buffer_info = [vk::DescriptorBufferInfo::default()
                .buffer(slot.uniform.buffer)
                .offset(0)
                .range(FLUID_UNIFORM_SIZE)];
            let writes = [vk::WriteDescriptorSet::default()
                .dst_set(*set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_info)];
            // SAFETY: set and buffer are live; descriptors are copied now.
            unsafe { device.update_descriptor_sets(&writes, &[]) };
        }
        let image_set = |set: vk::DescriptorSet, views: [(vk::ImageView, vk::Sampler); 3]| {
            let infos = views.map(|(view, sampler)| {
                [vk::DescriptorImageInfo::default()
                    .sampler(sampler)
                    .image_view(view)
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)]
            });
            let writes = [0_usize, 1, 2].map(|binding| {
                vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(binding as u32)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&infos[binding])
            });
            // SAFETY: views, samplers and the set outlive this state.
            unsafe { device.update_descriptor_sets(&writes, &[]) };
        };
        image_set(
            filter_a_set,
            [
                (depth_target.view, nearest_sampler),
                (thickness_target.view, nearest_sampler),
                (scene_copy.view, linear_sampler),
            ],
        );
        image_set(
            filter_b_set,
            [
                (ping_target.view, nearest_sampler),
                (thickness_target.view, nearest_sampler),
                (scene_copy.view, linear_sampler),
            ],
        );
        // NGQ10 revision 4: with the cleanup pass the final smoothed depth
        // lives in the ping target; without it, in the depth target.
        let final_depth_view = if profile.cleanup_radius_pixels != 0 {
            ping_target.view
        } else {
            depth_target.view
        };
        image_set(
            composite_set,
            [
                (final_depth_view, nearest_sampler),
                (thickness_target.view, nearest_sampler),
                (scene_copy.view, linear_sampler),
            ],
        );
        image_set(
            thickness_a_set,
            [
                (final_depth_view, nearest_sampler),
                (thickness_target.view, nearest_sampler),
                (scene_copy.view, linear_sampler),
            ],
        );
        image_set(
            thickness_b_set,
            [
                (final_depth_view, nearest_sampler),
                (thickness_ping.view, nearest_sampler),
                (scene_copy.view, linear_sampler),
            ],
        );

        let modules =
            crate::shader_assets::fluid_shader_modules().map_err(B0GpuContentError::ShaderAsset)?;
        let splat = create_pipeline(
            device,
            &[frame_layout],
            0,
            &modules.splat_vertex,
            &modules.splat_fragment,
            PipelineShape {
                color_formats: &[DEPTH_FORMAT, THICKNESS_FORMAT],
                depth_format: Some(depth_format),
                blends: &[blend_state(vk::BlendOp::MIN), blend_state(vk::BlendOp::ADD)],
                instanced: true,
            },
        )?;
        guard.pipelines.push((splat.pipeline, splat.layout));
        let filter = create_pipeline(
            device,
            &[frame_layout, image_layout],
            FILTER_PUSH_SIZE,
            &modules.screen_vertex,
            &modules.filter_fragment,
            PipelineShape {
                color_formats: &[DEPTH_FORMAT],
                depth_format: None,
                blends: &[opaque_blend_state()],
                instanced: false,
            },
        )?;
        guard.pipelines.push((filter.pipeline, filter.layout));
        let thickness = create_pipeline(
            device,
            &[frame_layout, image_layout],
            FILTER_PUSH_SIZE,
            &modules.screen_vertex,
            &modules.thickness_fragment,
            PipelineShape {
                color_formats: &[THICKNESS_FORMAT],
                depth_format: None,
                blends: &[opaque_blend_state()],
                instanced: false,
            },
        )?;
        guard.pipelines.push((thickness.pipeline, thickness.layout));
        let spray = create_pipeline(
            device,
            &[frame_layout],
            0,
            &modules.spray_vertex,
            &modules.spray_fragment,
            PipelineShape {
                color_formats: &[color_format],
                depth_format: Some(depth_format),
                blends: &[spray_blend_state()],
                instanced: true,
            },
        )?;
        guard.pipelines.push((spray.pipeline, spray.layout));
        let composite = create_pipeline(
            device,
            &[frame_layout, image_layout],
            0,
            &modules.screen_vertex,
            &modules.composite_fragment,
            PipelineShape {
                color_formats: &[color_format],
                depth_format: None,
                blends: &[opaque_blend_state()],
                instanced: false,
            },
        )?;
        guard.armed = false;
        Ok(Self {
            device: device.clone(),
            profile,
            extent,
            splat,
            filter,
            thickness,
            composite,
            spray,
            descriptor_pool,
            frame_layout,
            image_layout,
            frame_sets,
            filter_a_set,
            filter_b_set,
            thickness_a_set,
            thickness_b_set,
            composite_set,
            nearest_sampler,
            linear_sampler,
            depth_target,
            ping_target,
            thickness_target,
            thickness_ping,
            scene_copy,
            slots,
        })
    }

    /// Refreshes this frame slot's particle buffer from the current update.
    /// The caller has waited for the slot fence.
    pub(crate) fn prepare(
        &mut self,
        update: Option<&Arc<ParticleSurfaceUpdateV1>>,
        frame_slot_index: usize,
    ) -> Result<FluidUploadStats, B0GpuContentError> {
        let slot =
            self.slots
                .get_mut(frame_slot_index)
                .ok_or(B0GpuContentError::InvalidFramePlan(
                    "frame slot index is outside the particle surface ring",
                ))?;
        let Some(update) = update else {
            slot.uploaded_hash = None;
            slot.uploaded_count = 0;
            return Ok(FluidUploadStats::default());
        };
        if update.particle_count() > self.profile.particle_capacity {
            return Err(B0GpuContentError::InvalidFramePlan(
                "particle update exceeds the declared capacity",
            ));
        }
        if slot.uploaded_hash == Some(update.canonical_hash()) {
            return Ok(FluidUploadStats {
                uploads: 0,
                bytes: 0,
                particle_count: slot.uploaded_count,
            });
        }
        slot.particles.write(0, update.packed_positions())?;
        slot.uploaded_hash = Some(update.canonical_hash());
        slot.uploaded_count = update.particle_count();
        Ok(FluidUploadStats {
            uploads: 1,
            bytes: u64::try_from(update.packed_positions().len())
                .map_err(|_| B0GpuContentError::CountOverflow)?,
            particle_count: update.particle_count(),
        })
    }

    /// Records the whole pass after the opaque world rendering instance has
    /// ended. Returns `false` when there was nothing to draw.
    #[allow(
        clippy::too_many_arguments,
        reason = "every image the pass touches is named explicitly"
    )]
    pub(crate) fn record(
        &mut self,
        command_buffer: vk::CommandBuffer,
        frame_slot_index: usize,
        camera: Option<&B0CameraFrameV1>,
        swapchain_image: vk::Image,
        swapchain_view: vk::ImageView,
        scene_depth_view: vk::ImageView,
        sun_direction_intensity: [f32; 4],
        jitter: Option<ProjectionJitterV1>,
    ) -> Result<bool, B0GpuContentError> {
        let Some(camera) = camera else {
            return Ok(false);
        };
        let slot = self
            .slots
            .get(frame_slot_index)
            .ok_or(B0GpuContentError::InvalidFramePlan(
                "frame slot index is outside the particle surface ring",
            ))?;
        if slot.uploaded_count == 0 {
            return Ok(false);
        }
        let frame_set =
            *self
                .frame_sets
                .get(frame_slot_index)
                .ok_or(B0GpuContentError::InvalidFramePlan(
                    "frame slot index is outside the fluid sets",
                ))?;
        let (viewport, _) = camera_raster_region(camera.viewport, self.extent)?;
        let matrices = camera_matrices_jittered(camera, viewport, jitter)?;
        let view_sun = transform_direction(
            &matrices.view,
            [
                sun_direction_intensity[0],
                sun_direction_intensity[1],
                sun_direction_intensity[2],
            ],
        );
        let radius = self.profile.radius_micrometres as f32 / 1_000_000.0;
        let focal_px_y = 0.5 * self.extent.height as f32 / matrices.tan_half_y;
        let mut uniform = [0_u8; FLUID_UNIFORM_SIZE as usize];
        write_f32(&mut uniform[..64], &matrices.view);
        write_f32(&mut uniform[64..128], &matrices.projection);
        write_f32(
            &mut uniform[128..144],
            &[
                self.extent.width as f32,
                self.extent.height as f32,
                1.0 / self.extent.width as f32,
                1.0 / self.extent.height as f32,
            ],
        );
        write_f32(
            &mut uniform[144..160],
            &[
                radius,
                matrices.near,
                matrices.far,
                self.profile.thickness_scale,
            ],
        );
        write_f32(
            &mut uniform[160..176],
            &[
                self.profile.absorption_per_metre[0],
                self.profile.absorption_per_metre[1],
                self.profile.absorption_per_metre[2],
                self.profile.refraction_strength,
            ],
        );
        write_f32(
            &mut uniform[176..192],
            &[
                matrices.tan_half_x,
                matrices.tan_half_y,
                focal_px_y,
                EMPTY_DEPTH_METRES,
            ],
        );
        write_f32(
            &mut uniform[192..208],
            &[
                view_sun[0],
                view_sun[1],
                view_sun[2],
                sun_direction_intensity[3],
            ],
        );
        write_f32(
            &mut uniform[208..224],
            &[
                self.profile.spray_neighbour_threshold as f32,
                self.profile.spray_radius_micrometres as f32 / 1_000_000.0,
                self.profile.spray_alpha,
                self.profile.spray_cluster_threshold as f32,
            ],
        );
        write_f32(
            &mut uniform[224..240],
            &[
                self.profile.spray_streak_seconds,
                self.profile.spray_subdroplets as f32,
                self.profile.bulk_neighbour_count as f32,
                self.profile.edge_radius_scale,
            ],
        );
        write_f32(
            &mut uniform[240..256],
            &[self.profile.cleanup_radius_pixels as f32, 0.0, 0.0, 0.0],
        );
        slot.uniform.write(0, &uniform)?;

        let device = &self.device;
        let full = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.extent,
        };
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.extent.width as f32,
            height: self.extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [full];

        // 1. Copy the opaque scene colour for refraction.
        let to_transfer = [
            image_barrier(
                swapchain_image,
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
                self.scene_copy.image.image(),
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                (
                    vk::PipelineStageFlags2::FRAGMENT_SHADER,
                    vk::AccessFlags2::SHADER_SAMPLED_READ,
                ),
                (
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_WRITE,
                ),
            ),
        ];
        let copy = [vk::ImageCopy::default()
            .src_subresource(color_layers())
            .dst_subresource(color_layers())
            .extent(vk::Extent3D {
                width: self.extent.width,
                height: self.extent.height,
                depth: 1,
            })];
        let after_copy = [
            image_barrier(
                self.scene_copy.image.image(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                (
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_WRITE,
                ),
                (
                    vk::PipelineStageFlags2::FRAGMENT_SHADER,
                    vk::AccessFlags2::SHADER_SAMPLED_READ,
                ),
            ),
            image_barrier(
                swapchain_image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                (
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_READ,
                ),
                (
                    vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                    vk::AccessFlags2::COLOR_ATTACHMENT_WRITE
                        | vk::AccessFlags2::COLOR_ATTACHMENT_READ,
                ),
            ),
            // Fluid targets are rewritten from scratch every frame.
            image_barrier(
                self.depth_target.image.image(),
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                (
                    vk::PipelineStageFlags2::FRAGMENT_SHADER,
                    vk::AccessFlags2::SHADER_SAMPLED_READ,
                ),
                (
                    vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                    vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                ),
            ),
            image_barrier(
                self.thickness_target.image.image(),
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                (
                    vk::PipelineStageFlags2::FRAGMENT_SHADER,
                    vk::AccessFlags2::SHADER_SAMPLED_READ,
                ),
                (
                    vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                    vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                ),
            ),
        ];
        // The world pass wrote the scene depth; the splat pass tests against
        // it without writing.
        let depth_memory = [vk::MemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS)
            .src_access_mask(vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE)
            .dst_stage_mask(
                vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
            )
            .dst_access_mask(vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ)];
        // SAFETY: every image belongs to this device, the world rendering
        // instance has ended, and the swapchain was created with
        // transfer-source usage for this run.
        unsafe {
            device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&to_transfer),
            );
            device.cmd_copy_image(
                command_buffer,
                swapchain_image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                self.scene_copy.image.image(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &copy,
            );
            device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default()
                    .image_memory_barriers(&after_copy)
                    .memory_barriers(&depth_memory),
            );
        }

        // 2. Sphere splat: nearest depth (MIN) and thickness (ADD), occluded
        // by the scene depth.
        let depth_clear = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [EMPTY_DEPTH_METRES, 0.0, 0.0, 0.0],
            },
        };
        let zero_clear = vk::ClearValue {
            color: vk::ClearColorValue { float32: [0.0; 4] },
        };
        let splat_colors = [
            vk::RenderingAttachmentInfo::default()
                .image_view(self.depth_target.view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(depth_clear),
            vk::RenderingAttachmentInfo::default()
                .image_view(self.thickness_target.view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(zero_clear),
        ];
        let splat_depth = vk::RenderingAttachmentInfo::default()
            .image_view(scene_depth_view)
            .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::LOAD)
            .store_op(vk::AttachmentStoreOp::STORE);
        let splat_info = vk::RenderingInfo::default()
            .render_area(full)
            .layer_count(1)
            .color_attachments(&splat_colors)
            .depth_attachment(&splat_depth);
        let frame_sets = [frame_set];
        let particle_buffers = [slot.particles.buffer];
        let particle_offsets = [0];
        // SAFETY: the attachments are in the layouts declared above, the
        // instance buffer holds `uploaded_count` packed particles written
        // after this slot's fence completed, and the pipeline formats match.
        unsafe {
            device.cmd_begin_rendering(command_buffer, &splat_info);
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.splat.pipeline,
            );
            device.cmd_set_viewport(command_buffer, 0, &viewports);
            device.cmd_set_scissor(command_buffer, 0, &scissors);
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.splat.layout,
                0,
                &frame_sets,
                &[],
            );
            device.cmd_bind_vertex_buffers(command_buffer, 0, &particle_buffers, &particle_offsets);
            device.cmd_draw(command_buffer, 6, slot.uploaded_count, 0, 0);
            device.cmd_end_rendering(command_buffer);
        }

        // 3. Two separable smoothing passes: depth -> ping -> depth.
        let attachment_to_sampled = |image: vk::Image| {
            image_barrier(
                image,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                (
                    vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                    vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                ),
                (
                    vk::PipelineStageFlags2::FRAGMENT_SHADER,
                    vk::AccessFlags2::SHADER_SAMPLED_READ,
                ),
            )
        };
        let sampled_to_attachment = |image: vk::Image, old: vk::ImageLayout| {
            image_barrier(
                image,
                old,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                (
                    vk::PipelineStageFlags2::FRAGMENT_SHADER,
                    vk::AccessFlags2::SHADER_SAMPLED_READ,
                ),
                (
                    vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                    vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                ),
            )
        };
        let splat_done = [
            attachment_to_sampled(self.depth_target.image.image()),
            attachment_to_sampled(self.thickness_target.image.image()),
            sampled_to_attachment(self.ping_target.image.image(), vk::ImageLayout::UNDEFINED),
        ];
        let filter_a_done = [
            attachment_to_sampled(self.ping_target.image.image()),
            sampled_to_attachment(
                self.depth_target.image.image(),
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ),
        ];
        let filter_b_done = [attachment_to_sampled(self.depth_target.image.image())];
        // Cleanup (direction 0,0): depth -> ping again, so the final depth
        // is the ping target and the sets above bind it.
        let cleanup_barriers = [
            filter_b_done[0],
            sampled_to_attachment(
                self.ping_target.image.image(),
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ),
        ];
        let cleanup_done = [attachment_to_sampled(self.ping_target.image.image())];
        let cleanup = self.profile.cleanup_radius_pixels != 0;
        let mut filter_passes = vec![
            (
                &splat_done[..],
                self.ping_target.view,
                self.filter_a_set,
                [1.0_f32, 0.0],
            ),
            (
                &filter_a_done[..],
                self.depth_target.view,
                self.filter_b_set,
                [0.0_f32, 1.0],
            ),
        ];
        if cleanup {
            filter_passes.push((
                &cleanup_barriers[..],
                self.ping_target.view,
                self.filter_a_set,
                [0.0_f32, 0.0],
            ));
        }
        for (barriers, target_view, image_set, direction) in filter_passes {
            let colors = [vk::RenderingAttachmentInfo::default()
                .image_view(target_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::DONT_CARE)
                .store_op(vk::AttachmentStoreOp::STORE)];
            let info = vk::RenderingInfo::default()
                .render_area(full)
                .layer_count(1)
                .color_attachments(&colors);
            let sets = [frame_set, image_set];
            let mut push = [0_u8; FILTER_PUSH_SIZE as usize];
            write_f32(&mut push, &direction);
            // SAFETY: barriers order the previous writes before these reads,
            // the target is in attachment layout, and the fullscreen vertex
            // shader synthesizes three vertices without buffer access.
            unsafe {
                device.cmd_pipeline_barrier2(
                    command_buffer,
                    &vk::DependencyInfo::default().image_memory_barriers(barriers),
                );
                device.cmd_begin_rendering(command_buffer, &info);
                device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.filter.pipeline,
                );
                device.cmd_set_viewport(command_buffer, 0, &viewports);
                device.cmd_set_scissor(command_buffer, 0, &scissors);
                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.filter.layout,
                    0,
                    &sets,
                    &[],
                );
                device.cmd_push_constants(
                    command_buffer,
                    self.filter.layout,
                    vk::ShaderStageFlags::FRAGMENT,
                    0,
                    &push,
                );
                device.cmd_draw(command_buffer, 3, 1, 0, 0);
                device.cmd_end_rendering(command_buffer);
            }
        }

        // 3b. Two separable Gaussian passes over the thickness:
        // thickness -> thickness_ping -> thickness, sampling the smoothed depth.
        let thickness_a_barriers = [
            if cleanup {
                cleanup_done[0]
            } else {
                filter_b_done[0]
            },
            sampled_to_attachment(
                self.thickness_ping.image.image(),
                vk::ImageLayout::UNDEFINED,
            ),
        ];
        let thickness_b_barriers = [
            attachment_to_sampled(self.thickness_ping.image.image()),
            sampled_to_attachment(
                self.thickness_target.image.image(),
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ),
        ];
        let thickness_done = [attachment_to_sampled(self.thickness_target.image.image())];
        let thickness_passes = [
            (
                &thickness_a_barriers[..],
                self.thickness_ping.view,
                self.thickness_a_set,
                [1.0_f32, 0.0],
            ),
            (
                &thickness_b_barriers[..],
                self.thickness_target.view,
                self.thickness_b_set,
                [0.0_f32, 1.0],
            ),
        ];
        for (barriers, target_view, image_set, direction) in thickness_passes {
            let colors = [vk::RenderingAttachmentInfo::default()
                .image_view(target_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::DONT_CARE)
                .store_op(vk::AttachmentStoreOp::STORE)];
            let info = vk::RenderingInfo::default()
                .render_area(full)
                .layer_count(1)
                .color_attachments(&colors);
            let sets = [frame_set, image_set];
            let mut push = [0_u8; FILTER_PUSH_SIZE as usize];
            write_f32(&mut push, &direction);
            // SAFETY: same ordering argument as the depth filter passes.
            unsafe {
                device.cmd_pipeline_barrier2(
                    command_buffer,
                    &vk::DependencyInfo::default().image_memory_barriers(barriers),
                );
                device.cmd_begin_rendering(command_buffer, &info);
                device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.thickness.pipeline,
                );
                device.cmd_set_viewport(command_buffer, 0, &viewports);
                device.cmd_set_scissor(command_buffer, 0, &scissors);
                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.thickness.layout,
                    0,
                    &sets,
                    &[],
                );
                device.cmd_push_constants(
                    command_buffer,
                    self.thickness.layout,
                    vk::ShaderStageFlags::FRAGMENT,
                    0,
                    &push,
                );
                device.cmd_draw(command_buffer, 3, 1, 0, 0);
                device.cmd_end_rendering(command_buffer);
            }
        }

        // 4. Composite over the opaque swapchain image.
        let composite_colors = [vk::RenderingAttachmentInfo::default()
            .image_view(swapchain_view)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::LOAD)
            .store_op(vk::AttachmentStoreOp::STORE)];
        let composite_info = vk::RenderingInfo::default()
            .render_area(full)
            .layer_count(1)
            .color_attachments(&composite_colors);
        let composite_sets = [frame_set, self.composite_set];
        // SAFETY: the smoothed depth is sampled after its last write, the
        // swapchain image is back in attachment layout, and the composite
        // pipeline targets the swapchain format.
        unsafe {
            device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&thickness_done),
            );
            device.cmd_begin_rendering(command_buffer, &composite_info);
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.composite.pipeline,
            );
            device.cmd_set_viewport(command_buffer, 0, &viewports);
            device.cmd_set_scissor(command_buffer, 0, &scissors);
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.composite.layout,
                0,
                &composite_sets,
                &[],
            );
            device.cmd_draw(command_buffer, 3, 1, 0, 0);
            device.cmd_end_rendering(command_buffer);
        }

        // 5. Spray: particles below the neighbour threshold as soft discs,
        // alpha-blended over the composite and depth-tested against the
        // opaque scene (RGB only, the alpha coverage channel is untouched).
        if self.profile.spray_neighbour_threshold != 0 || self.profile.spray_cluster_threshold != 0
        {
            let spray_colors = [vk::RenderingAttachmentInfo::default()
                .image_view(swapchain_view)
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE)];
            let spray_depth = vk::RenderingAttachmentInfo::default()
                .image_view(scene_depth_view)
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE);
            let spray_info = vk::RenderingInfo::default()
                .render_area(full)
                .layer_count(1)
                .color_attachments(&spray_colors)
                .depth_attachment(&spray_depth);
            let color_written = [vk::MemoryBarrier2::default()
                .src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
                .dst_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(
                    vk::AccessFlags2::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                )];
            // SAFETY: the composite write is ordered before the blended
            // read/write, the depth attachment is only tested, and the
            // instance buffer is the one bound for the splat.
            unsafe {
                device.cmd_pipeline_barrier2(
                    command_buffer,
                    &vk::DependencyInfo::default().memory_barriers(&color_written),
                );
                device.cmd_begin_rendering(command_buffer, &spray_info);
                device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.spray.pipeline,
                );
                device.cmd_set_viewport(command_buffer, 0, &viewports);
                device.cmd_set_scissor(command_buffer, 0, &scissors);
                device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.spray.layout,
                    0,
                    &frame_sets,
                    &[],
                );
                device.cmd_bind_vertex_buffers(
                    command_buffer,
                    0,
                    &particle_buffers,
                    &particle_offsets,
                );
                device.cmd_draw(
                    command_buffer,
                    6 * self.profile.spray_subdroplets.clamp(1, 32),
                    slot.uploaded_count,
                    0,
                    0,
                );
                device.cmd_end_rendering(command_buffer);
            }
        }
        Ok(true)
    }

    pub(crate) fn allocation_stats(&self) -> Result<(u64, u64), B0GpuContentError> {
        let mut bytes = 0_u64;
        let mut allocations = 0_u64;
        for target in [
            &self.depth_target,
            &self.ping_target,
            &self.thickness_target,
            &self.thickness_ping,
            &self.scene_copy,
        ] {
            bytes = bytes
                .checked_add(target.image.allocation_size())
                .ok_or(B0GpuContentError::CountOverflow)?;
            allocations += 1;
        }
        for slot in &self.slots {
            bytes = bytes
                .checked_add(slot.particles.allocation_size())
                .and_then(|value| value.checked_add(slot.uniform.allocation_size()))
                .ok_or(B0GpuContentError::CountOverflow)?;
            allocations += 2;
        }
        Ok((bytes, allocations))
    }
}

impl Drop for FluidPassState {
    fn drop(&mut self) {
        // SAFETY: the owning context waits for device idle before dropping
        // the pass; pipelines go before layouts, sets before samplers and
        // views, views before their images.
        unsafe {
            for pipeline in [
                &self.splat,
                &self.filter,
                &self.thickness,
                &self.composite,
                &self.spray,
            ] {
                self.device.destroy_pipeline(pipeline.pipeline, None);
                self.device.destroy_pipeline_layout(pipeline.layout, None);
            }
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device
                .destroy_descriptor_set_layout(self.image_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.frame_layout, None);
            self.device.destroy_sampler(self.linear_sampler, None);
            self.device.destroy_sampler(self.nearest_sampler, None);
            for target in [
                &self.depth_target,
                &self.ping_target,
                &self.thickness_target,
                &self.thickness_ping,
                &self.scene_copy,
            ] {
                self.device.destroy_image_view(target.view, None);
            }
        }
    }
}

/// Destroys partially constructed objects when construction fails midway.
struct Teardown {
    device: ash::Device,
    layouts: Vec<vk::DescriptorSetLayout>,
    samplers: Vec<vk::Sampler>,
    pool: vk::DescriptorPool,
    pipelines: Vec<(vk::Pipeline, vk::PipelineLayout)>,
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
            for (pipeline, layout) in self.pipelines.drain(..) {
                self.device.destroy_pipeline(pipeline, None);
                self.device.destroy_pipeline_layout(layout, None);
            }
            if self.pool != vk::DescriptorPool::null() {
                self.device.destroy_descriptor_pool(self.pool, None);
            }
            for layout in self.layouts.drain(..) {
                self.device.destroy_descriptor_set_layout(layout, None);
            }
            for sampler in self.samplers.drain(..) {
                self.device.destroy_sampler(sampler, None);
            }
            for view in self.views.drain(..) {
                self.device.destroy_image_view(view, None);
            }
        }
    }
}

struct PipelineShape<'a> {
    color_formats: &'a [vk::Format],
    depth_format: Option<vk::Format>,
    blends: &'a [vk::PipelineColorBlendAttachmentState],
    instanced: bool,
}

fn create_pipeline(
    device: &ash::Device,
    set_layouts: &[vk::DescriptorSetLayout],
    push_size: u32,
    vertex_words: &[u32],
    fragment_words: &[u32],
    shape: PipelineShape<'_>,
) -> Result<FluidPipeline, B0GpuContentError> {
    let push_ranges = [vk::PushConstantRange {
        stage_flags: vk::ShaderStageFlags::FRAGMENT,
        offset: 0,
        size: push_size,
    }];
    let mut layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(set_layouts);
    if push_size > 0 {
        layout_info = layout_info.push_constant_ranges(&push_ranges);
    }
    // SAFETY: descriptor layouts are live; the push range is within the
    // guaranteed minimum.
    let layout = unsafe { device.create_pipeline_layout(&layout_info, None) }?;
    let vertex_info = vk::ShaderModuleCreateInfo::default().code(vertex_words);
    let fragment_info = vk::ShaderModuleCreateInfo::default().code(fragment_words);
    // SAFETY: checked-in SPIR-V passed structural validation.
    let vertex_module = match unsafe { device.create_shader_module(&vertex_info, None) } {
        Ok(module) => module,
        Err(error) => {
            // SAFETY: no pipeline depends on the layout yet.
            unsafe { device.destroy_pipeline_layout(layout, None) };
            return Err(error.into());
        }
    };
    // SAFETY: same conditions as the vertex module.
    let fragment_module = match unsafe { device.create_shader_module(&fragment_info, None) } {
        Ok(module) => module,
        Err(error) => {
            // SAFETY: neither object has dependants.
            unsafe {
                device.destroy_shader_module(vertex_module, None);
                device.destroy_pipeline_layout(layout, None);
            }
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
        let bindings = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: PARTICLE_SURFACE_STRIDE,
            input_rate: vk::VertexInputRate::INSTANCE,
        }];
        let attributes = [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32_UINT,
                offset: 12,
            },
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 16,
            },
            vk::VertexInputAttributeDescription {
                location: 3,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: 32,
            },
            vk::VertexInputAttributeDescription {
                location: 4,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: 48,
            },
        ];
        let vertex_input = if shape.instanced {
            vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_binding_descriptions(&bindings)
                .vertex_attribute_descriptions(&attributes)
        } else {
            vk::PipelineVertexInputStateCreateInfo::default()
        };
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
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
            .depth_test_enable(shape.depth_format.is_some())
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);
        let color_blend =
            vk::PipelineColorBlendStateCreateInfo::default().attachments(shape.blends);
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        let mut rendering = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(shape.color_formats);
        if let Some(depth_format) = shape.depth_format {
            rendering = rendering.depth_attachment_format(depth_format);
        }
        let create_info = vk::GraphicsPipelineCreateInfo::default()
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
        // SAFETY: every create-info slice and module stays live for the call.
        match unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
        } {
            Ok(pipelines) => Ok(pipelines[0]),
            Err((partial, error)) => {
                // SAFETY: partial pipelines are unowned results of this call.
                unsafe {
                    for pipeline in partial {
                        device.destroy_pipeline(pipeline, None);
                    }
                }
                Err(B0GpuContentError::from(error))
            }
        }
    };
    // SAFETY: pipeline creation copied all module state.
    unsafe {
        device.destroy_shader_module(fragment_module, None);
        device.destroy_shader_module(vertex_module, None);
    }
    match result {
        Ok(pipeline) => Ok(FluidPipeline { pipeline, layout }),
        Err(error) => {
            // SAFETY: no pipeline depends on the layout.
            unsafe { device.destroy_pipeline_layout(layout, None) };
            Err(error)
        }
    }
}

fn blend_state(op: vk::BlendOp) -> vk::PipelineColorBlendAttachmentState {
    vk::PipelineColorBlendAttachmentState::default()
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::ONE)
        .dst_color_blend_factor(vk::BlendFactor::ONE)
        .color_blend_op(op)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE)
        .alpha_blend_op(op)
        .color_write_mask(vk::ColorComponentFlags::R)
}

fn spray_blend_state() -> vk::PipelineColorBlendAttachmentState {
    vk::PipelineColorBlendAttachmentState::default()
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ZERO)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE)
        .alpha_blend_op(vk::BlendOp::ADD)
        .color_write_mask(
            vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B,
        )
}

fn opaque_blend_state() -> vk::PipelineColorBlendAttachmentState {
    vk::PipelineColorBlendAttachmentState::default()
        .blend_enable(false)
        .color_write_mask(vk::ColorComponentFlags::RGBA)
}

fn format_supports(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    format: vk::Format,
    required: vk::FormatFeatureFlags,
) -> bool {
    // SAFETY: read-only capability query on a device of this instance.
    let properties =
        unsafe { instance.get_physical_device_format_properties(physical_device, format) };
    properties.optimal_tiling_features.contains(required)
}

fn color_subresource() -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1)
}

fn color_layers() -> vk::ImageSubresourceLayers {
    vk::ImageSubresourceLayers::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
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
        .subresource_range(color_subresource())
}

fn write_f32(target: &mut [u8], values: &[f32]) {
    for (chunk, value) in target.chunks_exact_mut(4).zip(values) {
        chunk.copy_from_slice(&value.to_le_bytes());
    }
}

/// Applies the rotation part of a column-major view matrix to a direction.
fn transform_direction(view: &[f32; 16], direction: [f32; 3]) -> [f32; 3] {
    [
        view[0] * direction[0] + view[4] * direction[1] + view[8] * direction[2],
        view[1] * direction[0] + view[5] * direction[1] + view[9] * direction[2],
        view[2] * direction[0] + view[6] * direction[1] + view[10] * direction[2],
    ]
}
