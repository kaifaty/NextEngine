#version 450

layout(location = 0) in vec3 in_position;

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
    gl_Position = frame.shadow_view_projection * draw.model * vec4(in_position, 1.0);
}
