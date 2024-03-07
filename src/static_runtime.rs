use crate::{
    entity_system::SceneManager,
    gl_wrappers::Gl,
    rendering::{Renderer, Screen},
    resources::ResourceManager,
    scripting::Scripting,
};
use glfw::{
    fail_on_errors, init, Action, Context, GlfwReceiver, Key, Modifiers, MouseButton,
    OpenGlProfileHint, PWindow, SwapInterval, WindowEvent, WindowHint, WindowMode,
};
use std::mem::MaybeUninit;

const MAJOR: u32 = 4;
const MINOR: u32 = 3;
const CONTEXT_VERSION: WindowHint = WindowHint::ContextVersion(MAJOR, MINOR);
const OPENGL_PROFILE: WindowHint = WindowHint::OpenGlProfile(OpenGlProfileHint::Core);
const MODE: WindowMode<'_> = WindowMode::Windowed;
const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

static mut WINDOW:              MaybeUninit<PWindow>                            = MaybeUninit::uninit();
static mut RECEIVER:            MaybeUninit<GlfwReceiver<(f64, WindowEvent)>>   = MaybeUninit::uninit();
static mut SCREEN:              MaybeUninit<Screen>                             = MaybeUninit::uninit();
static mut RENDERER:            MaybeUninit<Renderer>                           = MaybeUninit::uninit();
static mut RESOURCE_MANAGER:    MaybeUninit<ResourceManager>                    = MaybeUninit::uninit();
static mut SCENE_MANAGER:       MaybeUninit<SceneManager>                       = MaybeUninit::uninit();
static mut SCRIPTING:           MaybeUninit<Scripting>                          = MaybeUninit::uninit();
static mut WINDOW_EVENTS:       WindowEvents                                    = WindowEvents::new();

fn window() -> &'static mut PWindow {
    unsafe { WINDOW.assume_init_mut() }
}
fn receiver() -> &'static GlfwReceiver<(f64, WindowEvent)> {
    unsafe { RECEIVER.assume_init_ref() }
}
fn screen() -> &'static mut Screen {
    unsafe { SCREEN.assume_init_mut() }
}
fn renderer() -> &'static mut Renderer {
    unsafe { RENDERER.assume_init_mut() }
}
fn resource_manager() -> &'static mut ResourceManager {
    unsafe { RESOURCE_MANAGER.assume_init_mut() }
}
fn scene_manager() -> &'static mut SceneManager {
    unsafe { SCENE_MANAGER.assume_init_mut() }
}
fn scripting() -> &'static Scripting {
    unsafe { SCRIPTING.assume_init_ref() }
}
fn events() -> &'static mut WindowEvents {
    unsafe { &mut WINDOW_EVENTS }
}

pub fn start() {
    let mut glfw = init(fail_on_errors!()).unwrap();
    glfw.window_hint(OPENGL_PROFILE);
    glfw.window_hint(CONTEXT_VERSION);
    let (mut window, receiver) = glfw.create_window(WIDTH, HEIGHT, "", MODE).unwrap();
    window.make_current();
    enable_polling(&mut window);
    glfw.set_swap_interval(SwapInterval::Sync(1));

    let gl = Gl::load();

    unsafe {
        RENDERER.write(Renderer::new(
            window.get_framebuffer_size(),
            window.get_context_version(),
            &gl,
        ));
        SCREEN.write(Screen::new(
            window.get_framebuffer_size(),
            window.get_context_version(),
            &gl,
        ));
        WINDOW.write(window);
        RECEIVER.write(receiver);
        SCRIPTING.write(Scripting::new());
        RESOURCE_MANAGER.write(ResourceManager::new());
        SCENE_MANAGER.write(SceneManager::default());
    }

    scripting().load_api();

    main_loop();

    unsafe {
        RENDERER.assume_init_drop();
        SCREEN.assume_init_drop();
        gl.unload();
        SCRIPTING.assume_init_drop();
        WINDOW.assume_init_drop();
    }
}

fn main_loop() {
    while !window().should_close() {
        process_events(&mut vec![renderer(), screen()]);
        render_iteration();
    }
}

fn process_events(framebuffer_size_callbacks: &mut Vec<&mut dyn FramebufferSizeCallback>) {
    window().glfw.poll_events();
    events().clear_events();
    for (_, event) in glfw::flush_messages(receiver()) {
        match event {
            WindowEvent::Key(Key::Enter, _, Action::Repeat, _) => {
                events().char_input.push('\n');
            }
            WindowEvent::Key(key, _, action @ (Action::Press | Action::Release), modifiers) => {
                events().key_input.push((key, action, modifiers));
                if key == Key::V && action == Action::Press && modifiers == Modifiers::Control {
                    if let Some(string) = window().get_clipboard_string() {
                        events().char_input.push_str(&string);
                    }
                }
                if key == Key::Enter && action == Action::Press {
                    events().char_input.push('\n');
                }
            }
            WindowEvent::Char(char_) => {
                events().char_input.push(char_);
            }
            WindowEvent::CursorPos(x, y) => {
                events().update_cursor_pos((x, y));
            }
            WindowEvent::MouseButton(button, action, modifiers) => {
                events()
                    .mouse_button_input
                    .push((button, action, modifiers));
            }
            WindowEvent::Scroll(x, y) => {}
            WindowEvent::FramebufferSize(w, h) if w != 0 && h != 0 => {
                for callback in framebuffer_size_callbacks.iter_mut() {
                    callback.framebuffer_size((w, h));
                }
            }
            WindowEvent::Focus(focused) => {}
            WindowEvent::FileDrop(paths) => {}
            _ => {}
        }
    }
}

fn render_iteration() {
    screen().render_offscreen(renderer().framebuffer());
    window().swap_buffers();
}

fn script_iteration() {}

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
        self.cursor_offset.0 = self.cursor_pos.0 - cursor_pos.0;
        self.cursor_offset.1 = self.cursor_pos.1 - cursor_pos.1;
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

    pub fn clear_events(&mut self) {
        self.key_input.clear();
        self.char_input.clear();
        self.mouse_button_input.clear();
    }
}

pub trait FramebufferSizeCallback {
    fn framebuffer_size(&mut self, size: (i32, i32));
}
