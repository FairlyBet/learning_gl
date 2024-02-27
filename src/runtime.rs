use crate::{
    application::Application,
    entity_system::SceneManager,
    gl_wrappers,
    rendering::{DefaultRenderer, Screen},
    resources::ResourceManager,
    scripting::Scripting,
};
use glfw::{Action, Context as _, Key, Modifiers, MouseButton, PWindow, WindowEvent};

#[derive(Debug)]
pub struct Runtime;

impl Runtime {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(self) {
        let mut app = Application::new();
        let mut resource_manager = ResourceManager::new();
        let mut scene_manager = SceneManager::default();
        let mut renderer = DefaultRenderer::new(
            app.window.get_framebuffer_size(),
            app.window.get_context_version(),
        );
        let mut screen = Screen::new(
            app.window.get_framebuffer_size(),
            app.window.get_context_version(),
        );
        let mut events = WindowEvents::default();
        let mut frame_time = 0.0;
        let scripting = Scripting::new();

        resource_manager.load_scripts(&scripting);
        scripting.create_wrappers(&scene_manager, &events, &app.window, &frame_time);
        scene_manager =
            SceneManager::from_scene_index(0, &mut resource_manager, &scripting).unwrap();


        let mut collect_time = 0.0;
        while !app.window.should_close() {
            app.window.glfw.set_time(0.0);

            Self::update_events(&mut app, &mut events, &mut vec![&mut renderer, &mut screen]);
            Self::script_iteration(&scripting, &mut scene_manager, &mut app.window);
            Self::render_iteration(&mut app);
            
            collect_time += frame_time;
            if collect_time >= 30.0 {
                scripting.gc_collect();
                println!("Collect");
                collect_time = 0.0;
            }

            frame_time = app.window.glfw.get_time();
        }
    }

    fn update_events(
        app: &mut Application,
        events: &mut WindowEvents,
        framebuffer_size_callbacks: &mut Vec<&mut dyn FramebufferSizeCallback>,
    ) {
        app.window.glfw.poll_events();
        events.clear_events();
        for (_, event) in glfw::flush_messages(&app.receiver) {
            match event {
                WindowEvent::Key(Key::Enter, _, Action::Repeat, _) => {
                    events.char_input.push('\n');
                }
                WindowEvent::Key(key, _, action @ (Action::Press | Action::Release), modifiers) => {
                    events.key_input.push((key, action, modifiers));
                    if key == Key::V && action == Action::Press && modifiers == Modifiers::Control {
                        if let Some(string) = app.window.get_clipboard_string() {
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
                WindowEvent::CursorPos(x, y) => {
                    events.update_cursor_pos((x, y));
                }
                WindowEvent::MouseButton(button, action, modifiers) => {
                    events.mouse_button_input.push((button, action, modifiers));
                }
                WindowEvent::Scroll(x, y) => {}
                WindowEvent::FramebufferSize(w, h) => {
                    if w == 0 || h == 0 {
                        continue;
                    }
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

    fn script_iteration(
        scripting: &Scripting,
        scene_manager: &mut SceneManager,
        window: &mut PWindow,
    ) {
        scripting.run_updates(scene_manager, window);
    }

    fn render_iteration(app: &mut Application) {
        gl_wrappers::clear(gl::COLOR_BUFFER_BIT);
        app.window.swap_buffers();
    }
}

#[derive(Default, Debug)]
pub struct WindowEvents {
    key_input: Vec<(Key, Action, Modifiers)>,
    char_input: String,
    mouse_button_input: Vec<(MouseButton, Action, Modifiers)>,
    cursor_offset: (f64, f64),
    cursor_pos: (f64, f64),
}

impl WindowEvents {
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
