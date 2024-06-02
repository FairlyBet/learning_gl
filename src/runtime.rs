use crate::{
    entity_system::SceneManager,
    gl_wrappers::Gl,
    rendering::{Renderer, Screen},
    resources::ResourceManager,
    scripting::Scripting,
};
use glfw::{
    fail_on_errors, Action, ClientApiHint, Context, CursorMode, GlfwReceiver, Key, Modifiers,
    MouseButton, OpenGlProfileHint, PWindow, SwapInterval, WindowEvent, WindowHint, WindowMode,
};
use spin_sleep::{SpinSleeper, SpinStrategy};
use core::result;
use std::time::Duration;

const CONTEXT_VERSION: WindowHint = WindowHint::ContextVersion(4, 6);
const OPENGL_PROFILE: WindowHint = WindowHint::OpenGlProfile(OpenGlProfileHint::Core);
const CLIENT_API: WindowHint = WindowHint::ClientApi(ClientApiHint::OpenGl);
const MODE: WindowMode<'static> = WindowMode::Windowed;
const SWAP_INTERVAL: SwapInterval = SwapInterval::Sync(1);
const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

pub fn run() {
    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    glfw.window_hint(CLIENT_API);
    glfw.window_hint(OPENGL_PROFILE);
    glfw.window_hint(CONTEXT_VERSION);
    let (mut window, receiver) = glfw.with_primary_monitor(|glfw, monitor| match monitor {
        Some(monitor) => {
            let mode = monitor.get_video_mode().unwrap();
            glfw.create_window(
                mode.width,
                mode.height,
                "v0.0.1",
                WindowMode::FullScreen(monitor),
            )
            .unwrap()
        }
        None => unreachable!(),
    });
    enable_polling(&mut window);
    window.set_cursor_mode(CursorMode::Disabled);
    window.set_cursor_pos(0.0, 0.0);
    window.set_raw_mouse_motion(true);
    window.set_resizable(false);
    window.make_current();
    glfw.set_swap_interval(SWAP_INTERVAL);

    let gl = Gl::load();

    let mut renderer = Renderer::new(
        window.get_framebuffer_size(),
        window.get_context_version(),
        &gl,
    );
    let mut screen = Screen::new(
        window.get_framebuffer_size(),
        window.get_context_version(),
        &gl,
    );
    let scripting = Scripting::new();
    let mut resource_manager = ResourceManager::new(&gl);
    let mut scene_manager = SceneManager::default();
    let mut events = WindowEvents::new();
    let mut frametime = 0.0;
    let mut sleep_time = Duration::ZERO;

    scripting.load_api(&mut scene_manager, &events, &window, &frametime);
    scene_manager.load_scene(0, &mut resource_manager, &scripting);
    scene_manager.framebuffer_size(window.get_framebuffer_size());

    while !window.should_close() {
        // Consider moving cursor to (0, 0) as its movement is unlimited and possibly could reach f64 accuracy limit
        window.glfw.set_time(0.0);
        process_events(
            &mut window,
            &receiver,
            &mut events,
            &mut sleep_time,
            &mut [&mut renderer, &mut screen, &mut scene_manager],
        );
        script_iteration(&scripting);
        render_iteration(
            &mut window,
            &screen,
            &renderer,
            &scene_manager,
            &resource_manager,
            &sleep_time,
        );
        frametime = window.glfw.get_time();
    }
}

