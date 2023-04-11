use glfw::{Action, Key};
use nalgebra_glm::Vec3;

use crate::{camera::Camera, input::Input};

const VELOCTITY: f32 = 0.1666;

pub struct CameraController<'a> {
    input: &'a Input<'a>,
}

impl<'a> CameraController<'_> {
    pub fn new(input: &'a Input) -> CameraController<'a> {
        CameraController { input }
    }

    pub fn update(&mut self, camera: &mut Camera) {
        let input = self.input;
        let mut delta = Vec3::zeros();
        if let Action::Press | Action::Release = input.get_key(Key::W) {
            delta.z += 1.0;
        }
        if let Action::Press | Action::Release = input.get_key(Key::A) {
            delta.x -= 1.0;
        }
        if let Action::Press | Action::Release = input.get_key(Key::S) {
            delta.z -= 1.0;
        }
        if let Action::Press | Action::Release = input.get_key(Key::D) {
            delta.x += 1.0;
        }
        delta = delta * VELOCTITY;
        camera.move_(&delta);
    }
}
