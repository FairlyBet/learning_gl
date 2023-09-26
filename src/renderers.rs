use crate::{
    data_structures::{Canvas, LightSource, Model, Transform, ViewObject},
    gl_wrappers::{BufferObject, Shader, ShaderProgram, Texture},
};
use glm::{Mat4x4, Vec3, Vec4};
use std::{ffi::c_void, mem::size_of, ptr};

pub struct ModelRenderer {
    shader_program: ShaderProgram,
    matrix_data_buffer: BufferObject,
    lighting_data_buffer: BufferObject,
}

impl ModelRenderer {
    pub fn new() -> Self {
        let vertex_shader =
            Shader::from_file(gl::VERTEX_SHADER, "src\\shaders\\basic.vert").unwrap();
        let fragment_shader =
            Shader::from_file(gl::FRAGMENT_SHADER, "src\\shaders\\basic.frag").unwrap();
        let lighting_shader =
            Shader::from_file(gl::FRAGMENT_SHADER, "src\\shaders\\directional-light.frag").unwrap();
        let color_scale_shader =
            Shader::from_file(gl::FRAGMENT_SHADER, "src\\shaders\\color-scale.frag").unwrap();
        let shader_program = ShaderProgram::new().unwrap();
        shader_program.attach_shader(&vertex_shader);
        shader_program.attach_shader(&fragment_shader);
        shader_program.attach_shader(&lighting_shader);
        shader_program.attach_shader(&color_scale_shader);
        shader_program.link();
        // println!("{}", shader_program.link_success());

        let matrix_data_buffer = BufferObject::new(gl::UNIFORM_BUFFER).unwrap();
        matrix_data_buffer.bind();
        matrix_data_buffer.buffer_data(
            size_of::<MatrixData>(),
            ptr::null() as *const c_void,
            gl::DYNAMIC_DRAW,
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
        camera: &ViewObject,
        transform: &Transform,
        model: &Model,
        light_source: &LightSource,
    ) {
        self.shader_program.use_();

        let matrix_data = MatrixData::new(
            camera.projection * camera.get_view() * transform.get_model(),
            transform.get_model(),
            glm::quat_to_mat4(&transform.orientation),
        );
        let lighting_data = LightingData::new(*light_source, camera.transform.position);

        self.matrix_data_buffer.bind();
        self.matrix_data_buffer.buffer_subdata(
            size_of::<MatrixData>(),
            (&matrix_data as *const MatrixData).cast(),
            0,
        );

        self.lighting_data_buffer.bind();
        self.lighting_data_buffer.buffer_subdata(
            size_of::<LightingData>(),
            (&lighting_data as *const LightingData).cast(),
            0,
        );

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

    fn draw_arrays() {
        todo!();
        // unsafe {
        //     gl::DrawArrays(gl::TRIANGLES, 0, triangle_count);
        // }
    }
}

enum UniformBufferBinding {
    MatrixData = 0,
    LightingData = 1,
}

#[repr(C)]
pub struct MatrixData {
    mvp: Mat4x4,
    model: Mat4x4,
    orientation: Mat4x4,
}

impl MatrixData {
    pub fn new(mvp: Mat4x4, model: Mat4x4, orientation: Mat4x4) -> Self {
        Self {
            mvp,
            model,
            orientation,
        }
    }
}

#[repr(C)]
pub struct LightingData {
    light_source: LightSource,
    viewer_position: Vec4,
}

impl LightingData {
    pub fn new(light_source: LightSource, viewer_position: Vec3) -> Self {
        Self {
            light_source,
            viewer_position: glm::vec3_to_vec4(&viewer_position),
        }
    }
}

pub struct ScreenRenderer {
    shader_program: ShaderProgram,
}

impl ScreenRenderer {
    pub fn new() -> Self {
        let shader_program = ShaderProgram::from_vert_frag_file(
            "src\\shaders\\texture-rendering.vert",
            "src\\shaders\\texture-rendering.frag",
        )
        .unwrap();
        Self { shader_program }
    }

    pub fn draw_texture(&self, canvas: &Canvas, texture: &Texture) {
        unsafe {
            texture.bind();
            canvas.bind();
            self.shader_program.use_();
            gl::DrawArrays(gl::TRIANGLES, 0, canvas.render_quad.triangle_count);
        }
    }
}
