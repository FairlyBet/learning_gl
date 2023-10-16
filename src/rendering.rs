use crate::{
    camera::Camera,
    data_3d::{Mesh, Model, VertexAttribute},
    gl_wrappers::{self, BufferObject, Renderbuffer, Shader, ShaderProgram, Texture},
    lighting::{LightObject, LightSource},
    linear::Transform,
};
use gl::types::GLenum;
use nalgebra_glm::{Mat4, Vec3};
use std::{ffi::c_void, mem::size_of, ptr};

enum UniformBufferBinding {
    MatrixData = 0,
    LightingData = 1,
}

#[repr(C)]
pub struct MatrixData {
    mvp: Mat4,
    model: Mat4,
    orientation: Mat4,
    light_space: Mat4,
}

impl MatrixData {
    pub fn new(mvp: Mat4, model: Mat4, orientation: Mat4, light_space: Mat4) -> Self {
        Self {
            mvp,
            model,
            orientation,
            light_space,
        }
    }
}

#[repr(C)]
pub struct LightingData {
    light_source: LightSource,
    viewer_position: Vec3,
}

impl LightingData {
    pub fn new(light_source: LightSource, viewer_position: Vec3) -> Self {
        Self {
            light_source,
            viewer_position,
        }
    }
}

pub struct Framebuffer {
    framebuffer: gl_wrappers::Framebuffer,
    pub sampler_buffer: Texture,
    depth_stencil_buffer: Renderbuffer,
    pub size: (i32, i32),
}

impl Framebuffer {
    pub fn new(size: (i32, i32), mag: GLenum, min: GLenum) -> Self {
        let sampler_buffer = Texture::new(gl::TEXTURE_2D).unwrap();
        sampler_buffer.bind();
        sampler_buffer.texture_data(size, ptr::null(), gl::UNSIGNED_BYTE, gl::RGB, gl::RGB);
        sampler_buffer.parameter(gl::TEXTURE_MIN_FILTER, min);
        sampler_buffer.parameter(gl::TEXTURE_MAG_FILTER, mag);

        let depth_stencil_buffer = Renderbuffer::new(gl::RENDERBUFFER).unwrap();
        depth_stencil_buffer.bind();
        depth_stencil_buffer.buffer_storage(size, gl::DEPTH24_STENCIL8);

        let framebuffer = gl_wrappers::Framebuffer::new(gl::FRAMEBUFFER).unwrap();
        framebuffer.bind();
        framebuffer.attach_texture2d(&sampler_buffer, gl::COLOR_ATTACHMENT0);
        framebuffer.attach_renderbuffer(&depth_stencil_buffer, gl::DEPTH_STENCIL_ATTACHMENT);
        gl_wrappers::Framebuffer::bind_default();

        Self {
            framebuffer,
            sampler_buffer,
            depth_stencil_buffer,
            size,
        }
    }

    pub fn new_shadowmap(size: (i32, i32), mag: GLenum, min: GLenum) -> Self {
        let sampler_buffer = Texture::new(gl::TEXTURE_2D).unwrap();
        sampler_buffer.bind();
        sampler_buffer.texture_data(
            size,
            ptr::null(),
            gl::FLOAT,
            gl::DEPTH_COMPONENT,
            gl::DEPTH_COMPONENT,
        );
        sampler_buffer.parameter(gl::TEXTURE_MIN_FILTER, min);
        sampler_buffer.parameter(gl::TEXTURE_MAG_FILTER, mag);

        let framebuffer = gl_wrappers::Framebuffer::new(gl::FRAMEBUFFER).unwrap();
        framebuffer.bind();
        framebuffer.attach_texture2d(&sampler_buffer, gl::DEPTH_ATTACHMENT);
        unsafe {
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);
        }
        gl_wrappers::Framebuffer::bind_default();

        Self {
            framebuffer,
            sampler_buffer,
            depth_stencil_buffer: Renderbuffer::new(gl::RENDERBUFFER).unwrap(),
            size,
        }
    }

    pub fn bind(&self) {
        self.framebuffer.bind();
        unsafe { gl::Viewport(0, 0, self.size.0, self.size.1) }
    }

    pub fn bind_default(size: (i32, i32)) {
        gl_wrappers::Framebuffer::bind_default();
        unsafe {
            gl::Viewport(0, 0, size.0, size.1);
        }
    }
}

pub struct Canvas {
    pub render_quad: Mesh,
}

impl Canvas {
    pub fn new() -> Self {
        let quad = Mesh::new(
            Mesh::QUAD_VERTICES_TEX_COORDS.len() * size_of::<f32>(),
            Mesh::QUAD_VERTICES_TEX_COORDS.as_ptr().cast(),
            ptr::null() as *const c_void,
            vec![VertexAttribute::Position, VertexAttribute::TexCoord],
            gl::STATIC_DRAW,
            6,
            0,
        );
        Self { render_quad: quad }
    }

