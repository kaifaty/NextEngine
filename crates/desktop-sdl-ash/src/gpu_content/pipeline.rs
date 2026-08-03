use ash::vk;
use next_contracts::presentation::{
    CameraProjectionProfileV1, CameraResultSampleV1, CameraRoleV1, CameraViewportV1,
    QuantizedPresentationTransformV1,
};
use next_render::B0CameraFrameV1;

use super::{B0GpuContentError, DRAW_PUSH_CONSTANT_SIZE, FRAME_UNIFORM_SIZE, VERTEX_STRIDE};

const B0_FRONT_FACE: vk::FrontFace = vk::FrontFace::COUNTER_CLOCKWISE;
const B0_DEPTH_COMPARE_OP: vk::CompareOp = vk::CompareOp::LESS_OR_EQUAL;
const UNORM16_MAX: u64 = u16::MAX as u64;
const MICROMETRES_PER_METRE: f64 = 1_000_000.0;

/// Fixed raster state for one pipeline built on the shared B0 shader
/// interface. The overlay variant reuses the checked-in SPIR-V modules and
/// only reconfigures blend, depth and culling (ADR-003 owns this boundary).
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct RasterFixedStateV1 {
    pub(super) blend_enable: bool,
    pub(super) depth_test_enable: bool,
    pub(super) depth_write_enable: bool,
    pub(super) cull_mode: vk::CullModeFlags,
}

const B0_RASTER_FIXED_STATE: RasterFixedStateV1 = RasterFixedStateV1 {
    blend_enable: false,
    depth_test_enable: true,
    depth_write_enable: true,
    cull_mode: vk::CullModeFlags::BACK,
};

/// Straight-alpha source-over compositing without depth or culling for the
/// semantic UI overlay quad.
pub(super) const UI_OVERLAY_RASTER_FIXED_STATE: RasterFixedStateV1 = RasterFixedStateV1 {
    blend_enable: true,
    depth_test_enable: false,
    depth_write_enable: false,
    cull_mode: vk::CullModeFlags::NONE,
};

pub(super) struct FrameRasterState {
    pub(super) view_projection_bytes: [u8; FRAME_UNIFORM_SIZE as usize],
    pub(super) viewport: vk::Viewport,
    pub(super) scissor: vk::Rect2D,
}

fn raster_depth_state(
    fixed: RasterFixedStateV1,
) -> vk::PipelineDepthStencilStateCreateInfo<'static> {
    vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(fixed.depth_test_enable)
        .depth_write_enable(fixed.depth_write_enable)
        .depth_compare_op(B0_DEPTH_COMPARE_OP)
        .depth_bounds_test_enable(false)
        .stencil_test_enable(false)
}

#[cfg(test)]
fn b0_depth_stencil_state() -> vk::PipelineDepthStencilStateCreateInfo<'static> {
    raster_depth_state(B0_RASTER_FIXED_STATE)
}

pub(super) struct PipelineState {
    device: ash::Device,
    pub(super) pipeline: vk::Pipeline,
    pub(super) layout: vk::PipelineLayout,
}

impl PipelineState {
    pub(super) fn new(
        device: &ash::Device,
        color_format: vk::Format,
        depth_format: vk::Format,
        frame_layout: vk::DescriptorSetLayout,
        texture_layout: vk::DescriptorSetLayout,
    ) -> Result<Self, B0GpuContentError> {
        Self::new_with_fixed_state(
            device,
            color_format,
            depth_format,
            frame_layout,
            texture_layout,
            B0_RASTER_FIXED_STATE,
        )
    }

    /// Builds the semantic UI overlay variant: same shader interface and push
    /// constants, straight-alpha blending, no depth testing, no culling.
    pub(super) fn new_ui_overlay(
        device: &ash::Device,
        color_format: vk::Format,
        depth_format: vk::Format,
        frame_layout: vk::DescriptorSetLayout,
        texture_layout: vk::DescriptorSetLayout,
    ) -> Result<Self, B0GpuContentError> {
        Self::new_with_fixed_state(
            device,
            color_format,
            depth_format,
            frame_layout,
            texture_layout,
            UI_OVERLAY_RASTER_FIXED_STATE,
        )
    }

