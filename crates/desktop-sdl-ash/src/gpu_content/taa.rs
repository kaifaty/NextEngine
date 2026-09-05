//! Scene look L4 (plan `look/04`): temporal anti-aliasing. After the scene
//! passes a fullscreen program reprojects the previous resolved history by
//! the G-buffer's motion vectors (the sky through the previous
//! view-projection), clips it to the current neighbourhood's variance box
//! and blends it with the jittered current frame; the result is copied back
//! into the scene target for the tone map. Renderer-local.

use ash::vk;

use super::B0GpuContentError;
use super::gbuffer::create_fullscreen_pipeline;
use super::resources::ImageAllocation;

/// The history buffers share the HDR scene format.
pub(crate) const TAA_HISTORY_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
/// The weight of the current frame in the blend.
pub(crate) const TAA_CURRENT_WEIGHT: f32 = 0.1;
/// The variance clip's width in standard deviations.
#[cfg(test)]
pub(crate) const TAA_CLIP_GAMMA: f32 = 1.0;
/// `previous_view_projection` (64), `params` (jitter delta xy, current
/// weight, first-frame flag), `viewport` (width, height, 1/width, 1/height).
const TAA_PUSH_CONSTANT_SIZE: u32 = 96;

struct Target {
    image: ImageAllocation,
    view: vk::ImageView,
}

/// Why the pass cannot be built, or `None` when it can (plan `look/04` G6).
pub(crate) const fn temporal_aa_fallback_reason(
    enabled: bool,
    gbuffer: bool,
    history_format_supported: bool,
) -> Option<&'static str> {
    if !enabled {
        Some("disabled by option")
    } else if !gbuffer {
        Some("no G-buffer prepass")
    } else if !history_format_supported {
        Some("R16G16B16A16_SFLOAT is not a sampleable colour attachment")
    } else {
        None
    }
}

/// Clips a history colour into the neighbourhood's variance box
/// (`mean ± gamma * deviation` per channel); the same rule as `taa.frag`.
#[cfg(test)]
pub(crate) fn clip_history(
    history: [f32; 3],
    mean: [f32; 3],
    deviation: [f32; 3],
    gamma: f32,
) -> [f32; 3] {
    let mut out = history;
    for channel in 0..3 {
        let low = mean[channel] - gamma * deviation[channel];
        let high = mean[channel] + gamma * deviation[channel];
        out[channel] = history[channel].clamp(low, high);
    }
    out
}

/// The blend of the clipped history and the current colour; the current
/// colour alone on the first frame.
#[cfg(test)]
pub(crate) fn blend_history(
    history: [f32; 3],
    current: [f32; 3],
    current_weight: f32,
    first_frame: bool,
) -> [f32; 3] {
    if first_frame {
        return current;
    }
    let mut out = [0.0_f32; 3];
    for channel in 0..3 {
        out[channel] =
            history[channel] * (1.0 - current_weight) + current[channel] * current_weight;
    }
    out
}

/// Owner of every device object of the pass; fields drop in dependency
/// order.
pub(crate) struct TemporalAaPassState {
    device: ash::Device,
    extent: vk::Extent2D,
    sampler: vk::Sampler,
    set_layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    /// One set per history target: reads the other target as the history.
    sets: [vk::DescriptorSet; 2],
    layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    history: [Target; 2],
    /// The target written this frame (the history read next frame).
    current: usize,
    /// No history yet (the first frame after creation).
    first_frame: bool,
    previous_view_projection: [f32; 16],
    previous_jitter_pixels: [f32; 2],
}

