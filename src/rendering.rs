use crate::{
    data3d::{self, Mesh, VertexAttribute},
    gl_wrappers::{self, BufferObject, Renderbuffer, Shader, ShaderProgram, Texture},
    lighting::LightData,
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
    pub mvp: Mat4,
    pub model: Mat4,
    pub orientation: Mat4,
    pub light_space: Mat4,
}

#[repr(C)]
pub struct LightingData {
    pub light_source: LightData,
    pub viewer_position: Vec3,
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

// pub struct ScreenQuad {
//     pub render_quad: Mesh,
// }

// impl ScreenQuad {
//     pub fn new() -> Self {
//         let quad = Mesh::new(
//             data3d::QUAD_VERTICES_TEX_COORDS.len() * size_of::<f32>(),
//             data3d::QUAD_VERTICES_TEX_COORDS.as_ptr().cast(),
//             ptr::null(),
//             vec![VertexAttribute::Position, VertexAttribute::TexCoord],
//             gl::STATIC_DRAW,
//             6,
//             0,
//         );
//         Self { render_quad: quad }
//     }

//     pub fn bind(&self) {
//         self.render_quad.bind();
//     }
// }

// pub struct RenderProgram {
//     shader_program: ShaderProgram,
//     matrix_data_buffer: BufferObject,
//     lighting_data_buffer: BufferObject,
// }

// impl RenderProgram {
//     pub fn new(shader_program: ShaderProgram) -> Self {
//         let matrix_data_buffer = BufferObject::new(gl::UNIFORM_BUFFER).unwrap();
//         matrix_data_buffer.bind();
//         matrix_data_buffer.buffer_data(
//             size_of::<MatrixData>(),
//             ptr::null() as *const c_void,
//             gl::DYNAMIC_DRAW, // or static?
//         );
//         let lighting_data_buffer = BufferObject::new(gl::UNIFORM_BUFFER).unwrap();
//         lighting_data_buffer.bind();
//         lighting_data_buffer.buffer_data(
//             size_of::<LightingData>(),
//             ptr::null() as *const c_void,
//             gl::DYNAMIC_DRAW,
//         );
//         matrix_data_buffer.bind_buffer_base(UniformBufferBinding::MatrixData as u32);
//         lighting_data_buffer.bind_buffer_base(UniformBufferBinding::LightingData as u32);

//         Self {
//             shader_program,
//             matrix_data_buffer,
//             lighting_data_buffer,
//         }
//     }

//     // pub fn draw(
//     //     &self,
//     //     camera: &Camera,
//     //     transform: &Transform,
//     //     model: &ModelIndex,
//     //     light: &mut LightSource,
//     // ) {
//     //     self.shader_program.use_();
//     //     self.fill_buffers(camera, transform, light);
//     //     for mesh in model.get_meshes() {
//     //         mesh.bind();
//     //         unsafe {
//     //             gl::DrawElements(
//     //                 gl::TRIANGLES,
//     //                 mesh.index_count,
//     //                 gl::UNSIGNED_INT,
//     //                 0 as *const _,
//     //             );
//     //         }
//     //     }
//     // }

//     // fn fill_buffers(&self, camera: &Camera, transform: &Transform, light: &mut LightSource) {
//     //     let matrix_data = MatrixData::new(
//     //         camera.projection_view() * transform.model(), // change
//     //         transform.model(),
//     //         glm::quat_to_mat4(&transform.orientation),
//     //         light.lightspace(),
//     //     );
//     //     self.matrix_data_buffer.bind();
//     //     self.matrix_data_buffer.buffer_subdata(
//     //         size_of::<MatrixData>(),
//     //         (&matrix_data as *const MatrixData).cast(),
//     //         0,
//     //     );

//     //     let lighting_data =
//     //         LightingData::new(light.get_data(), unsafe { (*camera.transform).position });
//     //     self.lighting_data_buffer.bind();
//     //     self.lighting_data_buffer.buffer_subdata(
//     //         size_of::<LightingData>(),
//     //         (&lighting_data as *const LightingData).cast(),
//     //         0,
//     //     );
//     // }

//     // pub fn draw_arrays() {
//     //     todo!();
//     // }
// }

// impl Default for RenderProgram {
//     fn default() -> Self {
//         let vertex_shader =
//             Shader::from_file(gl::VERTEX_SHADER, "src\\shaders\\main.vert").unwrap();
//         let fragment_shader =
//             Shader::from_file(gl::FRAGMENT_SHADER, "src\\shaders\\main.frag").unwrap();
//         let lighting_shader =
//             Shader::from_file(gl::FRAGMENT_SHADER, "src\\shaders\\lighting.frag").unwrap();
//         let shader_program = ShaderProgram::new().unwrap();
//         shader_program.attach_shader(&vertex_shader);
//         shader_program.attach_shader(&fragment_shader);
//         shader_program.attach_shader(&lighting_shader);
//         shader_program.link();
//         Self::new(shader_program)
//     }
// }

// pub struct ScreenQuadRenderer {
//     shader_program: ShaderProgram,
//     gamma_correction_uniform: i32,
//     pub gamma: f32,
// }

// impl ScreenQuadRenderer {
//     const DEFAULT_GAMMA: f32 = 2.2;
//     const GAMMA_CORRECTION_NAME: &str = "gamma_correction";

//     pub fn new() -> Self {
//         let shader_program = ShaderProgram::from_vert_frag_file(
//             "src\\shaders\\screen.vert",
//             "src\\shaders\\screen.frag",
//         )
//         .unwrap();
//         shader_program.use_();
//         let uniform = shader_program.get_uniform(Self::GAMMA_CORRECTION_NAME);
//         Self {
//             shader_program,
//             gamma: Self::DEFAULT_GAMMA,
//             gamma_correction_uniform: uniform,
//         }
//     }

//     pub fn draw_texture(&self, quad: &ScreenQuad, texture: &Texture) {
//         texture.bind();
//         quad.bind();
//         self.shader_program.use_();
//         unsafe {
//             gl::Uniform1f(self.gamma_correction_uniform, 1.0 / self.gamma);
//             gl::DrawArrays(gl::TRIANGLES, 0, quad.render_quad.triangle_count);
//         }
//     }
// }

// pub struct RenderPipeline {
//     offscreen_buffer: Framebuffer,
//     render_program: RenderProgram,
//     screen_quad: ScreenQuad,
//     screen_quad_renderer: ScreenQuadRenderer,
//     match_window_size: bool,
//     main_framebuffer_size: (i32, i32),
// }

// impl RenderPipeline {
//     pub fn new(framebuffer_size: (i32, i32)) -> Self {
//         let offscreen_buffer = Framebuffer::new(framebuffer_size, gl::LINEAR, gl::LINEAR);
//         let render_program = Default::default();
//         let screen_quad = ScreenQuad::new();
//         let screen_quad_renderer = ScreenQuadRenderer::new();
//         Self::setup();
//         Self {
//             offscreen_buffer,
//             render_program,
//             screen_quad,
//             screen_quad_renderer,
//             match_window_size: true,
//             main_framebuffer_size: framebuffer_size,
//         }
//     }

//     fn setup() {
//         unsafe {
//             gl::ClearColor(0.0, 0.0, 0.0, 1.0);
//             gl::Enable(gl::DEPTH_TEST);
//             gl::Enable(gl::CULL_FACE);
//         }
//     }

//     pub fn draw_cycle(&self) {
//         self.offscreen_buffer.bind();
//         unsafe {
//             gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
//         }
//         // draw
//         Framebuffer::bind_default(self.main_framebuffer_size);
//         self.screen_quad.bind();
//         unsafe {
//             gl::Clear(gl::COLOR_BUFFER_BIT);
//         }
//         self.screen_quad_renderer
//             .draw_texture(&self.screen_quad, &self.offscreen_buffer.sampler_buffer);
//     }

//     pub fn on_framebuffer_size(&mut self, size: (i32, i32)) {
//         self.main_framebuffer_size = size;
//         if self.match_window_size {
//             self.offscreen_buffer = Framebuffer::new(size, gl::LINEAR, gl::LINEAR);
//         }
//     }
// }
