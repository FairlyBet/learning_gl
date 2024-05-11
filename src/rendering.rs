use crate::{
    camera::Camera,
    data3d::{self, Mesh, MeshData, VertexAttribute},
    entity_system::SceneManager,
    gl_wrappers::{self, BufferObject, Gl, Renderbuffer, ShaderProgram, Texture},
    lighting::{LightData, LightSource},
    resources::MeshManager,
    runtime::FramebufferSizeCallback,
    shader::{
        self, DirectPBR, FragShader, MainShader, ScreenShaderFrag,
        ScreenShaderVert, VertShader,
    },
};
use gl::types::GLenum;
use glfw::Version;
use nalgebra_glm::{Mat4, Vec3};
use std::{
    marker::PhantomData,
    mem::{size_of, size_of_val, MaybeUninit},
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

pub const MAX_LIGHT_SOURCES_PER_FRAME: usize = 16;

#[derive(Debug)]
#[repr(C)]
pub struct LightingData {
    pub light_sources: [MaybeUninit<LightData>; MAX_LIGHT_SOURCES_PER_FRAME],
    pub viewer_position: Vec3,
    pub source_count: u32,
}

impl LightingData {
    fn new() -> Self {
        Self {
            light_sources: [MaybeUninit::zeroed(); MAX_LIGHT_SOURCES_PER_FRAME],
            viewer_position: Default::default(),
            source_count: Default::default(),
        }
    }
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

#[derive(Debug)]
pub struct Renderer<'a> {
    pd: PhantomData<&'a ()>,
    framebuffer: Framebuffer,
    shader_program: ShaderProgram,
    matrix_buffer: BufferObject,
    lighting_buffer: BufferObject,
}

impl<'a> Renderer<'a> {
    pub fn new(size: (i32, i32), context_version: Version, _: &'a Gl) -> Self {
        // let size = (size.0 / 4, size.1 / 4);
        let framebuffer = Framebuffer::new(size, gl::NEAREST, gl::NEAREST);

        let main_vert = MainShader::<VertShader>::new();
        let mut main_frag = MainShader::<FragShader>::new();
        let lighting_shader = DirectPBR::new();
        main_frag.attach_shader(&lighting_shader);

        let main_vert = shader::build_shader(&main_vert, context_version);
        let main_frag = shader::build_shader(&main_frag, context_version);
        let lighting_shader = shader::build_shader(&lighting_shader, context_version);

        let program = ShaderProgram::new().unwrap();
        program.attach_shader(&main_vert);
        program.attach_shader(&main_frag);
        program.attach_shader(&lighting_shader);
        program.link();
        assert!(program.link_success());
        let matrix_buffer = matrix_data_buffer();
        let lighting_buffer = lighting_data_buffer();

        Self {
            framebuffer,
            shader_program: program,
            matrix_buffer,
            lighting_buffer,
            pd: PhantomData::default(),
        }
    }

    pub fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }

    pub fn render(&self, scene_manager: &SceneManager, mesh_manager: &MeshManager) {
        let camera = match scene_manager.component_slice::<Camera>().first() {
            Some(cam) => cam,
            None => return,
        };
        if scene_manager.component_slice::<LightSource>().is_empty() {
            return;
        }

        let camera_transform = scene_manager.get_transform(camera.owner_id());

        let mut lighting_data = LightingData::new();
        let lights = scene_manager.component_slice::<LightSource>();
        let mut i = 0;
        for light_source in lights {
            if i == MAX_LIGHT_SOURCES_PER_FRAME {
                break;
            }
            let light_transform = scene_manager.get_transform(light_source.owner_id());
            lighting_data.light_sources[i].write(light_source.data.get_data(&light_transform));
            i += 1;
        }
        lighting_data.source_count = i as u32;
        lighting_data.viewer_position = camera_transform.global_position();

        self.lighting_buffer.bind();
        self.lighting_buffer.buffer_subdata(
            size_of::<LightingData>(),
            (&lighting_data as *const LightingData).cast(),
            0,
        );

        self.framebuffer.bind();
        self.shader_program.use_();
        Self::gl_enable();
        gl_wrappers::clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        for mesh_comp in scene_manager.component_slice::<Mesh>() {
            let mesh_transform = scene_manager.get_transform(mesh_comp.owner_id());
            let matrix_data = MatrixData {
                mvp: camera.data.projection_view(camera_transform) * mesh_transform.model(),
                model: mesh_transform.model(),
                orientation: glm::quat_to_mat4(&mesh_transform.orientation),
                light_space: glm::Mat4::identity(),
            };
            self.matrix_buffer.bind();
            self.matrix_buffer.buffer_subdata(
                size_of::<MatrixData>(),
                (&matrix_data as *const MatrixData).cast(),
                0,
            );

            for (mesh_data, material) in mesh_manager.mesh_n_material(&mesh_comp.data) {
                mesh_manager.textures().get(material.base_color).bind_to_unit(gl::TEXTURE0);
                mesh_manager.textures().get(material.metalness).bind_to_unit(gl::TEXTURE1);
                mesh_manager.textures().get(material.roughness).bind_to_unit(gl::TEXTURE2);
                mesh_manager.textures().get(material.ao).bind_to_unit(gl::TEXTURE3);
                mesh_manager.textures().get(material.normals).bind_to_unit(gl::TEXTURE4);
                mesh_manager.textures().get(material.displacement).bind_to_unit(gl::TEXTURE5);
                mesh_data.bind();
                unsafe {
                    gl::DrawElements(
                        gl::TRIANGLES,
                        mesh_data.index_count,
                        gl::UNSIGNED_INT,
                        0 as *const _,
                    );
                }
            }
        }
    }

    fn gl_enable() {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
        }
    }
}

