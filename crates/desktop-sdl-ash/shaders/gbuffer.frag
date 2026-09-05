#version 450

// Water look L8 (plan continuum-water/18): the G-buffer suite fragment
// stage. Four attachments: albedo + group mask, encoded normal + roughness,
// screen motion in pixels (previous frame to this one, +x right, +y down,
// jitter removed), linear depth (view-space distance along the camera axis).

layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec3 in_world_normal;
layout(location = 2) in vec4 in_current_clip;
layout(location = 3) in vec4 in_previous_clip;

layout(location = 0) out vec4 out_albedo_mask;
layout(location = 1) out vec4 out_normal_roughness;
layout(location = 2) out vec2 out_motion;
layout(location = 3) out float out_linear_depth;

layout(set = 0, binding = 0, std140) uniform GBufferUniforms {
    mat4 view_projection;
    mat4 previous_view_projection;
    vec4 viewport;
    vec4 jitter;
} frame;

layout(set = 1, binding = 0) uniform sampler2D base_color_texture;

layout(push_constant, std430) uniform DrawPushConstants {
    mat4 model;
    vec4 base_color_factor;
    vec4 material_params;
    vec4 meta;
} draw;

void main() {
    vec4 base_color = texture(base_color_texture, in_uv) * draw.base_color_factor;
    vec3 normal = dot(in_world_normal, in_world_normal) > 0.0001
        ? normalize(in_world_normal)
        : vec3(0.0, 1.0, 0.0);
    if (!gl_FrontFacing) {
        normal = -normal;
    }
    vec2 current_ndc = in_current_clip.xy / in_current_clip.w;
    vec2 previous_ndc = in_previous_clip.xy / in_previous_clip.w;
    vec2 motion = (current_ndc - previous_ndc) * 0.5 * frame.viewport.xy
        - (frame.jitter.xy - frame.jitter.zw);

    out_albedo_mask = vec4(base_color.rgb, draw.meta.y / 255.0);
    out_normal_roughness = vec4(normal * 0.5 + 0.5, draw.meta.z);
    out_motion = motion;
    out_linear_depth = in_current_clip.w;
}
