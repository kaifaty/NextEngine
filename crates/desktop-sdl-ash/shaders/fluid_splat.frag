#version 450

// Ellipsoid impostor (NGQ10 revision 4): the view ray through the fragment
// is intersected with the kernel ellipsoid; location 0 receives the nearest
// view-space depth (MIN blend), location 1 accumulates the chord through
// the ellipsoid as thickness (ADD blend). The scene depth attachment
// occludes both through the fixed-function depth test.

layout(location = 0) in vec3 in_view_center;
layout(location = 1) flat in vec3 in_g_row0;
layout(location = 2) flat in vec3 in_g_row1;
layout(location = 3) flat in vec3 in_g_row2;

layout(set = 0, binding = 0, std140) uniform FluidFrame {
    mat4 view;
    mat4 projection;
    vec4 viewport;
    vec4 params;
    vec4 absorption;
    vec4 focal;
    vec4 sun;
    vec4 spray;
    vec4 spray2;
    vec4 filter_params;
} frame;

layout(location = 0) out float out_depth;
layout(location = 1) out float out_thickness;

void main() {
    vec2 uv = gl_FragCoord.xy * frame.viewport.zw;
    vec2 ndc = uv * 2.0 - 1.0;
    vec3 direction = normalize(vec3(ndc.x * frame.focal.x, -ndc.y * frame.focal.y, -1.0));
    mat3 g = mat3(in_g_row0, in_g_row1, in_g_row2);
    vec3 gd = g * direction;
    vec3 gc = g * (-in_view_center);
    float a = dot(gd, gd);
    float b = 2.0 * dot(gd, gc);
    float c = dot(gc, gc) - 1.0;
    float disc = b * b - 4.0 * a * c;
    if (disc <= 0.0 || a <= 0.0) {
        discard;
    }
    float root = sqrt(disc);
    float t = (-b - root) / (2.0 * a);
    if (t <= 0.0) {
        discard;
    }
    vec3 view_position = direction * t;
    vec4 clip = frame.projection * vec4(view_position, 1.0);
    gl_FragDepth = clip.z / clip.w;
    out_depth = -view_position.z;
    out_thickness = (root / a) * frame.params.w;
}
