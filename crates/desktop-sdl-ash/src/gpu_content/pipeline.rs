use ash::vk;
use next_contracts::presentation::QuantizedPresentationTransformV1;

use super::{B0GpuContentError, DRAW_PUSH_CONSTANT_SIZE, FRAME_UNIFORM_SIZE, VERTEX_STRIDE};

const B0_FRONT_FACE: vk::FrontFace = vk::FrontFace::CLOCKWISE;

pub(super) struct PipelineState {
    device: ash::Device,
    pub(super) pipeline: vk::Pipeline,
    pub(super) layout: vk::PipelineLayout,
}

impl PipelineState {
    pub(super) fn new(
        device: &ash::Device,
        color_format: vk::Format,
        frame_layout: vk::DescriptorSetLayout,
        texture_layout: vk::DescriptorSetLayout,
    ) -> Result<Self, B0GpuContentError> {
        let set_layouts = [frame_layout, texture_layout];
        let push_constant_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            offset: 0,
            size: DRAW_PUSH_CONSTANT_SIZE,
        }];
        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&set_layouts)
            .push_constant_ranges(&push_constant_ranges);
        // SAFETY: descriptor layouts are live and the closed push range fits
        // Vulkan's minimum guaranteed 128-byte capacity.
        let layout = unsafe { device.create_pipeline_layout(&layout_info, None) }?;
        let result = create_graphics_pipeline(device, color_format, layout);
        match result {
            Ok(pipeline) => Ok(Self {
                device: device.clone(),
                pipeline,
                layout,
            }),
            Err(error) => {
                // SAFETY: no pipeline depends on the layout after failed
                // pipeline construction.
                unsafe { device.destroy_pipeline_layout(layout, None) };
                Err(error)
            }
        }
    }
}

impl Drop for PipelineState {
    fn drop(&mut self) {
        // SAFETY: both handles belong to this device and the pipeline is
        // destroyed before the layout it references.
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
        }
    }
}

fn create_graphics_pipeline(
    device: &ash::Device,
    color_format: vk::Format,
    layout: vk::PipelineLayout,
) -> Result<vk::Pipeline, B0GpuContentError> {
    let modules =
        crate::shader_assets::b0_shader_modules().map_err(B0GpuContentError::ShaderAsset)?;
    let vertex_info = vk::ShaderModuleCreateInfo::default().code(&modules.vertex);
    let fragment_info = vk::ShaderModuleCreateInfo::default().code(&modules.fragment);
    // SAFETY: decoded SPIR-V words remain live for the call and have validated
    // magic and alignment.
    let vertex_module = unsafe { device.create_shader_module(&vertex_info, None) }?;
    // SAFETY: fragment words meet the same offline-validated conditions.
    let fragment_module = match unsafe { device.create_shader_module(&fragment_info, None) } {
        Ok(module) => module,
        Err(error) => {
            // SAFETY: vertex module has no pipeline dependants yet.
            unsafe { device.destroy_shader_module(vertex_module, None) };
            return Err(error.into());
        }
    };

    let result = {
        let entry_point = c"main";
        let stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_module)
                .name(entry_point),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_module)
                .name(entry_point),
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
            .cull_mode(vk::CullModeFlags::BACK)
            // A positive-height Vulkan viewport reverses authored CCW NDC
            // winding in framebuffer coordinates.
            .front_face(B0_FRONT_FACE)
            .line_width(1.0);
        let multisample = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
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
        let create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization)
            .multisample_state(&multisample)
            .color_blend_state(&color_blend)
            .dynamic_state(&dynamic)
            .layout(layout)
            .push_next(&mut rendering);
        // SAFETY: every referenced create-info slice and shader module remains
        // live for the call; dynamic rendering declares the exact target format.
        match unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
        } {
            Ok(pipelines) => Ok(pipelines[0]),
            Err((partial, error)) => {
                // SAFETY: any partially created pipelines are owned results
                // from this failed call and have no external references.
                unsafe {
                    for pipeline in partial {
                        device.destroy_pipeline(pipeline, None);
                    }
                }
                Err(error.into())
            }
        }
    };

    // SAFETY: pipeline creation has finished copying module state, so both
    // temporary modules can be destroyed on success or failure.
    unsafe {
        device.destroy_shader_module(fragment_module, None);
        device.destroy_shader_module(vertex_module, None);
    }
    result
}

