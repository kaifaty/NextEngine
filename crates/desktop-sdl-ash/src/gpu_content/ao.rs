//! Scene look L3 (plan `look/03`): ground-truth ambient occlusion from the
//! G-buffer prepass. A fullscreen program reads the prepass's linear depth
//! and normals, integrates three horizon slices inside a one-metre radius
//! (GTAO) into an `R8` target, and a depth-aware separable blur smooths it;
//! the world material multiplies its sky irradiance by the result.
//! Renderer-local; nothing here enters gameplay, persistence or replay.

use ash::vk;

use super::B0GpuContentError;
use super::gbuffer::create_fullscreen_pipeline;
use super::resources::ImageAllocation;

/// The occlusion target format.
pub(crate) const AO_FORMAT: vk::Format = vk::Format::R8_UNORM;
/// The world-space radius the horizons are searched in.
pub(crate) const AO_RADIUS_METRES: f32 = 1.0;
pub(crate) const AO_SLICES: u32 = 3;
pub(crate) const AO_STEPS: u32 = 5;
/// `direction.xy` (texels, blur only), `radius`, spare; then the target's
/// width, height and their reciprocals.
const AO_PUSH_CONSTANT_SIZE: u32 = 32;
/// Revision 1: the occlusion runs at half the scene resolution.
const AO_RESOLUTION_DIVISOR: u32 = 2;

struct Target {
    image: ImageAllocation,
    view: vk::ImageView,
}

/// Owner of every device object of the pass; fields drop in dependency
/// order.
pub(crate) struct AmbientOcclusionPassState {
    device: ash::Device,
    extent: vk::Extent2D,
    sampler: vk::Sampler,
    set_layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    /// The occlusion set (depth, normal), the horizontal blur set (reads
    /// the raw target) and the vertical blur set (reads the blurred one).
    sets: [vk::DescriptorSet; 3],
    layout: vk::PipelineLayout,
    occlusion_pipeline: vk::Pipeline,
    blur_pipeline: vk::Pipeline,
    /// The raw occlusion, then the vertically blurred final.
    raw: Target,
    /// The horizontally blurred intermediate.
    blurred: Target,
}

/// Why the pass cannot be built, or `None` when it can (plan `look/03` G6).
pub(crate) const fn ambient_occlusion_fallback_reason(
    gbuffer: bool,
    r8_supported: bool,
) -> Option<&'static str> {
    if !gbuffer {
        Some("no G-buffer prepass")
    } else if !r8_supported {
        Some("R8_UNORM is not a sampleable colour attachment")
    } else {
        None
    }
}

/// The GTAO slice integral: the visibility of one slice given the projected
/// normal's angle `n` from the view vector (signed towards the positive
/// horizon) and the two horizon angles (negative side, positive side),
/// clamped to the normal's hemisphere. The same expression as `ao.frag`.
#[cfg(test)]
pub(crate) fn gtao_slice_visibility(n: f32, h1: f32, h2: f32) -> f32 {
    use std::f32::consts::FRAC_PI_2;
    let h1 = n + (h1 - n).max(-FRAC_PI_2);
    let h2 = n + (h2 - n).min(FRAC_PI_2);
    let arc = |h: f32| -(2.0 * h - n).cos() + n.cos() + 2.0 * h * n.sin();
    0.25 * (arc(h1) + arc(h2))
}

