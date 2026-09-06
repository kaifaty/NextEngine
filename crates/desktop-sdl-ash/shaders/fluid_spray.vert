#version 450

// ADR-102 spray layer (NGQ10 revision 3): every spray particle (few
// neighbours or a small connected component) is drawn as several
// sub-droplets, capsules jittered deterministically inside the particle
// sphere and stretched along the screen projection of the velocity.

layout(location = 0) in vec3 in_center_metres;
layout(location = 1) in uint in_flags;
layout(location = 2) in vec3 in_velocity;

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

layout(location = 0) out vec2 out_corner;
layout(location = 1) out float out_half_length;

uint wang_hash(uint seed) {
    seed = (seed ^ 61u) ^ (seed >> 16);
    seed *= 9u;
    seed = seed ^ (seed >> 4);
    seed *= 0x27d4eb2du;
    seed = seed ^ (seed >> 15);
    return seed;
}

float unit_float(uint seed) {
    return float(wang_hash(seed) & 0x00FFFFFFu) / 16777215.0 * 2.0 - 1.0;
}

void main() {
    const vec2 corners[6] = vec2[](
        vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(1.0, 1.0),
        vec2(-1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, 1.0)
    );
    vec2 corner = corners[gl_VertexIndex % 6];
    int sub = gl_VertexIndex / 6;
    uint neighbours = in_flags & 0xFFu;
    uint cluster = in_flags >> 8;
    bool spray = (frame.spray.x > 0.0 && float(neighbours) < frame.spray.x)
        || (frame.spray.w > 0.0 && float(cluster) < frame.spray.w);
    if (!spray || float(sub) >= frame.spray2.y) {
        gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
        out_corner = vec2(2.0);
        out_half_length = 0.0;
        return;
    }
    uint seed = uint(gl_InstanceIndex) * 7919u + uint(sub) * 104729u + 1u;
    vec3 jitter = vec3(unit_float(seed), unit_float(seed + 1u), unit_float(seed + 2u));
    jitter *= 0.9 / max(length(jitter), 1.0);
    vec3 world_center = in_center_metres + jitter * frame.params.x;
    vec3 view_center = (frame.view * vec4(world_center, 1.0)).xyz;
    vec3 view_velocity = (frame.view * vec4(in_velocity, 0.0)).xyz;
    float radius = frame.spray.y;
    float streak = min(length(view_velocity.xy) * frame.spray2.x, 0.15);
    vec2 along = length(view_velocity.xy) > 1e-4 ? normalize(view_velocity.xy) : vec2(1.0, 0.0);
    vec2 across = vec2(-along.y, along.x);
    float half_length = 0.5 * streak / radius;
    vec2 offset = across * corner.x * radius + along * corner.y * (radius + 0.5 * streak);
    vec3 view_position = view_center + vec3(offset, 0.0);
    out_corner = vec2(corner.x, corner.y * (1.0 + half_length));
    out_half_length = half_length;
    gl_Position = frame.projection * vec4(view_position, 1.0);
}
