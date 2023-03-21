use gl::types::{GLenum, GLuint};

/// The types of shader object.
pub enum ShaderType {
    /// Vertex shaders determine the position of geometry within the screen.
    VertexShader = gl::VERTEX_SHADER as isize,
    /// Fragment shaders determine the color output of geometry.
    ///
    /// Also other values, but mostly color.
    FragmentShader = gl::FRAGMENT_SHADER as isize,
}

/// A handle to a [Shader
/// Object](https://www.khronos.org/opengl/wiki/GLSL_Object#Shader_objects)
pub struct Shader(GLuint);

impl Shader {
    /// Makes a new shader.
    ///
    /// Prefer the [`Shader::from_source`](Shader::from_source) method.
    ///
    /// Possibly skip the direct creation of the shader object and use
    /// [`ShaderProgram::from_vert_frag`](ShaderProgram::from_vert_frag).
    pub fn new(type_: ShaderType) -> Option<Self> {
        let shader = unsafe { gl::CreateShader(type_ as GLenum) };
        if shader != 0 {
            Some(Self(shader))
        } else {
            None
        }
    }

    /// Returns shaders ID given by OpenGL
    pub fn get_id(&self) -> GLenum {
        self.0
    }

    /// Assigns a source string to the shader.
    ///
    /// Replaces any previously assigned source.
    pub fn set_source(&self, src: &str) {
        unsafe {
            gl::ShaderSource(
                self.0,
                1,
                &(src.as_bytes().as_ptr().cast()),
                &(src.len().try_into().unwrap()),
            );
        }
    }

    /// Compiles the shader based on the current source.
    pub fn compile(&self) {
        unsafe { gl::CompileShader(self.0) };
    }

    /// Checks if the last compile was successful or not.
    pub fn compile_success(&self) -> bool {
        let mut compiled = 0;
        unsafe { gl::GetShaderiv(self.0, gl::COMPILE_STATUS, &mut compiled) };
        compiled == i32::from(gl::TRUE)
    }

    /// Gets the info log for the shader.
    ///
    /// Usually you use this to get the compilation log when a compile failed.
    pub fn info_log(&self) -> String {
        let mut needed_len = 0;
        unsafe { gl::GetShaderiv(self.0, gl::INFO_LOG_LENGTH, &mut needed_len) };
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            gl::GetShaderInfoLog(
                self.0,
                v.capacity().try_into().unwrap(),
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    /// Marks a shader for deletion.
    ///
    /// Note: This _does not_ immediately delete the shader. It only marks it for
    /// deletion. If the shader has been previously attached to a program then the
    /// shader will stay allocated until it's unattached from that program.
    pub fn delete(self) {
        unsafe { gl::DeleteShader(self.0) };
    }

    /// Takes a shader type and source string and produces either the compiled
    /// shader or an error message.
    ///
    /// Prefer [`ShaderProgram::from_vert_frag`](ShaderProgram::from_vert_frag),
    /// it makes a complete program from the vertex and fragment sources all at
    /// once.
    pub fn from_source(type_: ShaderType, source: &str) -> Result<Self, String> {
        let id = Self::new(type_).ok_or_else(|| "Couldn't allocate new shader".to_string())?;
        id.set_source(source);
        id.compile();
        if id.compile_success() {
            Ok(id)
        } else {
            let out = id.info_log();
            id.delete();
            Err(out)
        }
    }
}
