use gl::types::{GLenum, GLuint};
use std::{fs::File, io::Read, path::Path};

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
