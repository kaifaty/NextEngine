#version 450

// Scene look L7 (plan look/07): the post chain's composite. The resolved HDR
// scene under the half-resolution volume (fog.frag, revision 3: in-scattered
// radiance and transmittance), the bloom chain, exposure, the ACES fitted
// curve and a colour-grading LUT, to the sRGB swapchain.

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
    mat4 shadow_cascades[3];    // plan look/02: the cascade view-projections
    vec4 shadow_extents;        // cascade extents in metres, spare
} lighting;

layout(set = 1, binding = 0) uniform sampler2D scene_color;
layout(set = 1, binding = 1) uniform sampler2D bloom;
layout(set = 1, binding = 2) uniform sampler2D linear_depth;
// The 32^3 grading LUT stored as a strip of 32 slices along u.
layout(set = 1, binding = 3) uniform sampler2D grading_lut;
// Revision 3: the volume at half resolution (rgb in-scatter, a transmittance).
layout(set = 1, binding = 4) uniform sampler2D fog_volume;

layout(push_constant, std430) uniform PostPushConstants {
    vec4 params;    // exposure, fog density at ground level, bloom mix, lut size
    vec4 fog;       // height scale (m), march distance (m), far distance (m), HG g
} post;

vec3 aces_fitted(vec3 x) {
    const float a = 2.51;
    const float b = 0.03;
    const float c = 2.43;
    const float d = 0.59;
    const float e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

vec3 grade(vec3 c) {
    float size = post.params.w;
    float slices = size - 1.0;
    c = clamp(c, 0.0, 1.0);
    float slice = c.b * slices;
    float s0 = floor(slice);
    float s1 = min(s0 + 1.0, slices);
    float t = slice - s0;
    float v = (c.g * slices + 0.5) / size;
    float u0 = (s0 * size + c.r * slices + 0.5) / (size * size);
    float u1 = (s1 * size + c.r * slices + 0.5) / (size * size);
    return mix(texture(grading_lut, vec2(u0, v)).rgb, texture(grading_lut, vec2(u1, v)).rgb, t);
}

void main() {
    vec2 size = vec2(textureSize(scene_color, 0));
    vec2 uv = gl_FragCoord.xy / size;
    vec4 scene = texelFetch(scene_color, ivec2(gl_FragCoord.xy), 0);
    vec3 color = max(scene.rgb, vec3(0.0));

    // Revision 3: the volume from the half-resolution target.
    vec4 volume = texture(fog_volume, uv);
    color = color * volume.a + max(volume.rgb, vec3(0.0));

    // Bloom, exposure, the curve and the grade.
    vec3 glow = texture(bloom, uv).rgb;
    color = mix(color, glow, post.params.z);
    vec3 exposed = color * post.params.x;
    out_color = vec4(grade(aces_fitted(exposed)), scene.a);
}