    fn new_with_fixed_state(
        device: &ash::Device,
        color_format: vk::Format,
        depth_format: vk::Format,
        frame_layout: vk::DescriptorSetLayout,
        texture_layout: vk::DescriptorSetLayout,
        fixed: RasterFixedStateV1,
    ) -> Result<Self, B0GpuContentError> {
        let layout = create_pipeline_layout(device, frame_layout, texture_layout)?;
        let result = create_graphics_pipeline(device, color_format, depth_format, layout, fixed);
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

fn create_pipeline_layout(
    device: &ash::Device,
    frame_layout: vk::DescriptorSetLayout,
    texture_layout: vk::DescriptorSetLayout,
) -> Result<vk::PipelineLayout, B0GpuContentError> {
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
    Ok(unsafe { device.create_pipeline_layout(&layout_info, None) }?)
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
    depth_format: vk::Format,
    layout: vk::PipelineLayout,
    fixed: RasterFixedStateV1,
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
            .cull_mode(fixed.cull_mode)
            // A positive-height Vulkan viewport reverses authored CCW NDC
            // winding in framebuffer coordinates.
            .front_face(B0_FRONT_FACE)
            .line_width(1.0);
        let multisample = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let depth_stencil = raster_depth_state(fixed);
        let blend_attachments = [vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(fixed.blend_enable)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::RGBA)];
        let color_blend =
            vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachments);
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        let color_formats = [color_format];
        let mut rendering = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&color_formats)
            .depth_attachment_format(depth_format);
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

pub(super) fn frame_raster_state(
    camera: Option<&B0CameraFrameV1>,
    target_extent: vk::Extent2D,
) -> Result<FrameRasterState, B0GpuContentError> {
    if target_extent.width == 0 || target_extent.height == 0 {
        return Err(invalid_frame_plan("render extent must be non-zero"));
    }

    match camera {
        Some(camera) => {
            validate_camera_frame(camera)?;
            let (viewport, scissor) = camera_raster_region(camera.viewport, target_extent)?;
            let matrix = camera_view_projection_matrix(camera, viewport)?;
            Ok(FrameRasterState {
                view_projection_bytes: matrix_bytes(matrix),
                viewport,
                scissor,
            })
        }
        None => fallback_raster_state(target_extent),
    }
}

fn fallback_raster_state(
    target_extent: vk::Extent2D,
) -> Result<FrameRasterState, B0GpuContentError> {
    let viewport = vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: target_extent.width as f32,
        height: target_extent.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    };
    let scissor = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: target_extent,
    };
    let aspect = f64::from(target_extent.width) / f64::from(target_extent.height);
    let fallback = f64_matrix_to_f32([
        1.0 / aspect,
        0.0,
        0.0,
        0.0,
        0.0,
        -1.0,
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
    ])?;
    Ok(FrameRasterState {
        view_projection_bytes: matrix_bytes(fallback),
        viewport,
        scissor,
    })
}

fn validate_camera_frame(camera: &B0CameraFrameV1) -> Result<(), B0GpuContentError> {
    if camera.camera_role != CameraRoleV1::PrimaryThirdPerson {
        return Err(invalid_frame_plan(
            "B0 camera role must be primary third person",
        ));
    }
    CameraViewportV1::new(
        camera.viewport.viewport_id,
        camera.viewport.origin_unorm16,
        camera.viewport.extent_unorm16,
    )
    .map_err(|_| invalid_frame_plan("camera viewport is invalid"))?;
    CameraProjectionProfileV1::new(
        camera.projection_profile.vertical_fov_millidegrees,
        camera.projection_profile.near_plane_micrometres,
        camera.projection_profile.far_plane_micrometres,
    )
    .map_err(|_| invalid_frame_plan("camera projection profile is invalid"))?;
    CameraResultSampleV1::validate(camera.current_result_sample)
        .map_err(|_| invalid_frame_plan("camera current result sample is invalid"))
}

