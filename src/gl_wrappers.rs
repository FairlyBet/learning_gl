use gl::types::{GLboolean, GLenum, GLint, GLsizei, GLuint};
// use image::DynamicImage;
use std::{
    ffi::{c_void, CString},
    fs::File,
    io::Read,
    path::Path,
};

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

    pub fn get_uniform(&self, name: &str) -> GLint {
        let c_name = CString::new(name).unwrap();
        unsafe { gl::GetUniformLocation(self.id, c_name.as_ptr().cast()) }
    }

    pub fn attach_shader(&self, shader: &Shader) {
        unsafe { gl::AttachShader(self.id, shader.id) };
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

    pub fn from_vert_frag_src(vert_src: &str, frag_src: &str) -> Result<Self, String> {
        let vert = Shader::from_source(gl::VERTEX_SHADER, vert_src)
            .map_err(|e| format!("Vertex Compile Error: {}", e))?;
        let frag = Shader::from_source(gl::FRAGMENT_SHADER, frag_src)
            .map_err(|e| format!("Fragment Compile Error: {}", e))?;

        ShaderProgram::from_vert_frag(vert, frag)
    }

    pub fn from_vert_frag_file(vert_file_name: &str, frag_file_name: &str) -> Result<Self, String> {
        let vert = Shader::from_file(gl::VERTEX_SHADER, vert_file_name)
            .map_err(|e| format!("Vertex Compile Error: {}", e))?;
        let frag = Shader::from_file(gl::FRAGMENT_SHADER, frag_file_name)
            .map_err(|e| format!("Fragment Compile Error: {}", e))?;

        ShaderProgram::from_vert_frag(vert, frag)
    }

    pub fn from_vert_frag(vert: Shader, frag: Shader) -> Result<Self, String> {
        let program = Self::new().ok_or_else(|| "Couldn't allocate a program".to_string())?;
        program.attach_shader(&vert);
        program.attach_shader(&frag);
        program.link();
        if program.link_success() {
            Ok(program)
        } else {
            let out = format!("Program Link Error: {}", program.info_log());
            Err(out)
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id) };
    }
}

pub struct Shader {
    pub id: GLuint,
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

    pub fn from_source(type_: GLenum, source: &str) -> Result<Self, String> {
        let shader = Self::new(type_).ok_or_else(|| "Couldn't allocate new shader".to_string())?;
        shader.set_source(source);
        shader.compile();
        if shader.compile_success() {
            Ok(shader)
        } else {
            let out = shader.info_log();
            Err(out)
        }
    }

    pub fn from_file(type_: GLenum, file_name: &str) -> Result<Self, String> {
        let source = Shader::read_source(file_name);
        Shader::from_source(type_, &source)
    }

    fn read_source(file_name: &str) -> String {
        let path = Path::new(file_name);
        let display = path.display();

        let mut file = match File::open(&path) {
            Err(why) => panic!("Couldn't open {}: {}", display, why),
            Ok(file) => file,
        };

        let mut source = String::new();
        match file.read_to_string(&mut source) {
            Err(why) => panic!("Couldn't read {}: {}", display, why),
            Ok(_) => {}
        }

        source
    }
}

// impl Drop for Shader {
//     fn drop(&mut self) {
//         unsafe { gl::DeleteShader(self.id) };
//     }
// }

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
}

impl Drop for VertexArrayObject {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}

pub struct BufferObject {
    id: GLuint,
    target: GLenum,
}

impl BufferObject {
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

    pub fn unbind(target: GLenum) {
        unsafe { gl::BindBuffer(target, 0) }
    }

    pub fn buffer_data(&self, size: usize, data: *const c_void, usage: GLenum) {
        unsafe {
            gl::BufferData(self.target, size as isize, data, usage);
        }
    }

    pub fn buffer_subdata(&self, size: usize, data: *const c_void, offset: u32) {
        unsafe {
            gl::BufferSubData(self.target, offset as isize, size as isize, data);
        }
    }

    pub fn bind_buffer_base(&self, index: GLuint) {
        unsafe {
            gl::BindBufferBase(self.target, index, self.id);
        }
    }
}

impl Drop for BufferObject {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &self.id) }
    }
}

pub struct Texture {
    pub id: GLuint,
    target: GLenum,
}

impl Texture {
    // pub fn from_bytes2d(size: (u32, u32), data: *const u8, format: GLenum) -> Option<Self> {
    //     Self::new(
    //         gl::TEXTURE_2D,
    //         size,
    //         data.cast(),
    //         gl::UNSIGNED_BYTE,
    //         format,
    //         format as i32,
    //     )
    // }

    pub fn new(target: GLenum) -> Option<Self> {
        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
        }
        if id != 0 {
            Some(Self { id, target })
        } else {
            None
        }
    }

    pub fn texture_data(
        &self,
        size: (i32, i32),
        data: *const c_void,
        type_: GLenum,
        format: GLenum,
        internal_format: GLenum,
    ) {
        unsafe {
            gl::TexImage2D(
                self.target,
                0,
                internal_format as i32,
                size.0,
                size.1,
                0,
                format,
                type_,
                data,
            );
        }
    }

    pub fn generate_mipmaps(&self) {
        unsafe {
            gl::GenerateMipmap(self.target);
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(self.target, self.id);
        }
    }

    pub fn bind_to_unit(&self, unit: GLenum) {
        unsafe {
            gl::ActiveTexture(unit);
            gl::BindTexture(self.target, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(self.target, 0);
        }
    }

    pub fn parameter(&self, pname: GLenum, param: GLenum) {
        unsafe {
            gl::TexParameteri(self.target, pname, param as GLint);
        }
    }

    // pub fn from_file(path: &str) -> Option<Self> {
    //     let image = image::open(path).unwrap();
    //     let bytes = image.as_bytes();
    //     let size = (image.width(), image.height());
    //     let channel: GLenum;
    //     if let DynamicImage::ImageRgba8(_) = image {
    //         channel = gl::RGBA;
    //     } else {
    //         channel = gl::RGB;
    //     }
    //     Self::from_bytes2d(size, bytes.as_ptr(), channel)
    // }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

pub struct Framebuffer {
    id: GLuint,
    target: GLenum,
}

impl Framebuffer {
    pub fn new(target: GLenum) -> Option<Self> {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut id);
        }
        if id != 0 {
            Some(Self { id, target })
        } else {
            None
        }
    }

    pub fn attach_texture2d(&self, texture: &Texture, attachment: GLenum) {
        unsafe {
            gl::FramebufferTexture2D(self.target, attachment, texture.target, texture.id, 0);
        }
    }

    pub fn attach_renderbuffer(&self, renderbuffer: &Renderbuffer, attachment: GLenum) {
        unsafe {
            gl::FramebufferRenderbuffer(
                self.target,
                attachment,
                renderbuffer.target,
                renderbuffer.id,
            );
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(self.target, self.id);
        }
    }

    pub fn bind_default() {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id);
        }
    }
}

pub struct Renderbuffer {
    id: GLuint,
    target: GLenum,
}

impl Renderbuffer {
    pub fn new(target: GLenum) -> Option<Self> {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenRenderbuffers(1, &mut id);
        }
        if id != 0 {
            Some(Self { id, target })
        } else {
            None
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindRenderbuffer(self.target, self.id);
        }
    }

    pub fn buffer_storage(&self, size: (i32, i32), internal_format: GLenum) {
        unsafe {
            gl::RenderbufferStorage(self.target, internal_format, size.0, size.1);
        }
    }
}

impl Drop for Renderbuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteRenderbuffers(1, &self.id);
        }
    }
}
