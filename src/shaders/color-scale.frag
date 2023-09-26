#version 420 core 

vec3 color_scale(vec3 frag_color) {
    int red = int(frag_color.r * 100);
    int green = int(frag_color.g * 100);
    int blue = int(frag_color.b * 100);
    red /= 10;
    green /= 10;
    blue /= 10;
    frag_color = vec3(red, green, blue) / 10;
    return frag_color;
}
