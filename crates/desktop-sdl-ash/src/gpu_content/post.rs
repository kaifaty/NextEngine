//! Scene look L7 (plan `look/07`): the post chain between the temporal
//! resolve and the swapchain. A physically based bloom (a half-resolution
//! mip chain, 13-tap downsample with the Karis average on the first level,
//! tent upsample), an exponential height fog marched at half resolution
//! through the cascaded shadow map (revision 3) with the sky's radiance
//! along the ray partitioned into a shadowed sun share (light shafts,
//! revision 2), exposure, the ACES curve and a colour-grading LUT. Replaces
//! the plan 01 resolve when built; that resolve is the fallback.
//! Renderer-local.

use ash::vk;

use super::B0GpuContentError;
use super::gbuffer::create_fullscreen_pipeline;
use super::resources::{BufferAllocation, ImageAllocation};

/// The bloom chain shares the HDR scene format.
pub(crate) const POST_BLOOM_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
/// The bloom chain's greatest depth (half resolution downward).
pub(crate) const POST_BLOOM_MAX_MIPS: u32 = 5;
/// The bloom's share of the composite (`mix(scene, bloom, mix)`).
pub(crate) const POST_BLOOM_MIX: f32 = 0.04;
/// The shadowed march's step count.
pub(crate) const POST_FOG_STEPS: u32 = 12;
/// The height fog's scale: the density falls by `e` per this height.
pub(crate) const POST_FOG_HEIGHT_SCALE_METRES: f32 = 12.0;
/// The shadowed march's length; beyond it the closed form with the
/// ambient term only.
pub(crate) const POST_FOG_MARCH_METRES: f32 = 60.0;
/// The sky's distance for the fog.
pub(crate) const POST_FOG_FAR_METRES: f32 = 200.0;
/// The Henyey-Greenstein anisotropy of the sun's in-scattering.
pub(crate) const POST_FOG_ANISOTROPY: f32 = 0.5;
/// The grading LUT's edge; stored as a `size² x size` strip.
pub(crate) const POST_LUT_SIZE: u32 = 32;
const POST_LUT_FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;
/// `params` (source texel xy, karis flag / target inverse xy).
const BLOOM_PUSH_CONSTANT_SIZE: u32 = 16;
/// `params` (exposure, ground density, bloom mix, lut size), `fog`
/// (height scale, march, far, anisotropy); the fog program reads `params`
/// as (ground density, 1 / width, 1 / height, spare).
const POST_PUSH_CONSTANT_SIZE: u32 = 32;

/// Why the chain cannot be built, or `None` when it can (plan `look/07`
/// G6).
pub(crate) const fn post_chain_fallback_reason(
    enabled: bool,
    gbuffer: bool,
    hdr_target: bool,
) -> Option<&'static str> {
    if !enabled {
        Some("disabled by option")
    } else if !gbuffer {
        Some("no G-buffer prepass")
    } else if !hdr_target {
        Some("no HDR scene target")
    } else {
        None
    }
}

/// The bloom chain's depth at a render extent: half resolution down to
/// where a side would fall under `8` texels, at most `POST_BLOOM_MAX_MIPS`.
pub(crate) fn bloom_mip_count(extent: vk::Extent2D) -> u32 {
    let mut count = 0;
    let (mut width, mut height) = (extent.width / 2, extent.height / 2);
    while count < POST_BLOOM_MAX_MIPS && width >= 8 && height >= 8 {
        count += 1;
        width /= 2;
        height /= 2;
    }
    count
}

fn level_extent(base: vk::Extent2D, level: u32) -> vk::Extent2D {
    vk::Extent2D {
        width: (base.width >> level).max(1),
        height: (base.height >> level).max(1),
    }
}

/// The Henyey-Greenstein phase function (the same as `post.frag`).
#[cfg(test)]
pub(crate) fn henyey_greenstein(cos_theta: f32, g: f32) -> f32 {
    let denominator = (1.0 + g * g - 2.0 * g * cos_theta).max(1e-4);
    (1.0 - g * g) / (4.0 * std::f32::consts::PI * denominator.powf(1.5))
}

/// The optical depth of the exponential height fog along a ray segment
/// `[a, b]` with vertical component `dy` from height `y0` (closed form,
/// the same as `post.frag`).
#[cfg(test)]
pub(crate) fn height_fog_optical_depth(
    sigma_ground: f32,
    height_scale: f32,
    y0: f32,
    dy: f32,
    a: f32,
    b: f32,
) -> f32 {
    let base = sigma_ground * (-y0.max(0.0) / height_scale).exp();
    if dy.abs() < 1e-4 {
        return base * (b - a);
    }
    let k = dy / height_scale;
    base * ((-k * a).exp() - (-k * b).exp()) / k
}

