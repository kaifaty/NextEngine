#version 450

layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform sampler2D overlay_texture;

void main() {
    out_color = texture(overlay_texture, in_uv);
}
