use crate::{
    shader_program::ShaderProgram, texture::Texture, vertex_array_object::VertexArrayObject,
    vertex_buffer_object::VertexBufferObject,
};

pub struct Renderer<'a> {
    vao: VertexArrayObject,
    texture: &'a Option<Texture>,
    shader_program: &'a ShaderProgram,
    draw_fn: &'a fn() -> (),
}

impl<'a> Renderer<'_> {
    pub fn new(
        buffer: &'a VertexBufferObject,
        texture: &'a Option<Texture>,
        shader_program: &'a ShaderProgram,
        draw_fn: &'a fn() -> (),
    ) -> Renderer<'a> {
        let vao = VertexArrayObject::new().unwrap();
        vao.bind();

        buffer.bind();

        VertexArrayObject::unbind();

        Renderer {
            vao,
            shader_program,
            texture,
            draw_fn,
        }
    }

    pub fn bind(&self) {
        self.vao.bind();
        if let Some(texture) = self.texture {
            texture.bind();
        }
        self.shader_program.use_();
    }

    pub fn get_shader_program(&self) -> &ShaderProgram {
        self.shader_program
    }

    pub fn get_texture(&self) -> &Option<Texture> {
        self.texture
    }
    pub fn draw(&self) {
        (self.draw_fn)();
    }
}