/// The cosine-weighted visibility of an open hemisphere around a normal
/// tilted by `tilt` from the view vector, as the slice sum computes it
/// (the projected-normal weight over `slices` slice directions).
#[cfg(test)]
pub(crate) fn open_hemisphere_visibility(tilt: f32, slices: usize) -> f32 {
    use std::f32::consts::{FRAC_PI_2, PI};
    // View along +z; the normal tilted towards +x.
    let normal = [tilt.sin(), 0.0, tilt.cos()];
    let mut total = 0.0;
    for slice in 0..slices {
        let phi = (slice as f32 + 0.5) * PI / slices as f32;
        // The slice axis in the screen plane (perpendicular to the view).
        let axis = [phi.cos(), phi.sin(), 0.0];
        let plane_normal = [-axis[1], axis[0], 0.0];
        let along_plane_normal = normal[0] * plane_normal[0] + normal[1] * plane_normal[1];
        let projected = [
            normal[0] - plane_normal[0] * along_plane_normal,
            normal[1] - plane_normal[1] * along_plane_normal,
            normal[2],
        ];
        let length = (projected[0] * projected[0]
            + projected[1] * projected[1]
            + projected[2] * projected[2])
            .sqrt();
        let unit = projected.map(|value| value / length);
        let towards_axis = unit[0] * axis[0] + unit[1] * axis[1];
        let n = unit[2].clamp(-1.0, 1.0).acos() * if towards_axis >= 0.0 { 1.0 } else { -1.0 };
        // An open hemisphere: the horizons sit on the tangent plane.
        total += length * gtao_slice_visibility(n, n - FRAC_PI_2, n + FRAC_PI_2);
    }
    total / slices as f32
}

