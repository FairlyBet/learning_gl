#version 430 core

in VertexData {
    vec2 tex_coord;
} vertex_data;

uniform sampler2D color_buffer;

layout(location = 0) out vec4 frag_color;
layout(location = 3) uniform float gamma;
layout(location = 4) uniform float exposure;

const float GRADE = 10.0;

vec3 color_grade(vec3 color) {
    ivec3 icolor = ivec3(color * GRADE * 10.0);
    icolor /= int(GRADE);
    return vec3(icolor) / 10.0;
}

void main() {
    vec4 color = texture(color_buffer, vertex_data.tex_coord);
    vec3 mapped = vec3(1.0) - exp(-color.rgb * exposure); // tone mapping
    vec3 corrected = pow(mapped, vec3(gamma)); // gamma correction
    vec3 grade = color_grade(corrected);
    frag_color = vec4(grade, color.a);
}