#version 450

layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_color;

void main() {
    float height = clamp(1.0 - in_uv.y, 0.0, 1.0);
    vec3 horizon = vec3(0.48, 0.60, 0.68);
    vec3 zenith = vec3(0.10, 0.20, 0.34);
    vec3 ground_haze = vec3(0.29, 0.36, 0.40);
    vec3 sky = mix(horizon, zenith, smoothstep(0.18, 1.0, height));
    sky = mix(ground_haze, sky, smoothstep(0.0, 0.16, height));
    out_color = vec4(sky, 1.0);
}
