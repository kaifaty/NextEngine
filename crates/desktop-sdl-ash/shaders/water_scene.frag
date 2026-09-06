#version 450

// Water look L2 + L3 (plan continuum-water/13): the water pass material of a
// `WaterSurface` dynamic ring drawn after the opaque scene. Set 3 carries the
// opaque scene colour copy, the scene depth and the water uniform; the rest
// is the B0 interface. Renderer-local constants only.

layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec3 in_world_position;
layout(location = 2) in vec3 in_world_normal;
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

layout(set = 1, binding = 0) uniform sampler2D base_color_texture;
layout(set = 2, binding = 0) uniform sampler2DArrayShadow shadow_map;

// Scene look L2 (plan look/02): the sun's visibility from the cascaded
// shadow map: the first cascade holding the receiver (moved along its
// normal by 1.5 texels) inside a 2% margin, a slope-scaled bias over the
// cascade's depth range, a 3x3 kernel of linear compare taps.
const float SHADOW_MAP_TEXELS = 2048.0;

float sun_visibility(vec3 world_position, vec3 normal, float n_dot_l) {
    for (int cascade = 0; cascade < 3; ++cascade) {
        float extent = lighting.shadow_extents[cascade];
        float texel_metres = extent / SHADOW_MAP_TEXELS;
        vec3 receiver = world_position + normal * texel_metres * 1.5;
        vec4 clip = lighting.shadow_cascades[cascade] * vec4(receiver, 1.0);
        vec3 coord = clip.xyz / clip.w;
        coord.xy = coord.xy * 0.5 + 0.5;
        if (any(lessThan(coord.xy, vec2(0.02))) || any(greaterThan(coord.xy, vec2(0.98)))
            || coord.z < 0.0 || coord.z > 1.0) {
            continue;
        }
        float range_scale = 44.0 / (1.375 * extent);
        float bias = max(0.0007 * (1.0 - n_dot_l), 0.00025) * range_scale;
        vec2 texel = vec2(1.0 / SHADOW_MAP_TEXELS);
        float visibility = 0.0;
        for (int y = -1; y <= 1; ++y) {
            for (int x = -1; x <= 1; ++x) {
                visibility += texture(
                    shadow_map,
                    vec4(coord.xy + vec2(x, y) * texel, float(cascade), coord.z - bias)
                );
            }
        }
        return visibility / 9.0;
    }
    return 1.0;
}
layout(set = 3, binding = 0) uniform sampler2D scene_color;
layout(set = 3, binding = 1) uniform sampler2D scene_depth;
layout(set = 3, binding = 3) uniform sampler2D reflection;
layout(set = 3, binding = 2, std140) uniform WaterUniforms {
    mat4 inverse_view_projection;
    vec4 viewport;      // width, height, 1/width, 1/height
    vec4 absorption;    // per-metre rgb, refraction strength
    vec4 shore;         // foam width (m), fade width (m), foam grey, run seconds
    vec4 under;         // plan 33: level (m), submerged (0/1), unused, unused
} water;

layout(push_constant, std430) uniform DrawPushConstants {
    mat4 model;
    vec4 base_color_factor;
} draw;

const float WATER_F0 = 0.02;
const float SUN_SPECULAR_EXPONENT = 240.0;
const float BODY_DIFFUSE_WEIGHT = 0.6;
const float DETAIL_NORMAL_OFFSET = 0.03;
// Plan 42: the detail tilt is whole at the camera and a fifth at 50 m.
const float DETAIL_FADE_METRES = 12.0;
const float REFLECTION_DISTORTION = 0.02;
const float CAUSTIC_STRENGTH = 1.0;
const float CAUSTIC_ABSORPTION_PER_METRE = 1.0;
// Plan 33: the surface seen from below.
const float WATER_INDEX = 1.333;
const vec3 WATER_UNDER_INSCATTER = vec3(0.05, 0.18, 0.28);
const vec3 WATER_UNDER_MIRROR = vec3(0.10, 0.28, 0.40);
const vec3 WATER_UNDER_ABSORPTION_PER_METRE = vec3(0.45, 0.12, 0.06);

vec3 scene_world_position(vec2 uv, float depth) {
    vec4 clip = vec4(uv * 2.0 - 1.0, depth, 1.0);
    vec4 world = water.inverse_view_projection * clip;
    return world.xyz / world.w;
}

