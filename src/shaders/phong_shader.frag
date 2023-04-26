#version 330 core

in vec3 normal; // make sure that normalized
in vec3 position; // interpolated from 3 world-space coodinates that form the triangle

uniform vec3 self_color;
uniform vec3 light_color;
uniform vec3 light_position;
uniform vec3 view_position;

void main() {
    float ambient_intensity = 0.25f;
    vec3 ambient = ambient_intensity * light_color;

    vec3 ray_direction = normalize(light_position - position);
    float diffuse_intensity = clamp(dot(normal, ray_direction), 0.0f, 1.0f);
    vec3 diffuse = diffuse_intensity * light_color;

    float specular_brightness = 0.5f;
    vec3 view_direction = normalize(view_position - position);
    vec3 reflect_direction = reflect(-ray_direction, normal);
    float specular_intensity = pow(max(dot(view_direction, reflect_direction), 0.0), 32);
    vec3 specular = specular_brightness * specular_intensity * light_color;

    vec3 frag_color = (ambient + diffuse + specular) * self_color;

    gl_FragColor = vec4(frag_color, 1.0f);
}
