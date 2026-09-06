#version 450

// Spray capsule: a soft, slightly bluish white droplet streak alpha-blended
// over the composited frame; RGB only, the swapchain alpha coverage channel
// is not written. Depth-tested against the opaque scene, no depth write.

layout(set = 0, binding = 0, std140) uniform FluidFrame {
    mat4 view;
    mat4 projection;
    vec4 viewport;
    vec4 params;
    vec4 absorption;
    vec4 focal;
    vec4 sun;
    vec4 spray;
    vec4 spray2;
    vec4 filter_params;
} frame;

layout(location = 0) in vec2 in_corner;
layout(location = 1) in float in_half_length;

layout(location = 0) out vec4 out_color;

void main() {
    vec2 capsule = vec2(in_corner.x, max(abs(in_corner.y) - in_half_length, 0.0));
    float radial = dot(capsule, capsule);
    if (radial > 1.0) {
        discard;
    }
    float alpha = frame.spray.z * (1.0 - radial);
    out_color = vec4(vec3(0.82, 0.90, 1.0), alpha);
}
