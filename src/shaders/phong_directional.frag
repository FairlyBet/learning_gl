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
    vec3 direction; // has to be normalized
};

in vec3 normal;
in vec3 position;
in vec2 tex_coord;

uniform vec3 view_position;
uniform Light light;
uniform Material material;

void main() {
    vec3 ambient = light.ambient * texture2D(material.diffuse, tex_coord).rgb;

    float diffuse_intensity = max(dot(normal, -light.direction), 0.0f);
    vec3 diffuse = light.diffuse * diffuse_intensity * texture(material.diffuse, tex_coord).rgb;

    vec3 inv_view_direction = normalize(view_position - position);
    vec3 reflect_direction = (reflect(light.direction, normal));
    float specular_intensity = pow(max(dot(inv_view_direction, reflect_direction), 0.0f), material.shininess);
    vec3 specular = light.specular * specular_intensity * texture2D(material.specular, tex_coord).rgb;

    vec3 frag_color = ambient + diffuse + specular;

    gl_FragColor = vec4(frag_color, 1.0f);
}
