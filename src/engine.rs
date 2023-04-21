use glfw::{
    Action, Context, CursorMode, Glfw, Key, OpenGlProfileHint, SwapInterval, Window, WindowEvent,
    WindowHint, WindowMode,
};
use nalgebra_glm::Mat4;
use std::sync::mpsc::Receiver;

use crate::{camera::Camera, object::Object, to_rad};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const FOV_Y: f32 = 45.0;
const NEAR: f32 = 0.1;
const FAR: f32 = 100.0;

pub struct Engine<'a> {
    glfw: Glfw,
    window: Window,
    receiver: Receiver<(f64, WindowEvent)>,

    camera: Camera,
    camera_updater: Option<fn(&Camera, f32) -> ()>,
    projection: Mat4,

    objects: Vec<&'a Object<'a>>,
}

impl<'a> Engine<'a> {
    pub fn new() -> Engine<'a> {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        glfw.window_hint(WindowHint::ContextVersion(3, 3));

        let (mut window, receiver) = glfw
            .create_window(WIDTH, HEIGHT, "", WindowMode::Windowed)
            .unwrap();
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);
        window.set_cursor_pos_polling(true);
        window.make_current();

        gl_loader::init_gl();
        gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Enable(gl::DEPTH_TEST);
        }

        Engine {
            glfw,
            receiver,
            camera: Camera::new(),
            camera_updater: None,
            objects: vec![],
            projection: glm::perspective(
                calculate_aspect(window.get_framebuffer_size()),
                to_rad(FOV_Y),
                NEAR,
                FAR,
            ),
            window,
        }
    }

    pub fn set_camera_updater(&mut self, updater: fn(&Camera, f32) -> ()) {
        self.camera_updater = Some(updater);
    }

    pub fn set_swap_interval(&mut self, interval: SwapInterval) {
        self.glfw.set_swap_interval(interval);
    }

    pub fn set_cursor_mode(&mut self, mode: CursorMode) {
        self.window.set_cursor_mode(mode);
    }

    pub fn add_object(&mut self, object: &'a Object<'a>) {
        self.objects.push(object);
    }

    pub fn main_loop(&mut self) {
        let mut frametime = 0.0;
        while !self.window.should_close() {
            self.glfw.set_time(0.0);
            self.window.set_cursor_pos(0.0, 0.0);
            self.glfw.poll_events();

            if let Some(updater) = self.camera_updater {
                updater(&self.camera, frametime);
            }

            handle_window_events(self);

            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }
            for object in &self.objects {
                object.get_renderer().bind();
                object.get_renderer().draw(&object.get_transform()); // Проброс трансформа в параметр
                                                                     // Очистка ресурсов из objects и просто
            }
            self.window.swap_buffers();

            frametime = self.glfw.get_time() as f32;
        }
        // end gl после очистки всех ресурсов
    }
}

impl<'a> Drop for Engine<'a> {
    fn drop(&mut self) {
        gl_loader::end_gl();
    }
}

fn handle_window_events(engine: &mut Engine) {
    for (_, event) in glfw::flush_messages(&engine.receiver) {
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                engine.window.set_should_close(true)
            }
            WindowEvent::FramebufferSize(width, height) => unsafe {
                let aspect = calculate_aspect((width, height));
                engine.projection = glm::perspective(aspect, to_rad(FOV_Y), NEAR, FAR);
                gl::Viewport(0, 0, width, height);
            },
            _ => {}
        }
    }
}

fn calculate_aspect(framebuffer_size: (i32, i32)) -> f32 {
    framebuffer_size.0 as f32 / framebuffer_size.1 as f32
}