fn camera_raster_region(
    viewport: CameraViewportV1,
    target_extent: vk::Extent2D,
) -> Result<(vk::Viewport, vk::Rect2D), B0GpuContentError> {
    let [left, top, right, bottom] = normalized_viewport_edges(viewport, target_extent);
    if ![left, top, right, bottom].into_iter().all(f64::is_finite) || right <= left || bottom <= top
    {
        return Err(invalid_frame_plan(
            "camera viewport cannot be represented by the active target",
        ));
    }

    let viewport = vk::Viewport {
        x: finite_f32(left)?,
        y: finite_f32(top)?,
        width: finite_f32(right - left)?,
        height: finite_f32(bottom - top)?,
        min_depth: 0.0,
        max_depth: 1.0,
    };

    let origin_x = scissor_floor(left, target_extent.width)?;
    let origin_y = scissor_floor(top, target_extent.height)?;
    let end_x = scissor_ceil(right, target_extent.width)?;
    let end_y = scissor_ceil(bottom, target_extent.height)?;
    let scissor_width = end_x
        .checked_sub(origin_x)
        .filter(|value| *value != 0)
        .ok_or_else(|| {
            invalid_frame_plan("camera viewport has no addressable horizontal pixels")
        })?;
    let scissor_height = end_y
        .checked_sub(origin_y)
        .filter(|value| *value != 0)
        .ok_or_else(|| invalid_frame_plan("camera viewport has no addressable vertical pixels"))?;
    let offset_x = i32::try_from(origin_x)
        .map_err(|_| invalid_frame_plan("camera viewport x offset exceeds Vulkan limits"))?;
    let offset_y = i32::try_from(origin_y)
        .map_err(|_| invalid_frame_plan("camera viewport y offset exceeds Vulkan limits"))?;

    Ok((
        viewport,
        vk::Rect2D {
            offset: vk::Offset2D {
                x: offset_x,
                y: offset_y,
            },
            extent: vk::Extent2D {
                width: scissor_width,
                height: scissor_height,
            },
        },
    ))
}

fn normalized_viewport_edges(viewport: CameraViewportV1, target_extent: vk::Extent2D) -> [f64; 4] {
    let origin_x = u64::from(viewport.origin_unorm16[0]);
    let origin_y = u64::from(viewport.origin_unorm16[1]);
    let right = origin_x + u64::from(viewport.extent_unorm16[0]);
    let bottom = origin_y + u64::from(viewport.extent_unorm16[1]);
    let width = f64::from(target_extent.width);
    let height = f64::from(target_extent.height);
    [
        origin_x as f64 * width / UNORM16_MAX as f64,
        origin_y as f64 * height / UNORM16_MAX as f64,
        right as f64 * width / UNORM16_MAX as f64,
        bottom as f64 * height / UNORM16_MAX as f64,
    ]
}

fn scissor_floor(value: f64, limit: u32) -> Result<u32, B0GpuContentError> {
    if !value.is_finite() || value < 0.0 || value > f64::from(limit) {
        return Err(invalid_frame_plan(
            "camera viewport scissor origin is outside the active target",
        ));
    }
    Ok(value.floor() as u32)
}

fn scissor_ceil(value: f64, limit: u32) -> Result<u32, B0GpuContentError> {
    if !value.is_finite() || value < 0.0 || value > f64::from(limit) {
        return Err(invalid_frame_plan(
            "camera viewport scissor edge is outside the active target",
        ));
    }
    Ok(value.ceil() as u32)
}

