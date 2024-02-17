use crate::{
    application::Application,
    entity_system::SceneManager,
    gl_wrappers,
    rendering::{DefaultRenderer, Screen},
    resources::ResourceManager,
    scene,
    scripting::Scripting,
};
use glfw::{Action, Context as _, Key, Modifiers, MouseButton, WindowEvent};
use rlua::{Function, RegistryKey, Table};
use std::{
    fs::{self, File},
    io::Write as _,
};

pub struct Runtime;

impl Runtime {
    pub fn new() -> Self {
        Self {}
    }

    fn update_events(
        app: &mut Application,
        events: &mut WindowEvents,
        framebuffer_size_callbacks: &mut Vec<&mut dyn FramebufferSizeCallback>,
    ) {
        app.glfw.poll_events();
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
                    if (w == 0 || h == 0) {
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

    fn render_iteration(app: &mut Application) {
        gl_wrappers::clear(gl::COLOR_BUFFER_BIT);
        app.window.swap_buffers();
    }

    fn script_iteration(scripting: &Scripting, object_key: &RegistryKey) {
        scripting.lua.context(|context| {
            let object = context.registry_value::<Table>(object_key).unwrap();
            let update = object.get::<_, Function>("update").unwrap();
            update.call::<_, ()>(object).unwrap();
        });
    }

    pub fn run(self, mut app: Application) {
        let scenes = scene::get_scenes();

        let start = match scenes.first() {
            Some(scene) => scene,
            None => return,
        };

        let mut resource_manager = ResourceManager::new();
        resource_manager.load(&start);

        let mut scene_manager = SceneManager::from_scene(&start, &resource_manager);

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

        let test_script = "
require \"assets.scripts.vector\"
        
local object = {}

function object:update()
    if Input.getKey(Keys.Space, Actions.Press) then
        print(Transform.getPosition(self))
    end

    if Input.getKeyHolded(Keys.W) then
        Transform.move(self, Vector:new(1, 1, 1) * frameTime())
    end
end

return object
        ";

        let scripting = Scripting::new();
        scripting.create_wrappers(&mut scene_manager, &events, &app.window, &frame_time);
        let object_key = scripting.lua.context(|context| {
            let chunk = context.load(test_script);
            let object = chunk.eval::<Table>().unwrap();
            let object_key = context.create_registry_value(object).unwrap();
            let object = context.registry_value::<Table>(&object_key).unwrap();
            scripting.register_object_id(&context, object, 1);
            object_key
        });

        // app.window.focus();
        let mut file = File::create("input.txt").unwrap();

        while !app.window.should_close() {
            app.glfw.set_time(0.0);

            // file.write(&events.char_input.as_bytes());
            Self::update_events(&mut app, &mut events, &mut vec![&mut renderer, &mut screen]);
            Self::script_iteration(&scripting, &object_key);
            Self::render_iteration(&mut app);

            frame_time = app.glfw.get_time();
        }
    }
}

#[derive(Default)]
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
