use crate::shader::{Shader, ShaderType};
use gl::types::GLuint;

/// A handle to a [Program
/// Object](https://www.khronos.org/opengl/wiki/GLSL_Object#Program_objects)
pub struct ShaderProgram(GLuint);

impl ShaderProgram {
    /// Allocates a new program object.
    ///
    /// Prefer [`ShaderProgram::from_vert_frag`](ShaderProgram::from_vert_frag),
    /// it makes a complete program from the vertex and fragment sources all at
    /// once.
    pub fn new() -> Option<Self> {
        let prog = unsafe { gl::CreateProgram() };
        if prog != 0 {
            Some(Self(prog))
        } else {
            None
        }
    }

    /// Returns shaders ID given by OpenGL
    pub fn get_id(&self) -> u32 {
        self.0
    }

    /// Attaches a shader object to this program object.
    pub fn attach_shader(&self, shader: &Shader) {
        unsafe { gl::AttachShader(self.0, shader.get_id()) };
    }

    /// Links the various attached, compiled shader objects into a usable program.
    pub fn link(&self) {
        unsafe { gl::LinkProgram(self.0) };
    }

    /// Checks if the last linking operation was successful.
    pub fn link_success(&self) -> bool {
        let mut success = 0;
        unsafe { gl::GetProgramiv(self.0, gl::LINK_STATUS, &mut success) };
        success == i32::from(gl::TRUE)
    }

    /// Gets the log data for this program.
    ///
    /// This is usually used to check the message when a program failed to link.
    pub fn info_log(&self) -> String {
        let mut needed_len = 0;
        unsafe { gl::GetProgramiv(self.0, gl::INFO_LOG_LENGTH, &mut needed_len) };
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            gl::GetProgramInfoLog(
                self.0,
                v.capacity() as i32,
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    /// Sets the program as the program to use when drawing.
    pub fn use_(&self) {
        unsafe { gl::UseProgram(self.0) };
    }

    /// Marks the program for deletion.
    ///
    /// Note: This _does not_ immediately delete the program. If the program is
    /// currently in use it won't be deleted until it's not the active program.
    /// When a program is finally deleted and attached shaders are unattached.
    pub fn delete(self) {
        unsafe { gl::DeleteProgram(self.0) };
    }

    /// Takes a vertex shader source string and a fragment shader source string
    /// and either gets you a working program object or gets you an error message.
    ///
    /// This is the preferred way to create a simple shader program in the common
    /// case. It's just less error prone than doing all the steps yourself.
    pub fn from_vert_frag(vert: &str, frag: &str) -> Result<Self, String> {
        let p = Self::new().ok_or_else(|| "Couldn't allocate a program".to_string())?;
        let v = Shader::from_source(ShaderType::VertexShader, vert)
            .map_err(|e| format!("Vertex Compile Error: {}", e))?;
        let f = Shader::from_source(ShaderType::FragmentShader, frag)
            .map_err(|e| format!("Fragment Compile Error: {}", e))?;
        p.attach_shader(&v);
        p.attach_shader(&f);
        p.link();
        v.delete();
        f.delete();
        if p.link_success() {
            Ok(p)
        } else {
            let out = format!("Program Link Error: {}", p.info_log());
            p.delete();
            Err(out)
        }
    }

    pub fn from_vert_frag_file(vert_file_name: &str, frag_file_name: &str) -> Result<Self, String> {
        let vert = Shader::get_src(vert_file_name);
        let frag = Shader::get_src(frag_file_name);
        ShaderProgram::from_vert_frag(vert.as_str(), frag.as_str())
    }
}
