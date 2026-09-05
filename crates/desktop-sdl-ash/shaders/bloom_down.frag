#version 450

// Scene look L7 (plan look/07): one bloom downsample step. The 13-tap
// filter of Jimenez (2014) over the finer level; the first step applies the
// Karis average per 4-tap group to keep fireflies out of the chain.

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D source;

layout(push_constant, std430) uniform BloomPushConstants {
    vec4 params;    // source texel width, height, karis flag, spare
} bloom;

float luma(vec3 c) {
    return dot(c, vec3(0.2126, 0.7152, 0.0722));
}

vec3 karis(vec3 a, vec3 b, vec3 c, vec3 d, bool enabled) {
    if (!enabled) {
        return (a + b + c + d) * 0.25;
    }
    float wa = 1.0 / (1.0 + luma(a));
    float wb = 1.0 / (1.0 + luma(b));
    float wc = 1.0 / (1.0 + luma(c));
    float wd = 1.0 / (1.0 + luma(d));
    return (a * wa + b * wb + c * wc + d * wd) / (wa + wb + wc + wd);
}

void main() {
    vec2 texel = bloom.params.xy;
    vec2 uv = gl_FragCoord.xy * 2.0 * texel;
    bool first = bloom.params.z > 0.5;
    vec3 a = texture(source, uv + texel * vec2(-2.0, -2.0)).rgb;
    vec3 b = texture(source, uv + texel * vec2(0.0, -2.0)).rgb;
    vec3 c = texture(source, uv + texel * vec2(2.0, -2.0)).rgb;
    vec3 d = texture(source, uv + texel * vec2(-2.0, 0.0)).rgb;
    vec3 e = texture(source, uv).rgb;
    vec3 f = texture(source, uv + texel * vec2(2.0, 0.0)).rgb;
    vec3 g = texture(source, uv + texel * vec2(-2.0, 2.0)).rgb;
    vec3 h = texture(source, uv + texel * vec2(0.0, 2.0)).rgb;
    vec3 i = texture(source, uv + texel * vec2(2.0, 2.0)).rgb;
    vec3 j = texture(source, uv + texel * vec2(-1.0, -1.0)).rgb;
    vec3 k = texture(source, uv + texel * vec2(1.0, -1.0)).rgb;
    vec3 l = texture(source, uv + texel * vec2(-1.0, 1.0)).rgb;
    vec3 m = texture(source, uv + texel * vec2(1.0, 1.0)).rgb;
    // Five overlapping 4-tap groups: the inner one weighs 0.5, the four
    // outer ones 0.125 each.
    vec3 color = karis(j, k, l, m, first) * 0.5;
    color += karis(a, b, d, e, first) * 0.125;
    color += karis(b, c, e, f, first) * 0.125;
    color += karis(d, e, g, h, first) * 0.125;
    color += karis(e, f, h, i, first) * 0.125;
    out_color = vec4(max(color, vec3(0.0)), 1.0);
}
