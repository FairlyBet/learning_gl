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
uniform sampler2D shadow_map;

in VertexData {
    vec3 position;
    vec3 normal;
    vec2 tex_coord;
    vec3 light_space_position;
} vertex;

const float AMBIENT_INTENSITY = 0.3;
const float SHININESS = 8;

float blinn_specular(vec3 to_ligth_source_direction, float shininess) {
    vec3 to_viewer_direction = normalize(viewer_position - vertex.position);
    vec3 halfway_direction = normalize(to_ligth_source_direction + to_viewer_direction);
    return pow(max(dot(vertex.normal, halfway_direction), 0), shininess);
}

float fragment_luminocity() {
    return 1;
    
    vec3 fragment_ndc = vertex.light_space_position * 0.5 + 0.5;
    float illuminated_depth = texture(shadow_map, fragment_ndc.xy).r;
    
    
    return float(fragment_ndc.z <= illuminated_depth);

}

void directional() {
    float diffuse_intensity = max(dot(vertex.normal, -light_source.direction), 0);
    float specular_intensity = blinn_specular(-light_source.direction, SHININESS);

    gl_FragColor = vec4((AMBIENT_INTENSITY + fragment_luminocity() * (diffuse_intensity + specular_intensity)) * light_source.color, 1);
}

void point() {
    vec3 to_light_source_direction = normalize(light_source.position - vertex.position);
    float diffuse_intensity = max(dot(vertex.normal, to_light_source_direction), 0);
    float specular_intensity = blinn_specular(to_light_source_direction, SHININESS);

    float distance = length(light_source.position - vertex.position);
    float attenuation = 1 / (light_source.constant + light_source.linear * distance + light_source.quadiratic * distance * distance);

    gl_FragColor = vec4((AMBIENT_INTENSITY + fragment_luminocity() * (diffuse_intensity + specular_intensity)) * light_source.color * attenuation, 1);
}

void spot() {
    vec3 to_light_source_direction = normalize(light_source.position - vertex.position);
    float diffuse_intensity = max(dot(vertex.normal, to_light_source_direction), 0);
    float specular_intensity = blinn_specular(to_light_source_direction, SHININESS);

    float distance = length(light_source.position - vertex.position);
    float attenuation = 1 / (light_source.constant + light_source.linear * distance + light_source.quadiratic * distance * distance);

    float theta = dot(to_light_source_direction, -light_source.direction);
    float epsilon = light_source.inner_cuttoff - light_source.outer_cuttoff;
    float edge_intensity = clamp((theta - light_source.outer_cuttoff) / epsilon, 0, 1);

    gl_FragColor = vec4((AMBIENT_INTENSITY + edge_intensity * fragment_luminocity() * (diffuse_intensity + specular_intensity)) * light_source.color * attenuation, 1);
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
