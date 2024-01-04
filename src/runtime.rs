use crate::{
    application::Application,
    asset_manager::AssetManager,
    entity_system::{self, CameraComponent, SceneChunk},
    rendering::{DefaultRenderer, Renderer, Screen},
    scene::{self, Scene},
    serializable,
};
use glfw::{Action, Context as _, Key, Modifiers, MouseButton, WindowEvent};

pub struct Runtime;

impl Runtime {
    pub fn new() -> Self {
        Self {}
    }

    fn update_input(app: &mut Application, input: &mut Input) {
        input.clear_events();
        app.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&app.receiver) {
            match event {
                WindowEvent::Key(key, _, action @ (Action::Press | Action::Release), modifiers) => {
                }
                WindowEvent::Char(char_) => {}
                WindowEvent::CursorPos(x, y) => {}
                WindowEvent::MouseButton(
                    button,
                    action @ (Action::Press | Action::Release),
                    modifiers,
                ) => {}
                WindowEvent::FramebufferSize(w, h) => {}
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

    fn handle_closing(app: &Application) -> bool {
        !app.window.should_close()
    }

    pub fn run(self, mut app: Application) {
        let scenes = scene::get_scenes();
        if let None = scenes.first() {
            return;
        }
        let first = scenes.first().unwrap();

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

        let mut input = Input::new();

        app.window.focus();

        let mut frame_time = 0.0;

        loop {
            app.glfw.set_time(0.0);
            Self::update_input(&mut app, &mut input);
        }
    }
}

pub struct Input {
    key_events: Vec<(Key, Action, Modifiers)>,
    mouse_button_events: Vec<(MouseButton, Action, Modifiers)>,
    cursor_pos: (u32, u32),
}

impl Input {
    pub fn new() -> Self {
        Self {
            key_events: vec![],
            mouse_button_events: vec![],
            cursor_pos: Default::default(),
        }
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
    }
}