/// The grade of one display-space colour: an S-curve contrast about
/// middle grey, a saturation lift, a cool lift at black and a warm gain at
/// white (plan `look/07`).
pub(crate) fn grade_display(rgb: [f32; 3]) -> [f32; 3] {
    const CONTRAST: f32 = 1.06;
    const SATURATION: f32 = 1.08;
    const PIVOT: f32 = 0.18_f32;
    const LIFT: [f32; 3] = [-0.010, -0.004, 0.012];
    const GAIN: [f32; 3] = [1.0, 0.99, 0.97];
    let pivot = PIVOT.powf(1.0 / 2.2);
    let mut out = [0.0_f32; 3];
    for axis in 0..3 {
        let v = rgb[axis].clamp(0.0, 1.0);
        // Contrast about the pivot in a log-like space.
        let contrasted = pivot * (v / pivot).max(1e-6).powf(CONTRAST);
        out[axis] = contrasted;
    }
    let luma = 0.2126 * out[0] + 0.7152 * out[1] + 0.0722 * out[2];
    for axis in 0..3 {
        let saturated = luma + (out[axis] - luma) * SATURATION;
        let v = saturated.clamp(0.0, 1.0);
        // Lift in the shadows (fading in over the first 5 % so black stays
        // black, plan revision 1), gain at white.
        let lifted = v + LIFT[axis] * (1.0 - v) * (v / 0.05).min(1.0);
        let gain_weight = (1.0 - v) * ((1.0 - v) / 0.05).min(1.0);
        out[axis] = (lifted * (1.0 + (GAIN[axis] - 1.0) * gain_weight)).clamp(0.0, 1.0);
    }
    out
}

/// The grading LUT as `RGBA8` texels of a `size² x size` strip: slice `b`
/// at `u = b * size + r`, `v = g`.
pub(crate) fn grading_lut() -> Vec<u8> {
    let size = POST_LUT_SIZE as usize;
    let mut texels = vec![0_u8; size * size * size * 4];
    for b in 0..size {
        for g in 0..size {
            for r in 0..size {
                let input = [r, g, b].map(|v| v as f32 / (size - 1) as f32);
                let graded = grade_display(input);
                let index = ((g * size * size) + b * size + r) * 4;
                for axis in 0..3 {
                    texels[index + axis] = (graded[axis] * 255.0).round() as u8;
                }
                texels[index + 3] = 255;
            }
        }
    }
    texels
}

struct Chain {
    image: ImageAllocation,
    views: Vec<vk::ImageView>,
}

/// Owner of every device object of the chain. Fields drop in dependency
/// order: pipelines, descriptors, views, images, buffers.
pub(crate) struct PostPassState {
    device: ash::Device,
    extent: vk::Extent2D,
    bloom_extent: vk::Extent2D,
    mip_count: u32,
    sampler: vk::Sampler,
    bloom_set_layout: vk::DescriptorSetLayout,
    fog_set_layout: vk::DescriptorSetLayout,
    post_set_layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    down_sets: Vec<vk::DescriptorSet>,
    up_sets: Vec<vk::DescriptorSet>,
    fog_set: vk::DescriptorSet,
    post_set: vk::DescriptorSet,
    bloom_layout: vk::PipelineLayout,
    down_pipeline: vk::Pipeline,
    up_pipeline: vk::Pipeline,
    fog_layout: vk::PipelineLayout,
    fog_pipeline: vk::Pipeline,
    post_layout: vk::PipelineLayout,
    post_pipeline: vk::Pipeline,
    down: Chain,
    up: Chain,
    /// Revision 3: the volume at half resolution (rgb in-scatter, a
    /// transmittance).
    fog_view: vk::ImageView,
    fog: ImageAllocation,
    lut_view: vk::ImageView,
    lut: ImageAllocation,
    lut_staging: BufferAllocation,
    lut_uploaded: bool,
}

