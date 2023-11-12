use crate::gl_wrappers::Gl;
use glfw::{
    Action, Context, Glfw, Key, Modifiers, OpenGlProfileHint, SwapInterval, Window, WindowEvent,
    WindowHint, WindowMode,
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
    glfw: Glfw,
    window: Window,
    receiver: Receiver<(f64, WindowEvent)>,
    event_sys: EventSys,
}

impl Application {
    pub fn new() -> Self {
        let gl = Gl::load();

        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(OPENGL_PROFILE);
        glfw.window_hint(CONTEXT_VERSION);

        let (mut window, receiver) = glfw
            .create_window(DEFAULT_WIDTH, DEFAULT_HEIGHT, DEFAULT_TITLE, DEFAULT_MODE)
            .unwrap();
        Self::enable_polling(&mut window);
        window.make_current();
        glfw.set_swap_interval(SwapInterval::Sync(VSYNC.into()));

        let mut event_sys = EventSys::new();

        Self {
            gl,
            glfw,
            window,
            receiver,
            event_sys,
        }
    }

    fn enable_polling(window: &mut Window) {
        window.set_key_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_framebuffer_size_polling(true);
    }

    pub fn run(mut self) {
        let mut frame_time = 0.0;
        loop {
            if self.window.should_close() {
                break;
            }
            self.glfw.set_time(0.0);
            self.event_sys.clear_key_events();
            self.glfw.poll_events();
            for (_, event) in glfw::flush_messages(&self.receiver) {
                match event {
                    WindowEvent::FramebufferSize(w, h) => {
                        self.event_sys.update_framebuffer_size((w, h));
                    }
                    WindowEvent::Key(key, _, action, modifier) => {
                        self.event_sys.push_key_event((key, action, modifier));
                    }
                    _ => {}
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
            frame_time = self.glfw.get_time();
        }
    }
}

pub struct EventSys {
    on_framebuffer_size: Vec<*mut dyn OnFramebufferSize>,
    key_events: Vec<(Key, Action, Modifiers)>,
}

impl EventSys {
    pub fn new() -> Self {
        Self {
            on_framebuffer_size: Vec::new(),
            key_events: vec![],
        }
    }

    pub fn subscribe_framebuffersize(&mut self, listener: *mut dyn OnFramebufferSize) {
        self.on_framebuffer_size.push(listener);
    }

    pub fn get_key_events(&self) -> &Vec<(Key, Action, Modifiers)> {
        &self.key_events
    }

    pub fn get_key_event(&self, event: (Key, Action, Modifiers)) -> bool {
        self.key_events.contains(&event)
    }

    fn update_framebuffer_size(&self, size: (i32, i32)) {
        for subscriber in &self.on_framebuffer_size {
            let subscriber = unsafe { &mut **subscriber as &mut dyn OnFramebufferSize };
            subscriber.on_framebuffer_size(size);
        }
    }

    fn push_key_event(&mut self, event: (Key, Action, Modifiers)) {
        self.key_events.push(event);
    }

    fn clear_key_events(&mut self) {
        self.key_events.clear();
    }
}

pub trait OnFramebufferSize {
    fn on_framebuffer_size(&mut self, size: (i32, i32));
}
