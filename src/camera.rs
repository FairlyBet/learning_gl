use glfw::{Action, Key, Window};
use nalgebra_glm::{vec3, Vec3};

static mut X: f32 = 0.0;
static mut Y: f32 = 0.0;
static mut Z: f32 = 0.0;
static VELOCITY: f32 = 0.2;

pub fn update(window: &Window) {
    unsafe {
        if let Action::Repeat | Action::Press = window.get_key(Key::W) {
            Y += 1.0;
        }
        if let Action::Repeat | Action::Press = window.get_key(Key::S) {
            Y -= 1.0;
        }
        if let Action::Repeat | Action::Press = window.get_key(Key::A) {
            X -= 1.0;
        }
        if let Action::Repeat | Action::Press = window.get_key(Key::D) {
            X += 1.0;
        }
        Y = Y.clamp(-1.0, 1.0);
        X = X.clamp(-1.0, 1.0);
    }
}

pub fn get_position() -> Vec3 {
    unsafe { vec3(X, Y, Z).normalize() * VELOCITY }
}
