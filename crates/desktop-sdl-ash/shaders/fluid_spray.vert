#version 450

// ADR-102 spray layer (NGQ10 revision 2): particles with fewer fluid
// neighbours than the threshold are drawn as small camera-facing discs
// after the surface composite instead of joining the surface splat.

layout(location = 0) in vec3 in_center_metres;
layout(location = 1) in uint in_neighbours;

layout(set = 0, binding = 0, std140) uniform FluidFrame {
    mat4 view;
    mat4 projection;
    vec4 viewport;
    vec4 params;
    vec4 absorption;
    vec4 focal;
    vec4 sun;
    vec4 spray;
} frame;

layout(location = 0) out vec2 out_corner;

void main() {
    const vec2 corners[6] = vec2[](
        vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(1.0, 1.0),
        vec2(-1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, 1.0)
    );
    vec2 corner = corners[gl_VertexIndex % 6];
    if (frame.spray.x <= 0.0 || float(in_neighbours) >= frame.spray.x) {
        gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
        out_corner = vec2(2.0);
        return;
    }
    vec3 view_center = (frame.view * vec4(in_center_metres, 1.0)).xyz;
    vec3 view_position = view_center + vec3(corner * frame.spray.y, 0.0);
    out_corner = corner;
    gl_Position = frame.projection * vec4(view_position, 1.0);
}
