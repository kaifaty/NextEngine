#version 450

// Scene look L4 (plan look/04): the temporal resolve. The history reprojected
// by the G-buffer's motion (the sky through the previous view-projection),
// clipped to the current 3x3 neighbourhood's variance box, blended with the
// current frame.

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

layout(set = 1, binding = 0) uniform sampler2D scene_color;
layout(set = 1, binding = 1) uniform sampler2D history_color;
layout(set = 1, binding = 2) uniform sampler2D motion_vectors;
layout(set = 1, binding = 3) uniform sampler2D linear_depth;

layout(push_constant, std430) uniform TemporalPushConstants {
    mat4 previous_view_projection;
    vec4 params;    // jitter delta xy (pixels), current weight, first frame
    vec4 viewport;  // width, height, 1/width, 1/height
} temporal;

const float CLIP_GAMMA = 1.0;

void main() {
    vec2 texel = temporal.viewport.zw;
    vec2 uv = gl_FragCoord.xy * texel;
    vec3 current = texture(scene_color, uv).rgb;
    float first_frame = temporal.params.w;
    float depth = texture(linear_depth, uv).r;
    vec2 motion;
    if (depth > 0.0) {
        motion = texture(motion_vectors, uv).xy;
    } else {
        // The sky: a far point through the previous view-projection.
        vec4 far = lighting.inverse_view_projection * vec4(uv * 2.0 - 1.0, 1.0, 1.0);
        vec3 direction = normalize(far.xyz / far.w - frame.camera_world_position.xyz);
        vec4 previous_clip = temporal.previous_view_projection
            * vec4(frame.camera_world_position.xyz + direction * 1000.0, 1.0);
        vec2 previous_ndc = previous_clip.xy / max(previous_clip.w, 1e-4);
        motion = ((uv * 2.0 - 1.0) - previous_ndc) * 0.5 * temporal.viewport.xy - temporal.params.xy;
    }
    vec2 history_uv = uv - motion * texel;
    bool offscreen = any(lessThan(history_uv, vec2(0.0))) || any(greaterThan(history_uv, vec2(1.0)));
    if (first_frame > 0.5 || offscreen) {
        out_color = vec4(current, 1.0);
        return;
    }
    // The neighbourhood's mean and deviation per channel.
    vec3 sum = vec3(0.0);
    vec3 sum_squares = vec3(0.0);
    for (int y = -1; y <= 1; ++y) {
        for (int x = -1; x <= 1; ++x) {
            vec3 sample_color = texture(scene_color, uv + vec2(x, y) * texel).rgb;
            sum += sample_color;
            sum_squares += sample_color * sample_color;
        }
    }
    vec3 mean = sum / 9.0;
    vec3 deviation = sqrt(max(sum_squares / 9.0 - mean * mean, vec3(0.0)));
    vec3 history = texture(history_color, history_uv).rgb;
    history = clamp(history, mean - CLIP_GAMMA * deviation, mean + CLIP_GAMMA * deviation);
    float weight = temporal.params.z;
    out_color = vec4(mix(history, current, weight), 1.0);
}
