#version 330 core

in vec3 global_position;
in vec3 rotated_normal;
in vec2 tex_coord;

uniform vec3 color;

void main() {
    gl_FragColor = vec4(color, 1.0);
}