use gl::types::{GLboolean, GLenum, GLint, GLsizei, GLuint};
use std::ffi::{c_void, CString};
use std::{fs::File, io::Read, path::Path};

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
        stride: usize,
        pointer: *const c_void,
    ) {
        unsafe {
            gl::VertexAttribPointer(index, size, type_, normalized, stride as GLsizei, pointer);
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

pub struct Shader {
    id: GLuint,
}

impl Shader {
    pub fn new(type_: GLenum) -> Option<Self> {
        let id = unsafe { gl::CreateShader(type_) };
        if id != 0 {
            Some(Self { id })
        } else {
            None
        }
    }

    pub fn get_id(&self) -> GLenum {
        self.id
    }

    pub fn set_source(&self, src: &str) {
        unsafe {
            gl::ShaderSource(
                self.id,
                1,
                &(src.as_bytes().as_ptr().cast()),
                &(src.len().try_into().unwrap()),
            );
        }
    }

    pub fn compile(&self) {
        unsafe { gl::CompileShader(self.id) };
    }

    pub fn compile_success(&self) -> bool {
        let mut compiled = 0;
        unsafe { gl::GetShaderiv(self.id, gl::COMPILE_STATUS, &mut compiled) };
        compiled == i32::from(gl::TRUE)
    }

    pub fn info_log(&self) -> String {
        let mut needed_len = 0;
        unsafe { gl::GetShaderiv(self.id, gl::INFO_LOG_LENGTH, &mut needed_len) };
        let mut v: Vec<u8> = Vec::with_capacity(needed_len.try_into().unwrap());
        let mut len_written = 0_i32;
        unsafe {
            gl::GetShaderInfoLog(
                self.id,
                v.capacity().try_into().unwrap(),
                &mut len_written,
                v.as_mut_ptr().cast(),
            );
            v.set_len(len_written.try_into().unwrap());
        }
        String::from_utf8_lossy(&v).into_owned()
    }

    pub fn delete(self) {
        unsafe { gl::DeleteShader(self.id) };
    }

    pub fn from_source(type_: GLenum, source: &str) -> Result<Self, String> {
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

    pub fn from_file(type_: GLenum, file_name: &str) -> Result<Self, String> {
        let source = Shader::get_src(file_name);
        Shader::from_source(type_, &source)
    }

    fn get_src(file_name: &str) -> String {
        let path = Path::new(file_name);
        let display = path.display();

        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => file,
        };

        let mut source = String::new();
        match file.read_to_string(&mut source) {
            Err(why) => panic!("couldn't read {}: {}", display, why),
            Ok(_) => {}
        }
        source
    }
}

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

    fn delete(&self) {
        unsafe { gl::DeleteBuffers(1, &self.id) }
    }
}

impl Drop for VertexBufferObject {
    fn drop(&mut self) {
        self.delete()
    }
}

// pub struct Texture {
//     id: GLuint,
//     target: GLenum,
// }

// impl Texture {
//     pub fn new(target: GLenum, texture_data: (Info, Data<u8>)) -> Option<Self> {
//         let mut id = 0;

//         let channel = match texture_data.0.components {
//             3 => gl::RGB,
//             4 => gl::RGBA,
//             _ => panic!("Something with channels"),
//         };
//         unsafe {
//             gl::GenTextures(1, &mut id);
//             gl::BindTexture(target, id);
//             gl::TexImage2D(
//                 target,
//                 0,
//                 channel as GLint,
//                 texture_data.0.width,
//                 texture_data.0.height,
//                 0,
//                 channel,
//                 gl::UNSIGNED_BYTE,
//                 texture_data.1.as_slice().as_ptr().cast(),
//             );
//             gl::GenerateMipmap(target);
//             gl::BindTexture(target, 0);
//         }
//         if id != 0 {
//             Some(Self { id, target })
//         } else {
//             None
//         }
//     }

//     pub fn bind(&self) {
//         unsafe {
//             gl::ActiveTexture(gl::TEXTURE0);
//             gl::BindTexture(self.target, self.id);
//         }
//     }

//     pub fn bind_to_unit(&self, unit: GLenum) {
//         unsafe {
//             gl::ActiveTexture(unit);
//             gl::BindTexture(self.target, self.id);
//             gl::ActiveTexture(gl::TEXTURE0);
//         }
//     }

//     pub fn unbind(&self) {
//         unsafe {
//             gl::BindTexture(self.target, 0);
//         }
//     }

//     pub fn parameter(&self, pname: GLenum, param: GLenum) {
//         unsafe {
//             gl::TexParameteri(self.target, pname, param as GLint);
//         }
//     }

//     fn delete(&self) {
//         unsafe {
//             gl::DeleteTextures(1, &self.id);
//         }
//     }

//     pub fn from_file(path: &str) -> Option<Self> {
//         let mut f = File::open(path).unwrap();
//         if let Some(img) = stb::image::stbi_load_from_reader(&mut f, Channels::Default) {
//             Texture::new(gl::TEXTURE_2D, img)
//         } else {
//             panic!("Can't load image")
//         }
//     }
// }

// impl Drop for Texture {
//     fn drop(&mut self) {
//         self.delete();
//     }
// }
