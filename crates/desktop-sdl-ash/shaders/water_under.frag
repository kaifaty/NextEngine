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

// Scene look L1 (plan look/01): the lighting block. The sun in irradiance
// units (normal to the sun), the sky's SH2 radiance, the fog's radiance at
// the horizon, and the Preetham sky's coefficients for the reflected sky.
layout(set = 0, binding = 1, std140) uniform LightingUniforms {
    mat4 inverse_view_projection;
    vec4 sun_radiance;          // rgb irradiance normal to the sun, w exposure
    vec4 sky_sh[9];             // SH2 radiance coefficients (rgb)
    vec4 fog;                   // rgb radiance at the horizon, w density per metre
    vec4 sky_zenith;            // zenith x, y, Y (raw), sun disc cos inner
    vec4 sky_perez_x[2];
    vec4 sky_perez_y[2];
    vec4 sky_perez_luminance[2];
    vec4 sky_params;            // ground albedo, turbidity, luminance scale, sun disc cos outer
} lighting;

const float LIGHTING_PI = 3.14159265;

// Irradiance at a normal from the SH2 radiance (Ramamoorthi and Hanrahan).
vec3 sh_irradiance(vec3 n) {
    const float c1 = 0.429043;
    const float c2 = 0.511664;
    const float c3 = 0.743125;
    const float c4 = 0.886227;
    const float c5 = 0.247708;
    vec3 l00 = lighting.sky_sh[0].rgb;
    vec3 l1m1 = lighting.sky_sh[1].rgb;
    vec3 l10 = lighting.sky_sh[2].rgb;
    vec3 l11 = lighting.sky_sh[3].rgb;
    vec3 l2m2 = lighting.sky_sh[4].rgb;
    vec3 l2m1 = lighting.sky_sh[5].rgb;
    vec3 l20 = lighting.sky_sh[6].rgb;
    vec3 l21 = lighting.sky_sh[7].rgb;
    vec3 l22 = lighting.sky_sh[8].rgb;
    float x = n.x;
    float y = n.y;
    float z = n.z;
    vec3 e = c1 * l22 * (x * x - y * y) + c3 * l20 * z * z + c4 * l00 - c5 * l20
        + 2.0 * c1 * (l2m2 * x * y + l21 * x * z + l2m1 * y * z)
        + 2.0 * c2 * (l11 * x + l1m1 * y + l10 * z);
    return max(e, vec3(0.0));
}

float sky_perez(vec4 a, vec4 b, float cos_theta, float gamma) {
    return (1.0 + a.x * exp(a.y / max(cos_theta, 0.01)))
        * (1.0 + a.z * exp(a.w * gamma) + b.x * cos(gamma) * cos(gamma));
}

// The Preetham sky's radiance along a direction (linear sRGB, the same
// units as the lighting block); below the horizon the horizon's value.
vec3 sky_radiance(vec3 direction) {
    vec3 sun = normalize(-frame.sun_direction_intensity.xyz);
    float cos_theta = max(direction.y, 0.01);
    float gamma = acos(clamp(dot(direction, sun), -1.0, 1.0));
    float theta_s = acos(clamp(sun.y, -1.0, 1.0));
    float x = lighting.sky_zenith.x
        * sky_perez(lighting.sky_perez_x[0], lighting.sky_perez_x[1], cos_theta, gamma)
        / sky_perez(lighting.sky_perez_x[0], lighting.sky_perez_x[1], 1.0, theta_s);
    float y = lighting.sky_zenith.y
        * sky_perez(lighting.sky_perez_y[0], lighting.sky_perez_y[1], cos_theta, gamma)
        / sky_perez(lighting.sky_perez_y[0], lighting.sky_perez_y[1], 1.0, theta_s);
    float big_y = lighting.sky_zenith.z
        * sky_perez(lighting.sky_perez_luminance[0], lighting.sky_perez_luminance[1], cos_theta, gamma)
        / sky_perez(lighting.sky_perez_luminance[0], lighting.sky_perez_luminance[1], 1.0, theta_s);
    big_y = max(big_y, 0.0);
    y = max(y, 1e-4);
    float big_x = x * big_y / y;
    float big_z = (1.0 - x - y) * big_y / y;
    vec3 rgb = vec3(
        3.2406 * big_x - 1.5372 * big_y - 0.4986 * big_z,
        -0.9689 * big_x + 1.8758 * big_y + 0.0415 * big_z,
        0.0557 * big_x - 0.2040 * big_y + 1.0570 * big_z
    );
    return max(rgb, vec3(0.0)) * lighting.sky_params.z;
}

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
    // Scene look L1: the inscatter constant is an LDR colour; the exposure
    // lane of the lighting block brings it into the scene's units.
    out_color = vec4(
        mix(WATER_UNDER_INSCATTER / lighting.sun_radiance.w, scene, transmittance),
        1.0
    );
}
