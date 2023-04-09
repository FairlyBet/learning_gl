use crate::{
    shader_program::ShaderProgram, vertex_array_object::VertexArrayObject,
    vertex_buffer_object::VertexBufferObject,
};

pub struct Object<'a> {
    vao: VertexArrayObject,
    program: &'a ShaderProgram,
    draw_fn: &'a fn() -> (),
}

impl<'a> Object<'a> {
    pub fn new(
        data: &Vec<&VertexBufferObject>,
        program: &'a ShaderProgram,
        draw_fn: &'a fn() -> (),
    ) -> Self {
        let vao = VertexArrayObject::new().unwrap();
        vao.bind();
        for buffer in data {
            buffer.bind();
        }
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

    pub fn get_program(&self) -> &ShaderProgram {
        self.program
    }

    pub fn bind(&self) {
        self.vao.bind();
        self.program.use_();
    }

    pub fn draw(&self) {
        (self.draw_fn)();
    }

    pub fn draw_extra<T>(&self, extra: T, f: fn(T, &Object)->())
    {
        f(extra, self);
        self.draw();
    }

    pub fn delete(self) {
        self.vao.delete();
    }
}
