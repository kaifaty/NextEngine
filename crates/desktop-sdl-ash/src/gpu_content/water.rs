//! Water look L2 + L3 (plan `continuum-water/13`): the water pass.
//!
//! `WaterSurface` dynamic rings draw after the opaque scene: the pass copies
//! the opaque swapchain colour (the ADR-102 scene-copy technique), samples
//! the scene depth read-only and shades the surface with refraction, depth
//! absorption, a soft shoreline and foam on top of the WL1 material. Every
//! value here is renderer-local; nothing reaches gameplay or a root.

use ash::vk;

use next_render::B0CameraFrameV1;

use super::pipeline::{CameraMatricesV1, camera_matrices, camera_raster_region};
use super::resources::{BufferAllocation, ImageAllocation};
use super::{B0GpuContentError, DRAW_PUSH_CONSTANT_SIZE, VERTEX_STRIDE};

/// Bytes of the water uniform block: inverse view-projection, viewport,
/// absorption and shore constants.
pub(crate) const WATER_UNIFORM_SIZE: vk::DeviceSize = 128;
/// Plan 13 constants (renderer-local): absorption per metre and the
/// refraction strength of the ADR-102 particle pass, the shore band.
pub(crate) const WATER_ABSORPTION_PER_METRE: [f32; 3] = [1.2, 0.5, 0.25];
pub(crate) const WATER_REFRACTION_STRENGTH: f32 = 0.08;
pub(crate) const WATER_FOAM_WIDTH_METRES: f32 = 0.12;
pub(crate) const WATER_FADE_WIDTH_METRES: f32 = 0.04;
pub(crate) const WATER_FOAM_GREY: f32 = 0.85;

struct Target {
    image: ImageAllocation,
    view: vk::ImageView,
}

struct WaterSlot {
    uniform: BufferAllocation,
    set: vk::DescriptorSet,
}

/// Owner of every device object of the pass. Fields drop in dependency
/// order: pipeline, descriptors, samplers, views, images, buffers.
pub(crate) struct WaterPassState {
    device: ash::Device,
    extent: vk::Extent2D,
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    descriptor_pool: vk::DescriptorPool,
    set_layout: vk::DescriptorSetLayout,
    linear_sampler: vk::Sampler,
    nearest_sampler: vk::Sampler,
    scene_copy: Target,
    slots: Vec<WaterSlot>,
}

