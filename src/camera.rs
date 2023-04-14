extern crate nalgebra_glm as glm;

use crate::to_rad;
use glm::{vec2, vec3};
use nalgebra_glm::{Mat4, Vec3};

pub struct Camera {
    position: Vec3,
    rotation: Vec3,
    direction: Vec3,
    up: Vec3,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Vec3::zeros(),
            rotation: Vec3::zeros(),
            direction: -Vec3::z(),
            up: Vec3::y(),
        }
    }

    pub fn get_view(&self) -> Mat4 {
        glm::look_at(&self.position, &(self.position + self.direction), &self.up)
    }

    pub fn get_position(&self) -> &Vec3 {
        &self.position
    }

    pub fn get_rotation(&self) -> &Vec3 {
        &self.rotation
    }

    // pub fn get_direction_and_up(&self) -> (Vec3, Vec3) {
    //     let mut direction = -Vec3::z();
    //     let mut up = Vec3::y();
    //     let yaw = to_rad(self.rotation.y);
    //     let pitch = to_rad(self.rotation.x);

    //     direction = glm::rotate_vec3(&direction, pitch, &Vec3::x_axis());
    //     direction = glm::rotate_vec3(&direction, yaw, &Vec3::y_axis());

    //     up = glm::rotate_vec3(&up, pitch, &Vec3::x_axis());
    //     up = glm::rotate_vec3(&up, yaw, &Vec3::y_axis());
    //     (direction, up)
    // }

    pub fn move_(&mut self, delta: &Vec3) {
        self.position += delta;
    }

    pub fn move_local(&mut self, delta: &Vec3) {
        let (local_forward, local_up) = (self.direction, self.up);

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
        self.up = glm::rotate_vec3(&self.up, pitch, &Vec3::x_axis());
        self.up = glm::rotate_vec3(&self.up, yaw, &Vec3::y_axis());
    }

    pub fn rotate_local(&mut self, rotation: &Vec3) {
        let pitch = to_rad(rotation.x);
        let yaw = to_rad(rotation.y);

        let new_forward = glm::rotate_vec3(&self.direction, yaw, &self.up);
        let new_right = glm::cross(&self.up, &(-new_forward));

        self.direction = glm::rotate_vec3(&new_forward, pitch, &new_right);
        // let new_up = glm::cross(&(-new_forward), &new_right);
        // -Vec3::z();
        // vec3(new_forward.x, 0.0, new_forward.z);
        // vec3(0.0,  , z)
        // let yaw = glm::angle(&new_forward, &Vec3::x());
    }
}
