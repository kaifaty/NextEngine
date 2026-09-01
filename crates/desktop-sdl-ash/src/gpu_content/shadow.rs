use ash::vk;
use next_render::B0FramePlanV1;

use super::pipeline::{draw_push_constant_bytes, frame_raster_state};
use super::resources::{SHADOW_MAP_EXTENT, ShadowMap};
use super::{
    B0GpuContent, B0GpuContentError, DRAW_PUSH_CONSTANT_SIZE, INDIRECT_COMMAND_STRIDE,
    VERTEX_STRIDE,
};

/// Fixed renderer-owned depth-only pipeline for the single outdoor shadow
/// map. Gameplay never observes this optional presentation resource.
pub(super) struct ShadowPipelineState {
    device: ash::Device,
    pub(super) pipeline: vk::Pipeline,
    pub(super) layout: vk::PipelineLayout,
}

pub(super) fn initialize_shadow_map(
    device: &ash::Device,
    queue: vk::Queue,
    queue_family_index: u32,
    shadow: &ShadowMap,
) -> Result<(), B0GpuContentError> {
    let pool_info = vk::CommandPoolCreateInfo::default()
        .queue_family_index(queue_family_index)
        .flags(vk::CommandPoolCreateFlags::TRANSIENT);
    // SAFETY: the queue family belongs to this device.
    let pool = unsafe { device.create_command_pool(&pool_info, None) }?;
    let result = (|| {
        let allocation_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        // SAFETY: the transient pool is live for the complete setup submit.
        let command = unsafe { device.allocate_command_buffers(&allocation_info) }?[0];
        let begin = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        // SAFETY: the fresh command buffer is recorded exactly once.
        unsafe { device.begin_command_buffer(command, &begin) }?;
        let subresource = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::DEPTH)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let barriers = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::NONE)
            .src_access_mask(vk::AccessFlags2::NONE)
            .dst_stage_mask(vk::PipelineStageFlags2::FRAGMENT_SHADER)
            .dst_access_mask(vk::AccessFlags2::SHADER_SAMPLED_READ)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::DEPTH_READ_ONLY_OPTIMAL)
            .image(shadow.image())
            .subresource_range(subresource)];
        let dependency = vk::DependencyInfo::default().image_memory_barriers(&barriers);
        // SAFETY: the new image starts in UNDEFINED and the barrier covers its
        // sole mip/layer.
        unsafe { device.cmd_pipeline_barrier2(command, &dependency) };
        // SAFETY: recording is active and references only the live shadow map.
        unsafe { device.end_command_buffer(command) }?;
        let commands = [command];
        let submits = [vk::SubmitInfo::default().command_buffers(&commands)];
        // SAFETY: the command buffer is executable and submitted to its own
        // queue family. Setup waits idle before returning ownership.
        unsafe {
            device.queue_submit(queue, &submits, vk::Fence::null())?;
            device.queue_wait_idle(queue)?;
        }
        Ok(())
    })();
    // SAFETY: setup either was not submitted or queue_wait_idle completed; a
    // device-loss error also permits child teardown during recovery.
    unsafe { device.destroy_command_pool(pool, None) };
    result
}

impl ShadowPipelineState {
    pub(super) fn new(
        device: &ash::Device,
        depth_format: vk::Format,
        frame_layout: vk::DescriptorSetLayout,
    ) -> Result<Self, B0GpuContentError> {
        let set_layouts = [frame_layout];
        let push_constant_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            offset: 0,
            size: DRAW_PUSH_CONSTANT_SIZE,
        }];
        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&set_layouts)
            .push_constant_ranges(&push_constant_ranges);
        // SAFETY: the descriptor layout is live and the closed 80-byte push
        // range is within Vulkan's guaranteed minimum capacity.
        let layout = unsafe { device.create_pipeline_layout(&layout_info, None) }?;
        match create_shadow_pipeline(device, depth_format, layout) {
            Ok(pipeline) => Ok(Self {
                device: device.clone(),
                pipeline,
                layout,
            }),
            Err(error) => {
                // SAFETY: failed construction leaves no pipeline dependant.
                unsafe { device.destroy_pipeline_layout(layout, None) };
                Err(error)
            }
        }
    }
}

impl Drop for ShadowPipelineState {
    fn drop(&mut self) {
        // SAFETY: the device is idle during owner teardown; the pipeline is
        // destroyed before its layout.
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
        }
    }
}

fn create_shadow_pipeline(
    device: &ash::Device,
    depth_format: vk::Format,
    layout: vk::PipelineLayout,
) -> Result<vk::Pipeline, B0GpuContentError> {
    let words = crate::shader_assets::shadow_vertex_shader_module()
        .map_err(B0GpuContentError::ShaderAsset)?;
    let module_info = vk::ShaderModuleCreateInfo::default().code(&words);
    // SAFETY: checked-in SPIR-V has already passed structural validation.
    let module = unsafe { device.create_shader_module(&module_info, None) }?;
    let result = {
        let stages = [vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(module)
            .name(c"main")];
        let bindings = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: VERTEX_STRIDE,
            input_rate: vk::VertexInputRate::VERTEX,
        }];
        let attributes = [vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 0,
        }];
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&bindings)
            .vertex_attribute_descriptions(&attributes);
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(true)
            .depth_bias_constant_factor(1.25)
            .depth_bias_slope_factor(1.75)
            .line_width(1.0);
        let multisample = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);
        let color_blend = vk::PipelineColorBlendStateCreateInfo::default();
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        let mut rendering =
            vk::PipelineRenderingCreateInfo::default().depth_attachment_format(depth_format);
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
        // SAFETY: all create-info slices remain live for the call and dynamic
        // rendering names the exact depth-only target format.
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
                Err(error.into())
            }
        }
    };
    // SAFETY: pipeline creation has copied all module state.
    unsafe { device.destroy_shader_module(module, None) };
    result
}

