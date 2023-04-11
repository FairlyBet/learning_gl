use glfw::{Action, Key};
use nalgebra_glm::Vec3;

use crate::{camera::Camera, input::Input};

pub struct CameraController<'a> {
    camera: &'a mut Camera,
    input: &'a Input<'a>,
    key_bindings: Actions,
}

impl<'a> CameraController<'_> {
    pub fn new(
        camera: &'a mut Camera,
        input: &'a Input,
        key_bindings: Actions,
    ) -> CameraController<'a> {
        CameraController {
            camera,
            input,
            key_bindings,
        }
    }

    pub fn update(&self) {
        let mut direction = Vec3::zeros();
        if self
            .input
            .get_key_with_action(Actions::map(Actions::Up), (Action::Press as i32| Action::Repeat as i32) as Action)
        {
        }
    }
}

#[derive(Clone, Copy)]
pub enum Actions {
    Up,
    Right,
    Down,
    Left,
    Shift,
}

impl Actions {
    pub fn map(value: Actions) -> Key {
        match value {
            Actions::Up => Key::W,
            Actions::Left => Key::A,
            Actions::Down => Key::S,
            Actions::Right => Key::D,
            Actions::Shift => Key::LeftShift,
        }
    }
}
