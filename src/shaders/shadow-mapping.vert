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

void main() {
    gl_Position = light_space * vec4(position, 1);
}
