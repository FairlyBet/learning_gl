use nalgebra_glm::{Mat4, Vec3};

pub struct Camera {
    position: Vec3,
    direction: Vec3,
    view: Mat4,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Vec3::zeros(),
            // direction: -*Vec3::z_axis(),
            // direction: Vec3::new(0.707, 0.707, 0.0), Требуется фикс
            view: Mat4::identity(),
        }
    }

    pub fn get_view(&self) -> Mat4 {
        self.view
    }

    pub fn get_position(&self) -> &Vec3 {
        &self.position
    }

    pub fn look_at(&mut self, target: &Vec3) {
        self.view = glm::look_at(&self.position, &target, &Vec3::y_axis());
    }

    pub fn move_(&mut self, delta: &Vec3) {
        self.position += delta;
        self.view = glm::translate(&Mat4::identity(), &-self.position);
    }

    pub fn move_local(&mut self, delta: &Vec3) {
        let y_axis = Vec3::y_axis();
        let forward = glm::normalize(&self.direction);
        let right = glm::cross(&y_axis, &(-forward));
        let right = glm::normalize(&right);
        let up = glm::cross(&(-forward), &right);
        let up = glm::normalize(&up);
        let delta = forward * delta.z + right * delta.x + up * delta.y;
        self.move_(&delta);
    }

    pub fn translate(&mut self, position: &Vec3) {
        self.position = *position;
        self.view = glm::translate(&Mat4::identity(), &-self.position);
    }
}
