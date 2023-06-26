extern crate nalgebra_glm as glm;
use glfw::{CursorMode, OpenGlProfileHint, WindowMode};
use glm::Vec3;
use nalgebra_glm::Mat4x4;

pub struct GlfwConfig {
    pub profile: OpenGlProfileHint,
    pub version: (u32, u32),
}

pub struct WindowConfig<'a> {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub mode: WindowMode<'a>,
    pub cursor_mode: CursorMode,
    pub vsync: bool,
}

pub struct Object {
    transform: Transform,
}

pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3, // radians
    pub scale: Vec3,
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