impl WaterPassState {
    /// Builds the pass; `Ok(Err(reason))` is the declared fallback (the
    /// WL1 material inside the world pass), `Err` a real device failure.
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
        frame_layout: vk::DescriptorSetLayout,
        texture_layout: vk::DescriptorSetLayout,
        shadow_layout: vk::DescriptorSetLayout,
        transfer_source: bool,
        depth_sampled: bool,
    ) -> Result<Result<Self, &'static str>, B0GpuContentError> {
        if !transfer_source {
            return Ok(Err("swapchain images carry no transfer-source usage"));
        }
        if !depth_sampled {
            return Ok(Err("scene depth format is not sampleable"));
        }
        if extent.width == 0 || extent.height == 0 {
            return Ok(Err("render extent is empty"));
        }
        let scene_copy = {
            let image = ImageAllocation::new(
                instance,
                physical_device,
                device,
                vk::Extent3D {
                    width: extent.width,
                    height: extent.height,
                    depth: 1,
                },
                color_format,
                vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            )?;
            let view_info = vk::ImageViewCreateInfo::default()
                .image(image.image())
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(color_format)
                .subresource_range(color_subresource());
            // SAFETY: the image is live and uses this exact colour format.
            let view = unsafe { device.create_image_view(&view_info, None) }?;
            Target { image, view }
        };
        let mut guard = Teardown {
            device: device.clone(),
            layouts: Vec::new(),
            samplers: Vec::new(),
            pool: vk::DescriptorPool::null(),
            pipeline: None,
            views: vec![scene_copy.view],
            armed: true,
        };
        let host = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let mut uniforms = Vec::with_capacity(frame_slot_count);
        for _ in 0..frame_slot_count {
            uniforms.push(BufferAllocation::new(
                instance,
                physical_device,
                device,
                WATER_UNIFORM_SIZE,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                host,
            )?);
        }

        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(2)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        ];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        // SAFETY: bindings are fixed pass values and retain no host pointers.
        let set_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None) }?;
        guard.layouts.push(set_layout);

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
        let linear_sampler = unsafe { device.create_sampler(&sampler(vk::Filter::LINEAR), None) }?;
        guard.samplers.push(linear_sampler);
        // SAFETY: same as above.
        let nearest_sampler =
            unsafe { device.create_sampler(&sampler(vk::Filter::NEAREST), None) }?;
        guard.samplers.push(nearest_sampler);

        let slot_count =
            u32::try_from(frame_slot_count).map_err(|_| B0GpuContentError::CountOverflow)?;
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: slot_count.saturating_mul(2),
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
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
        let mut slots = Vec::with_capacity(frame_slot_count);
        for (uniform, set) in uniforms.into_iter().zip(sets) {
            let image_info = [vk::DescriptorImageInfo::default()
                .sampler(linear_sampler)
                .image_view(scene_copy.view)
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
            let buffer_info = [vk::DescriptorBufferInfo::default()
                .buffer(uniform.buffer)
                .offset(0)
                .range(WATER_UNIFORM_SIZE)];
            let writes = [
                vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&image_info),
                vk::WriteDescriptorSet::default()
                    .dst_set(set)
                    .dst_binding(2)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&buffer_info),
            ];
            // SAFETY: set, view and buffer are live; descriptors are copied now.
            unsafe { device.update_descriptor_sets(&writes, &[]) };
            slots.push(WaterSlot { uniform, set });
        }

        let modules = crate::shader_assets::water_scene_shader_modules()
            .map_err(B0GpuContentError::ShaderAsset)?;
        let set_layouts = [frame_layout, texture_layout, shadow_layout, set_layout];
        let push_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            offset: 0,
            size: DRAW_PUSH_CONSTANT_SIZE,
        }];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&set_layouts)
            .push_constant_ranges(&push_ranges);
        // SAFETY: descriptor layouts are live and the push range is the B0
        // range within the guaranteed minimum.
        let layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }?;
        guard.pipeline = Some((vk::Pipeline::null(), layout));
        let pipeline = create_water_pipeline(
            device,
            layout,
            color_format,
            depth_format,
            &modules.vertex,
            &modules.fragment,
        )?;
        guard.pipeline = Some((pipeline, layout));

        guard.armed = false;
        Ok(Ok(Self {
            device: device.clone(),
            extent,
            pipeline,
            layout,
            descriptor_pool,
            set_layout,
            linear_sampler,
            nearest_sampler,
            scene_copy,
            slots,
        }))
    }

    pub(crate) const fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }

    pub(crate) const fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }

    pub(crate) fn set(
        &self,
        frame_slot_index: usize,
    ) -> Result<vk::DescriptorSet, B0GpuContentError> {
        self.slots.get(frame_slot_index).map(|slot| slot.set).ok_or(
            B0GpuContentError::InvalidFramePlan("frame slot index is outside the water pass ring"),
        )
    }

    /// Writes the slot's uniform for this camera and binds the frame's scene
    /// depth view; returns the raster viewport and scissor of the camera.
    pub(crate) fn prepare(
        &mut self,
        frame_slot_index: usize,
        camera: &B0CameraFrameV1,
        scene_depth_view: vk::ImageView,
    ) -> Result<(vk::Viewport, vk::Rect2D), B0GpuContentError> {
        let slot =
            self.slots
                .get_mut(frame_slot_index)
                .ok_or(B0GpuContentError::InvalidFramePlan(
                    "frame slot index is outside the water pass ring",
                ))?;
        let (viewport, scissor) = camera_raster_region(camera.viewport, self.extent)?;
        let matrices: CameraMatricesV1 = camera_matrices(camera, viewport)?;
        let view_projection = multiply(&matrices.projection, &matrices.view);
        let inverse = invert(&view_projection).ok_or(B0GpuContentError::InvalidFramePlan(
            "camera view-projection is not invertible",
        ))?;
        let mut bytes = [0_u8; WATER_UNIFORM_SIZE as usize];
        write_f32(&mut bytes[..64], &inverse);
        write_f32(
            &mut bytes[64..80],
            &[
                self.extent.width as f32,
                self.extent.height as f32,
                1.0 / self.extent.width as f32,
                1.0 / self.extent.height as f32,
            ],
        );
        write_f32(
            &mut bytes[80..96],
            &[
                WATER_ABSORPTION_PER_METRE[0],
                WATER_ABSORPTION_PER_METRE[1],
                WATER_ABSORPTION_PER_METRE[2],
                WATER_REFRACTION_STRENGTH,
            ],
        );
        write_f32(
            &mut bytes[96..112],
            &[
                WATER_FOAM_WIDTH_METRES,
                WATER_FADE_WIDTH_METRES,
                WATER_FOAM_GREY,
                0.0,
            ],
        );
        slot.uniform.write(0, &bytes)?;
        let depth_info = [vk::DescriptorImageInfo::default()
            .sampler(self.nearest_sampler)
            .image_view(scene_depth_view)
            .image_layout(vk::ImageLayout::DEPTH_READ_ONLY_OPTIMAL)];
        let writes = [vk::WriteDescriptorSet::default()
            .dst_set(slot.set)
            .dst_binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&depth_info)];
        // SAFETY: the slot's fence guarantees no pending command buffer
        // reads this set; the view belongs to the acquired swapchain slot.
        unsafe { self.device.update_descriptor_sets(&writes, &[]) };
        Ok((viewport, scissor))
    }

    /// Records the scene colour copy and moves the scene depth to read-only
    /// before the water rendering instance. The world rendering instance
    /// must have ended.
    pub(crate) fn record_begin(
        &self,
        command_buffer: vk::CommandBuffer,
        swapchain_image: vk::Image,
        depth_image: vk::Image,
    ) {
        let to_transfer = [
            image_barrier(
                swapchain_image,
                color_subresource(),
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
                color_subresource(),
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
                color_subresource(),
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
                color_subresource(),
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
            // The world pass wrote the depth; the water pass tests against
            // it and samples it, both read-only.
            image_barrier(
                depth_image,
                depth_subresource(),
                vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
                vk::ImageLayout::DEPTH_READ_ONLY_OPTIMAL,
                (
                    vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
                    vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
                ),
                (
                    vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
                        | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS
                        | vk::PipelineStageFlags2::FRAGMENT_SHADER,
                    vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ
                        | vk::AccessFlags2::SHADER_SAMPLED_READ,
                ),
            ),
        ];
        // SAFETY: every image belongs to this device, the world rendering
        // instance has ended, and the swapchain was created with
        // transfer-source usage for this run.
        unsafe {
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&to_transfer),
            );
            self.device.cmd_copy_image(
                command_buffer,
                swapchain_image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                self.scene_copy.image.image(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &copy,
            );
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&after_copy),
            );
        }
    }

    /// Returns the scene depth to attachment layout after the water
    /// rendering instance ended, for the particle pass and the overlay.
    pub(crate) fn record_end(&self, command_buffer: vk::CommandBuffer, depth_image: vk::Image) {
        let restore = [image_barrier(
            depth_image,
            depth_subresource(),
            vk::ImageLayout::DEPTH_READ_ONLY_OPTIMAL,
            vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            (
                vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS
                    | vk::PipelineStageFlags2::FRAGMENT_SHADER,
                vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags2::SHADER_SAMPLED_READ,
            ),
            (
                vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
                vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
            ),
        )];
        // SAFETY: the water rendering instance has ended on this command
        // buffer and the depth image belongs to the acquired slot.
        unsafe {
            self.device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&restore),
            );
        }
    }

    pub(crate) fn allocation_bytes(&self) -> vk::DeviceSize {
        self.scene_copy.image.allocation_size()
            + self
                .slots
                .iter()
                .map(|slot| slot.uniform.allocation_size())
                .sum::<vk::DeviceSize>()
    }
}