    pub fn bind(&self) {
        self.render_quad.bind();
    }
}

pub struct ModelRenderer {
    shader_program: ShaderProgram,
    matrix_data_buffer: BufferObject,
    lighting_data_buffer: BufferObject,
}

impl ModelRenderer {
    pub fn new() -> Self {
        let vertex_shader =
            Shader::from_file(gl::VERTEX_SHADER, "src\\shaders\\main.vert").unwrap();
        let fragment_shader =
            Shader::from_file(gl::FRAGMENT_SHADER, "src\\shaders\\main.frag").unwrap();
        let lighting_shader =
            Shader::from_file(gl::FRAGMENT_SHADER, "src\\shaders\\lighting.frag").unwrap();
        let color_grade_shader =
            Shader::from_file(gl::FRAGMENT_SHADER, "src\\shaders\\color-grade.frag").unwrap();
        let shader_program = ShaderProgram::new().unwrap();
        shader_program.attach_shader(&vertex_shader);
        shader_program.attach_shader(&fragment_shader);
        shader_program.attach_shader(&lighting_shader);
        shader_program.attach_shader(&color_grade_shader);
        shader_program.link();
        println!("{}", shader_program.link_success());

        let matrix_data_buffer = BufferObject::new(gl::UNIFORM_BUFFER).unwrap();
        matrix_data_buffer.bind();
        matrix_data_buffer.buffer_data(
            size_of::<MatrixData>(),
            ptr::null() as *const c_void,
            gl::DYNAMIC_DRAW, // or static?
        );
        let lighting_data_buffer = BufferObject::new(gl::UNIFORM_BUFFER).unwrap();
        lighting_data_buffer.bind();
        lighting_data_buffer.buffer_data(
            size_of::<LightingData>(),
            ptr::null() as *const c_void,
            gl::DYNAMIC_DRAW,
        );
        matrix_data_buffer.bind_buffer_base(UniformBufferBinding::MatrixData as u32);
        lighting_data_buffer.bind_buffer_base(UniformBufferBinding::LightingData as u32);

        Self {
            shader_program,
            matrix_data_buffer,
            lighting_data_buffer,
        }
    }

    pub fn draw(
        &self,
        camera: &Camera,
        transform: &Transform,
        model: &Model,
        light: &mut LightObject,
    ) {
        self.shader_program.use_();

        self.fill_buffers(camera, transform, light);

        for mesh in model.get_meshes() {
            mesh.bind();
            unsafe {
                gl::DrawElements(
                    gl::TRIANGLES,
                    mesh.index_count,
                    gl::UNSIGNED_INT,
                    0 as *const _,
                );
            }
        }
    }

    fn fill_buffers(&self, camera: &Camera, transform: &Transform, light: &mut LightObject) {
        let matrix_data = MatrixData::new(
            camera.projection_view() * transform.model(), // change
            transform.model(),
            glm::quat_to_mat4(&transform.orientation),
            light.lightspace(),
        );
        self.matrix_data_buffer.bind();
        self.matrix_data_buffer.buffer_subdata(
            size_of::<MatrixData>(),
            (&matrix_data as *const MatrixData).cast(),
            0,
        );

        let lighting_data = LightingData::new(light.get_source(), camera.transform.position);
        self.lighting_data_buffer.bind();
        self.lighting_data_buffer.buffer_subdata(
            size_of::<LightingData>(),
            (&lighting_data as *const LightingData).cast(),
            0,
        );
    }

    // pub fn draw_arrays() {
    //     todo!();
    // }
}

pub struct ScreenRenderer {
    shader_program: ShaderProgram,
    gamma_correction_uniform: i32,
    pub gamma: f32,
}

impl ScreenRenderer {
    pub const GAMMA: f32 = 2.2;

    pub fn new() -> Self {
        let shader_program = ShaderProgram::from_vert_frag_file(
            "src\\shaders\\screen.vert",
            "src\\shaders\\screen.frag",
        )
        .unwrap();
        shader_program.use_();
        let uniform = shader_program.get_uniform("gamma_correction");
        Self {
            shader_program,
            gamma: Self::GAMMA,
            gamma_correction_uniform: uniform,
        }
    }

    pub fn draw_texture(&self, canvas: &Canvas, texture: &Texture) {
        unsafe {
            texture.bind();
            canvas.bind();
            self.shader_program.use_();
            gl::Uniform1f(self.gamma_correction_uniform, 1.0 / self.gamma);
            gl::DrawArrays(gl::TRIANGLES, 0, canvas.render_quad.triangle_count);
        }
    }
}
