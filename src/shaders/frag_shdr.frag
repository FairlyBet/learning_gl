#version 330

in vec2 coord;

uniform sampler2D tex;

void main() {
    gl_FragColor = texture2D(tex, coord);
}
