use gl::types::{GLenum, GLint, GLuint};
use stb::image::{Data, Info};

pub struct Texture(GLuint, GLenum);

impl Texture {
    pub fn new(target: GLenum, texture_data: (Info, Data<u8>)) -> Self {
        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(target, id);
            gl::TexImage2D(
                target,
                0,
                gl::RGB as GLint,
                texture_data.0.width,
                texture_data.0.height,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                texture_data.1.as_slice().as_ptr().cast(),
            );
            gl::GenerateMipmap(target);
            gl::BindTexture(target, 0);
        }
        Texture(id, target)
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(self.1, self.0);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(self.1, 0);
        }
    }

    pub fn parameter(&self, pname: GLenum, param: GLenum) {
        self.bind();
        unsafe {
            gl::TexParameteri(self.0, pname, param as GLint);
        }
        self.unbind();
    }

    pub fn delete(self){
        unsafe{
            gl::DeleteTextures(1, &self.0);
        }
    }
}