fn camera_view_projection_matrix(
    camera: &B0CameraFrameV1,
    viewport: vk::Viewport,
) -> Result<[f32; 16], B0GpuContentError> {
    let eye = micrometres_to_metres(camera.current_result_sample.pose.translation_micrometres);
    let focus = micrometres_to_metres(camera.current_result_sample.focus_point_micrometres);
    let forward = normalize3(subtract3(focus, eye))
        .ok_or_else(|| invalid_frame_plan("camera eye and focus do not define a view direction"))?;
    let side = normalize3(cross3(forward, [0.0, 1.0, 0.0]))
        .ok_or_else(|| invalid_frame_plan("camera view direction is parallel to world up"))?;
    let up = cross3(side, forward);
    if !up.into_iter().all(f64::is_finite) {
        return Err(invalid_frame_plan("camera view basis is not finite"));
    }

    // Column-major right-handed view matrix. The camera looks down view -Z.
    let view = [
        side[0],
        up[0],
        -forward[0],
        0.0,
        side[1],
        up[1],
        -forward[1],
        0.0,
        side[2],
        up[2],
        -forward[2],
        0.0,
        -dot3(side, eye),
        -dot3(up, eye),
        dot3(forward, eye),
        1.0,
    ];

    let projection_profile = camera.projection_profile;
    let vertical_fov_radians =
        f64::from(projection_profile.vertical_fov_millidegrees) * std::f64::consts::PI / 180_000.0;
    let tan_half_fov = (vertical_fov_radians * 0.5).tan();
    let aspect = f64::from(viewport.width) / f64::from(viewport.height);
    let near = projection_profile.near_plane_micrometres as f64 / MICROMETRES_PER_METRE;
    let far = projection_profile.far_plane_micrometres as f64 / MICROMETRES_PER_METRE;
    if ![tan_half_fov, aspect, near, far]
        .into_iter()
        .all(f64::is_finite)
        || tan_half_fov <= 0.0
        || aspect <= 0.0
        || near <= 0.0
        || far <= near
    {
        return Err(invalid_frame_plan(
            "camera projection cannot be represented as finite RH_ZO math",
        ));
    }

    let inverse_tan = 1.0 / tan_half_fov;
    let depth_scale = far / (near - far);
    let depth_translation = far * near / (near - far);
    // Vulkan has a top-left framebuffer origin for a positive-height
    // viewport. Negating projection Y keeps camera +Y visually upward; the
    // second orientation reversal is paired with CCW framebuffer front faces.
    let projection = [
        inverse_tan / aspect,
        0.0,
        0.0,
        0.0,
        0.0,
        -inverse_tan,
        0.0,
        0.0,
        0.0,
        0.0,
        depth_scale,
        -1.0,
        0.0,
        0.0,
        depth_translation,
        0.0,
    ];
    f64_matrix_to_f32(multiply_column_major_4x4(projection, view))
}

fn micrometres_to_metres(values: [i64; 3]) -> [f64; 3] {
    values.map(|value| value as f64 / MICROMETRES_PER_METRE)
}

fn subtract3(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn dot3(left: [f64; 3], right: [f64; 3]) -> f64 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn cross3(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn normalize3(value: [f64; 3]) -> Option<[f64; 3]> {
    let length_squared = dot3(value, value);
    if !length_squared.is_finite() || length_squared <= 0.0 {
        return None;
    }
    let inverse_length = length_squared.sqrt().recip();
    let normalized = value.map(|component| component * inverse_length);
    normalized
        .into_iter()
        .all(f64::is_finite)
        .then_some(normalized)
}

fn multiply_column_major_4x4(left: [f64; 16], right: [f64; 16]) -> [f64; 16] {
    let mut product = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            product[column * 4 + row] = (0..4)
                .map(|index| left[index * 4 + row] * right[column * 4 + index])
                .sum();
        }
    }
    product
}

fn f64_matrix_to_f32(matrix: [f64; 16]) -> Result<[f32; 16], B0GpuContentError> {
    let mut converted = [0.0_f32; 16];
    for (destination, value) in converted.iter_mut().zip(matrix) {
        let converted_value = value as f32;
        if !value.is_finite() || !converted_value.is_finite() {
            return Err(invalid_frame_plan(
                "camera matrix contains a non-finite component",
            ));
        }
        *destination = converted_value;
    }
    Ok(converted)
}

fn finite_f32(value: f64) -> Result<f32, B0GpuContentError> {
    let converted = value as f32;
    if !value.is_finite() || !converted.is_finite() || converted < 0.0 {
        return Err(invalid_frame_plan(
            "camera viewport cannot be represented by Vulkan",
        ));
    }
    Ok(converted)
}

