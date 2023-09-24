#version 420 core

in VertexData {
    vec2 tex_coord;
} vertex_data;

uniform sampler2D screen_texture;

void main() {
    gl_FragColor = texture(screen_texture, vertex_data.tex_coord);
}
