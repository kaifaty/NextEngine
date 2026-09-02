#version 450

// Fullscreen triangle for the filter and composite passes; the fragment
// stages derive texture coordinates from gl_FragCoord.

void main() {
    vec2 position = vec2(
        gl_VertexIndex == 1 ? 3.0 : -1.0,
        gl_VertexIndex == 2 ? 3.0 : -1.0
    );
    gl_Position = vec4(position, 0.0, 1.0);
}
