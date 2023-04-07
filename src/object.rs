use crate::{vertex_array_object::VertexArrayObject, shader_program::ShaderProgram, vertex_buffer_object::VertexBufferObject};

pub struct Object<'a> {
    vao: VertexArrayObject,
    program: &'a ShaderProgram,
    draw_fn: &'a fn() -> (),
}

impl<'a> Object<'a> {
    pub fn new(
        data: &Vec<&VertexBufferObject>,
        program: &'a ShaderProgram,
        attrib_fn: &fn() -> (),
        draw_fn: &'a fn() -> (),
    ) -> Self {
        let vao = VertexArrayObject::new().unwrap();
        vao.bind();
        for buffer in data {
            buffer.bind();
        }
        attrib_fn();
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
