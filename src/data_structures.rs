use crate::{
    gl_wrappers::{self, VertexArrayObject, VertexBufferObject},
    updaters,
};
use gl::types::GLenum;
use glfw::{Action, CursorMode, Key, OpenGlProfileHint, Window, WindowMode};
use glm::Vec3;
use nalgebra_glm::{Mat4x4, Quat};
use russimp::{
    scene::{PostProcess, Scene},
    Vector3D,
};
use std::{f32::consts, mem::size_of, vec};

/* Подведем итог:
1. Матрица поворота полученная из кватерниона влияет на матрицу перемещения,
т.е. если применять поворот после перемещения то объект будет двигаться по кругу
на такой же угол как и поворот вокруг центра координат,
что полезно для создания матрицы вида
2. Если последовательно поворачивать один и тот же кватернион то к осям
поворота будет применяться поворот уже имеющийся в
кватернионе, поэтому для корректной работы необходимо поворачивать
нулевой кватернион отдельно для каждой оси и уже
полученный результат комбинировать перемножением */

pub const DEG_TO_RAD: f32 = consts::PI / 180.0;

pub fn to_rad(deg: f32) -> f32 {
    deg * DEG_TO_RAD
}

pub struct GlfwConfig {
    pub profile: OpenGlProfileHint,
    pub version: (u32, u32),
}

impl Default for GlfwConfig {
    fn default() -> Self {
        Self {
            profile: OpenGlProfileHint::Core,
            version: (3, 3),
        }
    }
}

pub struct WindowConfig<'a> {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub mode: WindowMode<'a>,
    pub cursor_mode: CursorMode,
    pub vsync: bool,
}

impl Default for WindowConfig<'_> {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            title: Default::default(),
            mode: WindowMode::Windowed,
            cursor_mode: CursorMode::Normal,
            vsync: true,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub orientation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            position: Vec3::zeros(),
            orientation: glm::quat_identity(),
            scale: Vec3::from_element(1.0),
        }
    }

    pub fn get_model(&self) -> Mat4x4 {
        let identity = glm::identity();

        let tranlation = glm::translate(&identity, &self.position);
        let rotation = glm::quat_to_mat4(&self.orientation);
        let scale = glm::scale(&identity, &self.scale);

        tranlation * rotation * scale // why rotation matrix affects translation one??!
    }

    pub fn set_rotation(&mut self, euler: &Vec3) {
        self.orientation = glm::quat_identity();
        self.rotate(euler);
    }

    pub fn move_(&mut self, delta: &Vec3) {
        self.position += *delta;
    }

    pub fn move_local(&mut self, delta: &Vec3) {
        let (local_right, local_upward, local_forward) = self.get_local_axises();
        self.position += local_right * delta.x + local_upward * delta.y + local_forward * delta.z;
    }

    pub fn rotate(&mut self, euler: &Vec3) {
        self.rotate_around(euler, &(*Vec3::x_axis(), *Vec3::y_axis(), *Vec3::z_axis()));
    }

    pub fn rotate_local(&mut self, euler: &Vec3) {
        let local_axises = self.get_local_axises();
        self.rotate_around(euler, &local_axises);
    }

    pub fn get_local_axises(&self) -> (Vec3, Vec3, Vec3) {
        let local_right = glm::quat_rotate_vec3(&self.orientation, &Vec3::x_axis());
        let local_upward = glm::quat_rotate_vec3(&self.orientation, &Vec3::y_axis());
        let local_forward = glm::quat_rotate_vec3(&self.orientation, &Vec3::z_axis());

        (local_right, local_upward, local_forward)
    }

    fn rotate_around(&mut self, euler: &Vec3, axises: &(Vec3, Vec3, Vec3)) {
        let euler_rad = euler * DEG_TO_RAD;
        let identity = glm::quat_identity();
        let x_rotation = glm::quat_rotate_normalized_axis(&identity, euler_rad.x, &axises.0);
        let y_rotation = glm::quat_rotate_normalized_axis(&identity, euler_rad.y, &axises.1);
        let z_rotation = glm::quat_rotate_normalized_axis(&identity, euler_rad.z, &axises.2);

        self.orientation = z_rotation * y_rotation * x_rotation * self.orientation;
    }
}

