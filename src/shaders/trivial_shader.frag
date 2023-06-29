#version 330

in vec3 color;
uniform vec3 self_color;

void main() {
    gl_FragColor = vec4(color, 1.0);
}