impl TemporalAaPassState {
    /// Builds the pass over the scene target and the G-buffer's motion and
    /// linear-depth views; `Ok(Err(reason))` is the declared fallback.
    #[allow(
        clippy::too_many_arguments,
        reason = "Vulkan ownership inputs are explicit at the private adapter boundary"
    )]
    pub(crate) fn try_new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        extent: vk::Extent2D,
        frame_layout: vk::DescriptorSetLayout,
        scene_view: vk::ImageView,
        motion_view: vk::ImageView,
        linear_depth_view: vk::ImageView,
    ) -> Result<Result<Self, &'static str>, B0GpuContentError> {
        // SAFETY: the physical-device handle belongs to this live instance
        // and the query returns format capability data only.
        let properties = unsafe {
            instance.get_physical_device_format_properties(physical_device, TAA_HISTORY_FORMAT)
        };
        let required = vk::FormatFeatureFlags::COLOR_ATTACHMENT
            | vk::FormatFeatureFlags::SAMPLED_IMAGE
            | vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR
            | vk::FormatFeatureFlags::TRANSFER_SRC;
        if let Some(reason) = temporal_aa_fallback_reason(
            true,
            true,
            properties.optimal_tiling_features.contains(required),
        ) {
            return Ok(Err(reason));
        }
        if extent.width == 0 || extent.height == 0 {
            return Ok(Err("render extent is empty"));
        }
        let modules = super::super::shader_assets::temporal_aa_shader_modules()
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
        let target = |guard: &mut Teardown| {
            let image = ImageAllocation::new(
                instance,
                physical_device,
                device,
                vk::Extent3D {
                    width: extent.width,
                    height: extent.height,
                    depth: 1,
                },
                TAA_HISTORY_FORMAT,
                vk::ImageUsageFlags::COLOR_ATTACHMENT
                    | vk::ImageUsageFlags::SAMPLED
                    | vk::ImageUsageFlags::TRANSFER_SRC,
            )?;
            let view_info = vk::ImageViewCreateInfo::default()
                .image(image.image())
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(TAA_HISTORY_FORMAT)
                .subresource_range(color_subresource());
            // SAFETY: the image is live and uses this exact format.
            let view = unsafe { device.create_image_view(&view_info, None) }?;
            guard.views.push(view);
            Ok::<Target, B0GpuContentError>(Target { image, view })
        };
        let history = [target(&mut guard)?, target(&mut guard)?];
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
        let bindings = [0, 1, 2, 3].map(|binding| {
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        });
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        // SAFETY: plain layout creation on the live device.
        let set_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None) }?;
        guard.layouts.push(set_layout);
        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 8,
        }];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(2)
            .pool_sizes(&pool_sizes);
        // SAFETY: the pool exactly covers the two sets.
        let pool = unsafe { device.create_descriptor_pool(&pool_info, None) }?;
        guard.pools.push(pool);
        let layouts = [set_layout; 2];
        let allocation_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool)
            .set_layouts(&layouts);
        // SAFETY: pool and layouts are live on this device.
        let allocated = unsafe { device.allocate_descriptor_sets(&allocation_info) }?;
        let sets = [allocated[0], allocated[1]];
        let info = |view: vk::ImageView| {
            [vk::DescriptorImageInfo::default()
                .sampler(sampler)
                .image_view(view)
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)]
        };
        let scene_info = info(scene_view);
        let motion_info = info(motion_view);
        let depth_info = info(linear_depth_view);
        let history_info = [info(history[0].view), info(history[1].view)];
        let mut writes = Vec::with_capacity(8);
        for (index, set) in sets.iter().enumerate() {
            // The set writing target `index` reads the other target.
            let inputs = [
                (0, &scene_info),
                (1, &history_info[1 - index]),
                (2, &motion_info),
                (3, &depth_info),
            ];
            for (binding, image_info) in inputs {
                writes.push(
                    vk::WriteDescriptorSet::default()
                        .dst_set(*set)
                        .dst_binding(binding)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(image_info),
                );
            }
        }
        // SAFETY: sets, views and the sampler are live; descriptors are
        // copied now.
        unsafe { device.update_descriptor_sets(&writes, &[]) };
        let push_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            offset: 0,
            size: TAA_PUSH_CONSTANT_SIZE,
        }];
        let set_layouts = [frame_layout, set_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&set_layouts)
            .push_constant_ranges(&push_ranges);
        // SAFETY: both set layouts are live.
        let layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }?;
        guard.pipeline_layouts.push(layout);
        let pipeline = create_fullscreen_pipeline(
            device,
            layout,
            TAA_HISTORY_FORMAT,
            &modules.vertex,
            &modules.fragment,
        )?;
        guard.pipelines.push(pipeline);
        guard.armed = false;
        Ok(Ok(Self {
            device: device.clone(),
            extent,
            sampler,
            set_layout,
            pool,
            sets,
            layout,
            pipeline,
            history,
            current: 0,
            first_frame: true,
            previous_view_projection: IDENTITY,
            previous_jitter_pixels: [0.0; 2],
        }))
    }

    pub(crate) fn allocation_bytes(&self) -> vk::DeviceSize {
        self.history[0].image.allocation_size() + self.history[1].image.allocation_size()
    }

    /// Records the resolve into this frame's history target and copies it
    /// back into the scene target. The scene target must be in attachment
    /// layout (it returns there); the motion and linear-depth images must
    /// be in transfer-source layout (they return there). No rendering
    /// instance may be active.
    #[allow(
        clippy::too_many_arguments,
        reason = "the frame's images and matrices are explicit at the private adapter boundary"
    )]
    pub(crate) fn record(
        &mut self,
        command_buffer: vk::CommandBuffer,
        frame_set: vk::DescriptorSet,
        scene_image: vk::Image,
        motion_image: vk::Image,
        linear_depth_image: vk::Image,
        view_projection: [f32; 16],
        jitter_pixels: [f32; 2],
    ) {
        let target = &self.history[self.current];
        let history = &self.history[1 - self.current];
        let first_frame = self.first_frame;
        // Inputs to shader-read layout: the scene target from attachment,
        // the G-buffer images from transfer-source, the history from
        // wherever the previous frame left it (shader-read after its copy).
        let inputs_in = [
            image_barrier(
                scene_image,
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
            ),
            image_barrier(
                motion_image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                (
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_READ,
                ),
                (
                    vk::PipelineStageFlags2::FRAGMENT_SHADER,
                    vk::AccessFlags2::SHADER_READ,
                ),
            ),
            image_barrier(
                linear_depth_image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                (
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_READ,
                ),
                (
                    vk::PipelineStageFlags2::FRAGMENT_SHADER,
                    vk::AccessFlags2::SHADER_READ,
                ),
            ),
            image_barrier(
                history.image.image(),
                if first_frame {
                    vk::ImageLayout::UNDEFINED
                } else {
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL
                },
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                (
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_READ,
                ),
                (
                    vk::PipelineStageFlags2::FRAGMENT_SHADER,
                    vk::AccessFlags2::SHADER_READ,
                ),
            ),
            image_barrier(
                target.image.image(),
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                (
                    vk::PipelineStageFlags2::FRAGMENT_SHADER | vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::SHADER_READ | vk::AccessFlags2::TRANSFER_READ,
                ),
                (
                    vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                    vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                ),
            ),
        ];
        // After the resolve: the target to transfer-source, the scene
        // target to transfer-destination for the copy.
        let to_copy = [
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
            ),
            image_barrier(
                scene_image,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                (
                    vk::PipelineStageFlags2::FRAGMENT_SHADER,
                    vk::AccessFlags2::SHADER_READ,
                ),
                (
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_WRITE,
                ),
            ),
        ];
        // After the copy: the scene target back to attachment layout, the
        // G-buffer images back to transfer-source; the target stays in
        // transfer-source (next frame's history).
        let after_copy = [
            image_barrier(
                scene_image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                (
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_WRITE,
                ),
                (
                    vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                    vk::AccessFlags2::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                ),
            ),
            image_barrier(
                motion_image,
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
            ),
            image_barrier(
                linear_depth_image,
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
            ),
            image_barrier(
                history.image.image(),
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
            ),
        ];
        let color_attachments = [vk::RenderingAttachmentInfo::default()
            .image_view(target.view)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)];
        let render_area = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.extent,
        };
        let rendering_info = vk::RenderingInfo::default()
            .render_area(render_area)
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
        let scissor = [render_area];
        let width = self.extent.width as f32;
        let height = self.extent.height as f32;
        let mut push = [0_u8; TAA_PUSH_CONSTANT_SIZE as usize];
        let lanes: Vec<f32> = self
            .previous_view_projection
            .iter()
            .copied()
            .chain([
                jitter_pixels[0] - self.previous_jitter_pixels[0],
                jitter_pixels[1] - self.previous_jitter_pixels[1],
                TAA_CURRENT_WEIGHT,
                if first_frame { 1.0 } else { 0.0 },
                width,
                height,
                1.0 / width.max(1.0),
                1.0 / height.max(1.0),
            ])
            .collect();
        for (lane, value) in lanes.iter().enumerate() {
            push[lane * 4..lane * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        let sets = [frame_set, self.sets[self.current]];
        let copy = [vk::ImageCopy::default()
            .src_subresource(color_layers())
            .dst_subresource(color_layers())
            .extent(vk::Extent3D {
                width: self.extent.width,
                height: self.extent.height,
                depth: 1,
            })];
        // SAFETY: every handle belongs to this pass, the G-buffer pass or
        // the scene target; the images are in the declared layouts and no
        // rendering instance is active around the resolve instance.
        unsafe {
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&inputs_in),
            );
            self.device
                .cmd_begin_rendering(command_buffer, &rendering_info);
            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
            self.device.cmd_set_viewport(command_buffer, 0, &viewport);
            self.device.cmd_set_scissor(command_buffer, 0, &scissor);
            self.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.layout,
                0,
                &sets,
                &[],
            );
            self.device.cmd_push_constants(
                command_buffer,
                self.layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                &push,
            );
            self.device.cmd_draw(command_buffer, 3, 1, 0, 0);
            self.device.cmd_end_rendering(command_buffer);
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&to_copy),
            );
            self.device.cmd_copy_image(
                command_buffer,
                target.image.image(),
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                scene_image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &copy,
            );
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&after_copy),
            );
        }
        self.previous_view_projection = view_projection;
        self.previous_jitter_pixels = jitter_pixels;
        self.first_frame = false;
        self.current = 1 - self.current;
    }
}

