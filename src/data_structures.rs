extern crate nalgebra_glm as glm;
use std::vec;

use glfw::{Action, CursorMode, Key, OpenGlProfileHint, Window, WindowMode};
use glm::Vec3;
use nalgebra_glm::Mat4x4;

use crate::updaters::{self, OnFrameBufferSizeChange, OnKeyPressed};

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

pub struct ViewObject {
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

pub trait Objected {
    fn get_object(&mut self) -> &mut Object;
}

impl Objected for Object {
    fn get_object(&mut self) -> &mut Object {
        self
    }
}

impl Objected for ViewObject {
    fn get_object(&mut self) -> &mut Object {
        &mut self.object
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

    pub fn temp_new() -> Self {
        let on_key_pressed = vec![OnKeyPressed {
            key: Key::Escape,
            action: Action::Press,
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