impl AmbientOcclusionPassState {
    /// Builds the pass over the G-buffer's linear depth and normal views;
    /// `Ok(Err(reason))` is the declared fallback, `Err` a device failure.
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
        linear_depth_view: vk::ImageView,
        normal_view: vk::ImageView,
    ) -> Result<Result<Self, &'static str>, B0GpuContentError> {
        // SAFETY: the physical-device handle belongs to this live instance
        // and the query returns format capability data only.
        let properties =
            unsafe { instance.get_physical_device_format_properties(physical_device, AO_FORMAT) };
        let required = vk::FormatFeatureFlags::COLOR_ATTACHMENT
            | vk::FormatFeatureFlags::SAMPLED_IMAGE
            | vk::FormatFeatureFlags::TRANSFER_SRC;
        if let Some(reason) = ambient_occlusion_fallback_reason(
            true,
            properties.optimal_tiling_features.contains(required),
        ) {
            return Ok(Err(reason));
        }
        if extent.width == 0 || extent.height == 0 {
            return Ok(Err("render extent is empty"));
        }
        let extent = vk::Extent2D {
            width: extent.width.div_ceil(AO_RESOLUTION_DIVISOR),
            height: extent.height.div_ceil(AO_RESOLUTION_DIVISOR),
        };
        let occlusion_modules = super::super::shader_assets::ambient_occlusion_shader_modules()
            .map_err(B0GpuContentError::ShaderAsset)?;
        let blur_modules = super::super::shader_assets::ambient_occlusion_blur_shader_modules()
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
                AO_FORMAT,
                vk::ImageUsageFlags::COLOR_ATTACHMENT
                    | vk::ImageUsageFlags::SAMPLED
                    | vk::ImageUsageFlags::TRANSFER_SRC,
            )?;
            let view_info = vk::ImageViewCreateInfo::default()
                .image(image.image())
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(AO_FORMAT)
                .subresource_range(color_subresource());
            // SAFETY: the image is live and uses this exact format.
            let view = unsafe { device.create_image_view(&view_info, None) }?;
            guard.views.push(view);
            Ok::<Target, B0GpuContentError>(Target { image, view })
        };
        let raw = target(&mut guard)?;
        let blurred = target(&mut guard)?;
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
        let bindings = [0, 1, 2].map(|binding| {
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
            descriptor_count: 9,
        }];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(3)
            .pool_sizes(&pool_sizes);
        // SAFETY: the pool exactly covers the three sets.
        let pool = unsafe { device.create_descriptor_pool(&pool_info, None) }?;
        guard.pools.push(pool);
        let layouts = [set_layout; 3];
        let allocation_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool)
            .set_layouts(&layouts);
        // SAFETY: pool and layouts are live on this device.
        let allocated = unsafe { device.allocate_descriptor_sets(&allocation_info) }?;
        let sets = [allocated[0], allocated[1], allocated[2]];
        let image_info = |view: vk::ImageView| {
            [vk::DescriptorImageInfo::default()
                .sampler(sampler)
                .image_view(view)
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)]
        };
        let depth_info = image_info(linear_depth_view);
        let normal_info = image_info(normal_view);
        let raw_info = image_info(raw.view);
        let blurred_info = image_info(blurred.view);
        let mut writes = Vec::with_capacity(9);
        for (set, input) in [
            (sets[0], &raw_info),
            (sets[1], &raw_info),
            (sets[2], &blurred_info),
        ] {
            for (binding, info) in [(0, &depth_info), (1, &normal_info), (2, input)] {
                writes.push(
                    vk::WriteDescriptorSet::default()
                        .dst_set(set)
                        .dst_binding(binding)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(info),
                );
            }
        }
        // SAFETY: the sets, views and sampler are live; descriptors are
        // copied now.
        unsafe { device.update_descriptor_sets(&writes, &[]) };
        let push_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            offset: 0,
            size: AO_PUSH_CONSTANT_SIZE,
        }];
        let set_layouts = [frame_layout, set_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&set_layouts)
            .push_constant_ranges(&push_ranges);
        // SAFETY: both set layouts are live.
        let layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }?;
        guard.pipeline_layouts.push(layout);
        let occlusion_pipeline = create_fullscreen_pipeline(
            device,
            layout,
            AO_FORMAT,
            &occlusion_modules.vertex,
            &occlusion_modules.fragment,
        )?;
        guard.pipelines.push(occlusion_pipeline);
        let blur_pipeline = create_fullscreen_pipeline(
            device,
            layout,
            AO_FORMAT,
            &blur_modules.vertex,
            &blur_modules.fragment,
        )?;
        guard.pipelines.push(blur_pipeline);
        guard.armed = false;
        Ok(Ok(Self {
            device: device.clone(),
            extent,
            sampler,
            set_layout,
            pool,
            sets,
            layout,
            occlusion_pipeline,
            blur_pipeline,
            raw,
            blurred,
        }))
    }

    /// The final occlusion view the world material samples.
    pub(crate) fn final_view(&self) -> vk::ImageView {
        self.raw.view
    }

    pub(crate) fn sampler(&self) -> vk::Sampler {
        self.sampler
    }

    /// The final occlusion image for a capture (`R8`, in transfer-source
    /// layout after [`Self::record_capture_ready`]) and its extent.
    pub(crate) fn capture_image(&self) -> (vk::Image, vk::Format, vk::Extent2D) {
        (self.raw.image.image(), AO_FORMAT, self.extent)
    }

    pub(crate) fn allocation_bytes(&self) -> vk::DeviceSize {
        self.raw.image.allocation_size() + self.blurred.image.allocation_size()
    }

    /// Records the three fullscreen passes. The G-buffer's linear depth and
    /// normal images must be in transfer-source layout (the prepass leaves
    /// them there); they return to it. No rendering instance may be active.
    pub(crate) fn record(
        &self,
        command_buffer: vk::CommandBuffer,
        frame_set: vk::DescriptorSet,
        linear_depth_image: vk::Image,
        normal_image: vk::Image,
    ) {
        let inputs_to_sampled = [linear_depth_image, normal_image].map(|image| {
            image_barrier(
                image,
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
            )
        });
        let inputs_back = [linear_depth_image, normal_image].map(|image| {
            image_barrier(
                image,
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
            )
        });
        // SAFETY: the images belong to the G-buffer pass and this pass, are
        // in the declared layouts, and no rendering instance is active.
        unsafe {
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&inputs_to_sampled),
            );
        }
        self.record_pass(
            command_buffer,
            frame_set,
            self.sets[0],
            self.occlusion_pipeline,
            &self.raw,
            [0.0, 0.0, AO_RADIUS_METRES, 0.0],
        );
        self.record_pass(
            command_buffer,
            frame_set,
            self.sets[1],
            self.blur_pipeline,
            &self.blurred,
            [1.0, 0.0, AO_RADIUS_METRES, 0.0],
        );
        self.record_pass(
            command_buffer,
            frame_set,
            self.sets[2],
            self.blur_pipeline,
            &self.raw,
            [0.0, 1.0, AO_RADIUS_METRES, 0.0],
        );
        // SAFETY: the passes have ended; the inputs return to the capture
        // layout the G-buffer pass left them in.
        unsafe {
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&inputs_back),
            );
        }
    }

    /// One fullscreen pass into `target`, which ends in shader-read layout.
    fn record_pass(
        &self,
        command_buffer: vk::CommandBuffer,
        frame_set: vk::DescriptorSet,
        set: vk::DescriptorSet,
        pipeline: vk::Pipeline,
        target: &Target,
        push: [f32; 4],
    ) {
        let to_attachment = [image_barrier(
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
        )];
        let to_sampled = [image_barrier(
            target.image.image(),
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
        let lanes = [
            push[0],
            push[1],
            push[2],
            push[3],
            width,
            height,
            1.0 / width.max(1.0),
            1.0 / height.max(1.0),
        ];
        let mut bytes = [0_u8; AO_PUSH_CONSTANT_SIZE as usize];
        for (lane, value) in lanes.iter().enumerate() {
            bytes[lane * 4..lane * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        let sets = [frame_set, set];
        // SAFETY: every handle belongs to this pass or the content's frame
        // set, the target is in attachment layout inside the instance, and
        // the fullscreen triangle reads no buffers.
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
                &bytes,
            );
            self.device.cmd_draw(command_buffer, 3, 1, 0, 0);
            self.device.cmd_end_rendering(command_buffer);
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&to_sampled),
            );
        }
    }

    /// After the world pass: the final target to transfer-source layout for
    /// a capture. No rendering instance may be active.
    pub(crate) fn record_capture_ready(&self, command_buffer: vk::CommandBuffer) {
        let barrier = [image_barrier(
            self.raw.image.image(),
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
        // SAFETY: the world pass has ended; the image is in shader-read
        // layout.
        unsafe {
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&barrier),
            );
        }
    }
}

