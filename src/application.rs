use crate::gl_wrappers::Gl;
use glfw::{
    fail_on_errors, Context as _, GlfwReceiver, OpenGlProfileHint, PWindow, SwapInterval,
    WindowEvent, WindowHint, WindowMode,
};

const CONTEXT_VERSION: WindowHint = WindowHint::ContextVersion(4, 4);
const OPENGL_PROFILE: WindowHint = WindowHint::OpenGlProfile(OpenGlProfileHint::Core);
const DEFAULT_WIDTH: u32 = 800;
const DEFAULT_HEIGHT: u32 = 600;
const DEFAULT_TITLE: &str = "New Window";
const DEFAULT_MODE: WindowMode<'_> = WindowMode::Windowed;
const VSYNC: bool = true;

pub struct Application {
    #[allow(unused)]
    gl: Gl,
    pub receiver: GlfwReceiver<(f64, WindowEvent)>,
    pub window: PWindow,
}

impl Application {
    pub fn new() -> Self {
        let mut glfw = glfw::init(fail_on_errors!()).unwrap();
        glfw.window_hint(OPENGL_PROFILE);
        glfw.window_hint(CONTEXT_VERSION);

        let (mut window, receiver) = glfw
            .create_window(DEFAULT_WIDTH, DEFAULT_HEIGHT, DEFAULT_TITLE, DEFAULT_MODE)
            .unwrap();
        window.make_current();
        Self::enable_polling(&mut window);
        glfw.set_swap_interval(SwapInterval::Sync(VSYNC.into()));

        let gl = Gl::load();

        Self {
            gl,
            receiver,
            window,
        }
    }

    fn enable_polling(window: &mut PWindow) {
        window.set_key_polling(true);
        window.set_char_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);
        window.set_scroll_polling(true);
        window.set_framebuffer_size_polling(true);
        window.set_focus_polling(true);
        window.set_drag_and_drop_polling(true);
    }
}