pub(super) fn identity_matrix_bytes() -> [u8; FRAME_UNIFORM_SIZE as usize] {
    matrix_bytes([
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ])
}

pub(super) fn frame_projection_bytes(extent: vk::Extent2D) -> [u8; FRAME_UNIFORM_SIZE as usize] {
    let aspect = extent.width as f32 / extent.height as f32;
    matrix_bytes([
        1.0 / aspect,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ])
}

fn matrix_bytes(matrix: [f32; 16]) -> [u8; FRAME_UNIFORM_SIZE as usize] {
    let mut bytes = [0_u8; FRAME_UNIFORM_SIZE as usize];
    for (destination, value) in bytes.chunks_exact_mut(4).zip(matrix) {
        destination.copy_from_slice(&value.to_le_bytes());
    }
    bytes
}

pub(super) fn draw_push_constant_bytes(
    transform: QuantizedPresentationTransformV1,
    base_color_rgba_unorm16: [u16; 4],
) -> [u8; DRAW_PUSH_CONSTANT_SIZE as usize] {
    let model = model_matrix(transform);
    let mut bytes = [0_u8; DRAW_PUSH_CONSTANT_SIZE as usize];
    for (destination, value) in bytes[..64].chunks_exact_mut(4).zip(model) {
        destination.copy_from_slice(&value.to_le_bytes());
    }
    for (destination, value) in bytes[64..].chunks_exact_mut(4).zip(base_color_rgba_unorm16) {
        destination.copy_from_slice(&(f32::from(value) / f32::from(u16::MAX)).to_le_bytes());
    }
    bytes
}

fn model_matrix(transform: QuantizedPresentationTransformV1) -> [f32; 16] {
    let q = transform
        .orientation_q30
        .map(|component| component as f32 / (1_u32 << 30) as f32);
    let [x, y, z, w] = q;
    let x2 = x + x;
    let y2 = y + y;
    let z2 = z + z;
    let xx = x * x2;
    let xy = x * y2;
    let xz = x * z2;
    let yy = y * y2;
    let yz = y * z2;
    let zz = z * z2;
    let wx = w * x2;
    let wy = w * y2;
    let wz = w * z2;
    let translation = transform
        .translation_micrometres
        .map(|component| component as f32 / 1_000_000.0);

    [
        1.0 - (yy + zz),
        xy + wz,
        xz - wy,
        0.0,
        xy - wz,
        1.0 - (xx + zz),
        yz + wx,
        0.0,
        xz + wy,
        yz - wx,
        1.0 - (xx + yy),
        0.0,
        translation[0],
        translation[1],
        translation[2],
        1.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_quantized_transform_encodes_identity_model() {
        assert_eq!(
            model_matrix(QuantizedPresentationTransformV1::default()),
            [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ]
        );
    }

    #[test]
    fn material_factor_uses_the_full_unorm16_range() {
        let bytes = draw_push_constant_bytes(
            QuantizedPresentationTransformV1::default(),
            [0, u16::MAX, 0, u16::MAX],
        );
        assert_eq!(&bytes[64..68], &0.0_f32.to_le_bytes());
        assert_eq!(&bytes[68..72], &1.0_f32.to_le_bytes());
        assert_eq!(&bytes[72..76], &0.0_f32.to_le_bytes());
        assert_eq!(&bytes[76..80], &1.0_f32.to_le_bytes());
    }

    #[test]
    fn positive_height_viewport_uses_clockwise_framebuffer_front_faces() {
        assert!(B0_FRONT_FACE == vk::FrontFace::CLOCKWISE);
    }
}
