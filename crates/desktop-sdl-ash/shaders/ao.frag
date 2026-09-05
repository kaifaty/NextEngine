#version 450

// Scene look L3 (plan look/03): ground-truth ambient occlusion over the
// G-buffer prepass. World positions from the linear depth and the lighting
// block's inverse view-projection, three horizon slices with five steps per
// side inside a one-metre radius, the GTAO slice integral.

layout(location = 0) out float out_occlusion;

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

layout(set = 1, binding = 0) uniform sampler2D linear_depth;
layout(set = 1, binding = 1) uniform sampler2D normal_roughness;
layout(set = 1, binding = 2) uniform sampler2D occlusion_input;

layout(push_constant, std430) uniform OcclusionPushConstants {
    vec4 params;    // blur direction (texels), radius (m), spare
    vec4 target;    // target width, height, 1/width, 1/height
} occlusion;

const int SLICES = 3;
const int STEPS = 5;
const float AO_PI = 3.14159265;

vec3 view_ray(vec2 uv) {
    vec4 far = lighting.inverse_view_projection * vec4(uv * 2.0 - 1.0, 1.0, 1.0);
    return normalize(far.xyz / far.w - frame.camera_world_position.xyz);
}

vec3 position_at(vec2 uv, vec3 forward) {
    float z = texture(linear_depth, uv).r;
    vec3 ray = view_ray(uv);
    return frame.camera_world_position.xyz + ray * (z / max(dot(ray, forward), 1e-3));
}

float slice_visibility(float n, float h1, float h2) {
    h1 = n + max(h1 - n, -0.5 * AO_PI);
    h2 = n + min(h2 - n, 0.5 * AO_PI);
    float a1 = -cos(2.0 * h1 - n) + cos(n) + 2.0 * h1 * sin(n);
    float a2 = -cos(2.0 * h2 - n) + cos(n) + 2.0 * h2 * sin(n);
    // Unbounded per slice: the projected-normal weight over the slices
    // brings an open hemisphere to one only in the average.
    return 0.25 * (a1 + a2);
}

void main() {
    // Revision 1: the target is half the scene; steps stay in scene texels.
    vec2 texel = 1.0 / vec2(textureSize(linear_depth, 0));
    vec2 uv = gl_FragCoord.xy * occlusion.target.zw;
    float z = texture(linear_depth, uv).r;
    if (z <= 0.0 || z > 5000.0) {
        out_occlusion = 1.0;
        return;
    }
    vec3 camera = frame.camera_world_position.xyz;
    vec3 forward = view_ray(vec2(0.5));
    vec3 position = position_at(uv, forward);
    vec3 normal = normalize(texture(normal_roughness, uv).xyz * 2.0 - 1.0);
    vec3 view = normalize(camera - position);
    float radius = occlusion.params.z;
    float pixel_angle = max(length(view_ray(uv + vec2(texel.x, 0.0)) - view_ray(uv)), 1e-5);
    float radius_pixels = clamp(radius / (z * pixel_angle), 2.0, 64.0);
    float noise = fract(52.9829189 * fract(dot(gl_FragCoord.xy, vec2(0.06711056, 0.00583715))));
    float step_jitter = fract(noise * 7.0);
    float visibility = 0.0;
    for (int slice = 0; slice < SLICES; ++slice) {
        float phi = (float(slice) + noise) * AO_PI / float(SLICES);
        vec2 direction = vec2(cos(phi), sin(phi));
        vec3 tangent = view_ray(uv + direction * texel) - view_ray(uv);
        vec3 axis = tangent - view * dot(tangent, view);
        if (dot(axis, axis) < 1e-12) {
            visibility += 1.0;
            continue;
        }
        axis = normalize(axis);
        vec3 plane_normal = normalize(cross(view, axis));
        vec3 projected = normal - plane_normal * dot(normal, plane_normal);
        float projected_length = length(projected);
        if (projected_length < 1e-4) {
            visibility += 1.0;
            continue;
        }
        projected /= projected_length;
        float n = acos(clamp(dot(projected, view), -1.0, 1.0)) * sign(dot(projected, axis) + 1e-6);
        float horizons[2] = float[2](-1.0, -1.0);
        for (int side = 0; side < 2; ++side) {
            float sign_side = side == 0 ? -1.0 : 1.0;
            for (int step = 0; step < STEPS; ++step) {
                float fraction = (float(step) + step_jitter + 0.5) / float(STEPS);
                vec2 sample_uv = uv + sign_side * direction * texel * radius_pixels * fraction;
                if (any(lessThan(sample_uv, vec2(0.0))) || any(greaterThan(sample_uv, vec2(1.0)))) {
                    break;
                }
                float sample_z = texture(linear_depth, sample_uv).r;
                if (sample_z <= 0.0 || sample_z > 5000.0) {
                    continue;
                }
                vec3 sample_position = position_at(sample_uv, forward);
                vec3 offset = sample_position - position;
                float distance_metres = length(offset);
                if (distance_metres > radius || distance_metres < 1e-4) {
                    continue;
                }
                float cos_horizon = dot(offset, view) / distance_metres;
                float falloff = clamp(1.0 - distance_metres * distance_metres / (radius * radius), 0.0, 1.0);
                horizons[side] = max(horizons[side], mix(-1.0, cos_horizon, falloff));
            }
        }
        float h1 = -acos(clamp(horizons[0], -1.0, 1.0));
        float h2 = acos(clamp(horizons[1], -1.0, 1.0));
        visibility += projected_length * slice_visibility(n, h1, h2);
    }
    out_occlusion = clamp(visibility / float(SLICES), 0.0, 1.0);
}
