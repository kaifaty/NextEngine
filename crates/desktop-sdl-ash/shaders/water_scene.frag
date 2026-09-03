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

layout(set = 1, binding = 0) uniform sampler2D base_color_texture;
layout(set = 2, binding = 0) uniform sampler2DShadow shadow_map;
layout(set = 3, binding = 0) uniform sampler2D scene_color;
layout(set = 3, binding = 1) uniform sampler2D scene_depth;
layout(set = 3, binding = 2, std140) uniform WaterUniforms {
    mat4 inverse_view_projection;
    vec4 viewport;      // width, height, 1/width, 1/height
    vec4 absorption;    // per-metre rgb, refraction strength
    vec4 shore;         // foam width (m), fade width (m), foam grey, unused
} water;

layout(push_constant, std430) uniform DrawPushConstants {
    mat4 model;
    vec4 base_color_factor;
} draw;

const float WATER_F0 = 0.02;
const float SUN_SPECULAR_EXPONENT = 240.0;
const float BODY_DIFFUSE_WEIGHT = 0.6;
const vec3 SKY_HORIZON = vec3(0.48, 0.60, 0.68);
const vec3 SKY_ZENITH = vec3(0.10, 0.20, 0.34);

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

    vec3 view = normalize(frame.camera_world_position.xyz - in_world_position);
    float cos_theta = clamp(dot(normal, view), 0.0, 1.0);
    float fresnel = WATER_F0 + (1.0 - WATER_F0) * pow(1.0 - cos_theta, 5.0);

    vec3 light_direction = normalize(-frame.sun_direction_intensity.xyz);
    float diffuse = max(dot(normal, light_direction), 0.0);
    vec4 shadow_clip = frame.shadow_view_projection * vec4(in_world_position, 1.0);
    vec3 shadow_coord = shadow_clip.xyz / shadow_clip.w;
    shadow_coord.xy = shadow_coord.xy * 0.5 + 0.5;
    float shadow_visibility = 1.0;
    if (shadow_coord.x >= 0.0 && shadow_coord.x <= 1.0
        && shadow_coord.y >= 0.0 && shadow_coord.y <= 1.0
        && shadow_coord.z >= 0.0 && shadow_coord.z <= 1.0) {
        vec2 texel = 1.0 / vec2(textureSize(shadow_map, 0));
        float receiver_bias = max(0.0007 * (1.0 - diffuse), 0.00025);
        shadow_visibility = 0.0;
        for (int y = -1; y <= 1; ++y) {
            for (int x = -1; x <= 1; ++x) {
                shadow_visibility += texture(
                    shadow_map,
                    vec3(shadow_coord.xy + vec2(x, y) * texel, shadow_coord.z - receiver_bias)
                );
            }
        }
        shadow_visibility /= 9.0;
    }

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

    // Lit water body (WL1) and the transmitted, absorbed scene behind it.
    float hemisphere = clamp(normal.y * 0.5 + 0.5, 0.0, 1.0);
    vec3 ambient = mix(
        frame.hemisphere_ground_color.rgb,
        frame.hemisphere_sky_color.rgb,
        hemisphere
    );
    vec3 body = base_color.rgb * (
        ambient + BODY_DIFFUSE_WEIGHT * diffuse * shadow_visibility * frame.sun_direction_intensity.w
    );
    vec3 transmittance = exp(-water.absorption.rgb * thickness);
    vec3 under = mix(body, scene * transmittance, transmittance);

    // Shoreline (revision 2): measured by the vertical water depth at the
    // scene point, so grazing views keep a thin band at walls and the crate;
    // fade into the scene over the fade width, foam over the foam width.
    float vertical_depth = max(in_world_position.y - behind.y, 0.0);
    float fade = smoothstep(0.0, water.shore.y, vertical_depth);
    float foam = 1.0 - smoothstep(0.0, water.shore.x, vertical_depth);
    vec3 shore_colour = mix(under, vec3(water.shore.z), foam * 0.6);

    // WL1 reflection, glint and fog.
    vec3 reflected = reflect(-view, normal);
    float elevation = clamp(reflected.y, 0.0, 1.0);
    vec3 reflected_sky = mix(SKY_HORIZON, SKY_ZENITH, smoothstep(0.0, 0.8, elevation));
    vec3 half_vector = normalize(light_direction + view);
    float specular = pow(max(dot(normal, half_vector), 0.0), SUN_SPECULAR_EXPONENT)
        * frame.sun_direction_intensity.w * shadow_visibility;
    vec3 lit_water = mix(shore_colour, reflected_sky, fresnel) + vec3(specular);
    vec3 lit_color = mix(texture(scene_color, uv).rgb, lit_water, fade);

    float world_distance = distance(in_world_position, frame.camera_world_position.xyz);
    float fog_amount = clamp(
        1.0 - exp(-world_distance * frame.fog_color_density.w),
        0.0,
        0.82
    );
    out_color = vec4(mix(lit_color, frame.fog_color_density.rgb, fog_amount), 1.0);
}
