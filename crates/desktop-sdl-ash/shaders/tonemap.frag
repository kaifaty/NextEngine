#version 450

// Scene look L1 (plan `look/01`): the HDR scene target to the sRGB swapchain.
// Exposure, the ACES fitted curve (Narkowicz), linear output; the swapchain's
// sRGB format encodes on write. Alpha passes through (the fluid pass writes
// its capture-only coverage there).

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D scene_color;

layout(push_constant, std430) uniform TonemapPushConstants {
    vec4 exposure_and_padding;
} tonemap;

vec3 aces_fitted(vec3 x) {
    const float a = 2.51;
    const float b = 0.03;
    const float c = 2.43;
    const float d = 0.59;
    const float e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

void main() {
    vec4 scene = texelFetch(scene_color, ivec2(gl_FragCoord.xy), 0);
    vec3 exposed = max(scene.rgb, vec3(0.0)) * tonemap.exposure_and_padding.x;
    out_color = vec4(aces_fitted(exposed), scene.a);
}
