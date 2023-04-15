use glfw::{
    Context, CursorMode, Glfw, OpenGlProfileHint, SwapInterval, Window, WindowEvent, WindowHint,
    WindowMode,
};
use std::sync::mpsc::Receiver;

use crate::camera::Camera;

pub struct Engine {
    glfw: Glfw,
    window: Option<Window>,
    camera: Option<Camera>,
    receiver: Option<Receiver<(f64, WindowEvent)>>,
    gl_is_loaded: bool,
}

impl Engine {
    pub fn new() {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        glfw.window_hint(WindowHint::ContextVersion(3, 3));

        Engine {
            glfw,
            window: None,
            camera: None,
            receiver: None,
            gl_is_loaded: false,
        };
    }

    pub fn create_window(&mut self, width: u32, height: u32) {
        let (window, receiver) = self
            .glfw
            .create_window(width, height, "", WindowMode::Windowed)
            .unwrap();
        self.window = window;
        self.receiver = receiver;

        self.window.set_key_polling(true);
        self.window.set_framebuffer_size_polling(true);
        self.window.set_cursor_pos_polling(true);

        self.window.make_current();
    }

    pub fn set_swap_interval(&mut self, interval: SwapInterval) {
        self.glfw.set_swap_interval(interval);
    }

    pub fn set_cursor_mode(&mut self, mode: CursorMode) {
        self.window.set_cursor_mode(mode);
    }

    pub fn load_texture() {}
}
