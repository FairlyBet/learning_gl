#version 330 core

in vec3 global_position;
in vec3 rotated_normal;
in vec2 tex_coord;

uniform vec3 viewer_position;
uniform vec3 light_direction;
uniform vec3 light_color;

uniform sampler2D diffuse;
uniform sampler2D specular;

void main() {
    float ambient_intensity = 0.25;

    float diffuse_intensity = max(dot(rotated_normal, -light_direction), 0.0);

    vec3 viewer_direction = normalize(viewer_position - global_position);
    vec3 reflect_direction = reflect(light_direction, rotated_normal);
    float specular_intensity = max(dot(viewer_direction, reflect_direction), 0.0);
    specular_intensity = pow(specular_intensity, 32);

    vec3 color = ((ambient_intensity + diffuse_intensity) * texture2D(diffuse, tex_coord).rgb + specular_intensity * texture2D(specular, tex_coord).rgb) * light_color;

    gl_FragColor = vec4(color, 1.0);
}