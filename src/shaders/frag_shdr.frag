#version 330

in vec3 color;
in vec2 tex_coord;

uniform sampler2D texture1;
uniform sampler2D texture2;

void main() {
    gl_FragColor = mix(texture2D(texture1, tex_coord), texture2D(texture2, tex_coord), .2);
}
