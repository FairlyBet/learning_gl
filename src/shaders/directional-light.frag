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
    int type;
};

in VertexData {
    vec3 global_position;
    vec3 normal;
    vec2 tex_coord;
} vertex_data;

vec3 compute_directional_light(LightSource ligth_source, vec3 viewer_position) {
    float ambient_intensity = 0.3;
    float diffuse_intensity = max(dot(vertex_data.normal, -ligth_source.direction), 0);

    vec3 viewer_direction = normalize(viewer_position - vertex_data.global_position);
    vec3 reflect_direction = reflect(ligth_source.direction, vertex_data.normal);
    // float specular_intensity = max(dot(viewer_direction, reflect_direction), 0);
    // specular_intensity = pow(specular_intensity, 8);

    return (ambient_intensity + diffuse_intensity) * ligth_source.color;
}