impl Drop for WaterPassState {
    fn drop(&mut self) {
        // SAFETY: the owner waits for device idle before dropping; children
        // are destroyed before parents.
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device
                .destroy_descriptor_set_layout(self.set_layout, None);
            self.device.destroy_sampler(self.linear_sampler, None);
            self.device.destroy_sampler(self.nearest_sampler, None);
            self.device.destroy_image_view(self.scene_copy.view, None);
        }
    }
}

/// Destroys partially constructed objects when construction fails midway.
struct Teardown {
    device: ash::Device,
    layouts: Vec<vk::DescriptorSetLayout>,
    samplers: Vec<vk::Sampler>,
    pool: vk::DescriptorPool,
    pipeline: Option<(vk::Pipeline, vk::PipelineLayout)>,
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
            if let Some((pipeline, layout)) = self.pipeline.take() {
                if pipeline != vk::Pipeline::null() {
                    self.device.destroy_pipeline(pipeline, None);
                }
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

fn create_water_pipeline(
    device: &ash::Device,
    layout: vk::PipelineLayout,
    color_format: vk::Format,
    depth_format: vk::Format,
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
        let bindings = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: VERTEX_STRIDE,
            input_rate: vk::VertexInputRate::VERTEX,
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
                format: vk::Format::R32G32_SFLOAT,
                offset: 12,
            },
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: vk::Format::R16G16B16A16_SNORM,
                offset: 20,
            },
        ];
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&bindings)
            .vertex_attribute_descriptions(&attributes);
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false)
            .line_width(1.0);
        let multisample = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        // Depth-tested against the opaque scene, never written: the depth
        // image is read-only and sampled during the pass.
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);
        let blend_attachments = [vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(false)
            .color_write_mask(vk::ColorComponentFlags::RGBA)];
        let color_blend =
            vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachments);
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        let color_formats = [color_format];
        let mut rendering = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&color_formats)
            .depth_attachment_format(depth_format);
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
        // live for the call; dynamic rendering declares the exact formats.
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

