use crate::{
    camera,
    linear::{self, Projection, Transform},
};
use nalgebra_glm::Mat4;

pub struct Camera {
    pub transform: *const Transform,
    projection: Projection,
    projection_matrix: Mat4,
}

impl Camera {
    pub fn new(transform: &Transform, projection: Projection) -> Self {
        Self {
            transform,
            projection,
            projection_matrix: projection.matrix(),
        }
    }

    pub fn projection_view(&self) -> Mat4 {
        unsafe { self.projection_matrix * linear::view_matrix(&*self.transform) }
    }

    pub fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
        self.projection_matrix = projection.matrix();
    }

    pub fn update_aspect(&mut self, framebuffer_size: (i32, i32)) {
        if let Projection::Perspective(_, fovy, near, far) = self.projection {
            self.projection =
                Projection::new_perspective(camera::aspect(framebuffer_size), fovy, near, far);
            self.projection_matrix = self.projection.matrix();
        }
    }
}

pub fn aspect(framebuffer_size: (i32, i32)) -> f32 {
    framebuffer_size.0 as f32 / framebuffer_size.1 as f32
}
