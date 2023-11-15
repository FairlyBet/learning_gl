use crate::{gl_wrappers::Gl, rendering::RenderPipeline};
use glfw::{
    Action, Context, Glfw, Key, Modifiers, MouseButton, OpenGlProfileHint, SwapInterval, Window,
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
    glfw: Glfw,
    window: Window,
    receiver: Receiver<(f64, WindowEvent)>,
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
        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);
        window.set_framebuffer_size_polling(true);
    }

    pub fn run(mut self) {
        let mut render_pipeline = RenderPipeline::new(self.window.get_framebuffer_size());
        let mut event_sys = EventSys::new();

        let mut frame_time = 0.0;
        loop {
            if self.window.should_close() {
                break;
            }

            self.glfw.set_time(0.0);
            self.glfw.poll_events();
            event_sys.clear_events();
            for (_, event) in glfw::flush_messages(&self.receiver) {
                match event {
                    WindowEvent::FramebufferSize(w, h) => {
                        render_pipeline.on_framebuffer_size((w, h));
                    }
                    WindowEvent::Key(key, _, action, modifier) => {
                        event_sys.key_events.push((key, action, modifier));
                    }
                    WindowEvent::MouseButton(button, action, modifier) => {
                        event_sys
                            .mouse_button_events
                            .push((button, action, modifier));
                    }
                    _ => {}
                }
            }

            render_pipeline.draw_cycle();
            self.window.swap_buffers();
            // std::thread::sleep(std::time::Duration::from_millis(100));

            frame_time = self.glfw.get_time();
        }
    }
}

pub struct EventSys {
    key_events: Vec<(Key, Action, Modifiers)>,
    mouse_button_events: Vec<(MouseButton, Action, Modifiers)>,
}

impl EventSys {
    pub fn new() -> Self {
        Self {
            key_events: vec![],
            mouse_button_events: vec![],
        }
    }

    // pub fn subscribe_framebuffersize(&mut self, listener: &'a mut dyn OnFramebufferSize) {
    //     self.on_framebuffer_size.push(listener);
    // }

    pub fn get_key(&self, key: (Key, Action, Modifiers)) -> bool {
        self.key_events.contains(&key)
    }

    pub fn get_mouse_button(&self, button: (MouseButton, Action, Modifiers)) -> bool {
        self.mouse_button_events.contains(&button)
    }

    // fn update_framebuffer_size(&self, size: (i32, i32)) {
    //     for subscriber in &self.on_framebuffer_size {
    //         let subscriber = unsafe { &mut **subscriber as &mut dyn OnFramebufferSize };
    //         subscriber.on_framebuffer_size(size);
    //     }
    // }

    fn clear_events(&mut self) {
        self.key_events.clear();
        self.mouse_button_events.clear();
    }
}

// pub trait OnFramebufferSize {
//     fn on_framebuffer_size(&mut self, size: (i32, i32));
// }
