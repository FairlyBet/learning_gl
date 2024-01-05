use crate::{
    application::Application,
    asset_manager::AssetManager,
    entity_system::SceneChunk,
    rendering::{DefaultRenderer, Screen},
    scene,
};
use glfw::{Action, Key, Modifiers, MouseButton, WindowEvent};
use std::{thread, time::Duration};

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
                WindowEvent::Key(key, _, action @ (Action::Press | Action::Release), modifiers) => {
                    events.key_events.push((key, action, modifiers));
                }
                WindowEvent::Char(char_) => {
                    events.char_events.push(char_);
                }
                WindowEvent::CursorPos(x, y) => {
                    events.update_cursor_pos((x, y));
                }
                WindowEvent::MouseButton(
                    button,
                    action @ (Action::Press | Action::Release),
                    modifiers,
                ) => {
                    events.mouse_button_events.push((button, action, modifiers));
                }
                WindowEvent::FramebufferSize(w, h) => {
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

    // fn script_iteration(context: &mut Context) {
    // let scripting = Scripting::new();
    // scripting.lua.context(|state| {
    //     let chunk = state.load("source");
    //     chunk.call(args)
    // });
    // let script_files = asset_loader::get_paths::<Scripting>();
    // script_files
    //     .iter()
    //     .for_each(|item| _ = scripting.execute_file(Path::new(&item)));
    // }
    // fn render_iteration(
    //     app: &mut Application,
    //     screen: &Screen,
    //     context: &Context,
    //     renderer: &impl Renderer,
    // ) {
    // renderer.render(
    //     &context.entity_system,
    //     &context.model_container,
    //     screen.offscreen_buffer(),
    // );
    // screen.render_offscreen();
    //     app.window.swap_buffers();
    // }

    pub fn run(self, mut app: Application) {
        let scenes = scene::get_scenes();
        let first = match scenes.first() {
            Some(scene) => scene,
            None => return,
        };

        let mut asset_manager = AssetManager::new();
        asset_manager.load(&first);

        let mut chunk = SceneChunk::from_scene(&first, &asset_manager);

        let mut renderer = DefaultRenderer::new(
            app.window.get_framebuffer_size(),
            app.window.get_context_version(),
        );

        let mut screen = Screen::new(
            app.window.get_framebuffer_size(),
            app.window.get_context_version(),
        );

        let mut input = WindowEvents::default();

        app.window.focus();

        let mut frame_time = 0.0;

        while !app.window.should_close() {
            app.glfw.set_time(0.0);
            Self::update_events(&mut app, &mut input, &mut vec![&mut renderer, &mut screen]);
            thread::sleep(Duration::from_millis(20));
            frame_time = app.glfw.get_time();
        }
    }
}

#[derive(Default)]
pub struct WindowEvents {
    key_events: Vec<(Key, Action, Modifiers)>,
    char_events: Vec<char>,
    mouse_button_events: Vec<(MouseButton, Action, Modifiers)>,
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
        self.key_events.contains(&key)
    }

    pub fn get_mouse_button(&self, button: (MouseButton, Action, Modifiers)) -> bool {
        self.mouse_button_events.contains(&button)
    }

    pub fn clear_events(&mut self) {
        self.key_events.clear();
        self.mouse_button_events.clear();
        self.char_events.clear();
    }
}

pub trait FramebufferSizeCallback {
    fn framebuffer_size(&mut self, size: (i32, i32));
}