pub struct ViewObject {
    pub transform: Transform,
    projection_matrix: Mat4x4,
}

impl ViewObject {
    pub fn new(projection: Projection) -> ViewObject {
        ViewObject {
            transform: Transform::new(),
            projection_matrix: projection.calculate_projecion(),
        }
    }

    pub fn get_view(&self) -> Mat4x4 {
        let identity = glm::identity();
        let translation = glm::translate(&identity, &(-self.transform.position));
        let rotation = glm::inverse(&glm::quat_to_mat4(&self.transform.orientation));

        rotation * translation // applying quat rotation after translation makes object rotate
                               // around coordinate center and around themselves simultaneoulsy
    }

    pub fn get_projection(&self) -> Mat4x4 {
        self.projection_matrix
    }

    pub fn set_projection(&mut self, projection: Projection) {
        self.projection_matrix = projection.calculate_projecion();
    }
}

#[derive(Clone, Copy)]
pub enum Projection {
    Orthographic(f32, f32, f32, f32, f32, f32),
    Perspective(f32, f32, f32, f32),
}

impl Projection {
    fn calculate_projecion(&self) -> Mat4x4 {
        match *self {
            Projection::Orthographic(left, right, bottom, top, znear, zfar) => {
                glm::ortho(left, right, bottom, top, znear, zfar)
            }
            Projection::Perspective(aspect, fovy, near, far) => {
                glm::perspective(aspect, to_rad(fovy), near, far)
            }
        }
    }
}

pub struct EngineApi<'a> {
    window: &'a Window,
    frametime: f32,
    should_close: bool,
    cursor_offset: (f32, f32),
}

impl<'a> EngineApi<'a> {
    pub fn new(window: &'a Window, frametime: f32, cursor_offset: (f32, f32)) -> Self {
        EngineApi {
            window,
            frametime,
            should_close: false,
            cursor_offset,
        }
    }

    pub fn get_key(&self, key: Key) -> Action {
        self.window.get_key(key)
    }

    pub fn get_cursor_pos(&self) -> (f32, f32) {
        let pos = self.window.get_cursor_pos();
        (pos.0 as f32, pos.1 as f32)
    }

    pub fn get_cursor_offset(&self) -> (f32, f32) {
        self.cursor_offset
    }

    pub fn get_frametime(&self) -> f32 {
        self.frametime
    }

    pub fn set_should_close_true(&mut self) {
        self.should_close = true;
    }

    pub fn get_should_close(&self) -> bool {
        self.should_close
    }
}

pub struct EventContainer {
    pub on_key_pressed: Vec<OnKeyPressed>,
    pub on_framebuffer_size_changed: Vec<OnFrameBufferSizeChange>,
}

impl EventContainer {
    pub fn new() -> Self {
        todo!();
    }

    pub fn new_minimal() -> Self {
        let on_key_pressed = vec![OnKeyPressed {
            callback: updaters::close_on_escape,
        }];
        let on_framebuffer_size_changed = vec![OnFrameBufferSizeChange {
            callback: updaters::on_framebuffer_size_change,
        }];
        EventContainer {
            on_key_pressed,
            on_framebuffer_size_changed,
        }
    }
}

pub struct OnKeyPressed {
    pub callback: fn(key: Key, action: Action, &mut EngineApi) -> (),
}

pub struct OnFrameBufferSizeChange {
    pub callback: fn(i32, i32) -> (),
}

