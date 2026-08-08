#version 450

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

layout(push_constant, std430) uniform DrawPushConstants {
    mat4 model;
    vec4 base_color_factor;
} draw;

void main() {
    vec4 base_color = texture(base_color_texture, in_uv) * draw.base_color_factor;
    vec3 face_normal = normalize(cross(dFdx(in_world_position), dFdy(in_world_position)));
    vec3 normal = dot(in_world_normal, in_world_normal) > 0.0001
        ? normalize(in_world_normal)
        : face_normal;
    if (!gl_FrontFacing) {
        normal = -normal;
    }
    vec3 light_direction = normalize(-frame.sun_direction_intensity.xyz);
    float diffuse = max(dot(normal, light_direction), 0.0);
    float hemisphere = clamp(normal.y * 0.5 + 0.5, 0.0, 1.0);
    vec3 ambient = mix(
        frame.hemisphere_ground_color.rgb,
        frame.hemisphere_sky_color.rgb,
        hemisphere
    );
    vec3 lit_color = base_color.rgb * (
        ambient + diffuse * frame.sun_direction_intensity.w
    );
    float world_distance = distance(in_world_position, frame.camera_world_position.xyz);
    float fog_amount = clamp(
        1.0 - exp(-world_distance * frame.fog_color_density.w),
        0.0,
        0.82
    );
    out_color = vec4(
        mix(lit_color, frame.fog_color_density.rgb, fog_amount),
        base_color.a
    );
}
