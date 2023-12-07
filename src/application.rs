use crate::{gl_wrappers::Gl, runtime::Runtime};
use glfw::{
    Action, Context as _, Glfw, Key, Modifiers, MouseButton, OpenGlProfileHint, SwapInterval, Window,
    WindowEvent, WindowHint, WindowMode,
};
use std::sync::mpsc::Receiver;

const CONTEXT_VERSION: WindowHint = WindowHint::ContextVersion(4, 2);
const OPENGL_PROFILE: WindowHint = WindowHint::OpenGlProfile(OpenGlProfileHint::Core);
const DEFAULT_WIDTH: u32 = 800;
const DEFAULT_HEIGHT: u32 = 600;
const DEFAULT_TITLE: &str = "New Window";
const DEFAULT_MODE: WindowMode<'_> = WindowMode::Windowed;
const VSYNC: bool = true;

pub struct Application {
    gl: Gl,
    pub glfw: Glfw,
    pub window: Window,
    pub receiver: Receiver<(f64, WindowEvent)>,
}

impl Application {
    pub fn new() -> Self {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(OPENGL_PROFILE);
        glfw.window_hint(CONTEXT_VERSION);

        let (mut window, receiver) = glfw
            .create_window(DEFAULT_WIDTH, DEFAULT_HEIGHT, DEFAULT_TITLE, DEFAULT_MODE)
            .unwrap();
        Self::enable_polling(&mut window);
        window.make_current();
        glfw.set_swap_interval(SwapInterval::Sync(VSYNC.into()));

        let gl = Gl::load();

        Self {
            gl,
            glfw,
            window,
            receiver,
        }
    }

    fn enable_polling(window: &mut Window) {
        window.set_key_polling(true);
        window.set_char_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);
        window.set_drag_and_drop_polling(true);
        window.set_framebuffer_size_polling(true);
        window.set_focus_polling(true);
    }

    pub fn run(mut self) {
        let runtime = Runtime::new();
        runtime.run(self);
    }
}
