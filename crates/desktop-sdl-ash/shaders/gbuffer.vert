#version 450

// Water look L8 (plan continuum-water/18): the G-buffer suite vertex stage.
// The B0 vertex layout drawn again with the frame's jittered view-projection
// and, per draw, the previous rendered frame's model matrix (storage entry
// `meta.x`) under the previous jittered view-projection, so the fragment
// stage can emit screen motion.

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in vec4 in_normal_snorm;

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec3 out_world_normal;
layout(location = 2) out vec4 out_current_clip;
layout(location = 3) out vec4 out_previous_clip;

layout(set = 0, binding = 0, std140) uniform GBufferUniforms {
    mat4 view_projection;
    mat4 previous_view_projection;
    vec4 viewport;  // width, height, 1/width, 1/height
    vec4 jitter;    // current xy, previous xy (pixels)
} frame;

layout(set = 0, binding = 1, std430) readonly buffer PreviousModels {
    mat4 models[];
} history;

layout(push_constant, std430) uniform DrawPushConstants {
    mat4 model;
    vec4 base_color_factor;
    vec4 material_params;  // metallic, roughness, emissive intensity, 0
    vec4 meta;  // draw index, group, roughness, 0
} draw;

void main() {
    vec4 world_position = draw.model * vec4(in_position, 1.0);
    gl_Position = frame.view_projection * world_position;
    out_current_clip = gl_Position;
    uint index = uint(draw.meta.x + 0.5);
    vec4 previous_world = history.models[index] * vec4(in_position, 1.0);
    out_previous_clip = frame.previous_view_projection * previous_world;
    out_uv = in_uv;
    if (dot(in_normal_snorm.xyz, in_normal_snorm.xyz) > 0.0001) {
        out_world_normal = normalize(transpose(inverse(mat3(draw.model))) * in_normal_snorm.xyz);
    } else {
        out_world_normal = vec3(0.0);
    }
}
