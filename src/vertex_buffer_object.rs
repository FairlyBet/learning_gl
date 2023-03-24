use gl::types::{GLenum, GLuint};

/// Basic wrapper for a [Vertex Array
/// Object](https://www.khronos.org/opengl/wiki/Vertex_Specification#Vertex_Array_Object).
/// The types of buffer object that you can have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferType {
    /// Array Buffers holds arrays of vertex data for drawing.
    ArrayBuffer = gl::ARRAY_BUFFER,
    /// Element Array Buffers hold indexes of what vertexes to use for drawing.
    ElementArrayBuffer = gl::ELEMENT_ARRAY_BUFFER,
}

/// Basic wrapper for a [Buffer
/// Object](https://www.khronos.org/opengl/wiki/Buffer_Object).
pub struct VertexBufferObject(pub GLuint);
impl VertexBufferObject {
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
        unsafe { gl::BindBuffer(type_, self.0) }
    }

    /// Clear the current vertex buffer binding for the given type.
    pub fn clear_binding(type_: BufferType) {
        unsafe { gl::BindBuffer(type_  , 0) }
    }

    /// Delete buffer
    pub fn delete(self) {
        unsafe { gl::DeleteBuffers(1, &self.0) }
    }
}
