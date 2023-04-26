#version 330

uniform vec3 self_color;

void main() {
    gl_FragColor = vec4(self_color, 1.0);
}
