#version 420 core 

void color_grade() {
    ivec3 icolor = ivec3(gl_FragColor.rgb * 100);
    icolor /= 10;
    gl_FragColor = vec4(vec3(icolor) / 10, 1);
}
