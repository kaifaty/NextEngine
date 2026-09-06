#version 450

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in vec4 in_normal_snorm;

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec3 out_world_position;
layout(location = 2) out vec3 out_world_normal;

layout(set = 0, binding = 0, std140) uniform FrameUniforms {
    mat4 view_projection;
    mat4 shadow_view_projection;
    vec4 camera_world_position;
    vec4 sun_direction_intensity;
    vec4 hemisphere_sky_color;
    vec4 hemisphere_ground_color;
    vec4 fog_color_density;
} frame;

layout(push_constant, std430) uniform DrawPushConstants {
    mat4 model;
    vec4 base_color_factor;
} draw;

void main() {
    vec4 world_position = draw.model * vec4(in_position, 1.0);
    gl_Position = frame.view_projection * world_position;
    out_uv = in_uv;
    out_world_position = world_position.xyz;
    if (dot(in_normal_snorm.xyz, in_normal_snorm.xyz) > 0.0001) {
        out_world_normal = normalize(transpose(inverse(mat3(draw.model))) * in_normal_snorm.xyz);
    } else {
        out_world_normal = vec3(0.0);
    }
}
