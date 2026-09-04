#version 450

// Plan continuum-water/35: the wet band above every water ring's level. A
// fullscreen pass inside the water pass (after the scene copy, before the
// rings, never with a submerged eye): every scene pixel inside a ring's
// widened plan and within the band over that ring's level is darkened and
// given a sun gloss from the depth-reconstructed normal. Renderer-local
// constants only; the sets are the water pass's own.

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
layout(set = 3, binding = 4, std140) uniform WaterRings {
    vec4 plans[8];      // min x, min z, max x, max z (metres)
    vec4 levels[8];     // level (metres), unused
    vec4 count;         // ring count, unused
} rings;

const float WET_BAND_METRES = 0.15;
const float WET_PLAN_MARGIN_METRES = 0.25;
const float WET_BELOW_METRES = 0.02;
const float WET_DARKEN = 0.55;
const float WET_GLOSS = 0.35;
const float WET_GLOSS_EXPONENT = 48.0;

vec3 scene_world_position(vec2 uv, float depth) {
    vec4 clip = vec4(uv * 2.0 - 1.0, depth, 1.0);
    vec4 world = water.inverse_view_projection * clip;
    return world.xyz / world.w;
}

void main() {
    vec2 uv = gl_FragCoord.xy * water.viewport.zw;
    float depth = texture(scene_depth, uv).r;
    if (depth >= 1.0) {
        discard;
    }
    vec3 point = scene_world_position(uv, depth);
    float wet = 0.0;
    int count = int(rings.count.x);
    for (int index = 0; index < 8; ++index) {
        if (index >= count) {
            break;
        }
        vec4 plan = rings.plans[index];
        if (point.x < plan.x - WET_PLAN_MARGIN_METRES || point.x > plan.z + WET_PLAN_MARGIN_METRES
            || point.z < plan.y - WET_PLAN_MARGIN_METRES || point.z > plan.w + WET_PLAN_MARGIN_METRES) {
            continue;
        }
        float height = point.y - rings.levels[index].x;
        if (height < -WET_BELOW_METRES) {
            continue;
        }
        wet = max(wet, 1.0 - smoothstep(0.0, WET_BAND_METRES, max(height, 0.0)));
    }
    if (wet <= 0.0) {
        discard;
    }
    vec2 texel = water.viewport.zw;
    vec3 right = scene_world_position(uv + vec2(texel.x, 0.0), texture(scene_depth, uv + vec2(texel.x, 0.0)).r);
    vec3 down = scene_world_position(uv + vec2(0.0, texel.y), texture(scene_depth, uv + vec2(0.0, texel.y)).r);
    vec3 normal = normalize(cross(right - point, down - point));
    vec3 view = normalize(frame.camera_world_position.xyz - point);
    if (dot(normal, view) < 0.0) {
        normal = -normal;
    }
    vec3 light = normalize(-frame.sun_direction_intensity.xyz);
    vec3 half_vector = normalize(light + view);
    float gloss = pow(max(dot(normal, half_vector), 0.0), WET_GLOSS_EXPONENT)
        * frame.sun_direction_intensity.w * WET_GLOSS * wet;
    vec3 scene = texture(scene_color, uv).rgb;
    out_color = vec4(scene * mix(1.0, WET_DARKEN, wet) + vec3(gloss), 1.0);
}
