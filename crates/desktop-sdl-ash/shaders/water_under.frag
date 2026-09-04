#version 450

// Plan continuum-water/33: the water between a submerged eye and everything
// else. A fullscreen pass inside the water pass, after the scene copy and
// before the rings: every pixel's path through the water (from the eye to
// the scene point behind the pixel, clipped at the level plane when that
// point lies above the level) attenuates the scene by plan 13's absorption
// and adds the in-scatter colour. Renderer-local constants only; the sets
// are the water pass's own (set 0 the B0 frame block, set 3 the water set).

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0, std140) uniform FrameUniforms {
    mat4 view_projection;
    mat4 shadow_view_projection;
    vec4 camera_world_position;
    vec4 sun_direction_intensity;
    vec4 hemisphere_sky_color;
    vec4 hemisphere_ground_color;
    vec4 fog_color_density;
} frame;

layout(set = 3, binding = 0) uniform sampler2D scene_color;
layout(set = 3, binding = 1) uniform sampler2D scene_depth;
layout(set = 3, binding = 2, std140) uniform WaterUniforms {
    mat4 inverse_view_projection;
    vec4 viewport;      // width, height, 1/width, 1/height
    vec4 absorption;    // per-metre rgb, refraction strength
    vec4 shore;         // foam width (m), fade width (m), foam grey, run seconds
    vec4 under;         // level (m), submerged (0/1), unused, unused
} water;

const vec3 WATER_UNDER_INSCATTER = vec3(0.05, 0.18, 0.28);
// Revision 2: the eye inside the water sees metres of it; plan 13's
// absorption (a look down into shallow water) loses red within one metre,
// so the underwater path uses its own clear-pool constant.
const vec3 WATER_UNDER_ABSORPTION_PER_METRE = vec3(0.45, 0.12, 0.06);

vec3 scene_world_position(vec2 uv, float depth) {
    vec4 clip = vec4(uv * 2.0 - 1.0, depth, 1.0);
    vec4 world = water.inverse_view_projection * clip;
    return world.xyz / world.w;
}

void main() {
    vec2 uv = gl_FragCoord.xy * water.viewport.zw;
    float depth = texture(scene_depth, uv).r;
    vec3 eye = frame.camera_world_position.xyz;
    vec3 behind = scene_world_position(uv, depth);
    float level = water.under.x;
    // The path stops at the level plane when the scene point lies above it
    // (the sky, the walls' tops): the eye is below the level here.
    if (behind.y > level && behind.y > eye.y) {
        float t = clamp((level - eye.y) / (behind.y - eye.y), 0.0, 1.0);
        behind = mix(eye, behind, t);
    }
    float path = distance(eye, behind);
    vec3 transmittance = exp(-WATER_UNDER_ABSORPTION_PER_METRE * path);
    vec3 scene = texture(scene_color, uv).rgb;
    out_color = vec4(mix(WATER_UNDER_INSCATTER, scene, transmittance), 1.0);
}
