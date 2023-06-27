extern crate nalgebra_glm as glm;
use glfw::{CursorMode, OpenGlProfileHint, WindowMode};
use glm::Vec3;
use nalgebra_glm::Mat4x4;

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
            width: 800,
            height: 600,
            title: Default::default(),
            mode: WindowMode::Windowed,
            cursor_mode: CursorMode::Normal,
            vsync: true,
        }
    }
}

pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3, // radians
    pub scale: Vec3,
}

pub struct Object {
    transform: Transform,
    // extensions: 
}

impl Object {
    pub fn get_model(&self) -> Mat4x4 {
        let mut res = Mat4x4::from_diagonal_element(1.0);
        res = glm::translate(&res, &self.transform.position);

        res = glm::rotate(&res, self.transform.rotation.x, &Vec3::x_axis());
        res = glm::rotate(&res, self.transform.rotation.y, &Vec3::y_axis());
        res = glm::rotate(&res, self.transform.rotation.z, &Vec3::z_axis());

        res = glm::scale(&res, &self.transform.scale);

        res
    }
}

struct ViewObject {
    object: Object,
    type_: ViewType,
}

enum ViewType {
    Orthographic(f32, f32, f32, f32, f32, f32),
    Perspective(f32, f32, f32, f32),
}

pub struct SceneConfig<'a> {
    objects: Vec<Object>,
    views: Vec<ViewObject>,
    active_view: &'a ViewObject,
}
