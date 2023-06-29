use crate::updaters;
use glfw::{Action, CursorMode, Key, OpenGlProfileHint, Window, WindowMode};
use glm::Vec3;
use nalgebra_glm::{Mat4x4, Quat};
use std::vec;

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

#[derive(Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub orientation: Quat, // radians
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
        let mut model = glm::identity();

        model = glm::translate(&model, &self.position);
        model = glm::quat_to_mat4(&self.orientation) * model;
        model = glm::scale(&model, &self.scale);

        model
    }

    pub fn move_(&mut self, delta: Vec3) {
        self.position += delta;
    }

    pub fn move_local(&mut self, delta: Vec3) {
        let mut local_forward = -(*Vec3::z_axis());

        // local_forward = glm::rotate_vec3(&local_forward, self.rotation.x, &Vec3::x_axis());
        // local_forward = glm::rotate_vec3(&local_forward, self.rotation.y, &Vec3::y_axis());
        // local_forward = glm::rotate_vec3(&local_forward, self.rotation.z, &Vec3::z_axis());

        // let quat = glm::quat_ // перечитать статью
    }

    pub fn rotate(&mut self, euler: Vec3) {
        self.orientation = glm::quat_rotate(&self.orientation, euler.x, &Vec3::x_axis());
        self.orientation = glm::quat_rotate(&self.orientation, euler.y, &Vec3::y_axis());
        self.orientation = glm::quat_rotate(&self.orientation, euler.z, &Vec3::z_axis());
    }

    pub fn rotate_local(&mut self, euler: Vec3) {}
}

pub struct ViewObject {
    pub transform: Transform,
    view: Mat4x4,
    projection: Mat4x4,
}

impl ViewObject {
    pub fn new(type_: ViewType, transform: Transform) -> ViewObject {
        ViewObject {
            transform,
            view: Mat4x4::zeros(),
            projection: type_.calculate_projecion(),
        }
    }

    pub fn get_view(&self) -> Mat4x4 {
        let mut direction = -(*Vec3::z_axis());

        // direction = glm::rotate_vec3(&direction, self.transform.orientation.x, &Vec3::x_axis());
        // direction = glm::rotate_vec3(&direction, self.transform.orientation.y, &Vec3::y_axis());
        // direction = glm::rotate_vec3(&direction, self.transform.orientation.z, &Vec3::z_axis());

        glm::look_at(
            &(self.transform.position),
            &(self.transform.position + direction),
            &(*Vec3::y_axis()),
        )
    }

    pub fn get_projection(&self) -> Mat4x4 {
        self.projection
    }

    pub fn set_view_type(&mut self, type_: ViewType) {
        self.projection = type_.calculate_projecion();
    }
}

#[derive(Clone, Copy)]
pub enum ViewType {
    Orthographic(f32, f32, f32, f32, f32, f32),
    Perspective(f32, f32, f32, f32),
}

impl ViewType {
    fn calculate_projecion(&self) -> Mat4x4 {
        match *self {
            ViewType::Orthographic(left, right, bottom, top, znear, zfar) => {
                glm::ortho(left, right, bottom, top, znear, zfar)
            }
            ViewType::Perspective(aspect, fovy, near, far) => {
                glm::perspective(aspect, fovy, near, far)
            }
        }
    }
}

pub struct EngineApi<'a> {
    window: &'a Window,
    frametime: f32,
    should_close: bool,
}

impl<'a> EngineApi<'a> {
    pub fn new(window: &'a Window, frametime: f32) -> Self {
        EngineApi {
            window,
            frametime,
            should_close: false,
        }
    }

    pub fn get_key(&self, key: Key) -> Action {
        self.window.get_key(key)
    }

    pub fn get_cursor_pos(&self) -> (f64, f64) {
        self.window.get_cursor_pos()
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