pub fn load_model(path: &str) -> GlMesh {
    let scene = Scene::from_file(
        path,
        vec![
            PostProcess::Triangulate,
            PostProcess::FlipUVs,
            PostProcess::OptimizeGraph,
            PostProcess::OptimizeMeshes,
        ],
    )
    .unwrap();
    let mesh = &scene.meshes[0];
    let vert = &mesh.vertices;
    let mut ind = Vec::<u32>::with_capacity(mesh.faces.len() * 3);

    for face in mesh.faces.iter() {
        for index in face.0.iter() {
            ind.push(*index);
        }
    }
    // texture loading
    GlMesh::from_assimp(vert, Some(&ind), gl::STATIC_DRAW)
}

pub struct GlMesh {
    vao: VertexArrayObject,
    vertices: VertexBufferObject,
    indecies: Option<VertexBufferObject>,
    triangles_count: i32,
    indecies_count: i32,
}

impl GlMesh {
    pub const CUBE_MESH: [f32; 108] = [
        -0.5, -0.5, -0.5, 0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5,
        -0.5, -0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5,
        -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5,
        -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5,
        0.5, -0.5, 0.5, 0.5, 0.5, 0.5, -0.5, -0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5,
        -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5,
        0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5,
    ];

    pub fn from_vertices(vertices: &Vec<f32>, usage: GLenum) -> Self {
        let vao = VertexArrayObject::new().unwrap();
        vao.bind();

        let vbo = VertexBufferObject::new(gl::ARRAY_BUFFER).unwrap();
        vbo.bind();
        vbo.buffer_data(
            vertices.len() * size_of::<f32>(),
            vertices.as_ptr().cast(),
            usage,
        );
        gl_wrappers::configure_attribute(0, 3, gl::FLOAT, gl::FALSE, 0, 0 as *const _);
        gl_wrappers::enable_attribute(0);
        VertexArrayObject::unbind();
        vbo.unbind();

        let triangles_count = vertices.len() as i32 / 3;

        Self {
            vao,
            vertices: vbo,
            triangles_count,
            indecies: None,
            indecies_count: 0,
        }
    }

    pub fn from_assimp(
        vertices: &Vec<Vector3D>,
        indecies: Option<&Vec<u32>>,
        usage: GLenum,
    ) -> Self {
        let vao = VertexArrayObject::new().unwrap();
        vao.bind();

        let vbo = VertexBufferObject::new(gl::ARRAY_BUFFER).unwrap();
        vbo.bind();
        vbo.buffer_data(
            vertices.len() * size_of::<Vector3D>(),
            vertices.as_ptr().cast(),
            usage,
        );

        // возможно этого здесь не должно быть
        // как вариант конфигурация должна производиться глобально
        // для всех вао
        gl_wrappers::configure_attribute(0, 3, gl::FLOAT, gl::FALSE, 0, 0 as *const _);
        gl_wrappers::enable_attribute(0);

        let ind: Option<VertexBufferObject>;
        match indecies {
            Some(index_data) => {
                let ebo = VertexBufferObject::new(gl::ELEMENT_ARRAY_BUFFER).unwrap();
                ebo.bind();
                ebo.buffer_data(
                    index_data.len() * size_of::<u32>(),
                    index_data.as_ptr().cast(),
                    usage,
                );
                ind = Some(ebo);
            }
            None => ind = None,
        }
        VertexArrayObject::unbind();
        vbo.unbind();
        if let Some(ebo) = &ind {
            ebo.unbind();
        }

        let triangles_count = vertices.len() as i32;
        let indecies_count = match indecies {
            Some(vec) => vec.len() as i32,
            None => 0,
        };

        Self {
            vao,
            vertices: vbo,
            triangles_count,
            indecies: ind,
            indecies_count,
        }
    }

    pub fn bind(&self) {
        self.vao.bind();
    }

    pub fn unbind(&self) {
        VertexArrayObject::unbind();
    }

    pub fn draw(&self) {
        self.bind();
        if let Some(_) = &self.indecies {
            unsafe {
                gl::DrawElements(
                    gl::TRIANGLES,
                    self.indecies_count,
                    gl::UNSIGNED_INT,
                    0 as *const _,
                );
            }
        } else {
            unsafe {
                gl::DrawArrays(gl::TRIANGLES, 0, self.triangles_count);
            }
        }
    }
}
