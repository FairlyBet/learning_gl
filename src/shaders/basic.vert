#version 420 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 tex_coord;

layout (std140, binding = 0) uniform MatrixData {
    mat4 mvp;
    mat4 model;
    mat4 orientation;
};

out VertexData {
    vec3 global_position;
    vec3 normal;
    vec2 tex_coord;
} vertex_data;

void main() {
    vertex_data.global_position = (model * vec4(position, 1.0)).xyz;
    vertex_data.normal = (orientation * vec4(normal, 1.0)).xyz;
    vertex_data.tex_coord = tex_coord;

    gl_Position = mvp * vec4(position, 1.0);
}
