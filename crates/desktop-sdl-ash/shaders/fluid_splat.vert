#version 450

// ADR-102 presentation-only particle surface: one camera-facing quad per
// particle (six vertices from gl_VertexIndex, centre from the per-instance
// stream). Nothing here is gameplay authority.

layout(location = 0) in vec3 in_center_metres;
layout(location = 1) in uint in_flags;      // neighbours (low byte), cluster size (above)
layout(location = 2) in vec3 in_velocity;   // metres per second, unused here

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
} frame;

layout(location = 0) out vec3 out_view_center;
layout(location = 1) out vec2 out_corner;
layout(location = 2) out float out_radius;

bool is_spray(uint neighbours, uint cluster) {
    return (frame.spray.x > 0.0 && float(neighbours) < frame.spray.x)
        || (frame.spray.w > 0.0 && float(cluster) < frame.spray.w);
}

void main() {
    const vec2 corners[6] = vec2[](
        vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(1.0, 1.0),
        vec2(-1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, 1.0)
    );
    vec2 corner = corners[gl_VertexIndex % 6];
    uint neighbours = in_flags & 0xFFu;
    uint cluster = in_flags >> 8;
    // NGQ10 revision 2/3: spray particles belong to the spray pass;
    // collapse their quad outside the clip volume.
    if (is_spray(neighbours, cluster)) {
        gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
        out_view_center = vec3(0.0);
        out_corner = vec2(2.0);
        out_radius = frame.params.x;
        return;
    }
    // NGQ10 revision 3: the splat radius grows from the edge scale at the
    // spray threshold to the full radius at the bulk neighbour count.
    float span = max(frame.spray2.z - frame.spray.x, 1.0);
    float t = clamp((float(neighbours) - frame.spray.x) / span, 0.0, 1.0);
    float radius = frame.params.x * (frame.spray.x > 0.0 ? mix(frame.spray2.w, 1.0, t) : 1.0);
    vec3 view_center = (frame.view * vec4(in_center_metres, 1.0)).xyz;
    vec3 view_position = view_center + vec3(corner * radius, 0.0);
    out_view_center = view_center;
    out_corner = corner;
    out_radius = radius;
    gl_Position = frame.projection * vec4(view_position, 1.0);
}
