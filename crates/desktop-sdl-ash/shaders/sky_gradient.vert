#version 450

layout(location = 0) out vec2 out_uv;

void main() {
    vec2 position = vec2(
        gl_VertexIndex == 1 ? 3.0 : -1.0,
        gl_VertexIndex == 2 ? 3.0 : -1.0
    );
    out_uv = position * 0.5 + 0.5;
    gl_Position = vec4(position, 0.9999, 1.0);
}
