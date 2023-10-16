#version 420 core

layout (location = 0) in vec3 position;
layout (location = 2) in vec2 tex_coord;

out VertexData {
    vec2 tex_coord;
} vertex_data;

void main() {
    vertex_data.tex_coord = tex_coord;
    gl_Position = vec4(position, 1.0);
}
