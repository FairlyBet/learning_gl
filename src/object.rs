use crate::{vertex_array_object::VertexArrayObject, shader_program::ShaderProgram, vertex_buffer_object::VertexBufferObject};

pub struct Object<'a> {
    vao: VertexArrayObject,
    program: &'a ShaderProgram,
    draw_fn: fn() -> (),
}

impl<'a> Object<'a> {
    pub fn new(
        data: &Vec<&VertexBufferObject>,
        program: &'a ShaderProgram,
        attribute_configurer: &fn() -> (),
        draw_fn: fn() -> (),
    ) -> Self {
        let vao = VertexArrayObject::new().unwrap();
        vao.bind();
        for buffer in data {
            buffer.bind();
        }
        attribute_configurer();
        VertexArrayObject::clear_binding();
        for buffer in data {
            buffer.unbind();
        }
        Object {
            vao,
            program,
            draw_fn,
        }
    }

    pub fn bind(&self) {
        self.vao.bind();
        self.program.use_();
    }

    pub fn draw(&self) {
        (self.draw_fn)();
    }

    pub fn delete(self) {
        self.vao.delete();
    }
}
