use crate::{
    data3d::{self, Mesh, VertexAttribute},
    entity_system::{CameraComponent, LightComponent, MeshComponent, SceneChunk},
    gl_wrappers::{self, BufferObject, Renderbuffer, ShaderProgram, Texture},
    lighting::LightData,
    resources::RangeContainer,
    runtime::FramebufferSizeCallback,
    shader::{
        self, DefaultLightShader, MainFragmentShader, MainShader, MainVertexShader,
        ScreenShaderFrag, ScreenShaderVert,
    },
};
use gl::types::GLenum;
use glfw::Version;
use memoffset::offset_of;
use nalgebra_glm::{Mat4, Vec3};
use std::{
    mem::{size_of, size_of_val},
    ptr,
};

// Добавить сущности ShadowMapping / CascadingShadowMapping ?

pub enum BindingPoints {
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
    buffer.bind_buffer_base(BindingPoints::MatrixData as u32);
    buffer
}

fn lighting_data_buffer() -> BufferObject {
    let buffer = BufferObject::new(gl::UNIFORM_BUFFER).unwrap();
    buffer.bind();
    buffer.buffer_data(size_of::<LightingData>(), ptr::null(), gl::DYNAMIC_DRAW);
    buffer.bind_buffer_base(BindingPoints::LightingData as u32);
    buffer
}

pub trait Renderer {
    fn render(&self, entity_system: &SceneChunk, model_container: &RangeContainer<Mesh>);
}

pub struct DefaultRenderer {
    framebuffer: Framebuffer,
    shader_program: ShaderProgram,
    matrix_buffer: BufferObject,
    lighting_buffer: BufferObject,
}

impl DefaultRenderer {
    pub fn new(size: (i32, i32), version: Version) -> Self {
        let framebuffer = Framebuffer::new(size, gl::LINEAR, gl::LINEAR);

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
            framebuffer,
            shader_program: program,
            matrix_buffer,
            lighting_buffer,
        }
    }

    fn gl_config() {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
        }
    }
}

impl Renderer for DefaultRenderer {
    fn render(&self, entity_system: &SceneChunk, model_container: &RangeContainer<Mesh>) {
        let camera_comp = match entity_system.component_slice::<CameraComponent>().first() {
            Some(camera) => camera,
            None => return,
        };
        let light_comp = match entity_system.component_slice::<LightComponent>().first() {
            Some(light) => light,
            None => return,
        };
        let mesh_components = entity_system.component_slice::<MeshComponent>();

        let camera_transform = entity_system.get_transfom(camera_comp.owner_id);
        let light_transform = entity_system.get_transfom(light_comp.owner_id);

        let lighting_data = LightingData {
            light_data: light_comp.light_source.get_data(light_transform),
            viewer_position: Vec3::zeros(),
        };
        self.lighting_buffer.bind();
        self.lighting_buffer.buffer_subdata(
            size_of::<LightingData>(),
            (&lighting_data as *const LightingData).cast(),
            0,
        );

        self.framebuffer.bind();
        self.shader_program.use_();
        Self::gl_config();
        gl_wrappers::clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        for mesh in mesh_components {
            let mesh_transform = entity_system.get_transfom(mesh.owner_id);

            let matrix_data = MatrixData {
                mvp: camera_comp.camera.projection_view(camera_transform) * mesh_transform.model(),
                model: mesh_transform.model(),
                orientation: glm::quat_to_mat4(&mesh_transform.orientation),
                light_space: light_comp.light_source.lightspace(light_transform),
            };

            self.matrix_buffer.bind();
            self.matrix_buffer.buffer_subdata(
                size_of::<MatrixData>(),
                (&matrix_data as *const MatrixData).cast(),
                0,
            );
            self.lighting_buffer.bind();
            self.lighting_buffer.buffer_subdata(
                size_of::<Vec3>(),
                (&mesh_transform.position as *const Vec3).cast(),
                offset_of!(LightingData, viewer_position) as u32,
            );

            // render_meshes(model_container.get_model(mesh.model_index));
        }
    }
}

impl FramebufferSizeCallback for DefaultRenderer {
    fn framebuffer_size(&mut self, size: (i32, i32)) {
        self.framebuffer = Framebuffer::new(size, gl::LINEAR, gl::LINEAR);
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

pub struct Screen {
    resolution: (i32, i32),
    program: ShaderProgram,
    quad: Mesh,
    gamma_correction: f32,
}

impl Screen {
    pub fn new(resolution: (i32, i32), gl_version: Version) -> Self {
        let vert = shader::build_shader(&ScreenShaderVert::new(), gl_version);
        let frag = shader::build_shader(&ScreenShaderFrag::new(), gl_version);
        let program = ShaderProgram::new().unwrap();
        program.attach_shader(&vert);
        program.attach_shader(&frag);
        program.link();
        assert!(program.link_success());
        program.use_();
        let gamma = 2.2f32;
        unsafe {
            gl::Uniform1f(ScreenShaderFrag::GAMMA_CORRECTION_LOCATION, 1.0 / gamma);
        }
        let quad = Mesh::new(
            6,
            size_of_val(data3d::QUAD_VERTICES_TEX_COORDS),
            data3d::QUAD_VERTICES_TEX_COORDS.as_ptr().cast(),
            vec![VertexAttribute::Position, VertexAttribute::TexCoord],
            0,
            ptr::null(),
            gl::STATIC_DRAW,
        );

        Self {
            resolution,
            program,
            quad,
            gamma_correction: gamma,
        }
    }

    pub fn render_offscreen(&self, offscreen: &Framebuffer) {
        Framebuffer::bind_default(self.resolution);
        gl_wrappers::clear(gl::COLOR_BUFFER_BIT);
        self.program.use_();
        self.quad.bind();
        offscreen.sampler_buffer.bind();
        unsafe {
            gl::Disable(gl::DEPTH_TEST | gl::STENCIL_TEST);
            gl::DrawArrays(gl::TRIANGLES, 0, self.quad.vertex_count);
        }
    }

    pub fn set_gamma(&mut self, gamma: f32) {
        self.gamma_correction = gamma;
        self.program.use_();
        unsafe {
            gl::Uniform1f(ScreenShaderFrag::GAMMA_CORRECTION_LOCATION, 1.0 / gamma);
        }
    }

    pub fn set_resolution(&mut self, resolution: (i32, i32)) {
        self.resolution = resolution;
    }
}

impl FramebufferSizeCallback for Screen {
    fn framebuffer_size(&mut self, size: (i32, i32)) {
        self.resolution = size;
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
        assert!(gl_wrappers::Framebuffer::is_complete());

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
        assert!(gl_wrappers::Framebuffer::is_complete());

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
