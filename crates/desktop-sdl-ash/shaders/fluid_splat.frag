#version 450

// Sphere impostor: location 0 receives the nearest view-space depth (MIN
// blend), location 1 accumulates the sphere thickness (ADD blend). The scene
// depth attachment occludes both through the fixed-function depth test.

layout(location = 0) in vec3 in_view_center;
layout(location = 1) in vec2 in_corner;

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

layout(location = 0) out float out_depth;
layout(location = 1) out float out_thickness;

void main() {
    float radial = dot(in_corner, in_corner);
    if (radial > 1.0) {
        discard;
    }
    float radius = frame.params.x;
    float height = sqrt(1.0 - radial);
    vec3 view_position = in_view_center + vec3(in_corner * radius, height * radius);
    vec4 clip = frame.projection * vec4(view_position, 1.0);
    gl_FragDepth = clip.z / clip.w;
    out_depth = -view_position.z;
    out_thickness = 2.0 * height * radius * frame.params.w;
}
