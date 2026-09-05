#version 450

// Water look L5 (plan continuum-water/15): the B0 material for the mirrored
// reflection pass; fragments below the mirror plane (the `w` lane of the
// camera position, written only for this pass) are discarded. Scene look
// L1 shading otherwise identical to `b0_textured`.
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
layout(set = 2, binding = 0) uniform sampler2DShadow shadow_map;

layout(push_constant, std430) uniform DrawPushConstants {
    mat4 model;
    vec4 base_color_factor;
    vec4 material_params;   // metallic, roughness, emissive intensity, 0
} draw;

void main() {
    if (in_world_position.y < frame.camera_world_position.w) {
        discard;
    }
    vec4 base_color = texture(base_color_texture, in_uv) * draw.base_color_factor;

    vec3 face_normal = normalize(cross(dFdx(in_world_position), dFdy(in_world_position)));
    vec3 normal = dot(in_world_normal, in_world_normal) > 0.0001
        ? normalize(in_world_normal)
        : face_normal;
    if (!gl_FrontFacing) {
        normal = -normal;
    }

    vec3 light_direction = normalize(-frame.sun_direction_intensity.xyz);
    float n_dot_l = max(dot(normal, light_direction), 0.0);
    vec4 shadow_clip = frame.shadow_view_projection * vec4(in_world_position, 1.0);
    vec3 shadow_coord = shadow_clip.xyz / shadow_clip.w;
    shadow_coord.xy = shadow_coord.xy * 0.5 + 0.5;
    float shadow_visibility = 1.0;
    if (shadow_coord.x >= 0.0 && shadow_coord.x <= 1.0
        && shadow_coord.y >= 0.0 && shadow_coord.y <= 1.0
        && shadow_coord.z >= 0.0 && shadow_coord.z <= 1.0) {
        vec2 texel = 1.0 / vec2(textureSize(shadow_map, 0));
        float receiver_bias = max(0.0007 * (1.0 - n_dot_l), 0.00025);
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

    // Scene look L1: Lambert under the sky's SH irradiance and the sun, and
    // a Cook-Torrance GGX lobe for the sun (metallic and roughness from the
    // material record).
    vec3 view = normalize(frame.camera_world_position.xyz - in_world_position);
    vec3 half_vector = normalize(light_direction + view);
    float n_dot_v = max(dot(normal, view), 1e-4);
    float n_dot_h = max(dot(normal, half_vector), 0.0);
    float v_dot_h = max(dot(view, half_vector), 0.0);
    float metallic = clamp(draw.material_params.x, 0.0, 1.0);
    float roughness = clamp(draw.material_params.y, 0.04, 1.0);
    float alpha = roughness * roughness;
    float alpha2 = alpha * alpha;
    float denominator = n_dot_h * n_dot_h * (alpha2 - 1.0) + 1.0;
    float distribution = alpha2 / (LIGHTING_PI * denominator * denominator);
    float visibility = 0.5 / (
        n_dot_l * sqrt(n_dot_v * n_dot_v * (1.0 - alpha2) + alpha2)
        + n_dot_v * sqrt(n_dot_l * n_dot_l * (1.0 - alpha2) + alpha2)
        + 1e-5
    );
    vec3 f0 = mix(vec3(0.04), base_color.rgb, metallic);
    vec3 fresnel = f0 + (1.0 - f0) * pow(1.0 - v_dot_h, 5.0);
    vec3 specular = distribution * visibility * fresnel;
    vec3 diffuse_colour = base_color.rgb * (1.0 - metallic);
    vec3 sun = lighting.sun_radiance.rgb * n_dot_l * shadow_visibility;
    vec3 lit_color = diffuse_colour / LIGHTING_PI * (sh_irradiance(normal) + sun)
        + specular * sun
        + base_color.rgb * draw.material_params.z;

    float world_distance = distance(in_world_position, frame.camera_world_position.xyz);
    float fog_amount = clamp(1.0 - exp(-world_distance * lighting.fog.w), 0.0, 0.82);
    out_color = vec4(mix(lit_color, lighting.fog.rgb, fog_amount), base_color.a);
}
