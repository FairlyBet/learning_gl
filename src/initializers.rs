use crate::data_structures::{GlfwConfig, WindowConfig};
use glfw::{Context, Glfw, SwapInterval, Window, WindowEvent, WindowHint};
use std::sync::mpsc::Receiver;

pub fn init_from_config(config: GlfwConfig) -> Glfw {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(WindowHint::OpenGlProfile(config.profile));
    glfw.window_hint(WindowHint::ContextVersion(
        config.version.0,
        config.version.1,
    ));

    glfw
}

pub fn create_from_config(
    config: WindowConfig,
    glfw: &mut Glfw,
) -> (Window, Receiver<(f64, WindowEvent)>) {
    let (mut window, receiver) = glfw
        .create_window(config.width, config.height, &config.title, config.mode)
        .unwrap();

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_cursor_mode(config.cursor_mode);

    window.make_current();

    init_gl();

    if config.vsync {
        glfw.set_swap_interval(SwapInterval::Sync(1));
    } else {
        glfw.set_swap_interval(SwapInterval::None);
    }

    (window, receiver)
}

fn init_gl() {
    gl_loader::init_gl();
    gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);
}

pub fn init_rendering() {
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::ClearColor(0.2, 0.15, 0.15, 1.0);
    }
}
