#version 430 core

in VertexData {
    vec2 tex_coord;
} vertex_data;

uniform sampler2D color_buffer;

layout(location = 0) out vec4 frag_color;
layout(location = 3) uniform float gamma;
layout(location = 4) uniform float exposure;

void main() {
    vec4 color = texture(color_buffer, vertex_data.tex_coord);
    vec3 mapped = vec3(1.0) - exp(-color.rgb * exposure); // tone mapping
    vec3 corrected = pow(mapped, vec3(gamma)); // gamma correction
    frag_color = vec4(corrected, color.a);
}