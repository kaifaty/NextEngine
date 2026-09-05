#version 450

// Scene look L1 (plan look/01): fullscreen triangle at the far depth; the
// fragment stage rebuilds the world direction from the clip position.

layout(location = 0) out vec2 out_ndc;

void main() {
    vec2 position = vec2(
        gl_VertexIndex == 1 ? 3.0 : -1.0,
        gl_VertexIndex == 2 ? 3.0 : -1.0
    );
    out_ndc = position;
    gl_Position = vec4(position, 0.9999, 1.0);
}
