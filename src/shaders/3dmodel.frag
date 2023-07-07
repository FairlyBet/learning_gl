#version 330 core

in vec3 global_position;
in vec3 rotated_normal;
in vec2 tex_coord;

uniform vec3 viewer_position;
uniform vec3 light_direction;

void main() {
    float ambient = 0.2;

    float diffuse = max(dot(rotated_normal, -light_direction), 0);

    vec3 viewer_direction = normalize(viewer_position - global_position);
    vec3 reflect_direction = reflect(light_direction, rotated_normal);
    float specular = max(dot(viewer_direction, reflect_direction), 0);
    specular = pow(specular, 32);

    vec3 color = vec3(0.8) * (ambient + diffuse + specular);

    gl_FragColor = vec4(color, 1);
}