fn process_events(
    window: &mut PWindow,
    receiver: &GlfwReceiver<(f64, WindowEvent)>,
    events: &mut WindowEvents,
    sleep_period: &mut Duration,
    framebuffer_size_callbacks: &mut [&mut dyn FramebufferSizeCallback],
) {
    window.glfw.poll_events();
    events.clear_events();

    for (_, event) in glfw::flush_messages(receiver) {
        match event {
            WindowEvent::Key(Key::Enter, _, Action::Repeat, _) => {
                events.char_input.push('\n');
            }
            WindowEvent::Key(key, _, action @ (Action::Press | Action::Release), modifiers) => {
                events.key_input.push((key, action, modifiers));
                if key == Key::V && action == Action::Press && modifiers == Modifiers::Control {
                    if let Some(string) = window.get_clipboard_string() {
                        events.char_input.push_str(&string);
                    }
                }
                if key == Key::Enter && action == Action::Press {
                    events.char_input.push('\n');
                }
            }
            WindowEvent::Char(char_) => {
                events.char_input.push(char_);
            }
            WindowEvent::MouseButton(button, action, modifiers) => {
                events.mouse_button_input.push((button, action, modifiers));
            }
            WindowEvent::Scroll(x, y) => {} // could be y, x
            WindowEvent::FramebufferSize(w, h) if w != 0 && h != 0 => {
                for callback in framebuffer_size_callbacks.iter_mut() {
                    callback.framebuffer_size((w, h));
                }
            }
            WindowEvent::FileDrop(paths) => {}
            WindowEvent::Iconify(i) => {
                *sleep_period = Duration::from_millis(33 * (i as u64));
            }
            _ => {}
        }
    }

    events.update_cursor_pos(window.get_cursor_pos());
}

fn script_iteration(scripting: &Scripting) {
    scripting.run_updates();
    scripting.gc_collect();
}

fn render_iteration(
    window: &mut PWindow,
    screen: &Screen,
    renderer: &Renderer,
    scene_manager: &SceneManager,
    resource_manager: &ResourceManager,
    sleep_period: &Duration,
) {
    renderer.render(scene_manager, &resource_manager.mesh_manager());
    screen.render_offscreen(renderer.framebuffer());
    window.swap_buffers();
    unsafe {
        gl::Finish();
    }
    let sleeper = SpinSleeper::default().with_spin_strategy(SpinStrategy::YieldThread);
    sleeper.sleep(sleep_period.clone());
}

fn enable_polling(window: &mut PWindow) {
    window.set_key_polling(true);
    window.set_char_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_scroll_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_drag_and_drop_polling(true);
    window.set_iconify_polling(true);
}

#[derive(Debug)]
pub struct WindowEvents {
    key_input: Vec<(Key, Action, Modifiers)>,
    char_input: String,
    mouse_button_input: Vec<(MouseButton, Action, Modifiers)>,
    cursor_offset: (f64, f64),
    cursor_pos: (f64, f64),
}

impl WindowEvents {
    pub const fn new() -> Self {
        Self {
            key_input: Vec::new(),
            char_input: String::new(),
            mouse_button_input: Vec::new(),
            cursor_offset: (0.0, 0.0),
            cursor_pos: (0.0, 0.0),
        }
    }

    fn update_cursor_pos(&mut self, cursor_pos: (f64, f64)) {
        self.cursor_offset.0 = cursor_pos.0 - self.cursor_pos.0;
        self.cursor_offset.1 = cursor_pos.1 - self.cursor_pos.1;
        self.cursor_pos = cursor_pos;
    }

    pub fn get_key(&self, key: (Key, Action, Modifiers)) -> bool {
        if key.2.is_empty() {
            self.key_input
                .iter()
                .any(|item| item.0 == key.0 && item.1 == key.1)
        } else {
            self.key_input.contains(&key)
        }
    }

    pub fn get_mouse_button(&self, button: (MouseButton, Action, Modifiers)) -> bool {
        if button.2.is_empty() {
            self.mouse_button_input
                .iter()
                .any(|item| item.0 == button.0 && item.1 == button.1)
        } else {
            self.mouse_button_input.contains(&button)
        }
    }

    pub fn get_cursor_pos(&self) -> (f64, f64) {
        self.cursor_pos
    }

    pub fn get_cursor_offset(&self) -> (f64, f64) {
        self.cursor_offset
    }

    fn clear_events(&mut self) {
        self.key_input.clear();
        self.char_input.clear();
        self.mouse_button_input.clear();
        self.cursor_offset = (0.0, 0.0);
    }
}

pub trait FramebufferSizeCallback {
    fn framebuffer_size(&mut self, size: (i32, i32));
}

pub type Result<T> = result::Result<T, Error>;

pub enum Error {
    MemoryError,
    UnsupportedAlignment(&'static str),
}
