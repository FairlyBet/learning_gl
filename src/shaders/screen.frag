#version 440 core

in VertexData {
    vec2 tex_coord;
} vertex_data;

uniform sampler2D screen_texture;
layout(location = 3) uniform float gamma_correction;

void main() {
    vec4 color = texture(screen_texture, vertex_data.tex_coord);
    vec3 correction = pow(color.rgb, vec3(gamma_correction));
    gl_FragColor = vec4(correction, color.a);
}
