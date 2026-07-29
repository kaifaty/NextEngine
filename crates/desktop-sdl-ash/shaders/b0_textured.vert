#version 450

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;

layout(location = 0) out vec2 out_uv;

layout(set = 0, binding = 0, std140) uniform FrameUniforms {
    mat4 view_projection;
} frame;

layout(push_constant, std430) uniform DrawPushConstants {
    mat4 model;
    vec4 base_color_factor;
} draw;

void main() {
    gl_Position = frame.view_projection * draw.model * vec4(in_position, 1.0);
    out_uv = in_uv;
}
