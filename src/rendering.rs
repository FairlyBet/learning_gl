use crate::{
    data3d::{Mesh, ModelContainer},
    entity_system::{CameraComponent, EntitySystem, LightComponent, MeshComponent},
    gl_wrappers::{self, BufferObject, Renderbuffer, ShaderProgram, Texture},
    lighting::LightData,
    shader::{self, DefaultLightShader, MainFragmentShader, MainShader, MainVertexShader},
};
use gl::types::GLenum;
use glfw::Version;
use memoffset::offset_of;
use nalgebra_glm::{Mat4, Vec3};
use std::{mem::size_of, ptr};

enum BufferBindingPoint {
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
    pub light_data: LightData,
    pub viewer_position: Vec3,
}

fn matrix_data_buffer() -> BufferObject {
    let buffer = BufferObject::new(gl::UNIFORM_BUFFER).unwrap();
    buffer.bind();
    buffer.buffer_data(size_of::<MatrixData>(), ptr::null(), gl::DYNAMIC_DRAW);
    buffer.bind_buffer_base(BufferBindingPoint::MatrixData as u32);
    buffer
}

fn lighting_data_buffer() -> BufferObject {
    let buffer = BufferObject::new(gl::UNIFORM_BUFFER).unwrap();
    buffer.bind();
    buffer.buffer_data(size_of::<LightingData>(), ptr::null(), gl::DYNAMIC_DRAW);
    buffer.bind_buffer_base(BufferBindingPoint::MatrixData as u32);
    buffer
}

pub trait Renderer {
    fn render(&self, entity_system: &EntitySystem, models: &ModelContainer);
}

pub struct DefaultRenderer {
    shader_program: ShaderProgram,
    matrix_buffer: BufferObject,
    lighting_buffer: BufferObject,
}

impl DefaultRenderer {
    pub fn new(version: Version) -> Self {
        let main_vert_src = MainShader::<MainVertexShader>::new();
        let main_vert = shader::build_shader(&main_vert_src, version);

        let mut main_frag_src = MainShader::<MainFragmentShader>::new();
        let light_shader_src = DefaultLightShader::new();
        main_frag_src.attach_shader(&light_shader_src);
        let light_shader = shader::build_shader(&light_shader_src, version);
        let main_frag = shader::build_shader(&main_frag_src, version);

        let program = ShaderProgram::new().unwrap();
        program.attach_shader(&main_vert);
        program.attach_shader(&main_frag);
        program.attach_shader(&light_shader);
        program.link();
        assert!(program.link_success());

        let matrix_buffer = matrix_data_buffer();
        let lighting_buffer = lighting_data_buffer();

        Self {
            shader_program: program,
            matrix_buffer,
            lighting_buffer,
        }
    }
}

impl Renderer for DefaultRenderer {
    fn render(&self, entity_system: &EntitySystem, model_container: &ModelContainer) {
        let camera = match entity_system.component_slice::<CameraComponent>().first() {
            Some(camera) => camera,
            None => return,
        };
        let light = match entity_system.component_slice::<LightComponent>().first() {
            Some(light) => light,
            None => return,
        };
        let meshes = entity_system.component_slice::<MeshComponent>();

        self.shader_program.use_();
        let camera_transform = entity_system.get_transfom(camera.owner_id);
        let light_transform = entity_system.get_transfom(light.owner_id);

        let lighting_data = LightingData {
            light_data: light.light_source.get_data(light_transform),
            viewer_position: Vec3::zeros(),
        };
        self.lighting_buffer.bind();
        self.lighting_buffer.buffer_subdata(
            size_of::<LightingData>(),
            (&lighting_data as *const LightingData).cast(),
            0,
        );

        for mesh in meshes {
            let transform = entity_system.get_transfom(mesh.owner_id);
            let matrix_data = MatrixData {
                mvp: camera.camera.projection_view(camera_transform) * transform.model(),
                model: transform.model(),
                orientation: glm::quat_to_mat4(&transform.orientation),
                light_space: light.light_source.lightspace(light_transform),
            };

            self.matrix_buffer.bind();
            self.matrix_buffer.buffer_subdata(
                size_of::<MatrixData>(),
                (&matrix_data as *const MatrixData).cast(),
                0,
            );
            self.lighting_buffer.bind();
            self.matrix_buffer.buffer_subdata(
                size_of::<Vec3>(),
                (&transform.position as *const Vec3).cast(),
                offset_of!(LightingData, viewer_position) as u32,
            );

            render_meshes(model_container.get_meshes(mesh.model_index));
        }
    }
}

fn render_meshes(meshes: &[Mesh]) {
    for mesh in meshes {
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

// pub fn draw(
//     &self,
//     camera: &Camera,
//     transform: &Transform,
//     model: &ModelIndex,
//     light: &mut LightSource,
// ) {
//     self.shader_program.use_();
//     self.fill_buffers(camera, transform, light);
//     for mesh in model.get_meshes() {
//         mesh.bind();
//         unsafe {
//             gl::DrawElements(
//                 gl::TRIANGLES,
//                 mesh.index_count,
//                 gl::UNSIGNED_INT,
//                 0 as *const _,
//             );
//         }
//     }
// }
// fn fill_buffers(&self, camera: &Camera, transform: &Transform, light: &mut LightSource) {
//     let matrix_data = MatrixData::new(
//         camera.projection_view() * transform.model(), // change
//         transform.model(),
//         glm::quat_to_mat4(&transform.orientation),
//         light.lightspace(),
//     );
//     self.matrix_data_buffer.bind();
//     self.matrix_data_buffer.buffer_subdata(
//         size_of::<MatrixData>(),
//         (&matrix_data as *const MatrixData).cast(),
//         0,
//     );
//     let lighting_data =
//         LightingData::new(light.get_data(), unsafe { (*camera.transform).position });
//     self.lighting_data_buffer.bind();
//     self.lighting_data_buffer.buffer_subdata(
//         size_of::<LightingData>(),
//         (&lighting_data as *const LightingData).cast(),
//         0,
//     );
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