impl PostPassState {
    /// Builds the chain over the scene target and the G-buffer's linear
    /// depth; `Ok(Err(reason))` is the declared fallback.
    #[allow(
        clippy::too_many_arguments,
        reason = "Vulkan ownership inputs are explicit at the private adapter boundary"
    )]
    pub(crate) fn try_new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        extent: vk::Extent2D,
        swapchain_format: vk::Format,
        frame_layout: vk::DescriptorSetLayout,
        shadow_layout: vk::DescriptorSetLayout,
        scene_view: vk::ImageView,
        linear_depth_view: vk::ImageView,
    ) -> Result<Result<Self, &'static str>, B0GpuContentError> {
        // SAFETY: the physical-device handle belongs to this live instance
        // and the query returns format capability data only.
        let properties = unsafe {
            instance.get_physical_device_format_properties(physical_device, POST_BLOOM_FORMAT)
        };
        let required = vk::FormatFeatureFlags::COLOR_ATTACHMENT
            | vk::FormatFeatureFlags::SAMPLED_IMAGE
            | vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR;
        if !properties.optimal_tiling_features.contains(required) {
            return Ok(Err(
                "R16G16B16A16_SFLOAT is not a linearly sampleable colour attachment",
            ));
        }
        let mip_count = bloom_mip_count(extent);
        if mip_count == 0 {
            return Ok(Err("render extent is too small for the bloom chain"));
        }
        let down_modules = super::super::shader_assets::bloom_down_shader_modules()
            .map_err(B0GpuContentError::ShaderAsset)?;
        let up_modules = super::super::shader_assets::bloom_up_shader_modules()
            .map_err(B0GpuContentError::ShaderAsset)?;
        let post_modules = super::super::shader_assets::post_shader_modules()
            .map_err(B0GpuContentError::ShaderAsset)?;
        let fog_modules = super::super::shader_assets::fog_shader_modules()
            .map_err(B0GpuContentError::ShaderAsset)?;
        let mut guard = Teardown {
            device: device.clone(),
            views: Vec::new(),
            samplers: Vec::new(),
            layouts: Vec::new(),
            pools: Vec::new(),
            pipeline_layouts: Vec::new(),
            pipelines: Vec::new(),
            armed: true,
        };
        let bloom_extent = vk::Extent2D {
            width: (extent.width / 2).max(1),
            height: (extent.height / 2).max(1),
        };
        let chain = |guard: &mut Teardown| {
            let image = ImageAllocation::new_mipped(
                instance,
                physical_device,
                device,
                vk::Extent3D {
                    width: bloom_extent.width,
                    height: bloom_extent.height,
                    depth: 1,
                },
                POST_BLOOM_FORMAT,
                vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
                mip_count,
            )?;
            let mut views = Vec::with_capacity(mip_count as usize);
            for level in 0..mip_count {
                let view_info = vk::ImageViewCreateInfo::default()
                    .image(image.image())
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(POST_BLOOM_FORMAT)
                    .subresource_range(level_range(level));
                // SAFETY: the image is live and uses this exact format.
                let view = unsafe { device.create_image_view(&view_info, None) }?;
                guard.views.push(view);
                views.push(view);
            }
            Ok::<Chain, B0GpuContentError>(Chain { image, views })
        };
        let down = chain(&mut guard)?;
        let up = chain(&mut guard)?;
        let fog = ImageAllocation::new(
            instance,
            physical_device,
            device,
            vk::Extent3D {
                width: bloom_extent.width,
                height: bloom_extent.height,
                depth: 1,
            },
            POST_BLOOM_FORMAT,
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
        )?;
        let fog_view_info = vk::ImageViewCreateInfo::default()
            .image(fog.image())
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(POST_BLOOM_FORMAT)
            .subresource_range(level_range(0));
        // SAFETY: the image is live and uses this exact format.
        let fog_view = unsafe { device.create_image_view(&fog_view_info, None) }?;
        guard.views.push(fog_view);

        // The grading LUT: a strip image filled from a host buffer on the
        // first recorded frame.
        let lut = ImageAllocation::new(
            instance,
            physical_device,
            device,
            vk::Extent3D {
                width: POST_LUT_SIZE * POST_LUT_SIZE,
                height: POST_LUT_SIZE,
                depth: 1,
            },
            POST_LUT_FORMAT,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        )?;
        let lut_view_info = vk::ImageViewCreateInfo::default()
            .image(lut.image())
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(POST_LUT_FORMAT)
            .subresource_range(level_range(0));
        // SAFETY: the image is live and uses this exact format.
        let lut_view = unsafe { device.create_image_view(&lut_view_info, None) }?;
        guard.views.push(lut_view);
        let lut_texels = grading_lut();
        let lut_staging = BufferAllocation::new(
            instance,
            physical_device,
            device,
            lut_texels.len() as vk::DeviceSize,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        lut_staging.write(0, &lut_texels)?;

        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .min_lod(0.0)
            .max_lod(0.0);
        // SAFETY: plain sampler creation on the live device.
        let sampler = unsafe { device.create_sampler(&sampler_info, None) }?;
        guard.samplers.push(sampler);

        let binding = |index: u32| {
            vk::DescriptorSetLayoutBinding::default()
                .binding(index)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        };
        let bloom_bindings = [binding(0), binding(1)];
        let bloom_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&bloom_bindings);
        // SAFETY: plain layout creation on the live device.
        let bloom_set_layout =
            unsafe { device.create_descriptor_set_layout(&bloom_layout_info, None) }?;
        guard.layouts.push(bloom_set_layout);
        let post_bindings = [binding(0), binding(1), binding(2), binding(3), binding(4)];
        let post_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&post_bindings);
        // SAFETY: plain layout creation on the live device.
        let post_set_layout =
            unsafe { device.create_descriptor_set_layout(&post_layout_info, None) }?;
        guard.layouts.push(post_set_layout);
        let fog_bindings = [binding(0)];
        let fog_layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&fog_bindings);
        // SAFETY: plain layout creation on the live device.
        let fog_set_layout =
            unsafe { device.create_descriptor_set_layout(&fog_layout_info, None) }?;
        guard.layouts.push(fog_set_layout);

        let bloom_set_count = mip_count * 2;
        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: bloom_set_count * 2 + 6,
        }];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(bloom_set_count + 2)
            .pool_sizes(&pool_sizes);
        // SAFETY: the pool exactly covers the sets below.
        let pool = unsafe { device.create_descriptor_pool(&pool_info, None) }?;
        guard.pools.push(pool);
        let mut layouts = vec![bloom_set_layout; bloom_set_count as usize];
        layouts.push(post_set_layout);
        layouts.push(fog_set_layout);
        let allocation_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool)
            .set_layouts(&layouts);
        // SAFETY: pool and layouts are live on this device.
        let allocated = unsafe { device.allocate_descriptor_sets(&allocation_info) }?;
        let down_sets = allocated[..mip_count as usize].to_vec();
        let up_sets = allocated[mip_count as usize..bloom_set_count as usize].to_vec();
        let post_set = allocated[bloom_set_count as usize];
        let fog_set = allocated[bloom_set_count as usize + 1];

        let info = |view: vk::ImageView| {
            [vk::DescriptorImageInfo::default()
                .sampler(sampler)
                .image_view(view)
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)]
        };
        let mut image_infos = Vec::new();
        let mut targets: Vec<(vk::DescriptorSet, u32, usize)> = Vec::new();
        for level in 0..mip_count as usize {
            // Downsample level `n` reads the scene (n = 0) or level n - 1.
            let source = if level == 0 {
                scene_view
            } else {
                down.views[level - 1]
            };
            image_infos.push(info(source));
            targets.push((down_sets[level], 0, image_infos.len() - 1));
            targets.push((down_sets[level], 1, image_infos.len() - 1));
            // Upsample level `n` reads the coarser result (the deepest
            // downsample or the next upsample) and its own downsample.
            let coarser = if level + 2 >= mip_count as usize {
                down.views[mip_count as usize - 1]
            } else {
                up.views[level + 1]
            };
            image_infos.push(info(coarser));
            targets.push((up_sets[level], 0, image_infos.len() - 1));
            image_infos.push(info(down.views[level]));
            targets.push((up_sets[level], 1, image_infos.len() - 1));
        }
        let bloom_view = if mip_count >= 2 {
            up.views[0]
        } else {
            down.views[0]
        };
        for (binding, view) in [
            (0, scene_view),
            (1, bloom_view),
            (2, linear_depth_view),
            (3, lut_view),
            (4, fog_view),
        ] {
            image_infos.push(info(view));
            targets.push((post_set, binding, image_infos.len() - 1));
        }
        image_infos.push(info(linear_depth_view));
        targets.push((fog_set, 0, image_infos.len() - 1));
        let writes: Vec<vk::WriteDescriptorSet> = targets
            .iter()
            .map(|(set, binding, index)| {
                vk::WriteDescriptorSet::default()
                    .dst_set(*set)
                    .dst_binding(*binding)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&image_infos[*index])
            })
            .collect();
        // SAFETY: sets, views and the sampler are live; descriptors are
        // copied now.
        unsafe { device.update_descriptor_sets(&writes, &[]) };

        let bloom_push = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            offset: 0,
            size: BLOOM_PUSH_CONSTANT_SIZE,
        }];
        let bloom_set_layouts = [bloom_set_layout];
        let bloom_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&bloom_set_layouts)
            .push_constant_ranges(&bloom_push);
        // SAFETY: the set layout is live.
        let bloom_layout = unsafe { device.create_pipeline_layout(&bloom_layout_info, None) }?;
        guard.pipeline_layouts.push(bloom_layout);
        let down_pipeline = create_fullscreen_pipeline(
            device,
            bloom_layout,
            POST_BLOOM_FORMAT,
            &down_modules.vertex,
            &down_modules.fragment,
        )?;
        guard.pipelines.push(down_pipeline);
        let up_pipeline = create_fullscreen_pipeline(
            device,
            bloom_layout,
            POST_BLOOM_FORMAT,
            &up_modules.vertex,
            &up_modules.fragment,
        )?;
        guard.pipelines.push(up_pipeline);
        let post_push = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            offset: 0,
            size: POST_PUSH_CONSTANT_SIZE,
        }];
        let fog_set_layouts = [frame_layout, fog_set_layout, shadow_layout];
        let fog_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&fog_set_layouts)
            .push_constant_ranges(&post_push);
        // SAFETY: every set layout is live.
        let fog_layout = unsafe { device.create_pipeline_layout(&fog_layout_info, None) }?;
        guard.pipeline_layouts.push(fog_layout);
        let fog_pipeline = create_fullscreen_pipeline(
            device,
            fog_layout,
            POST_BLOOM_FORMAT,
            &fog_modules.vertex,
            &fog_modules.fragment,
        )?;
        guard.pipelines.push(fog_pipeline);
        let post_set_layouts = [frame_layout, post_set_layout];
        let post_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&post_set_layouts)
            .push_constant_ranges(&post_push);
        // SAFETY: every set layout is live.
        let post_layout = unsafe { device.create_pipeline_layout(&post_layout_info, None) }?;
        guard.pipeline_layouts.push(post_layout);
        let post_pipeline = create_fullscreen_pipeline(
            device,
            post_layout,
            swapchain_format,
            &post_modules.vertex,
            &post_modules.fragment,
        )?;
        guard.pipelines.push(post_pipeline);
        guard.armed = false;
        Ok(Ok(Self {
            device: device.clone(),
            extent,
            bloom_extent,
            mip_count,
            sampler,
            bloom_set_layout,
            fog_set_layout,
            post_set_layout,
            pool,
            down_sets,
            up_sets,
            fog_set,
            post_set,
            bloom_layout,
            down_pipeline,
            up_pipeline,
            fog_layout,
            fog_pipeline,
            post_layout,
            post_pipeline,
            down,
            up,
            fog_view,
            fog,
            lut_view,
            lut,
            lut_staging,
            lut_uploaded: false,
        }))
    }

    pub(crate) fn mip_count(&self) -> u32 {
        self.mip_count
    }

    pub(crate) fn allocation_bytes(&self) -> vk::DeviceSize {
        self.down.image.allocation_size()
            + self.up.image.allocation_size()
            + self.fog.allocation_size()
            + self.lut.allocation_size()
    }

    /// Records the chain: the bloom over the scene target, then the
    /// composite into the swapchain image. The scene target must be in
    /// attachment layout and ends in transfer-source layout (as the plan
    /// 01 resolve leaves it); the linear-depth image must be in
    /// transfer-source layout (it returns there). No rendering instance
    /// may be active.
    #[allow(
        clippy::too_many_arguments,
        reason = "the frame's images and sets are explicit at the private adapter boundary"
    )]
    pub(crate) fn record(
        &mut self,
        command_buffer: vk::CommandBuffer,
        frame_set: vk::DescriptorSet,
        shadow_set: vk::DescriptorSet,
        scene_image: vk::Image,
        linear_depth_image: vk::Image,
        swapchain_view: vk::ImageView,
        exposure: f32,
        fog_density: f32,
    ) {
        let fragment_read = (
            vk::PipelineStageFlags2::FRAGMENT_SHADER,
            vk::AccessFlags2::SHADER_READ,
        );
        let attachment_write = (
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
        );
        let transfer_read = (
            vk::PipelineStageFlags2::TRANSFER,
            vk::AccessFlags2::TRANSFER_READ,
        );
        let inputs_in = [
            image_barrier(
                scene_image,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                attachment_write,
                fragment_read,
                level_range(0),
            ),
            image_barrier(
                linear_depth_image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                transfer_read,
                fragment_read,
                level_range(0),
            ),
        ];
        // SAFETY: the images are in the declared layouts and no rendering
        // instance is active.
        unsafe {
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&inputs_in),
            );
        }
        if !self.lut_uploaded {
            self.record_lut_upload(command_buffer);
            self.lut_uploaded = true;
        }

        // The bloom chain down, then up.
        for level in 0..self.mip_count {
            let target_extent = level_extent(self.bloom_extent, level);
            let source_extent = if level == 0 {
                self.extent
            } else {
                level_extent(self.bloom_extent, level - 1)
            };
            let mut push = [0_u8; BLOOM_PUSH_CONSTANT_SIZE as usize];
            let values = [
                1.0 / source_extent.width as f32,
                1.0 / source_extent.height as f32,
                if level == 0 { 1.0 } else { 0.0 },
                0.0,
            ];
            for (index, value) in values.iter().enumerate() {
                push[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
            }
            self.render_level(
                command_buffer,
                self.down.image.image(),
                self.down.views[level as usize],
                level,
                target_extent,
                self.down_pipeline,
                self.bloom_layout,
                &[self.down_sets[level as usize]],
                &push,
            );
        }
        for level in (0..self.mip_count.saturating_sub(1)).rev() {
            let target_extent = level_extent(self.bloom_extent, level);
            let coarser_extent = level_extent(self.bloom_extent, level + 1);
            let mut push = [0_u8; BLOOM_PUSH_CONSTANT_SIZE as usize];
            let values = [
                1.0 / coarser_extent.width as f32,
                1.0 / coarser_extent.height as f32,
                1.0 / target_extent.width as f32,
                1.0 / target_extent.height as f32,
            ];
            for (index, value) in values.iter().enumerate() {
                push[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
            }
            self.render_level(
                command_buffer,
                self.up.image.image(),
                self.up.views[level as usize],
                level,
                target_extent,
                self.up_pipeline,
                self.bloom_layout,
                &[self.up_sets[level as usize]],
                &push,
            );
        }

        // Revision 3: the volume at half resolution.
        let mut push = [0_u8; POST_PUSH_CONSTANT_SIZE as usize];
        let values = [
            fog_density,
            1.0 / self.bloom_extent.width as f32,
            1.0 / self.bloom_extent.height as f32,
            0.0,
            POST_FOG_HEIGHT_SCALE_METRES,
            POST_FOG_MARCH_METRES,
            POST_FOG_FAR_METRES,
            POST_FOG_ANISOTROPY,
        ];
        for (index, value) in values.iter().enumerate() {
            push[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        self.render_level(
            command_buffer,
            self.fog.image(),
            self.fog_view,
            0,
            self.bloom_extent,
            self.fog_pipeline,
            self.fog_layout,
            &[frame_set, self.fog_set, shadow_set],
            &push,
        );

        // The composite into the swapchain image.
        let mut push = [0_u8; POST_PUSH_CONSTANT_SIZE as usize];
        let values = [
            exposure,
            fog_density,
            POST_BLOOM_MIX,
            POST_LUT_SIZE as f32,
            POST_FOG_HEIGHT_SCALE_METRES,
            POST_FOG_MARCH_METRES,
            POST_FOG_FAR_METRES,
            POST_FOG_ANISOTROPY,
        ];
        for (index, value) in values.iter().enumerate() {
            push[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
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
        let viewport = [full_viewport(self.extent)];
        let scissor = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.extent,
        }];
        let outputs_out = [
            image_barrier(
                scene_image,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                fragment_read,
                transfer_read,
                level_range(0),
            ),
            image_barrier(
                linear_depth_image,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                fragment_read,
                transfer_read,
                level_range(0),
            ),
        ];
        // SAFETY: every input is in shader-read layout, the swapchain image
        // is in attachment layout, and every handle belongs to this pass,
        // the content or the live swapchain.
        unsafe {
            self.device
                .cmd_begin_rendering(command_buffer, &rendering_info);
            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.post_pipeline,
            );
            self.device.cmd_set_viewport(command_buffer, 0, &viewport);
            self.device.cmd_set_scissor(command_buffer, 0, &scissor);
            self.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.post_layout,
                0,
                &[frame_set, self.post_set],
                &[],
            );
            self.device.cmd_push_constants(
                command_buffer,
                self.post_layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                &push,
            );
            self.device.cmd_draw(command_buffer, 3, 1, 0, 0);
            self.device.cmd_end_rendering(command_buffer);
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&outputs_out),
            );
        }
    }

    fn record_lut_upload(&self, command_buffer: vk::CommandBuffer) {
        let transfer_write = (
            vk::PipelineStageFlags2::TRANSFER,
            vk::AccessFlags2::TRANSFER_WRITE,
        );
        let to_transfer = [image_barrier(
            self.lut.image(),
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            (vk::PipelineStageFlags2::TOP_OF_PIPE, vk::AccessFlags2::NONE),
            transfer_write,
            level_range(0),
        )];
        let to_sampled = [image_barrier(
            self.lut.image(),
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            transfer_write,
            (
                vk::PipelineStageFlags2::FRAGMENT_SHADER,
                vk::AccessFlags2::SHADER_READ,
            ),
            level_range(0),
        )];
        let region = [vk::BufferImageCopy::default()
            .buffer_offset(0)
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(0)
                    .base_array_layer(0)
                    .layer_count(1),
            )
            .image_extent(vk::Extent3D {
                width: POST_LUT_SIZE * POST_LUT_SIZE,
                height: POST_LUT_SIZE,
                depth: 1,
            })];
        // SAFETY: the staging buffer holds the LUT texels, the image is
        // fresh, and no rendering instance is active.
        unsafe {
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&to_transfer),
            );
            self.device.cmd_copy_buffer_to_image(
                command_buffer,
                self.lut_staging.buffer,
                self.lut.image(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &region,
            );
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&to_sampled),
            );
        }
    }

    /// One fullscreen program into one mip level of a chain image: the
    /// level from any layout (its contents are replaced) to attachment,
    /// the draw, then to shader-read for the next step.
    #[allow(
        clippy::too_many_arguments,
        reason = "the level's handles are explicit at the private adapter boundary"
    )]
    fn render_level(
        &self,
        command_buffer: vk::CommandBuffer,
        image: vk::Image,
        view: vk::ImageView,
        level: u32,
        extent: vk::Extent2D,
        pipeline: vk::Pipeline,
        layout: vk::PipelineLayout,
        sets: &[vk::DescriptorSet],
        push: &[u8],
    ) {
        let to_attachment = [image_barrier(
            image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            (
                vk::PipelineStageFlags2::FRAGMENT_SHADER,
                vk::AccessFlags2::SHADER_READ,
            ),
            (
                vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            ),
            level_range(level),
        )];
        let to_sampled = [image_barrier(
            image,
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
            level_range(level),
        )];
        let color_attachments = [vk::RenderingAttachmentInfo::default()
            .image_view(view)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)];
        let rendering_info = vk::RenderingInfo::default()
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            })
            .layer_count(1)
            .color_attachments(&color_attachments);
        let viewport = [full_viewport(extent)];
        let scissor = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        }];
        // SAFETY: the level belongs to this pass, its inputs are in
        // shader-read layout, and no rendering instance is active.
        unsafe {
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&to_attachment),
            );
            self.device
                .cmd_begin_rendering(command_buffer, &rendering_info);
            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline,
            );
            self.device.cmd_set_viewport(command_buffer, 0, &viewport);
            self.device.cmd_set_scissor(command_buffer, 0, &scissor);
            self.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                layout,
                0,
                sets,
                &[],
            );
            self.device.cmd_push_constants(
                command_buffer,
                layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                push,
            );
            self.device.cmd_draw(command_buffer, 3, 1, 0, 0);
            self.device.cmd_end_rendering(command_buffer);
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&to_sampled),
            );
        }
    }
}

