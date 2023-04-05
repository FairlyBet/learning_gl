#version 330

in vec3 color;
in vec2 tex_coord;

uniform sampler2D in_texture;

void main() {
    gl_FragColor = texture(in_texture, tex_coord) * vec4(color, 1.0);
}