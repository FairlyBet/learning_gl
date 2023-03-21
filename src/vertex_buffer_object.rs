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
pub struct Buffer(pub GLuint);
impl Buffer {
    /// Makes a new vertex buffer
    pub fn new() -> Option<Self> {
        let mut vbo = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
        }
        if vbo != 0 {
            Some(Self(vbo))
        } else {
            None
        }
    }

    /// Bind this vertex buffer for the given type
    pub fn bind(&self, type_: BufferType) {
        unsafe { gl::BindBuffer(type_ as GLenum, self.0) }
    }

    /// Clear the current vertex buffer binding for the given type.
    pub fn clear_binding(ty: BufferType) {
        unsafe { gl::BindBuffer(ty as GLenum, 0) }
    }

    /// Delete buffer
    pub fn delete(self) {
        unsafe { gl::DeleteBuffers(1, &self.0) }
    }
}
