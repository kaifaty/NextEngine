#version 450

// Scene look L7 (plan look/07, revision 3): the volume at half resolution.
// An exponential height fog marched through the cascaded shadow map (one
// dithered compare tap per step) with the sky model's radiance along the
// ray partitioned into a shadowed sun share and an ambient share; writes
// the in-scattered radiance (rgb) and the transmittance (a).

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

layout(set = 1, binding = 0) uniform sampler2D linear_depth;
layout(set = 2, binding = 0) uniform sampler2DArrayShadow shadow_map;

layout(push_constant, std430) uniform FogPushConstants {
    vec4 params;    // ground density, 1 / target width, 1 / target height, spare
    vec4 fog;       // height scale (m), march distance (m), far distance (m), HG g
} post;

const int FOG_STEPS = 12;
const float POST_PI = 3.14159265;
const float SHADOW_MAP_TEXELS = 2048.0;

// Scene look L2 (plan look/02): the sun's visibility from the cascaded
// shadow map at a point of the volume: one compare tap (the dithered
// march integrates), no receiver normal.
float sun_visibility(vec3 world_position) {
    for (int cascade = 0; cascade < 3; ++cascade) {
        float extent = lighting.shadow_extents[cascade];
        vec4 clip = lighting.shadow_cascades[cascade] * vec4(world_position, 1.0);
        vec3 coord = clip.xyz / clip.w;
        coord.xy = coord.xy * 0.5 + 0.5;
        if (any(lessThan(coord.xy, vec2(0.02))) || any(greaterThan(coord.xy, vec2(0.98)))
            || coord.z < 0.0 || coord.z > 1.0) {
            continue;
        }
        float range_scale = 44.0 / (1.375 * extent);
        float bias = 0.00025 * range_scale;
        return texture(shadow_map, vec4(coord.xy, float(cascade), coord.z - bias));
    }
    return 1.0;
}

vec3 view_ray(vec2 uv) {
    vec4 far = lighting.inverse_view_projection * vec4(uv * 2.0 - 1.0, 1.0, 1.0);
    return normalize(far.xyz / far.w - frame.camera_world_position.xyz);
}

float henyey_greenstein(float cos_theta, float g) {
    float denominator = 1.0 + g * g - 2.0 * g * cos_theta;
    return (1.0 - g * g) / (4.0 * POST_PI * pow(max(denominator, 1e-4), 1.5));
}

// The optical depth of the exponential height fog along a ray segment
// `[a, b]` (closed form; `dy` is the ray's vertical component).
float height_fog_depth(float sigma_ground, float height_scale, float y0, float dy, float a, float b) {
    float base = sigma_ground * exp(-max(y0, 0.0) / height_scale);
    if (abs(dy) < 1e-4) {
        return base * (b - a);
    }
    float k = dy / height_scale;
    return base * (exp(-k * a) - exp(-k * b)) / k;
}

float sky_perez(vec4 a, vec4 b, float cos_theta, float gamma) {
    return (1.0 + a.x * exp(a.y / max(cos_theta, 0.01)))
        * (1.0 + a.z * exp(a.w * gamma) + b.x * cos(gamma) * cos(gamma));
}

// The Preetham sky's radiance along a direction (as sky_analytic.frag);
// revision 2: the in-scattered radiance of the volume along a view ray
// (aerial perspective), rays below two degrees read the horizon.
vec3 sky_radiance(vec3 direction) {
    vec3 sun = normalize(-frame.sun_direction_intensity.xyz);
    float cos_theta = max(direction.y, 0.035);
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

float luma(vec3 c) {
    return dot(c, vec3(0.2126, 0.7152, 0.0722));
}

void main() {
    vec2 uv = gl_FragCoord.xy * post.params.yz;
    vec3 camera = frame.camera_world_position.xyz;
    vec3 ray = view_ray(uv);
    vec3 forward = view_ray(vec2(0.5));
    float z = texture(linear_depth, uv).r;
    float far_distance = post.fog.z;
    float distance = (z <= 0.0 || z > 5000.0) ? far_distance : z / max(dot(ray, forward), 1e-3);
    distance = min(distance, far_distance);

    float sigma_ground = post.params.x;
    float height_scale = post.fog.x;
    float march = min(distance, post.fog.y);
    float step_length = march / float(FOG_STEPS);
    float dither = fract(52.9829189 * fract(dot(gl_FragCoord.xy, vec2(0.06711056, 0.00583715))));
    vec3 to_sun = normalize(-frame.sun_direction_intensity.xyz);
    float phase = henyey_greenstein(dot(ray, to_sun), post.fog.w);
    float sun_term = luma(lighting.sun_radiance.rgb) * phase;
    float sun_share = sun_term / max(sun_term + luma(lighting.fog.rgb), 1e-4);
    vec3 in_scatter_radiance = sky_radiance(ray);
    vec3 scatter = vec3(0.0);
    float transmittance = 1.0;
    for (int i = 0; i < FOG_STEPS; ++i) {
        float t = (float(i) + dither) * step_length;
        vec3 p = camera + ray * t;
        float sigma = sigma_ground * exp(-max(p.y, 0.0) / height_scale);
        float attenuation = exp(-sigma * step_length);
        float lit = mix(1.0 - sun_share, 1.0, sun_visibility(p));
        scatter += transmittance * (1.0 - attenuation) * in_scatter_radiance * lit;
        transmittance *= attenuation;
    }
    if (distance > march) {
        float depth = height_fog_depth(sigma_ground, height_scale, camera.y, ray.y, march, distance);
        float remainder = exp(-depth);
        scatter += transmittance * (1.0 - remainder) * in_scatter_radiance;
        transmittance *= remainder;
    }
    out_color = vec4(scatter, transmittance);
}
