#version 330 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 yielding_tex_coord;

uniform mat4 mvp;
uniform mat4 model;
uniform mat4 orientation;

out vec3 global_position;
out vec3 rotated_normal;
out vec2 tex_coord;

void main() {
    global_position = (model * vec4(position, 1)).xyz;
    rotated_normal = (orientation * vec4(normal, 1)).xyz;
    tex_coord = yielding_tex_coord;

    gl_Position = mvp * vec4(position, 1);
}