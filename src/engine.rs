use glfw::{
    Context, CursorMode, Glfw, OpenGlProfileHint, SwapInterval, Window, WindowEvent, WindowHint,
    WindowMode,
};
use std::sync::mpsc::Receiver;

use crate::camera::Camera;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

pub struct Engine {
    glfw: Glfw,
    window: Window,
    receiver: Receiver<(f64, WindowEvent)>,
    camera: Camera,
    camera_updater: Option<fn(&Camera, f32) -> ()>,
}

impl Engine {
    pub fn new() -> Engine {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        glfw.window_hint(WindowHint::ContextVersion(3, 3));

        let (mut window, receiver) = glfw
            .create_window(WIDTH, HEIGHT, "", WindowMode::Windowed)
            .unwrap();
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);
        window.set_cursor_pos_polling(true);
        window.make_current();

        gl_loader::init_gl();
        gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Enable(gl::DEPTH_TEST);
        }

        Engine {
            glfw,
            window,
            receiver,
            camera: Camera::new(),
            camera_updater: None,
        }
    }

    pub fn set_camera_updater(&mut self, updater: fn(&Camera, f32) -> ()) {
        self.camera_updater = Some(updater);
    }

    pub fn set_swap_interval(&mut self, interval: SwapInterval) {
        self.glfw.set_swap_interval(interval);
    }

    pub fn set_cursor_mode(&mut self, mode: CursorMode) {
        self.window.set_cursor_mode(mode);
    }

    pub fn load_texture() {}

    pub fn main_loop(&mut self) {
        let frametime = 0.0;
        while !self.window.should_close() {
            self.glfw.set_time(0.0);
            self.window.set_cursor_pos(0.0, 0.0);
            self.glfw.poll_events();

            if let Some(updater) = self.camera_updater {
                updater(&self.camera, frametime);
            }
        }
    }
}
