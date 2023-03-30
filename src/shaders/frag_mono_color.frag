#version 330

in vec3 input_color;

void main() {
    gl_FragColor = vec4(input_color, 1.0);
}