fn color_subresource() -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1)
}

fn depth_subresource() -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::default()
        .aspect_mask(vk::ImageAspectFlags::DEPTH)
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
    subresource: vk::ImageSubresourceRange,
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
        .subresource_range(subresource)
}

fn write_f32(target: &mut [u8], values: &[f32]) {
    for (chunk, value) in target.chunks_exact_mut(4).zip(values) {
        chunk.copy_from_slice(&value.to_le_bytes());
    }
}

/// Column-major `left * right`.
fn multiply(left: &[f32; 16], right: &[f32; 16]) -> [f64; 16] {
    let mut out = [0.0_f64; 16];
    for column in 0..4 {
        for row in 0..4 {
            let mut sum = 0.0_f64;
            for k in 0..4 {
                sum += f64::from(left[k * 4 + row]) * f64::from(right[column * 4 + k]);
            }
            out[column * 4 + row] = sum;
        }
    }
    out
}

/// General 4x4 inverse (column-major) by cofactors; `None` when singular.
pub(crate) fn invert(m: &[f64; 16]) -> Option<[f32; 16]> {
    let mut inv = [0.0_f64; 16];
    inv[0] = m[5] * m[10] * m[15] - m[5] * m[11] * m[14] - m[9] * m[6] * m[15]
        + m[9] * m[7] * m[14]
        + m[13] * m[6] * m[11]
        - m[13] * m[7] * m[10];
    inv[4] = -m[4] * m[10] * m[15] + m[4] * m[11] * m[14] + m[8] * m[6] * m[15]
        - m[8] * m[7] * m[14]
        - m[12] * m[6] * m[11]
        + m[12] * m[7] * m[10];
    inv[8] = m[4] * m[9] * m[15] - m[4] * m[11] * m[13] - m[8] * m[5] * m[15]
        + m[8] * m[7] * m[13]
        + m[12] * m[5] * m[11]
        - m[12] * m[7] * m[9];
    inv[12] = -m[4] * m[9] * m[14] + m[4] * m[10] * m[13] + m[8] * m[5] * m[14]
        - m[8] * m[6] * m[13]
        - m[12] * m[5] * m[10]
        + m[12] * m[6] * m[9];
    inv[1] = -m[1] * m[10] * m[15] + m[1] * m[11] * m[14] + m[9] * m[2] * m[15]
        - m[9] * m[3] * m[14]
        - m[13] * m[2] * m[11]
        + m[13] * m[3] * m[10];
    inv[5] = m[0] * m[10] * m[15] - m[0] * m[11] * m[14] - m[8] * m[2] * m[15]
        + m[8] * m[3] * m[14]
        + m[12] * m[2] * m[11]
        - m[12] * m[3] * m[10];
    inv[9] = -m[0] * m[9] * m[15] + m[0] * m[11] * m[13] + m[8] * m[1] * m[15]
        - m[8] * m[3] * m[13]
        - m[12] * m[1] * m[11]
        + m[12] * m[3] * m[9];
    inv[13] = m[0] * m[9] * m[14] - m[0] * m[10] * m[13] - m[8] * m[1] * m[14]
        + m[8] * m[2] * m[13]
        + m[12] * m[1] * m[10]
        - m[12] * m[2] * m[9];
    inv[2] = m[1] * m[6] * m[15] - m[1] * m[7] * m[14] - m[5] * m[2] * m[15]
        + m[5] * m[3] * m[14]
        + m[13] * m[2] * m[7]
        - m[13] * m[3] * m[6];
    inv[6] = -m[0] * m[6] * m[15] + m[0] * m[7] * m[14] + m[4] * m[2] * m[15]
        - m[4] * m[3] * m[14]
        - m[12] * m[2] * m[7]
        + m[12] * m[3] * m[6];
    inv[10] = m[0] * m[5] * m[15] - m[0] * m[7] * m[13] - m[4] * m[1] * m[15]
        + m[4] * m[3] * m[13]
        + m[12] * m[1] * m[7]
        - m[12] * m[3] * m[5];
    inv[14] = -m[0] * m[5] * m[14] + m[0] * m[6] * m[13] + m[4] * m[1] * m[14]
        - m[4] * m[2] * m[13]
        - m[12] * m[1] * m[6]
        + m[12] * m[2] * m[5];
    inv[3] = -m[1] * m[6] * m[11] + m[1] * m[7] * m[10] + m[5] * m[2] * m[11]
        - m[5] * m[3] * m[10]
        - m[9] * m[2] * m[7]
        + m[9] * m[3] * m[6];
    inv[7] = m[0] * m[6] * m[11] - m[0] * m[7] * m[10] - m[4] * m[2] * m[11]
        + m[4] * m[3] * m[10]
        + m[8] * m[2] * m[7]
        - m[8] * m[3] * m[6];
    inv[11] = -m[0] * m[5] * m[11] + m[0] * m[7] * m[9] + m[4] * m[1] * m[11]
        - m[4] * m[3] * m[9]
        - m[8] * m[1] * m[7]
        + m[8] * m[3] * m[5];
    inv[15] = m[0] * m[5] * m[10] - m[0] * m[6] * m[9] - m[4] * m[1] * m[10]
        + m[4] * m[2] * m[9]
        + m[8] * m[1] * m[6]
        - m[8] * m[2] * m[5];
    let determinant = m[0] * inv[0] + m[1] * inv[4] + m[2] * inv[8] + m[3] * inv[12];
    if !determinant.is_finite() || determinant.abs() < 1e-30 {
        return None;
    }
    let scale = 1.0 / determinant;
    let mut out = [0.0_f32; 16];
    for (index, value) in inv.iter().enumerate() {
        let scaled = value * scale;
        if !scaled.is_finite() {
            return None;
        }
        out[index] = scaled as f32;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::{invert, multiply};

    #[test]
    fn inverse_reproduces_the_identity() {
        let matrix = [
            2.0, 0.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.0, 1.0, -1.0, 1.0, 2.0, 3.0, 1.0,
        ];
        let inverse = invert(&matrix).expect("invertible");
        let identity = multiply(&inverse, &matrix.map(|value| value as f32));
        for (index, value) in identity.iter().enumerate() {
            let expected = if index % 5 == 0 { 1.0 } else { 0.0 };
            assert!((value - expected).abs() < 1e-5, "{index}: {value}");
        }
        assert!(invert(&[0.0; 16]).is_none());
    }
}
