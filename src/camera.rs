extern crate nalgebra_glm as glm;

use crate::to_rad;
use nalgebra_glm::{Mat4, Vec3};

pub struct Camera {
    position: Vec3,
    direction: Vec3,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Vec3::zeros(),
            direction: -Vec3::z(),
        }
    }

    pub fn get_view(&self) -> Mat4 {
        glm::look_at(
            &self.position,
            &(self.position + self.direction),
            &Vec3::y(),
        )
    }

    pub fn get_position(&self) -> &Vec3 {
        &self.position
    }

    pub fn move_(&mut self, delta: &Vec3) {
        self.position += delta;
    }

    pub fn move_local(&mut self, delta: &Vec3) {
        let (local_forward, local_up) = (self.direction, Vec3::y());

        let mut local_right = glm::cross(&local_up, &(-local_forward));
        local_right = glm::normalize(&local_right);

        let delta = local_forward * delta.z + local_right * delta.x + local_up * delta.y;
        self.move_(&delta);
    }

    pub fn translate(&mut self, position: &Vec3) {
        self.position = *position;
    }

    pub fn rotate(&mut self, rotation: &Vec3) {
        let yaw = to_rad(rotation.y);
        let pitch = to_rad(rotation.x);
        self.direction = glm::rotate_vec3(&self.direction, pitch, &Vec3::x_axis());
        self.direction = glm::rotate_vec3(&self.direction, yaw, &Vec3::y_axis());
    }

    pub fn rotate_local(&mut self, rotation: &Vec3) {
        let pitch = to_rad(rotation.x);
        let yaw = to_rad(rotation.y);

        let new_forward = glm::rotate_vec3(&self.direction, yaw, &Vec3::y());
        let new_right = glm::cross(&Vec3::y(), &(-new_forward));

        self.direction = glm::rotate_vec3(&new_forward, pitch, &new_right);
    }
}