impl Drop for PostPassState {
    fn drop(&mut self) {
        // SAFETY: the owner waits for device idle before dropping; children
        // go before parents, the image and buffer allocations drop after.
        unsafe {
            self.device.destroy_pipeline(self.post_pipeline, None);
            self.device.destroy_pipeline(self.fog_pipeline, None);
            self.device.destroy_pipeline(self.up_pipeline, None);
            self.device.destroy_pipeline(self.down_pipeline, None);
            self.device.destroy_pipeline_layout(self.post_layout, None);
            self.device.destroy_pipeline_layout(self.fog_layout, None);
            self.device.destroy_pipeline_layout(self.bloom_layout, None);
            self.device.destroy_descriptor_pool(self.pool, None);
            self.device
                .destroy_descriptor_set_layout(self.post_set_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.fog_set_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.bloom_set_layout, None);
            self.device.destroy_sampler(self.sampler, None);
            self.device.destroy_image_view(self.fog_view, None);
            self.device.destroy_image_view(self.lut_view, None);
            for view in self.up.views.drain(..).chain(self.down.views.drain(..)) {
                self.device.destroy_image_view(view, None);
            }
        }
    }
}

/// Destroys partially constructed objects when construction fails midway.
struct Teardown {
    device: ash::Device,
    views: Vec<vk::ImageView>,
    samplers: Vec<vk::Sampler>,
    layouts: Vec<vk::DescriptorSetLayout>,
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

fn level_range(level: u32) -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(level)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1)
}

