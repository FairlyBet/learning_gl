#version 330

uniform vec3 self_color;
uniform vec3 light_color;

void main() {
    gl_FragColor = vec4(self_color * light_color, 1.0);
}
