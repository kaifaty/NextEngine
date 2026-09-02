#version 450

// Separable narrow-range depth smoothing (after Truong and Yuksel 2018):
// samples nearer than the centre by more than the low band belong to another
// surface and are ignored, samples farther by more than the high band are
// clamped to the band, everything else takes a screen-space Gaussian weight
// whose width follows the projected particle radius.

layout(set = 0, binding = 0, std140) uniform FluidFrame {
    mat4 view;
    mat4 projection;
    vec4 viewport;
    vec4 params;
    vec4 absorption;
    vec4 focal;
    vec4 sun;
} frame;

layout(set = 1, binding = 0) uniform sampler2D depth_in;
layout(set = 1, binding = 1) uniform sampler2D unused_thickness;
layout(set = 1, binding = 2) uniform sampler2D unused_scene;

layout(push_constant, std430) uniform FilterPush {
    vec2 direction;
} push;

layout(location = 0) out float out_depth;

void main() {
    vec2 uv = gl_FragCoord.xy * frame.viewport.zw;
    float center = texture(depth_in, uv).r;
    float empty = frame.focal.w;
    if (center >= empty) {
        out_depth = center;
        return;
    }
    float radius = frame.params.x;
    float projected = frame.focal.z * radius / max(center, 1e-4);
    float sigma = clamp(projected * 1.5, 1.0, 24.0);
    int extent = int(min(ceil(2.0 * sigma), 32.0));
    float low = 2.0 * radius;
    float high = 4.0 * radius;
    vec2 step = push.direction * frame.viewport.zw;
    float sum = 0.0;
    float weight_sum = 0.0;
    for (int i = -extent; i <= extent; ++i) {
        float sample_depth = texture(depth_in, uv + step * float(i)).r;
        if (sample_depth >= empty) {
            continue;
        }
        if (sample_depth < center - low) {
            continue;
        }
        sample_depth = min(sample_depth, center + high);
        float weight = exp(-float(i * i) / (2.0 * sigma * sigma));
        sum += weight * sample_depth;
        weight_sum += weight;
    }
    out_depth = weight_sum > 0.0 ? sum / weight_sum : center;
}
