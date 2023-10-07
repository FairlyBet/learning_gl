#version 420 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 tex_coord;

layout (std140, binding = 0) uniform MatrixData {
    mat4 mvp;
    mat4 model;
    mat4 orientation;
    mat4 light_space;
};

out VertexData {
    vec3 position;
    vec3 normal;
    vec2 tex_coord;
    vec3 light_space_position;
} vertex;

void main() {
    vertex.position = (model * vec4(position, 1)).xyz;
    vertex.normal = (orientation * vec4(normal, 1)).xyz;
    vertex.tex_coord = tex_coord;
    vec4 light_space_position = (light_space * model * vec4(position, 1));
    vertex.light_space_position = light_space_position.xyz / light_space_position.w;
    gl_Position = mvp * vec4(position, 1);
}
