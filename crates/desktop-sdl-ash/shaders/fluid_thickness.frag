#version 450

// Separable Gaussian smoothing of the accumulated thickness (NGQ10
// revision 2): the width follows the projected particle radius at the
// smoothed depth, samples outside the fluid silhouette are ignored, so the
// per-sphere lattice no longer shows through the refracted interior.

layout(set = 0, binding = 0, std140) uniform FluidFrame {
    mat4 view;
    mat4 projection;
    vec4 viewport;
    vec4 params;
    vec4 absorption;
    vec4 focal;
    vec4 sun;
    vec4 spray;
} frame;

layout(set = 1, binding = 0) uniform sampler2D depth_in;
layout(set = 1, binding = 1) uniform sampler2D thickness_in;
layout(set = 1, binding = 2) uniform sampler2D unused_scene;

layout(push_constant, std430) uniform FilterPush {
    vec2 direction;
} push;

layout(location = 0) out float out_thickness;

void main() {
    vec2 uv = gl_FragCoord.xy * frame.viewport.zw;
    float center_depth = texture(depth_in, uv).r;
    float empty = frame.focal.w;
    if (center_depth >= empty) {
        out_thickness = texture(thickness_in, uv).r;
        return;
    }
    float radius = frame.params.x;
    float projected = frame.focal.z * radius / max(center_depth, 1e-4);
    float sigma = clamp(projected * 1.5, 1.0, 24.0);
    int extent = int(min(ceil(2.0 * sigma), 32.0));
    vec2 step = push.direction * frame.viewport.zw;
    float sum = 0.0;
    float weight_sum = 0.0;
    for (int i = -extent; i <= extent; ++i) {
        vec2 sample_uv = uv + step * float(i);
        if (texture(depth_in, sample_uv).r >= empty) {
            continue;
        }
        float weight = exp(-float(i * i) / (2.0 * sigma * sigma));
        sum += weight * texture(thickness_in, sample_uv).r;
        weight_sum += weight;
    }
    out_thickness = weight_sum > 0.0 ? sum / weight_sum : texture(thickness_in, uv).r;
}
