#version 450

// ADR-102 presentation-only particle surface (NGQ10 revision 4): one
// camera-facing quad per particle covering the view-space bounding box of
// its anisotropic kernel ellipsoid (Yu and Turk 2010). A zero kernel means
// an isotropic sphere at the profile radius. Nothing here is gameplay
// authority.

layout(location = 0) in vec3 in_center_metres;
layout(location = 1) in uint in_flags;      // neighbours (low byte), cluster size (above)
layout(location = 2) in vec3 in_velocity;   // metres per second, unused here
layout(location = 3) in vec4 in_kernel_a;   // xx xy xz yy
layout(location = 4) in vec4 in_kernel_b;   // yz zz 0 0

layout(set = 0, binding = 0, std140) uniform FluidFrame {
    mat4 view;
    mat4 projection;
    vec4 viewport;   // width, height, 1/width, 1/height
    vec4 params;     // radius_m, near_m, far_m, thickness_scale
    vec4 absorption; // per-metre absorption rgb, refraction strength
    vec4 focal;      // tan_half_x, tan_half_y, focal_px_y, empty_depth
    vec4 sun;        // view-space sun direction xyz, intensity
    vec4 spray;      // neighbour threshold, sub-droplet radius_m, spray alpha, cluster threshold
    vec4 spray2;     // streak seconds, sub-droplets, bulk neighbours, edge radius scale
    vec4 filter_params; // cleanup radius px, 0, 0, 0
} frame;

layout(location = 0) out vec3 out_view_center;
layout(location = 1) flat out vec3 out_g_row0;
layout(location = 2) flat out vec3 out_g_row1;
layout(location = 3) flat out vec3 out_g_row2;

bool is_spray(uint neighbours, uint cluster) {
    return (frame.spray.x > 0.0 && float(neighbours) < frame.spray.x)
        || (frame.spray.w > 0.0 && float(cluster) < frame.spray.w);
}

mat3 inverse_symmetric(mat3 m) {
    float a = m[0][0], b = m[1][0], c = m[2][0];
    float e = m[1][1], f = m[2][1], k = m[2][2];
    float A = e * k - f * f;
    float B = f * c - b * k;
    float C = b * f - e * c;
    float det = a * A + b * B + c * C;
    float inv = 1.0 / (abs(det) < 1e-20 ? 1e-20 : det);
    return mat3(
        A * inv, B * inv, C * inv,
        B * inv, (a * k - c * c) * inv, (b * c - a * f) * inv,
        C * inv, (b * c - a * f) * inv, (a * e - b * b) * inv);
}

void main() {
    const vec2 corners[6] = vec2[](
        vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(1.0, 1.0),
        vec2(-1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, 1.0)
    );
    vec2 corner = corners[gl_VertexIndex % 6];
    uint neighbours = in_flags & 0xFFu;
    uint cluster = in_flags >> 8;
    if (is_spray(neighbours, cluster)) {
        gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
        out_view_center = vec3(0.0);
        out_g_row0 = vec3(0.0);
        out_g_row1 = vec3(0.0);
        out_g_row2 = vec3(0.0);
        return;
    }
    // World kernel (unit-space map), isotropic fallback at the profile
    // radius graded by neighbours (revision 3) when no kernel was sent.
    mat3 g_world;
    if (dot(in_kernel_a, in_kernel_a) + dot(in_kernel_b.xy, in_kernel_b.xy) > 0.0) {
        g_world = mat3(
            in_kernel_a.x, in_kernel_a.y, in_kernel_a.z,
            in_kernel_a.y, in_kernel_a.w, in_kernel_b.x,
            in_kernel_a.z, in_kernel_b.x, in_kernel_b.y);
    } else {
        float span = max(frame.spray2.z - frame.spray.x, 1.0);
        float t = clamp((float(neighbours) - frame.spray.x) / span, 0.0, 1.0);
        float radius = frame.params.x
            * (frame.spray.x > 0.0 ? mix(frame.spray2.w, 1.0, t) : 1.0);
        g_world = mat3(1.0 / radius);
    }
    mat3 rotation = mat3(frame.view);
    mat3 g_view = rotation * g_world * transpose(rotation);
    mat3 g_inverse = inverse_symmetric(g_view);
    // Bounding-box half extents of the ellipsoid along the view axes.
    vec3 extent = vec3(
        length(vec3(g_inverse[0][0], g_inverse[1][0], g_inverse[2][0])),
        length(vec3(g_inverse[0][1], g_inverse[1][1], g_inverse[2][1])),
        length(vec3(g_inverse[0][2], g_inverse[1][2], g_inverse[2][2])));
    vec3 view_center = (frame.view * vec4(in_center_metres, 1.0)).xyz;
    float z_near = max(-view_center.z - extent.z, 1e-4);
    vec4 clip = frame.projection * vec4(view_center, 1.0);
    vec2 half_size = vec2(frame.projection[0][0], frame.projection[1][1]) * extent.xy / z_near;
    clip.xy += corner * half_size * clip.w;
    out_view_center = view_center;
    out_g_row0 = vec3(g_view[0][0], g_view[1][0], g_view[2][0]);
    out_g_row1 = vec3(g_view[0][1], g_view[1][1], g_view[2][1]);
    out_g_row2 = vec3(g_view[0][2], g_view[1][2], g_view[2][2]);
    gl_Position = clip;
}
