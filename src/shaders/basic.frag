#version 420 core

struct LightSource {
    vec3 color;
    vec3 position;
    vec3 direction;
    float constant;
    float linear;
    float quadiratic;
    float inner_cuttoff;
    float outer_cuttoff;
    int type; // type 0 - Directional, type 1 - Point, type 2 - Spot
};

layout (std140, binding = 1) uniform LightingData {
    LightSource light_souce;
    vec3 viewer_position;
};

in VertexData {
    vec3 global_position;
    vec3 normal;
    vec2 tex_coord;
} vertex_data;

vec3 compute_directional_light(LightSource ligth_source, vec3 viewer_position);

void main() {
    vec3 frag_color = vec3(0);
    if (light_souce.type == 0) {
        frag_color = compute_directional_light(light_souce, viewer_position);
    }
    gl_FragColor = vec4(frag_color, 0);
}