fn full_viewport(extent: vk::Extent2D) -> vk::Viewport {
    vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: extent.width as f32,
        height: extent.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    }
}

fn image_barrier(
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    source: (vk::PipelineStageFlags2, vk::AccessFlags2),
    destination: (vk::PipelineStageFlags2, vk::AccessFlags2),
    range: vk::ImageSubresourceRange,
) -> vk::ImageMemoryBarrier2<'static> {
    vk::ImageMemoryBarrier2::default()
        .src_stage_mask(source.0)
        .src_access_mask(source.1)
        .dst_stage_mask(destination.0)
        .dst_access_mask(destination.1)
        .old_layout(old_layout)
        .new_layout(new_layout)
        .image(image)
        .subresource_range(range)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plan look/07 G3: the phase integrates to one over the sphere.
    #[test]
    fn henyey_greenstein_integrates_to_one() {
        let steps = 4000;
        let mut integral = 0.0_f64;
        for i in 0..steps {
            let cos_theta = -1.0 + (i as f64 + 0.5) * 2.0 / steps as f64;
            integral += f64::from(henyey_greenstein(cos_theta as f32, POST_FOG_ANISOTROPY))
                * 2.0
                * std::f64::consts::PI
                * (2.0 / steps as f64);
        }
        assert!((integral - 1.0).abs() < 0.01, "{integral}");
    }

    /// Plan look/07 G3: the closed form matches a numeric integral for a
    /// level ray, and a climbing ray thins by the height scale.
    #[test]
    fn height_fog_depth_matches_the_integral() {
        let sigma = 0.006;
        let closed =
            height_fog_optical_depth(sigma, POST_FOG_HEIGHT_SCALE_METRES, 1.0, 0.0, 0.0, 100.0);
        let mut numeric = 0.0_f32;
        let steps = 10_000;
        for i in 0..steps {
            let t = (i as f32 + 0.5) * 100.0 / steps as f32;
            numeric +=
                sigma * (-(1.0_f32) / POST_FOG_HEIGHT_SCALE_METRES).exp() * (100.0 / steps as f32);
            let _ = t;
        }
        assert!(
            (closed - numeric).abs() / numeric < 0.01,
            "{closed} {numeric}"
        );
        // A ray climbing 24 m over 100 m: the density at its top is under
        // 1 / e² of the ground density.
        let dy = 24.0 / 100.0;
        let climbing =
            height_fog_optical_depth(sigma, POST_FOG_HEIGHT_SCALE_METRES, 0.0, dy, 0.0, 100.0);
        let mut numeric_climb = 0.0_f32;
        for i in 0..steps {
            let t = (i as f32 + 0.5) * 100.0 / steps as f32;
            numeric_climb +=
                sigma * (-(t * dy) / POST_FOG_HEIGHT_SCALE_METRES).exp() * (100.0 / steps as f32);
        }
        assert!(
            (climbing - numeric_climb).abs() / numeric_climb < 0.01,
            "{climbing} {numeric_climb}"
        );
        let top_density = sigma * (-(24.0_f32) / POST_FOG_HEIGHT_SCALE_METRES).exp();
        assert!(top_density < sigma / (std::f32::consts::E * std::f32::consts::E));
    }

    #[test]
    fn bloom_chain_has_five_levels_at_the_reference_extent() {
        assert_eq!(
            bloom_mip_count(vk::Extent2D {
                width: 960,
                height: 540
            }),
            5
        );
        assert_eq!(
            bloom_mip_count(vk::Extent2D {
                width: 30,
                height: 30
            }),
            1
        );
        assert_eq!(
            bloom_mip_count(vk::Extent2D {
                width: 8,
                height: 8
            }),
            0
        );
    }

    /// Plan look/07 G3: the grade is monotonic on the grey axis and keeps
    /// black and white within a step.
    #[test]
    fn grading_lut_is_monotonic_on_grey() {
        let lut = grading_lut();
        let size = POST_LUT_SIZE as usize;
        let mut previous = [0_u8; 3];
        for i in 0..size {
            let index = ((i * size * size) + i * size + i) * 4;
            let texel = [lut[index], lut[index + 1], lut[index + 2]];
            assert!(
                texel
                    .iter()
                    .zip(previous)
                    .all(|(now, before)| *now >= before),
                "{i}: {texel:?} < {previous:?}"
            );
            previous = texel;
        }
        let black = &lut[..3];
        let last = ((size - 1) * size * size + (size - 1) * size + (size - 1)) * 4;
        let white = &lut[last..last + 3];
        assert!(black.iter().all(|value| *value <= 1), "{black:?}");
        assert!(white.iter().all(|value| *value >= 254), "{white:?}");
    }

    #[test]
    fn fallback_reasons_name_each_missing_precondition() {
        assert_eq!(post_chain_fallback_reason(true, true, true), None);
        assert_eq!(
            post_chain_fallback_reason(false, true, true),
            Some("disabled by option")
        );
        assert_eq!(
            post_chain_fallback_reason(true, false, true),
            Some("no G-buffer prepass")
        );
        assert_eq!(
            post_chain_fallback_reason(true, true, false),
            Some("no HDR scene target")
        );
    }
}
