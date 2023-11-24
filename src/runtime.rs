use crate::rendering::RenderPipeline;
use glfw::{Action, Context as _, Glfw, Key, Modifiers, MouseButton, Window, WindowEvent};
use std::{
    path::{Path, PathBuf},
    sync::mpsc::Receiver, fs,
};

pub struct Runtime;

impl Runtime {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(
        self,
        glfw: &mut Glfw,
        window: &mut Window,
        receiver: &Receiver<(f64, WindowEvent)>,
    ) {
        let mut render_pipeline = RenderPipeline::new(window.get_framebuffer_size());
        let mut event_sys = InputEvents::new();

        let mut frame_time = 0.0;
        loop {
            if window.should_close() {
                break;
            }

            glfw.set_time(0.0);
            glfw.poll_events();
            event_sys.clear_events();
            for (_, event) in glfw::flush_messages(receiver) {
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
            window.swap_buffers();
            // std::thread::sleep(std::time::Duration::from_millis(100));

            frame_time = glfw.get_time();
        }
    }
}

pub struct InputEvents {
    key_events: Vec<(Key, Action, Modifiers)>,
    mouse_button_events: Vec<(MouseButton, Action, Modifiers)>,
}

impl InputEvents {
    pub fn new() -> Self {
        Self {
            key_events: vec![],
            mouse_button_events: vec![],
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

struct AssetManager {
    meshes: Vec<String>,
}

impl AssetManager {
    const ASSET_PATH: &str = "assets";
    const MESHES_PATH: &str = "meshes";

    pub fn read_meshes(&mut self) {
        let path = Path::new(Self::ASSET_PATH).join(Self::MESHES_PATH);
        let entries = fs::read_dir(path).unwrap();
        for entry in entries {
            entry
        }
    }
}
