#version 450

// Scene look L2 (plan look/02): the depth-only caster program draws one
// cascade per rendering instance; the cascade index rides the spare lane of
// the draw push block and selects its view-projection in the lighting block.

layout(location = 0) in vec3 in_position;

layout(set = 0, binding = 0, std140) uniform FrameUniforms {
    mat4 view_projection;
    mat4 shadow_view_projection;
    vec4 camera_world_position;
    vec4 sun_direction_intensity;
    vec4 hemisphere_sky_color;
    vec4 hemisphere_ground_color;
    vec4 fog_color_density;
} frame;

layout(set = 0, binding = 1, std140) uniform LightingUniforms {
    mat4 inverse_view_projection;
    vec4 sun_radiance;
    vec4 sky_sh[9];
    vec4 fog;
    vec4 sky_zenith;
    vec4 sky_perez_x[2];
    vec4 sky_perez_y[2];
    vec4 sky_perez_luminance[2];
    vec4 sky_params;
    mat4 shadow_cascades[3];
    vec4 shadow_extents;
} lighting;

layout(push_constant, std430) uniform DrawPushConstants {
    mat4 model;
    vec4 base_color_factor;
    vec4 material_params;   // w: the cascade index
} draw;

void main() {
    int cascade = clamp(int(draw.material_params.w + 0.5), 0, 2);
    gl_Position = lighting.shadow_cascades[cascade] * draw.model * vec4(in_position, 1.0);
}
