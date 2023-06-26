use glfw::{CursorMode, OpenGlProfileHint, WindowMode};
use nalgebra_glm::Mat4x4;
extern crate nalgebra_glm as glm;

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
    transform: Mat4x4
}