impl Drop for TemporalAaPassState {
    fn drop(&mut self) {
        // SAFETY: the owner waits for device idle before dropping; children
        // are destroyed before parents.
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
            self.device.destroy_descriptor_pool(self.pool, None);
            self.device
                .destroy_descriptor_set_layout(self.set_layout, None);
            self.device.destroy_sampler(self.sampler, None);
            for target in &self.history {
                self.device.destroy_image_view(target.view, None);
            }
        }
    }
}

const IDENTITY: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Plan look/04 G3: the clip keeps a history inside the box and pulls one
    /// outside onto it; the first frame returns the current colour.
    #[test]
    fn history_is_clipped_and_blended() {
        let mean = [1.0, 2.0, 3.0];
        let deviation = [0.5, 0.5, 0.5];
        assert_eq!(
            clip_history([1.2, 1.8, 3.1], mean, deviation, TAA_CLIP_GAMMA),
            [1.2, 1.8, 3.1]
        );
        assert_eq!(
            clip_history([4.0, 0.0, 3.0], mean, deviation, TAA_CLIP_GAMMA),
            [1.5, 1.5, 3.0]
        );
        assert_eq!(blend_history([0.0; 3], [1.0; 3], 0.1, true), [1.0; 3]);
        let blended = blend_history([0.0; 3], [1.0; 3], 0.1, false);
        assert!((blended[0] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn fallback_reasons_name_each_missing_precondition() {
        assert_eq!(temporal_aa_fallback_reason(true, true, true), None);
        assert_eq!(
            temporal_aa_fallback_reason(false, true, true),
            Some("disabled by option")
        );
        assert_eq!(
            temporal_aa_fallback_reason(true, false, true),
            Some("no G-buffer prepass")
        );
        assert_eq!(
            temporal_aa_fallback_reason(true, true, false),
            Some("R16G16B16A16_SFLOAT is not a sampleable colour attachment")
        );
    }
}
