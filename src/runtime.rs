use crate::{
    application::Application,
    asset_loader,
    data3d::ModelContainer,
    entity_system::{CameraComponent, EntitySystem, MeshComponent},
    scene::{self, Scene},
    serializable,
};
use glfw::{Action, Context as _, Key, Modifiers, MouseButton, WindowEvent};

pub struct Runtime;

impl Runtime {
    pub fn new() -> Self {
        Self {}
    }

    fn process_window_events(app: &mut Application) {
        app.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&app.receiver) {
            match event {
                WindowEvent::Key(key, _, action @ (Action::Press | Action::Release), modifiers) => {
                }
                WindowEvent::Char(char_) => {}
                WindowEvent::CursorPos(x, y) => {}
                WindowEvent::FramebufferSize(w, h) => {}
                WindowEvent::Focus(focused) => {}
                WindowEvent::FileDrop(paths) => {}
                _ => {}
            }
        }
    }

    fn update_input() {}

    fn script_cycle() {}

    fn render_cycle(app: &mut Application) {
        app.window.swap_buffers();
    }

    fn handle_closing(app: &Application) -> bool {
        !app.window.should_close()
    }

    pub fn run(self, mut app: Application) {
        app.window.focus();
        let context = Context::init().expect("Scene integrity is violated");
        let mut frame_time = 0.0;
        while Self::handle_closing(&app) {
            app.glfw.set_time(0.0);
            Self::process_window_events(&mut app);
            Self::update_input();
            Self::script_cycle();
            Self::render_cycle(&mut app);
            frame_time = app.glfw.get_time();
        }

        // let mut input = Input::new();
        // let mut render_pipeline = RenderPipeline::new(app.window.get_framebuffer_size());
        // let mut frame_time = 0.0;
        // while !app.window.should_close() {
        //     app.glfw.set_time(0.0);
        //     app.glfw.poll_events();
        //     input.clear_events();
        //     for (_, event) in glfw::flush_messages(&app.receiver) {
        //         match event {
        //             WindowEvent::FramebufferSize(w, h) => {
        //                 render_pipeline.on_framebuffer_size((w, h));
        //             }
        //             WindowEvent::Key(key, _, action, modifier) => {
        //                 if action == Action::Repeat {
        //                     return;
        //                 }
        //                 input.key_events.push((key, action, modifier));
        //             }
        //             WindowEvent::MouseButton(button, action, modifier) => {
        //                 input.mouse_button_events.push((button, action, modifier));
        //             }
        //             _ => {}
        //         }
        //     }
        //     render_pipeline.draw_cycle();
        //     app.window.swap_buffers();
        //     // std::thread::sleep(std::time::Duration::from_millis(100));
        //     frame_time = app.glfw.get_time();
        // }
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

struct Context {
    entity_system: EntitySystem,
    model_container: ModelContainer,
    scenes: Vec<Scene>,
}

impl Context {
    pub fn init() -> Result<Self, ()> {
        let model_container = asset_loader::load_all_models();
        let scenes = scene::get_scenes();
        let initial = scenes.get(0).ok_or(())?;
        let entity_system = Self::load_scene(initial, &model_container);

        Ok(Self {
            entity_system,
            model_container,
            scenes,
        })
    }

    fn load_scene(scene: &Scene, model_container: &ModelContainer) -> EntitySystem {
        let mut entity_system = EntitySystem::from_scene(scene);
        let mesh_components = Self::mesh_components(scene, &model_container);
        entity_system.attach_components(mesh_components);
        let cameras = scene.read_vec::<serializable::CameraComponent>();
        let mut camera_components = Vec::<CameraComponent>::with_capacity(cameras.len());
        for camera in cameras {
            camera_components.push(camera.into());
        }
        entity_system.attach_components(camera_components);
        entity_system
    }

    fn mesh_components(scene: &Scene, model_container: &ModelContainer) -> Vec<MeshComponent> {
        let meshes = scene.read_vec::<serializable::MeshComponent>();
        let mut mesh_components = Vec::with_capacity(meshes.capacity());
        for mesh in meshes {
            let mesh_component = MeshComponent {
                model_index: model_container.get_model_index(&mesh.mesh_path),
                owner_id: mesh.owner_id,
            };
            mesh_components.push(mesh_component);
        }
        mesh_components
    }
}
