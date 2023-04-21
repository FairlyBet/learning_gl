use std::ffi::{c_void, CString};

use crate::shader::Shader;
use gl::types::{GLboolean, GLenum, GLint, GLsizei, GLuint};

#[derive(PartialEq, Eq)]
pub struct ShaderProgram {
    id: GLuint,
}

impl ShaderProgram {
    pub fn new() -> Option<Self> {
        let id = unsafe { gl::CreateProgram() };
        if id != 0 {
            Some(Self { id })
        } else {
            None
        }
    }

    pub fn get_id(&self) -> GLuint {
        self.id
    }

    pub fn get_uniform(&self, name: &str) -> GLint {
        let c_name = CString::new(name).unwrap();
        unsafe { gl::GetUniformLocation(self.id, c_name.as_ptr().cast()) }
    }

    pub fn configure_attribute(
        index: GLuint,
        size: GLint,
        type_: GLenum,
        normalized: GLboolean,
        stride: GLsizei,
        pointer: *const c_void,
    ) {
        unsafe {
            gl::VertexAttribPointer(index, size, type_, normalized, stride, pointer);
        }
    }

    pub fn enable_attribute(index: GLuint) {
        unsafe {
            gl::EnableVertexAttribArray(index);
        }
    }

    pub fn attach_shader(&self, shader: &Shader) {
        unsafe { gl::AttachShader(self.id, shader.get_id()) };
    }

    pub fn link(&self) {
        unsafe { gl::LinkProgram(self.id) };
    }

    pub fn link_success(&self) -> bool {
        let mut success = 0;
        unsafe { gl::GetProgramiv(self.id, gl::LINK_STATUS, &mut success) };
        success == i32::from(gl::TRUE)
    }

    pub fn info_log(&self) -> String {
        let mut needed_len = 0;
        unsafe { gl::GetProgramiv(self.id, gl::INFO_LOG_LENGTH, &mut needed_len) };
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            gl::GetProgramInfoLog(
                self.id,
                v.capacity() as i32,
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    pub fn use_(&self) {
        unsafe { gl::UseProgram(self.id) };
    }

    fn delete(&self) {
        unsafe { gl::DeleteProgram(self.id) };
    }

    pub fn from_vert_frag(vert: &str, frag: &str) -> Result<Self, String> {
        let p = Self::new().ok_or_else(|| "Couldn't allocate a program".to_string())?;
        let v = Shader::from_source(gl::VERTEX_SHADER, vert)
            .map_err(|e| format!("Vertex Compile Error: {}", e))?;
        let f = Shader::from_source(gl::FRAGMENT_SHADER, frag)
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
        let vert = Shader::from_file(gl::VERTEX_SHADER, vert_file_name).unwrap();
        let frag = Shader::from_file(gl::FRAGMENT_SHADER, frag_file_name).unwrap();
        let p = Self::new().ok_or_else(|| "Couldn't allocate a program".to_string())?;
        p.attach_shader(&vert);
        p.attach_shader(&frag);
        p.link();
        vert.delete();
        frag.delete();
        if p.link_success() {
            Ok(p)
        } else {
            let out = format!("Program Link Error: {}", p.info_log());
            p.delete();
            Err(out)
        }
    }

    pub fn unuse() {
        unsafe {
            gl::UseProgram(0);
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        ShaderProgram::unuse();
        self.delete();
    }
}