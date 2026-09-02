#version 450

// Shades the smoothed particle surface: view-space normal from the depth
// gradient, Fresnel between the refracted opaque scene (attenuated by the
// accumulated thickness) and the sky gradient, plus a sun highlight. Pixels
// without fluid are discarded so the opaque frame stays untouched. Fluid
// pixels write alpha 0: the swapchain is presented with opaque composite
// alpha, so the channel is only a coverage diagnostic for frame captures.

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
} frame;

layout(set = 1, binding = 0) uniform sampler2D depth_in;
layout(set = 1, binding = 1) uniform sampler2D thickness_in;
layout(set = 1, binding = 2) uniform sampler2D scene_in;

layout(location = 0) out vec4 out_color;

vec3 view_position(vec2 uv, float depth) {
    vec2 ndc = uv * 2.0 - 1.0;
    return vec3(ndc.x * depth * frame.focal.x, -ndc.y * depth * frame.focal.y, -depth);
}

void main() {
    vec2 uv = gl_FragCoord.xy * frame.viewport.zw;
    float empty = frame.focal.w;
    float depth = texture(depth_in, uv).r;
    if (depth >= empty) {
        discard;
    }
    vec2 texel = frame.viewport.zw;
    vec3 position = view_position(uv, depth);

    float depth_right = texture(depth_in, uv + vec2(texel.x, 0.0)).r;
    float depth_left = texture(depth_in, uv - vec2(texel.x, 0.0)).r;
    vec3 ddx = view_position(uv + vec2(texel.x, 0.0), depth_right) - position;
    vec3 ddx2 = position - view_position(uv - vec2(texel.x, 0.0), depth_left);
    if (depth_right >= empty || abs(ddx2.z) < abs(ddx.z)) {
        ddx = ddx2;
    }
    float depth_down = texture(depth_in, uv + vec2(0.0, texel.y)).r;
    float depth_up = texture(depth_in, uv - vec2(0.0, texel.y)).r;
    vec3 ddy = view_position(uv + vec2(0.0, texel.y), depth_down) - position;
    vec3 ddy2 = position - view_position(uv - vec2(0.0, texel.y), depth_up);
    if (depth_down >= empty || abs(ddy2.z) < abs(ddy.z)) {
        ddy = ddy2;
    }
    vec3 normal = normalize(cross(ddx, ddy));
    if (normal.z < 0.0) {
        normal = -normal;
    }

    vec3 to_camera = normalize(-position);
    float facing = max(dot(normal, to_camera), 0.0);
    float fresnel = 0.02 + 0.98 * pow(1.0 - facing, 5.0);

    float thickness = texture(thickness_in, uv).r;
    vec2 refraction = normal.xy * frame.absorption.w * clamp(thickness, 0.0, 1.0);
    vec2 scene_uv = clamp(uv + refraction, vec2(0.0), vec2(1.0));
    vec3 scene = texture(scene_in, scene_uv).rgb;
    vec3 attenuation = exp(-thickness * frame.absorption.rgb);
    vec3 refracted = scene * attenuation;

    vec3 world_up = normalize((frame.view * vec4(0.0, 1.0, 0.0, 0.0)).xyz);
    vec3 reflected = reflect(-to_camera, normal);
    float elevation = clamp(dot(reflected, world_up) * 0.5 + 0.5, 0.0, 1.0);
    vec3 horizon = vec3(0.48, 0.60, 0.68);
    vec3 zenith = vec3(0.10, 0.20, 0.34);
    vec3 sky = mix(horizon, zenith, smoothstep(0.35, 1.0, elevation));

    vec3 light = normalize(-frame.sun.xyz);
    vec3 half_vector = normalize(light + to_camera);
    float specular = pow(max(dot(normal, half_vector), 0.0), 96.0) * frame.sun.w;

    vec3 color = mix(refracted, sky, fresnel) + vec3(specular) * 0.6;
    out_color = vec4(color, 0.0);
}
