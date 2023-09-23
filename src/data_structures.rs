use crate::gl_wrappers::{self, Texture, VertexArrayObject, BufferObject};
use gl::types::GLenum;
use glfw::{Action, CursorMode, Key, OpenGlProfileHint, Window, WindowMode};
use glm::{Vec3, Vec4};
use nalgebra_glm::{Mat4x4, Quat};
use russimp::{
    scene::{PostProcess, Scene},
    Vector2D, Vector3D,
};
use std::{f32::consts, ffi::c_void, mem::size_of};

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
            version: (4, 2),
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
            width: 800,
            height: 600,
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

        tranlation * rotation * scale
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
    pub projection: Mat4x4,
}

impl ViewObject {
    pub fn new(projection: Projection) -> ViewObject {
        ViewObject {
            transform: Transform::new(),
            projection: projection.calculate_matrix(),
        }
    }

    pub fn get_view(&self) -> Mat4x4 {
        let identity = glm::identity();
        let translation = glm::translate(&identity, &(-self.transform.position));
        let rotation = glm::inverse(&glm::quat_to_mat4(&self.transform.orientation));

        rotation * translation // applying quat rotation after translation makes object rotate
                               // around coordinate center and around themselves simultaneoulsy
    }
}

#[derive(Clone, Copy)]
pub enum Projection {
    Orthographic(f32, f32, f32, f32, f32, f32),
    Perspective(f32, f32, f32, f32),
}

impl Projection {
    pub fn calculate_matrix(&self) -> Mat4x4 {
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

pub fn load_model(path: &str, post_pocess: Vec<PostProcess>) -> Model {
    let scene = Scene::from_file(path, post_pocess).unwrap();

    let mut meshes = Vec::<Mesh>::with_capacity(scene.meshes.len());
    for mesh in &scene.meshes {
        let vertex_count = mesh.vertices.len();
        let mut vertex_data = Vec::<VertexData>::with_capacity(vertex_count);

        for i in 0..vertex_count {
            let position = mesh.vertices[i];
            let normal = mesh.normals[i];
            let tex_coord: Vector2D;
            if let Some(tex_coords) = &(mesh.texture_coords[0]) {
                tex_coord = Vector2D {
                    x: tex_coords[i].x,
                    y: tex_coords[i].y,
                };
            } else {
                tex_coord = Default::default();
            }

            let vertex = VertexData {
                position,
                normal,
                tex_coord,
            };

            vertex_data.push(vertex);
        }

        let mut index_data = Vec::<u32>::with_capacity(mesh.faces.len() * 3);
        for face in &mesh.faces {
            for index in &face.0 {
                index_data.push(*index);
            }
        }

        let mesh = Mesh::from_vertex_data(&vertex_data, &index_data, gl::STATIC_DRAW);
        meshes.push(mesh);
    }
    // let diffuse = "assets\\meshes\\diffuse.jpg";
    // let specular = "assets\\meshes\\specular.jpg";

    // let material = Material::from_file(diffuse, specular);
    // let mesh = GlMesh::from_pointer(
    //     GlMesh::CUBE_VERTICES.as_ptr().cast(),
    //     GlMesh::CUBE_VERTICES.len() * size_of::<f32>(),
    //     gl::STATIC_DRAW,
    //     36,
    // );
    Model::new(meshes)
}

#[repr(C)]
pub struct VertexData {
    pub position: Vector3D,
    pub normal: Vector3D,
    pub tex_coord: Vector2D,
}

pub struct Mesh {
    vao: VertexArrayObject,
    vbo: BufferObject,
    ebo: BufferObject,
    pub triangle_count: i32,
    pub index_count: i32,
}

impl Mesh {
    pub const CUBE_VERTICES_AND_NORMALS: [f32; 216] = [
        -0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.5, 0.5, -0.5, 0.0,
        0.0, -1.0, 0.5, 0.5, -0.5, 0.0, 0.0, -1.0, -0.5, 0.5, -0.5, 0.0, 0.0, -1.0, -0.5, -0.5,
        -0.5, 0.0, 0.0, -1.0, -0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.5,
        0.5, 0.5, 0.0, 0.0, 1.0, 0.5, 0.5, 0.5, 0.0, 0.0, 1.0, -0.5, 0.5, 0.5, 0.0, 0.0, 1.0, -0.5,
        -0.5, 0.5, 0.0, 0.0, 1.0, -0.5, 0.5, 0.5, -1.0, 0.0, 0.0, -0.5, 0.5, -0.5, -1.0, 0.0, 0.0,
        -0.5, -0.5, -0.5, -1.0, 0.0, 0.0, -0.5, -0.5, -0.5, -1.0, 0.0, 0.0, -0.5, -0.5, 0.5, -1.0,
        0.0, 0.0, -0.5, 0.5, 0.5, -1.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.0, 0.5, 0.5, -0.5,
        1.0, 0.0, 0.0, 0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 0.5, -0.5,
        0.5, 1.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.0, -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 0.5,
        -0.5, -0.5, 0.0, -1.0, 0.0, 0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 0.5, -0.5, 0.5, 0.0, -1.0, 0.0,
        -0.5, -0.5, 0.5, 0.0, -1.0, 0.0, -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, -0.5, 0.5, -0.5, 0.0,
        1.0, 0.0, 0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 0.5, 0.5, 0.5, 0.0, 1.0, 0.0, 0.5, 0.5, 0.5, 0.0,
        1.0, 0.0, -0.5, 0.5, 0.5, 0.0, 1.0, 0.0, -0.5, 0.5, -0.5, 0.0, 1.0, 0.0,
    ];

    pub fn from_pointer(
        data: *const c_void,
        size: usize,
        usage: GLenum,
        triangle_count: i32,
    ) -> Self {
        let vao = VertexArrayObject::new().unwrap();
        vao.bind();

        let vertex_buffer = BufferObject::new(gl::ARRAY_BUFFER).unwrap();
        vertex_buffer.bind();
        vertex_buffer.buffer_data(size, data, usage);

        let element_buffer = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER).unwrap();

        gl_wrappers::configure_attribute(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * size_of::<f32>(),
            0 as *const _,
        );
        gl_wrappers::configure_attribute(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * size_of::<f32>(),
            (3 * size_of::<f32>()) as *const _,
        );
        // gl_wrappers::configure_attribute(
        //     2,
        //     2,
        //     gl::FLOAT,
        //     gl::FALSE,
        //     8 * size_of::<f32>(),
        //     (6 * size_of::<f32>()) as *const _,
        // );
        gl_wrappers::enable_attribute(0);
        gl_wrappers::enable_attribute(1);
        // gl_wrappers::enable_attribute(2);

        Mesh {
            vao,
            vbo: vertex_buffer,
            ebo: element_buffer,
            triangle_count,
            index_count: 0,
        }
    }

    pub fn from_vertex_data(
        vertex_data: &Vec<VertexData>,
        index_data: &Vec<u32>,
        usage: GLenum,
    ) -> Mesh {
        let vao = VertexArrayObject::new().unwrap();
        vao.bind();

        let vertex_buffer = BufferObject::new(gl::ARRAY_BUFFER).unwrap();
        vertex_buffer.bind();
        vertex_buffer.buffer_data(
            vertex_data.len() * size_of::<VertexData>(),
            vertex_data.as_ptr().cast(),
            usage,
        );

        let element_buffer = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER).unwrap();
        element_buffer.bind();
        element_buffer.buffer_data(
            index_data.len() * size_of::<u32>(),
            index_data.as_ptr().cast(),
            usage,
        );

        // configure in other place
        // use ShaderInput to configure
        Mesh::configure_vertex_attributes();

        let triangle_count = vertex_data.len() as i32 / 3;
        let index_count = index_data.len() as i32;

        Mesh {
            vao,
            vbo: vertex_buffer,
            ebo: element_buffer,
            triangle_count,
            index_count,
        }
    }

