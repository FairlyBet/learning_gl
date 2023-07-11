use crate::{
    data_structures::{EngineApi, Projection, ViewObject},
    get_aspect,
};
use glfw::{Action, Key};
use glm::Mat4;
use nalgebra_glm::{vec3, Vec3};

pub fn update_viewport(w: i32, h: i32) {
    unsafe {
        gl::Viewport(0, 0, w, h);
    }
}

pub fn update_perspective(w: i32, h: i32) -> Mat4 {
    let aspect = get_aspect((w, h));
    Projection::Perspective(aspect, 45.0, 0.1, 100.0).calculate_matrix()
}

pub fn default_camera_controller(camera: &mut ViewObject, api: &EngineApi) {
    let sensitivity = 0.07;
    let pos = api.get_cursor_offset();
    let x = pos.0;
    let y = pos.1;

    let local_rotation = vec3(-y, 0.0, 0.0) * sensitivity;
    let global_rotation = vec3(0.0, -x, 0.0) * sensitivity;

    let mut delta = Vec3::zeros();
    let velocity = 5.0;
    if let Action::Press | Action::Repeat = api.get_key(Key::W) {
        delta.z -= 1.0;
    }
    if let Action::Press | Action::Repeat = api.get_key(Key::A) {
        delta.x -= 1.0;
    }
    if let Action::Press | Action::Repeat = api.get_key(Key::S) {
        delta.z += 1.0;
    }
    if let Action::Press | Action::Repeat = api.get_key(Key::D) {
        delta.x += 1.0;
    }
    if delta.magnitude() > 0.0 {
        delta = glm::normalize(&delta); // returning nan
    }
    delta *= velocity * api.get_frametime();

    camera.transform.rotate(&global_rotation);
    camera.transform.rotate_local(&local_rotation);
    camera.transform.move_local(&delta);
}
