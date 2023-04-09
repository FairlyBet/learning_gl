use std::ffi::c_void;

use gl::types::{GLenum, GLuint};

/// Basic wrapper for a [Vertex Array
/// Object](https://www.khronos.org/opengl/wiki/Vertex_Specification#Vertex_Array_Object).
/// The types of buffer object that you can have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    /// Array Buffers holds arrays of vertex data for drawing.
    ArrayBuffer = gl::ARRAY_BUFFER as isize,
    /// Element Array Buffers hold indexes of what vertexes to use for drawing.
    ElementArrayBuffer = gl::ELEMENT_ARRAY_BUFFER as isize,
}

/// Basic wrapper for a [Buffer
/// Object](https://www.khronos.org/opengl/wiki/Buffer_Object).
pub struct VertexBufferObject(GLuint, GLenum);
impl VertexBufferObject {
    /// Makes a new vertex buffer
    pub fn new(type_: BufferType) -> Option<Self> {
        let mut vbo = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
        }
        if vbo != 0 {
            Some(Self(vbo, type_ as GLenum))
        } else {
            None
        }
    }

    /// Bind this vertex buffer for the given type
    pub fn bind(&self) {
        unsafe { gl::BindBuffer(self.1, self.0) }
    }

    /// Clear the current vertex buffer binding for the given type.
    pub fn unbind(&self) {
        unsafe { gl::BindBuffer(self.1, 0) }
    }

    pub fn buffer_data(&self, data: *const c_void, size: usize, usage: GLenum) {
        unsafe {
            gl::BufferData(self.1, size as isize, data, usage);
        }
    }

    /// Delete buffer
    pub fn delete(self) {
        unsafe { gl::DeleteBuffers(1, &self.0) }
    }
}
