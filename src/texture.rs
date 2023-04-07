use gl::types::{GLenum, GLint, GLuint};
use stb::image::{Data, Info};

pub struct Texture {
    id: GLuint,
    target: GLenum,
}

impl Texture {
    pub fn new(target: GLenum, texture_data: (Info, Data<u8>)) -> Self {
        let mut id = 0;
        let channel = match texture_data.0.components {
            3 => gl::RGB,
            4 => gl::RGBA,
            _ => panic!("something with channels"),
        };
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(target, id);
            gl::TexImage2D(
                target,
                0,
                channel as GLint,
                texture_data.0.width,
                texture_data.0.height,
                0,
                channel,
                gl::UNSIGNED_BYTE,
                texture_data.1.as_slice().as_ptr().cast(),
            );
            gl::GenerateMipmap(target);
            gl::BindTexture(target, 0);
        }
        Texture { id, target }
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

    pub fn delete(self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
