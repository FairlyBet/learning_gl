use glfw::{
    Context, CursorMode, Glfw, OpenGlProfileHint, SwapInterval, Window, WindowEvent, WindowHint,
    WindowMode,
};
use std::sync::mpsc::Receiver;

use crate::camera::Camera;

pub struct Engine {
    glfw: Glfw,
    window: Option<Window>,
    receiver: Option<Receiver<(f64, WindowEvent)>>,
    camera: Camera,
    camera_updater: Option< fn(Camera) -> ()>,
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
            receiver: None,
            camera: Camera::new(),
            camera_updater: None,
            gl_is_loaded: false,
        };
    }

    pub fn create_window(&mut self, width: u32, height: u32) {
        let (mut window, receiver) = self
            .glfw
            .create_window(width, height, "", WindowMode::Windowed)
            .unwrap();
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);
        window.set_cursor_pos_polling(true);
        window.make_current();

        if !self.gl_is_loaded {
            gl_loader::init_gl();
            gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);
            self.gl_is_loaded = true;
        }

        self.window = Some(window);
        self.receiver = Some(receiver);
    }

    pub fn set_swap_interval(&mut self, interval: SwapInterval) {
        self.glfw.set_swap_interval(interval);
    }

    pub fn set_cursor_mode(&mut self, mode: CursorMode) {
        self.window.as_mut().unwrap().set_cursor_mode(mode);
    }

    pub fn load_texture() {}
}
