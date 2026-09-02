#version 450

// ADR-102 presentation-only particle surface: one camera-facing quad per
// particle (six vertices from gl_VertexIndex, centre from the per-instance
// stream). Nothing here is gameplay authority.

layout(location = 0) in vec3 in_center_metres;

layout(set = 0, binding = 0, std140) uniform FluidFrame {
    mat4 view;
    mat4 projection;
    vec4 viewport;   // width, height, 1/width, 1/height
    vec4 params;     // radius_m, near_m, far_m, thickness_scale
    vec4 absorption; // per-metre absorption rgb, refraction strength
    vec4 focal;      // tan_half_x, tan_half_y, focal_px_y, empty_depth
    vec4 sun;        // view-space sun direction xyz, intensity
} frame;

layout(location = 0) out vec3 out_view_center;
layout(location = 1) out vec2 out_corner;

void main() {
    const vec2 corners[6] = vec2[](
        vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(1.0, 1.0),
        vec2(-1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, 1.0)
    );
    vec2 corner = corners[gl_VertexIndex % 6];
    vec3 view_center = (frame.view * vec4(in_center_metres, 1.0)).xyz;
    vec3 view_position = view_center + vec3(corner * frame.params.x, 0.0);
    out_view_center = view_center;
    out_corner = corner;
    gl_Position = frame.projection * vec4(view_position, 1.0);
}
