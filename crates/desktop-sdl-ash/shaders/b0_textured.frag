#version 450

layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform sampler2D base_color_texture;

layout(push_constant, std430) uniform DrawPushConstants {
    mat4 model;
    vec4 base_color_factor;
} draw;

void main() {
    out_color = texture(base_color_texture, in_uv) * draw.base_color_factor;
}