    fn configure_vertex_attributes() {
        gl_wrappers::configure_attribute(
            AttributeLocation::Position as u32,
            3,
            gl::FLOAT,
            gl::FALSE,
            8 * size_of::<f32>(),
            0 as *const _,
        );
        gl_wrappers::configure_attribute(
            AttributeLocation::Normal as u32,
            3,
            gl::FLOAT,
            gl::FALSE,
            8 * size_of::<f32>(),
            (3 * size_of::<f32>()) as *const _,
        );
        gl_wrappers::configure_attribute(
            AttributeLocation::TexCoord as u32,
            2,
            gl::FLOAT,
            gl::FALSE,
            8 * size_of::<f32>(),
            (6 * size_of::<f32>()) as *const _,
        );
        gl_wrappers::enable_attribute(AttributeLocation::Position as u32);
        gl_wrappers::enable_attribute(AttributeLocation::Normal as u32);
        gl_wrappers::enable_attribute(AttributeLocation::TexCoord as u32);
    }

    pub fn bind(&self) {
        self.vao.bind();
    }

    pub fn unbind(&self) {
        VertexArrayObject::unbind(); // antipattern
    }
}

pub struct Model {
    meshes: Vec<Mesh>,
}

impl Model {
    pub fn new(meshes: Vec<Mesh>) -> Self {
        Self { meshes }
    }

    pub fn get_meshes(&self) -> &Vec<Mesh> {
        &self.meshes
    }
}