impl<'a> FramebufferSizeCallback for Renderer<'a> {
    fn framebuffer_size(&mut self, size: (i32, i32)) {
        if self.framebuffer.size != size {
            self.framebuffer = Framebuffer::new(size, gl::LINEAR, gl::LINEAR);
        }
    }
}

#[derive(Debug)]
pub struct Screen<'a> {
    pd: PhantomData<&'a ()>,
    size: (i32, i32),
    program: ShaderProgram,
    quad: MeshData,
    gamma: f32,
}

impl<'a> Screen<'a> {
    pub fn new(size: (i32, i32), context_version: Version, _: &'a Gl) -> Self {
        let vert = shader::build_shader(&ScreenShaderVert::new(), context_version);
        let frag = shader::build_shader(&ScreenShaderFrag::new(), context_version);
        let program = ShaderProgram::new().unwrap();
        program.attach_shader(&vert);
        program.attach_shader(&frag);
        program.link();
        assert!(program.link_success());
        program.use_();
        let gamma = 2.2f32;
        unsafe {
            gl::Uniform1f(ScreenShaderFrag::GAMMA_LOCATION, 1.0 / gamma);
            gl::Uniform1f(ScreenShaderFrag::EXPOSURE_LOCATION, 1.0);
        }
        let quad = MeshData::new(
            6,
            size_of_val(data3d::QUAD_VERTICES_TEX_COORDS),
            data3d::QUAD_VERTICES_TEX_COORDS.as_ptr().cast(),
            vec![VertexAttribute::Position, VertexAttribute::TexCoord],
            0,
            ptr::null(),
            gl::STATIC_DRAW,
        );

        Self {
            size,
            program,
            quad,
            gamma,
            pd: PhantomData::default(),
        }
    }

    pub fn render_offscreen(&self, offscreen: &Framebuffer) {
        Framebuffer::bind_default(self.size);
        self.program.use_();
        self.quad.bind();
        offscreen.color_buffer.bind();
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::STENCIL_TEST);
            gl::DrawArrays(gl::TRIANGLES, 0, self.quad.vertex_count);
        }
    }

    pub fn set_gamma(&mut self, gamma: f32) {
        self.gamma = gamma;
        self.program.use_();
        unsafe {
            gl::Uniform1f(ScreenShaderFrag::GAMMA_LOCATION, 1.0 / gamma);
        }
    }

    pub fn set_exposure(&self, exposure: f32) {
        self.program.use_();
        unsafe {
            gl::Uniform1f(ScreenShaderFrag::EXPOSURE_LOCATION, exposure);
        }
    }

    pub fn set_size(&mut self, size: (i32, i32)) {
        self.size = size;
    }
}

impl<'a> FramebufferSizeCallback for Screen<'a> {
    fn framebuffer_size(&mut self, size: (i32, i32)) {
        if self.size != size {
            self.set_size(size)
        }
    }
}

#[derive(Debug)]
pub struct Framebuffer {
    framebuffer: gl_wrappers::Framebuffer,
    pub color_buffer: Texture,
    depth_stencil_buffer: Renderbuffer,
    pub size: (i32, i32),
}

impl Framebuffer {
    pub fn new(size: (i32, i32), mag: GLenum, min: GLenum) -> Self {
        let color_buffer = Texture::new(gl::TEXTURE_2D).unwrap();
        color_buffer.bind();
        color_buffer.texture_data(size, ptr::null(), gl::FLOAT, gl::RGBA, gl::RGBA16F);
        color_buffer.parameter(gl::TEXTURE_MIN_FILTER, min);
        color_buffer.parameter(gl::TEXTURE_MAG_FILTER, mag);

        let depth_stencil_buffer = Renderbuffer::new(gl::RENDERBUFFER).unwrap();
        depth_stencil_buffer.bind();
        depth_stencil_buffer.buffer_storage(size, gl::DEPTH24_STENCIL8);

        let framebuffer = gl_wrappers::Framebuffer::new(gl::FRAMEBUFFER).unwrap();
        framebuffer.bind();
        framebuffer.attach_texture2d(&color_buffer, gl::COLOR_ATTACHMENT0);
        framebuffer.attach_renderbuffer(&depth_stencil_buffer, gl::DEPTH_STENCIL_ATTACHMENT);
        assert!(gl_wrappers::Framebuffer::is_complete());

        gl_wrappers::Framebuffer::bind_default();

        Self {
            framebuffer,
            color_buffer,
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
            color_buffer: sampler_buffer,
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
