pub(crate) const VERTEX_SHADER_SRC: &str = "#version 330 core
layout (location = 0) in vec3 aPos;

void main()
{
    gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
}";

pub(crate) const MONO_COLOR_FRAG_SHDR_SRC: &str = "#version 330 core
out vec4 FragColor;
uniform vec4 inputColor;

void main()
{
    FragColor = inputColor;
}";
