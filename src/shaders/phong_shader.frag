#version 330 core

struct Material {
    sampler2D diffuse;
    sampler2D specular;
    float shininess;
};

struct Light {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    vec3 position;
};

in vec3 normal;
in vec3 position;
in vec2 tex_coord;

uniform vec3 view_position;
uniform Light light;
uniform Material material;

void main() {
    vec3 ambient = light.ambient * vec3(texture2D(material.diffuse, tex_coord));

    vec3 inv_ray_direction = normalize(light.position - position);
    float diffuse_intensity = max(dot(normal, inv_ray_direction), 0.0f);
    vec3 diffuse = light.diffuse * diffuse_intensity * vec3(texture(material.diffuse, tex_coord));

    vec3 inv_view_direction = normalize(view_position - position);
    vec3 reflect_direction = normalize(reflect(-inv_ray_direction, normal));
    float specular_intensity = pow(max(dot(inv_view_direction, reflect_direction), 0.0f), material.shininess);
    vec3 specular = light.specular * specular_intensity * vec3(texture2D(material.specular, tex_coord));

    vec3 frag_color = ambient + diffuse + specular;

    gl_FragColor = vec4(frag_color, 1.0f);
}
