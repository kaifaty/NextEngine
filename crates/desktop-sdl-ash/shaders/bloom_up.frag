#version 450

// Scene look L7 (plan look/07): one bloom upsample step. A 3x3 tent over
// the coarser level added to the downsampled level of this resolution.

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D coarser;
layout(set = 0, binding = 1) uniform sampler2D same_level;

layout(push_constant, std430) uniform BloomPushConstants {
    vec4 params;    // coarser texel width, height, target 1/width, 1/height
} bloom;

void main() {
    vec2 uv = gl_FragCoord.xy * bloom.params.zw;
    vec2 texel = bloom.params.xy;
    vec3 sum = vec3(0.0);
    sum += texture(coarser, uv + texel * vec2(-1.0, -1.0)).rgb;
    sum += texture(coarser, uv + texel * vec2(0.0, -1.0)).rgb * 2.0;
    sum += texture(coarser, uv + texel * vec2(1.0, -1.0)).rgb;
    sum += texture(coarser, uv + texel * vec2(-1.0, 0.0)).rgb * 2.0;
    sum += texture(coarser, uv).rgb * 4.0;
    sum += texture(coarser, uv + texel * vec2(1.0, 0.0)).rgb * 2.0;
    sum += texture(coarser, uv + texel * vec2(-1.0, 1.0)).rgb;
    sum += texture(coarser, uv + texel * vec2(0.0, 1.0)).rgb * 2.0;
    sum += texture(coarser, uv + texel * vec2(1.0, 1.0)).rgb;
    out_color = vec4(sum / 16.0 + texture(same_level, uv).rgb, 1.0);
}
