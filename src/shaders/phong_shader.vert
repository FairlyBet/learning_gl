#version 330 core

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec2 in_tex_coord;

out vec3 position;
out vec3 normal;
out vec2 tex_coord;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main() {
    gl_Position = projection * view * model * vec4(in_position, 1.0f);

    position = vec3(model * vec4(in_position, 1.0f));
    normal = mat3(transpose(inverse(model))) * in_normal; // removes non-uniform scaling affect, has to be done on cpu
    normal = normalize(normal); // must be normalized
    tex_coord = in_tex_coord;
}