void main() {
    vec4 base_color = texture(base_color_texture, in_uv) * draw.base_color_factor;

    vec3 face_normal = normalize(cross(dFdx(in_world_position), dFdy(in_world_position)));
    vec3 normal = dot(in_world_normal, in_world_normal) > 0.0001
        ? normalize(in_world_normal)
        : face_normal;
    if (!gl_FrontFacing) {
        normal = -normal;
    }
    // WL4 detail normal: two world-space sine gradients moving with the run
    // clock, a small tilt on top of the ring normal.
    {
        float t = water.shore.w;
        vec2 p = in_world_position.xz;
        vec2 d1 = normalize(vec2(0.83, 0.56));
        vec2 d2 = normalize(vec2(-0.42, 0.91));
        float k1 = 6.2831853 / 0.18;
        float k2 = 6.2831853 / 0.11;
        float g1 = cos(k1 * dot(d1, p) - k1 * 0.35 * t);
        float g2 = cos(k2 * dot(d2, p) - k2 * 0.5 * t);
        vec2 tilt = (d1 * g1 + d2 * g2) * DETAIL_NORMAL_OFFSET;
        // Plan 42: 0.11-0.18 m waves alias past a few metres and shimmer
        // far away; fade the tilt with the distance from the camera.
        float detail_distance = distance(frame.camera_world_position.xyz, in_world_position);
        tilt *= DETAIL_FADE_METRES / (DETAIL_FADE_METRES + detail_distance);
        normal = normalize(normal + vec3(tilt.x, 0.0, tilt.y));
    }

    vec3 view = normalize(frame.camera_world_position.xyz - in_world_position);
    float cos_theta = clamp(dot(normal, view), 0.0, 1.0);
    float fresnel = WATER_F0 + (1.0 - WATER_F0) * pow(1.0 - cos_theta, 5.0);

    // Plan 33: a submerged eye sees the back face. Snell's law for water
    // to air: beyond the critical angle the surface is a mirror of the
    // water itself; inside it the scene above shows through the refracted
    // offset, blended by Schlick's Fresnel of the refracted angle. The path
    // from the eye to the surface is fogged like the pass before the rings.
    if (water.under.y > 0.5 && !gl_FrontFacing) {
        vec2 uv_below = gl_FragCoord.xy * water.viewport.zw;
        float sin_i = sqrt(max(1.0 - cos_theta * cos_theta, 0.0));
        float sin_t = WATER_INDEX * sin_i;
        vec3 colour;
        if (sin_t >= 1.0) {
            colour = WATER_UNDER_MIRROR / lighting.sun_radiance.w;
        } else {
            float cos_t = sqrt(1.0 - sin_t * sin_t);
            float fresnel_below = WATER_F0 + (1.0 - WATER_F0) * pow(1.0 - cos_t, 5.0);
            vec2 refracted = clamp(
                uv_below + normal.xz * water.absorption.w,
                water.viewport.zw,
                1.0 - water.viewport.zw
            );
            vec3 above = texture(scene_color, refracted).rgb;
            colour = mix(above, WATER_UNDER_MIRROR / lighting.sun_radiance.w, fresnel_below);
        }
        float path = distance(frame.camera_world_position.xyz, in_world_position);
        vec3 transmittance = exp(-WATER_UNDER_ABSORPTION_PER_METRE * path);
        out_color = vec4(
            mix(WATER_UNDER_INSCATTER / lighting.sun_radiance.w, colour, transmittance),
            1.0
        );
        return;
    }

    vec3 light_direction = normalize(-frame.sun_direction_intensity.xyz);
    float diffuse = max(dot(normal, light_direction), 0.0);
    float shadow_visibility = sun_visibility(in_world_position, normal, diffuse);

    // Path length through the water: from the surface point to the opaque
    // scene point behind it on the same pixel ray.
    vec2 uv = gl_FragCoord.xy * water.viewport.zw;
    float surface_depth = gl_FragCoord.z;
    float behind_depth = texture(scene_depth, uv).r;
    vec3 behind = scene_world_position(uv, behind_depth);
    float thickness = max(distance(behind, in_world_position), 0.0);

    // Refraction: offset by the surface normal, scaled by the thickness; a
    // refracted sample that lies in front of the surface falls back.
    vec2 offset = normal.xz * water.absorption.w * min(thickness, 1.0);
    vec2 refracted_uv = clamp(uv + offset, water.viewport.zw, 1.0 - water.viewport.zw);
    float refracted_depth = texture(scene_depth, refracted_uv).r;
    vec3 scene = texture(scene_color, uv).rgb;
    if (refracted_depth >= surface_depth) {
        scene = texture(scene_color, refracted_uv).rgb;
        behind = scene_world_position(refracted_uv, refracted_depth);
        thickness = max(distance(behind, in_world_position), 0.0);
    }

    // WL6 caustics (plan 17 revision 2): two animated sine lattices over
    // the scene point under the surface, sharpened, fading with the
    // vertical depth of that point.
    {
        float t = water.shore.w;
        vec2 p = behind.xz;
        vec2 c1 = vec2(cos(radians(30.0)), sin(radians(30.0)));
        vec2 c2 = vec2(cos(radians(120.0)), sin(radians(120.0)));
        float k1 = 6.2831853 / 0.35;
        float k2 = 6.2831853 / 0.23;
        float a = 0.5 + 0.5 * sin(k1 * dot(c1, p) - k1 * 0.25 * t);
        float b = 0.5 + 0.5 * sin(k2 * dot(c2, p) - k2 * 0.4 * t);
        float caustic = pow(a * b, 2.0);
        float depth_below = max(in_world_position.y - behind.y, 0.0);
        float submerged = smoothstep(0.0, 0.05, depth_below);
        scene *= 1.0
            + CAUSTIC_STRENGTH * caustic * exp(-CAUSTIC_ABSORPTION_PER_METRE * depth_below)
            * submerged;
    }

    // Lit water body (WL1) and the transmitted, absorbed scene behind it.
    vec3 body = base_color.rgb / LIGHTING_PI * (
        sh_irradiance(normal)
        + BODY_DIFFUSE_WEIGHT * diffuse * shadow_visibility * lighting.sun_radiance.rgb
    );
    vec3 transmittance = exp(-water.absorption.rgb * thickness);
    vec3 under = mix(body, scene * transmittance, transmittance);

    // Shoreline (revision 2): measured by the vertical water depth at the
    // scene point, so grazing views keep a thin band at walls and the crate;
    // fade into the scene over the fade width, foam over the foam width.
    float vertical_depth = max(in_world_position.y - behind.y, 0.0);
    float fade = smoothstep(0.0, water.shore.y, vertical_depth);
    float foam = 1.0 - smoothstep(0.0, water.shore.x, vertical_depth);
    vec3 shore_colour = mix(under, vec3(water.shore.z / lighting.sun_radiance.w), foam * 0.6);

    // WL1 reflection, glint and fog.
    vec3 reflected = reflect(-view, normal);
    vec3 reflected_sky = sky_radiance(reflected);
    // WL5: the mirrored scene at this pixel, distorted by the ring normal,
    // over the analytic sky where nothing reflects (alpha 0).
    vec2 reflection_uv = clamp(
        uv + normal.xz * REFLECTION_DISTORTION,
        water.viewport.zw,
        1.0 - water.viewport.zw
    );
    vec4 mirrored = texture(reflection, reflection_uv);
    reflected_sky = mix(reflected_sky, mirrored.rgb, mirrored.a);
    vec3 half_vector = normalize(light_direction + view);
    vec3 specular = pow(max(dot(normal, half_vector), 0.0), SUN_SPECULAR_EXPONENT)
        * (SUN_SPECULAR_EXPONENT + 8.0) / (8.0 * LIGHTING_PI)
        * lighting.sun_radiance.rgb * shadow_visibility;
    vec3 lit_water = mix(shore_colour, reflected_sky, fresnel) + specular;
    vec3 lit_color = mix(texture(scene_color, uv).rgb, lit_water, fade);

    float world_distance = distance(in_world_position, frame.camera_world_position.xyz);
    float fog_amount = clamp(
        1.0 - exp(-world_distance * lighting.fog.w),
        0.0,
        0.82
    );
    out_color = vec4(mix(lit_color, lighting.fog.rgb, fog_amount), 1.0);
}
