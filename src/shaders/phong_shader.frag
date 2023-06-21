#version 330 core

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
};

in vec3 normal;
in vec3 position;

uniform vec3 light_color;
uniform vec3 light_position;
uniform vec3 view_position;

uniform Material material;

void main() {

    float ambient_intensity = (0.212671f * material.ambient.r + 0.715160f * material.ambient.g + 0.072169 * material.ambient.b) / (0.212671f * material.diffuse.r + 0.715160f * material.diffuse.g + 0.072169 * material.diffuse.b);
    vec3 ambient = material.ambient;

    vec3 inv_ray_direction = normalize(light_position - position);
    float diffuse_intensity = max(dot(normal, inv_ray_direction), 0.0f);
    vec3 diffuse = diffuse_intensity * material.diffuse;

    vec3 inv_view_direction = normalize(view_position - position);
    vec3 reflect_direction = normalize(reflect(-inv_ray_direction, normal));
    float specular_intensity = pow(max(dot(inv_view_direction, reflect_direction), 0.0f), material.shininess * 128);
    vec3 specular = specular_intensity * material.specular;

    vec3 frag_color = (ambient + diffuse + specular) * light_color;

    gl_FragColor = vec4(frag_color, 1.0f);
}
