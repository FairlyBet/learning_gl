pub(crate) const VERTEX_SHADER_SRC: &str = "#version 330 core
layout (location = 0) in vec3 aPos;

void main()
{
    gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
}";

pub(crate) const MONO_COLOR_FRAG_SHDR_SRC: &str = "#version 330 core
in vec3 out_color;
out vec4 FragColor;

void main()
{
    FragColor = vec4(out_color, 1.0);
}";

pub(crate) const VERTEX_SHADER_WITH_COL_SRC: &str = "#version 330 core
layout (location = 0) in vec3 pos;
layout (location = 1) in vec3 col;

out vec3 out_color;

void main()
{
    gl_Position = vec4(pos, 1.0);
    out_color = col;
}";
