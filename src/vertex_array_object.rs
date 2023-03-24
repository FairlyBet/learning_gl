use gl::types::GLuint;

/// Basic wrapper for a [Vertex Array
/// Object](https://www.khronos.org/opengl/wiki/Vertex_Specification#Vertex_Array_Object).
pub struct VertexArrayObject(GLuint);

impl VertexArrayObject {
    /// Creates a new vertex array object
    pub fn new() -> Option<Self> {
        let mut vao = 0;
        unsafe { gl::GenVertexArrays(1, &mut vao) };
        if vao != 0 {
            Some(Self(vao))
        } else {
            None
        }
    }

    /// Bind this vertex array as the current vertex array object
    pub fn bind(&self) {
        unsafe { gl::BindVertexArray(self.0) }
    }

    /// Clear the current vertex array object binding.
    pub fn clear_binding() {
        unsafe { gl::BindVertexArray(0) }
    }

    /// Delete vertex array
    pub fn delete(self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.0);
        }
    }
}
