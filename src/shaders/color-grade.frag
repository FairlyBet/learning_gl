#version 420 core

const float GRADE = 8;

void color_grade() {
    ivec3 icolor = ivec3(gl_FragColor.rgb * GRADE * 10);
    icolor /= int(GRADE);
    gl_FragColor = vec4(vec3(icolor) / 10, 1);
}
