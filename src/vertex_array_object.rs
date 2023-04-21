use gl::types::GLuint;

pub struct VertexArrayObject {
    id: GLuint,
}

impl VertexArrayObject {
    pub fn new() -> Option<Self> {
        let mut id = 0;
        unsafe { gl::GenVertexArrays(1, &mut id) };
        if id != 0 {
            Some(Self { id })
        } else {
            None
        }
    }

    pub fn bind(&self) {
        unsafe { gl::BindVertexArray(self.id) }
    }

    pub fn unbind() {
        unsafe { gl::BindVertexArray(0) }
    }

    fn delete(&self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}

impl Drop for VertexArrayObject {
    fn drop(&mut self) {
        self.delete();
    }
}