// pub struct ShaderInput {
//     shader_program: gl_wrappers::ShaderProgram,
//     mvp_location: i32,
//     model_location: i32,
//     orientation_location: i32,
//     viewer_position_location: i32,
//     light_direction_location: i32,
//     light_color_location: i32,
//     diffuse_location: i32,
//     specular_location: i32,
// }

// impl ShaderInput {
//     pub fn new() -> Self {
//         let shader_program = gl_wrappers::ShaderProgram::from_vert_frag_file(
//             "src\\shaders\\3d-model.vert",
//             "src\\shaders\\phong-directional.frag",
//         )
//         .unwrap();
//         shader_program.use_();

//         let mvp_location = shader_program.get_uniform(&UniformInput::Mvp.get_uniform_string());
//         let model_location = shader_program.get_uniform(&UniformInput::Model.get_uniform_string());
//         let orientation_location =
//             shader_program.get_uniform(&UniformInput::Orientation.get_uniform_string());
//         let viewer_position_location =
//             shader_program.get_uniform(&UniformInput::ViewerPosition.get_uniform_string());
//         let light_direction_location =
//             shader_program.get_uniform(&UniformInput::LightDirection.get_uniform_string());
//         let light_color_location =
//             shader_program.get_uniform(&UniformInput::LightColor.get_uniform_string());
//         let diffuse_location =
//             shader_program.get_uniform(&UniformInput::Diffuse.get_uniform_string());
//         let specular_location =
//             shader_program.get_uniform(&UniformInput::Specular.get_uniform_string());

//         ShaderInput {
//             shader_program,
//             mvp_location,
//             model_location,
//             orientation_location,
//             viewer_position_location,
//             light_direction_location,
//             light_color_location,
//             diffuse_location,
//             specular_location,
//         }
//     }

//     pub fn use_(&self) {
//         self.shader_program.use_();
//     }

//     pub fn draw(
//         &self,
//         transform: &Transform,
//         model: &Model,
//         viewer: &ViewObject,
//         light: &LightSource,
//     ) {
//         unsafe {
//             gl::UniformMatrix4fv(
//                 self.mvp_location,
//                 1,
//                 gl::FALSE,
//                 glm::value_ptr(
//                     &(viewer.projection_matrix * viewer.get_view() * transform.get_model()),
//                 )
//                 .as_ptr()
//                 .cast(),
//             );
//             gl::UniformMatrix4fv(
//                 self.model_location,
//                 1,
//                 gl::FALSE,
//                 glm::value_ptr(&transform.get_model()).as_ptr().cast(),
//             );
//             gl::UniformMatrix4fv(
//                 self.orientation_location,
//                 1,
//                 gl::FALSE,
//                 glm::value_ptr(&glm::quat_to_mat4(&transform.orientation))
//                     .as_ptr()
//                     .cast(),
//             );
//             gl::Uniform3fv(
//                 self.viewer_position_location,
//                 1,
//                 glm::value_ptr(&viewer.transform.position).as_ptr().cast(),
//             );
//             gl::Uniform3fv(
//                 self.light_direction_location,
//                 1,
//                 glm::value_ptr(&light.direction).as_ptr().cast(),
//             );
//             gl::Uniform3fv(
//                 self.light_color_location,
//                 1,
//                 glm::value_ptr(&light.color).as_ptr().cast(),
//             );
//             gl::Uniform1i(self.diffuse_location, 0);
//             gl::Uniform1i(self.specular_location, 1);
//         }

//         model.draw();
//     }
// }

pub enum AttributeLocation {
    Position = 0,
    Normal = 1,
    TexCoord = 2,
}

pub struct Material {
    diffuse: Texture,
    specular: Texture,
}

impl Material {
    pub fn from_file(diffuse_path: &str, specular_path: &str) -> Self {
        let diffuse = Texture::from_file(diffuse_path).unwrap();
        let specular = Texture::from_file(specular_path).unwrap();

        Self { diffuse, specular }
    }

    pub fn bind(&self) {
        self.diffuse.bind_to_unit(gl::TEXTURE0);
        self.specular.bind_to_unit(gl::TEXTURE1);
    }
}

#[derive(Clone, Copy)]
pub enum LightType {
    Directional = 0,
    Point = 1,
    Spot = 2,
}

impl Default for LightType {
    fn default() -> Self {
        LightType::Directional
    }
}

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct LightSource {
    color: Vec4,
    position: Vec4,
    direction: Vec4,
    constant: f32,
    linear: f32,
    quadratic: f32,
    inner_cutoff: f32,
    outer_cutoff: f32,
    type_: LightType,
}

impl LightSource {
    pub fn new_directional(color: Vec3, direction: Vec3) -> Self {
        let mut source: LightSource = Default::default();
        
        source.color = glm::vec3_to_vec4(&color);
        source.direction = glm::vec3_to_vec4(&direction);
        source.type_ = LightType::Directional;
        source
    }
}