impl Drop for AmbientOcclusionPassState {
    fn drop(&mut self) {
        // SAFETY: the owner waits for device idle before dropping; children
        // are destroyed before parents.
        unsafe {
            self.device.destroy_pipeline(self.occlusion_pipeline, None);
            self.device.destroy_pipeline(self.blur_pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
            self.device.destroy_descriptor_pool(self.pool, None);
            self.device
                .destroy_descriptor_set_layout(self.set_layout, None);
            self.device.destroy_sampler(self.sampler, None);
            self.device.destroy_image_view(self.raw.view, None);
            self.device.destroy_image_view(self.blurred.view, None);
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

fn color_subresource() -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
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
        .subresource_range(color_subresource())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

    /// Plan look/03 G3: the slice integral is one for open horizons at a
    /// normal along the view, less inside 45 degrees, monotone.
    #[test]
    fn slice_integral_is_one_when_open_and_falls_with_the_horizons() {
        assert!((gtao_slice_visibility(0.0, -FRAC_PI_2, FRAC_PI_2) - 1.0).abs() < 1e-5);
        let half = gtao_slice_visibility(0.0, -FRAC_PI_4, FRAC_PI_4);
        assert!(half < 0.8 && half > 0.3, "{half}");
        // The positive horizon rises from the view vector to the hemisphere's
        // edge: visibility grows with it.
        let mut previous = 0.0;
        for step in 0..=20 {
            let h = step as f32 * (FRAC_PI_2 / 20.0);
            let value = gtao_slice_visibility(0.0, -FRAC_PI_2, h);
            assert!(value >= previous - 1e-6, "{h}: {value} < {previous}");
            previous = value;
        }
        // A tilted normal reads open once the slices are averaged with the
        // projected-normal weight (the shader's sum).
        for tilt in [0.0_f32, 0.5, 1.0, 1.3] {
            let value = open_hemisphere_visibility(tilt, 256);
            assert!((value - 1.0).abs() < 0.01, "tilt {tilt}: {value}");
        }
    }

    #[test]
    fn fallback_reasons_name_each_missing_precondition() {
        assert_eq!(ambient_occlusion_fallback_reason(true, true), None);
        assert_eq!(
            ambient_occlusion_fallback_reason(false, true),
            Some("no G-buffer prepass")
        );
        assert_eq!(
            ambient_occlusion_fallback_reason(true, false),
            Some("R8_UNORM is not a sampleable colour attachment")
        );
    }
}