const fn invalid_frame_plan(reason: &'static str) -> B0GpuContentError {
    B0GpuContentError::InvalidFramePlan(reason)
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
    use next_contracts::ids::{AssetId, ContentHash, PersistentId};
    use next_contracts::presentation::CameraInterpolationPolicyV1;
    use next_contracts::project::AssetRevisionRefV1;

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
    fn vulkan_y_projection_and_positive_viewport_preserve_ccw_front_faces() {
        assert!(B0_FRONT_FACE == vk::FrontFace::COUNTER_CLOCKWISE);
    }

    #[test]
    fn b0_depth_uses_conventional_zero_to_one_less_or_equal_testing() {
        let state = b0_depth_stencil_state();
        assert!(state.depth_test_enable == vk::TRUE);
        assert!(state.depth_write_enable == vk::TRUE);
        assert!(state.depth_compare_op == vk::CompareOp::LESS_OR_EQUAL);
        assert!(state.depth_bounds_test_enable == vk::FALSE);
        assert!(state.stencil_test_enable == vk::FALSE);
    }

    #[test]
    fn typed_camera_builds_vulkan_rh_zo_view_projection() {
        let camera = camera_frame(CameraViewportV1::full(0), [0, 0, 3_000_000], [0, 0, 0]);
        let state = frame_raster_state(
            Some(&camera),
            vk::Extent2D {
                width: 800,
                height: 800,
            },
        )
        .expect("camera raster state");
        let matrix = matrix_from_bytes(state.view_projection_bytes);

        let near = transform_homogeneous(matrix, [0.0, 0.0, 2.0, 1.0]);
        let far = transform_homogeneous(matrix, [0.0, 0.0, -7.0, 1.0]);
        let focus = transform_homogeneous(matrix, [0.0, 0.0, 0.0, 1.0]);
        let above_focus = transform_homogeneous(matrix, [0.0, 1.0, 0.0, 1.0]);

        assert_approx(near[2] / near[3], 0.0);
        assert_approx(far[2] / far[3], 1.0);
        assert_approx(focus[0] / focus[3], 0.0);
        assert_approx(focus[1] / focus[3], 0.0);
        assert!(
            above_focus[1] / above_focus[3] < 0.0,
            "projection Y must place world up toward Vulkan's top-left framebuffer origin"
        );
    }

    #[test]
    fn typed_viewport_controls_dynamic_viewport_and_scissor() {
        let camera = camera_frame(
            CameraViewportV1::new(7, [2_570, 1_275], [25_700, 12_750]).expect("typed viewport"),
            [0, 0, 3_000_000],
            [0, 0, 0],
        );
        let state = frame_raster_state(
            Some(&camera),
            vk::Extent2D {
                width: 255,
                height: 257,
            },
        )
        .expect("camera raster state");

        assert_approx(state.viewport.x, 10.0);
        assert_approx(state.viewport.y, 5.0);
        assert_approx(state.viewport.width, 100.0);
        assert_approx(state.viewport.height, 50.0);
        assert!(state.scissor.offset == vk::Offset2D { x: 10, y: 5 });
        assert!(
            state.scissor.extent
                == vk::Extent2D {
                    width: 100,
                    height: 50
                }
        );
    }

    #[test]
    fn absent_camera_uses_deterministic_full_target_fallback() {
        let extent = vk::Extent2D {
            width: 960,
            height: 540,
        };
        let first = frame_raster_state(None, extent).expect("fallback");
        let second = frame_raster_state(None, extent).expect("fallback");
        let matrix = matrix_from_bytes(first.view_projection_bytes);

        assert_eq!(first.view_projection_bytes, second.view_projection_bytes);
        assert_approx(first.viewport.x, 0.0);
        assert_approx(first.viewport.y, 0.0);
        assert_approx(first.viewport.width, 960.0);
        assert_approx(first.viewport.height, 540.0);
        assert!(first.scissor.offset == vk::Offset2D { x: 0, y: 0 });
        assert!(first.scissor.extent == extent);
        assert!(matrix[5] < 0.0, "fallback must use the same Vulkan Y flip");
    }

    #[test]
    fn coincident_camera_eye_and_focus_fail_closed() {
        let camera = camera_frame(
            CameraViewportV1::full(0),
            [0, 1_000_000, 0],
            [0, 1_000_000, 0],
        );
        assert!(matches!(
            frame_raster_state(
                Some(&camera),
                vk::Extent2D {
                    width: 800,
                    height: 600
                }
            ),
            Err(B0GpuContentError::InvalidFramePlan(
                "camera eye and focus do not define a view direction"
            ))
        ));
    }

    #[test]
    fn camera_parallel_to_world_up_fails_closed() {
        let camera = camera_frame(CameraViewportV1::full(0), [0, 0, 0], [0, 1_000_000, 0]);
        assert!(matches!(
            frame_raster_state(
                Some(&camera),
                vk::Extent2D {
                    width: 800,
                    height: 600
                }
            ),
            Err(B0GpuContentError::InvalidFramePlan(
                "camera view direction is parallel to world up"
            ))
        ));
    }

    #[test]
    fn malformed_typed_camera_fields_fail_before_gpu_use() {
        let mut camera = camera_frame(CameraViewportV1::full(0), [0, 0, 3_000_000], [0, 0, 0]);
        camera.projection_profile.vertical_fov_millidegrees = 0;
        assert!(matches!(
            frame_raster_state(
                Some(&camera),
                vk::Extent2D {
                    width: 800,
                    height: 600
                }
            ),
            Err(B0GpuContentError::InvalidFramePlan(
                "camera projection profile is invalid"
            ))
        ));

        camera.projection_profile.vertical_fov_millidegrees = 90_000;
        camera.viewport = CameraViewportV1 {
            viewport_id: 0,
            origin_unorm16: [u16::MAX, 0],
            extent_unorm16: [1, u16::MAX],
        };
        assert!(matches!(
            frame_raster_state(
                Some(&camera),
                vk::Extent2D {
                    width: 800,
                    height: 600
                }
            ),
            Err(B0GpuContentError::InvalidFramePlan(
                "camera viewport is invalid"
            ))
        ));
    }

    #[test]
    fn non_finite_matrix_component_fails_closed() {
        let mut matrix = [0.0; 16];
        matrix[0] = f64::INFINITY;
        assert!(matches!(
            f64_matrix_to_f32(matrix),
            Err(B0GpuContentError::InvalidFramePlan(
                "camera matrix contains a non-finite component"
            ))
        ));
    }

    fn camera_frame(
        viewport: CameraViewportV1,
        eye_micrometres: [i64; 3],
        focus_micrometres: [i64; 3],
    ) -> B0CameraFrameV1 {
        let result = CameraResultSampleV1 {
            pose: QuantizedPresentationTransformV1 {
                translation_micrometres: eye_micrometres,
                ..QuantizedPresentationTransformV1::default()
            },
            focus_point_micrometres: focus_micrometres,
        };
        B0CameraFrameV1 {
            camera_record_hash: ContentHash::from_bytes([1; 32]),
            camera_id: PersistentId::from_bytes([2; 16]),
            camera_role: CameraRoleV1::PrimaryThirdPerson,
            viewport,
            projection_profile: CameraProjectionProfileV1::new(90_000, 1_000_000, 10_000_000)
                .expect("projection"),
            exposure_profile_revision: AssetRevisionRefV1 {
                asset_id: AssetId::from_bytes([3; 16]),
                record_sha256: ContentHash::from_bytes([4; 32]),
            },
            previous_result_sample: result,
            current_result_sample: result,
            cut: false,
            interpolation_policy: CameraInterpolationPolicyV1::LinearPose,
        }
    }

    fn matrix_from_bytes(bytes: [u8; FRAME_UNIFORM_SIZE as usize]) -> [f32; 16] {
        let mut matrix = [0.0; 16];
        for (destination, source) in matrix.iter_mut().zip(bytes.chunks_exact(4)) {
            *destination = f32::from_le_bytes(source.try_into().expect("one matrix component"));
        }
        matrix
    }

    fn transform_homogeneous(matrix: [f32; 16], value: [f32; 4]) -> [f32; 4] {
        let mut transformed = [0.0; 4];
        for row in 0..4 {
            transformed[row] = (0..4)
                .map(|column| matrix[column * 4 + row] * value[column])
                .sum();
        }
        transformed
    }

    fn assert_approx(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 1.0e-5,
            "expected {expected}, got {actual}"
        );
    }
}