impl B0GpuContent {
    pub(crate) fn record_shadow(
        &self,
        command_buffer: vk::CommandBuffer,
        plan: &B0FramePlanV1,
        target_extent: vk::Extent2D,
        frame_slot_index: usize,
    ) -> Result<(), B0GpuContentError> {
        let (Some(shadow), Some(pipeline), Some(shadow_set)) = (
            self.shadow_map.as_ref(),
            self.shadow_pipeline.as_ref(),
            self.descriptors.shadow_set,
        ) else {
            return Ok(());
        };
        if plan.catalog_hash != self.catalog_hash {
            return Err(B0GpuContentError::InvalidFramePlan(
                "shadow catalog hash does not match uploaded resources",
            ));
        }
        let frame_uniform = self.frame_uniforms.get(frame_slot_index).ok_or(
            B0GpuContentError::InvalidFramePlan(
                "shadow frame slot index is outside the uniform ring",
            ),
        )?;
        let frame_set = self
            .descriptors
            .frame_sets
            .get(frame_slot_index)
            .copied()
            .ok_or(B0GpuContentError::InvalidFramePlan(
                "shadow frame slot index is outside the descriptor ring",
            ))?;
        debug_assert_ne!(shadow_set, vk::DescriptorSet::null());
        let raster_state = frame_raster_state(plan.camera.as_ref(), target_extent)?;
        frame_uniform.write(0, &raster_state.view_projection_bytes)?;

        let subresource = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::DEPTH)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let to_depth = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::FRAGMENT_SHADER)
            .src_access_mask(vk::AccessFlags2::SHADER_SAMPLED_READ)
            .dst_stage_mask(
                vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
            )
            .dst_access_mask(
                vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
            )
            .old_layout(vk::ImageLayout::DEPTH_READ_ONLY_OPTIMAL)
            .new_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
            .image(shadow.image())
            .subresource_range(subresource)];
        let to_depth_dependency = vk::DependencyInfo::default().image_memory_barriers(&to_depth);
        // SAFETY: setup initialized the sole subresource to depth-read-only;
        // this command executes on the same ordered graphics queue.
        unsafe {
            self.geometry
                .device
                .cmd_pipeline_barrier2(command_buffer, &to_depth_dependency);
        }

        let clear = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        };
        let depth_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(shadow.view())
            .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(clear);
        let render_area = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: SHADOW_MAP_EXTENT,
                height: SHADOW_MAP_EXTENT,
            },
        };
        let rendering = vk::RenderingInfo::default()
            .render_area(render_area)
            .layer_count(1)
            .depth_attachment(&depth_attachment);
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: SHADOW_MAP_EXTENT as f32,
            height: SHADOW_MAP_EXTENT as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [render_area];
        let vertex_buffers = [self.geometry.buffer];
        let vertex_offsets = [0];
        let frame_sets = [frame_set];
        // SAFETY: all resources belong to the recording device, the depth
        // attachment is in the declared layout, and fixed ranges match the
        // checked-in depth-only shader interface.
        unsafe {
            self.geometry
                .device
                .cmd_begin_rendering(command_buffer, &rendering);
            self.geometry.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.pipeline,
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
                pipeline.layout,
                0,
                &frame_sets,
                &[],
            );
        }
        for draw in plan.draws.iter().filter(|draw| draw.casts_shadow) {
            let push_constants =
                draw_push_constant_bytes(draw.transform, draw.base_color_rgba_unorm16);
            if let Some(binding) = self.dynamic_draw_binding(draw.mesh_revision, frame_slot_index) {
                // SAFETY: the slot ring was refreshed after this slot's fence
                // completed, both buffers are live host-visible allocations
                // of this device, and the immutable stream is rebound after.
                unsafe {
                    self.geometry.device.cmd_push_constants(
                        command_buffer,
                        pipeline.layout,
                        vk::ShaderStageFlags::VERTEX,
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
                }
                continue;
            }
            let draw_key = super::DrawKey {
                mesh_revision: draw.mesh_revision,
                first_index: draw.first_index,
                index_count: draw.index_count,
            };
            let indirect_offset = self.draw_offsets.get(&draw_key).copied().ok_or(
                B0GpuContentError::ResourceMissing("shadow indexed-indirect command"),
            )?;
            // SAFETY: the fixed push range and initialized indirect command
            // match the depth-only pipeline and uploaded geometry.
            unsafe {
                self.geometry.device.cmd_push_constants(
                    command_buffer,
                    pipeline.layout,
                    vk::ShaderStageFlags::VERTEX,
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
        // SAFETY: one depth-only rendering instance is active and ends here.
        unsafe { self.geometry.device.cmd_end_rendering(command_buffer) };

        let to_sample = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(
                vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
            )
            .src_access_mask(vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::FRAGMENT_SHADER)
            .dst_access_mask(vk::AccessFlags2::SHADER_SAMPLED_READ)
            .old_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
            .new_layout(vk::ImageLayout::DEPTH_READ_ONLY_OPTIMAL)
            .image(shadow.image())
            .subresource_range(subresource)];
        let to_sample_dependency = vk::DependencyInfo::default().image_memory_barriers(&to_sample);
        // SAFETY: depth rendering has ended and the following world pass reads
        // this same subresource only through the declared compare sampler.
        unsafe {
            self.geometry
                .device
                .cmd_pipeline_barrier2(command_buffer, &to_sample_dependency);
        }
        Ok(())
    }
}
