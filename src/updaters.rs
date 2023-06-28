use crate::data_structures::{EngineApi, ViewObject};
use glfw::{Action, Key};
use nalgebra_glm::{vec3, Vec3};

pub fn on_framebuffer_size_change(w: i32, h: i32) {
    unsafe {
        gl::Viewport(0, 0, w, h);
    }
}

pub fn close_on_escape(key: Key, action: Action, api: &mut EngineApi) {
    if key == Key::Escape && action == Action::Press {
        api.set_should_close_true();
    }
}

pub fn default_camera_controller(camera: &mut ViewObject, api: EngineApi) {
    let sensitivity = 2.0;
    let pos = api.get_cursor_pos();
    let x = pos.0 as f32;
    let y = pos.1 as f32;
    let local_rotation = vec3(-y, 0.0, 0.0) * sensitivity * api.get_frametime();
    let global_rotation = vec3(0.0, -x, 0.0) * sensitivity * api.get_frametime();

    let mut delta = Vec3::zeros();
    let velocity = 5.0;
    if let Action::Press | Action::Repeat = api.get_key(Key::W) {
        delta.z += 1.0;
    }
    if let Action::Press | Action::Repeat = api.get_key(Key::A) {
        delta.x -= 1.0;
    }
    if let Action::Press | Action::Repeat = api.get_key(Key::S) {
        delta.z -= 1.0;
    }
    if let Action::Press | Action::Repeat = api.get_key(Key::D) {
        delta.x += 1.0;
    }
    if delta.magnitude() > 0.0 {
        delta = glm::normalize(&delta); // returning nan
    }
    delta *= velocity * api.get_frametime();

    // camera.rotate(&global_rotation);
    // camera.rotate_local(&local_rotation);
    // camera.move_local(&delta);
}