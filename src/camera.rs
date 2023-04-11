use nalgebra_glm::{Mat4, Vec3};

pub struct Camera {
    position: Vec3,
    view: Mat4,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Vec3::zeros(),
            view: Mat4::identity(),
        }
    }

    pub fn get_view(&self) -> Mat4 {
        self.view
    }

    pub fn look_at(&mut self, target: &Vec3) {
        self.view = glm::look_at(&self.position, &target, &Vec3::y_axis());
    }

    pub fn move_(&mut self, delta: &Vec3) {
        self.position += delta;
        self.view = glm::translate(&Mat4::identity(), &-self.position);
    }

    pub fn translate(&mut self, position: &Vec3) {
        self.position = *position;
        self.view = glm::translate(&Mat4::identity(), &-self.position);
    }
}
