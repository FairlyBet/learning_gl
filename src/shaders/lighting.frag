#version 420 core

struct LightSource {
    vec3 color;
    int type; // type 0 - Directional, type 1 - Point, type 2 - Spot
    vec3 position;
    float constant;
    vec3 direction;
    float linear;
    float quadiratic;
    float inner_cuttoff;
    float outer_cuttoff;
};

layout (std140, binding = 1) uniform LightingData {
    LightSource light_source;
    vec3 viewer_position;
};

in VertexData {
    vec3 position;
    vec3 normal;
    vec2 tex_coord;
} vertex;

const float AMBIENT_INTENSITY = 0.2;

void directional() {
    float diffuse_intensity = max(dot(vertex.normal, -light_source.direction), 0);

    vec3 viewer_direction = normalize(viewer_position - vertex.position);
    vec3 reflect_direction = reflect(light_source.direction, vertex.normal);
    float specular_intensity = pow(max(dot(viewer_direction, reflect_direction), 0), 16);

    gl_FragColor = vec4((AMBIENT_INTENSITY + diffuse_intensity + specular_intensity) * light_source.color, 1);
}

void point() {
    vec3 to_light_source_direction = normalize(light_source.position - vertex.position);
    float diffuse_intensity = max(dot(vertex.normal, to_light_source_direction), 0);

    vec3 viewer_direction = normalize(viewer_position - vertex.position);
    vec3 reflect_direction = reflect(-to_light_source_direction, vertex.normal);
    float specular_intensity = pow(max(dot(viewer_direction, reflect_direction), 0), 16);

    float distance = length(light_source.position - vertex.position);
    float attenuation = 1 / (light_source.constant + light_source.linear * distance + light_source.quadiratic * distance * distance);

    gl_FragColor = vec4((AMBIENT_INTENSITY + diffuse_intensity + specular_intensity) * light_source.color * attenuation, 1);
}

void spot() {
    vec3 to_light_source_direction = normalize(light_source.position - vertex.position);
    float diffuse_intensity = max(dot(vertex.normal, to_light_source_direction), 0);

    vec3 viewer_direction = normalize(viewer_position - vertex.position);
    vec3 reflect_direction = reflect(-to_light_source_direction, vertex.normal);
    float specular_intensity = pow(max(dot(viewer_direction, reflect_direction), 0), 16);

    float distance = length(light_source.position - vertex.position);
    float attenuation = 1 / (light_source.constant + light_source.linear * distance + light_source.quadiratic * distance * distance);

    float theta = dot(to_light_source_direction, -light_source.direction);
    float epsilon = light_source.inner_cuttoff - light_source.outer_cuttoff;
    float edge_intensity = clamp((theta - light_source.outer_cuttoff) / epsilon, 0, 1);

    gl_FragColor = vec4((AMBIENT_INTENSITY + (diffuse_intensity + specular_intensity) * edge_intensity) * light_source.color * attenuation, 1);
}

void compute_lighting() {
    if (light_source.type == 0) {
        directional();
    } else if (light_source.type == 1) {
        point();
    } else if (light_source.type == 2) {
        spot();
    }
}
