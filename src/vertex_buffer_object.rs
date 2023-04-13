use gl::types::{GLenum, GLuint};
use std::ffi::c_void;

pub struct VertexBufferObject {
    id: GLuint,
    target: GLenum,
}

impl VertexBufferObject {
    pub fn new(target: GLenum) -> Option<Self> {
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        if id != 0 {
            Some(Self { id, target })
        } else {
            None
        }
    }

    pub fn bind(&self) {
        unsafe { gl::BindBuffer(self.target, self.id) }
    }

    pub fn unbind(&self) {
        unsafe { gl::BindBuffer(self.target, 0) }
    }

    pub fn buffer_data(&self, size: usize, data: *const c_void, usage: GLenum) {
        unsafe {
            gl::BufferData(self.target, size as isize, data, usage);
        }
    }

    pub fn delete(self) {
        unsafe { gl::DeleteBuffers(1, &self.id) }
    }
}
