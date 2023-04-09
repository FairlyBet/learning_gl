#version 330 core

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 tex_coord;

uniform mat4 translation;

out vec2 coord;

void main() {
    coord = tex_coord;
    gl_Position = translation * vec4(pos, 1.0);
}
