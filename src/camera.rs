use crate::{
    camera,
    linear::{self, Projection, Transform},
};
use nalgebra_glm::Mat4;

pub struct Camera {
    projection: Projection,
    projection_matrix: Mat4,
}

impl Camera {
    pub fn new(projection: Projection) -> Self {
        Self {
            projection,
            projection_matrix: projection.matrix(),
        }
    }

    pub fn projection_view(&self, transform: &Transform) -> Mat4 {
        self.projection_matrix * linear::view_matrix(transform)
    }

    pub fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
        self.projection_matrix = projection.matrix();
    }

    pub fn update_aspect(&mut self, framebuffer_size: (i32, i32)) {
        match self.projection {
            Projection::Orthographic {
                left,
                right,
                bottom,
                top,
                znear,
                zfar,
            } => todo!(),
            Projection::Perspective {
                aspect,
                fovy,
                near,
                far,
            } => {
                self.projection =
                    Projection::new_perspective(camera::aspect(framebuffer_size), fovy, near, far);
                self.projection_matrix = self.projection.matrix();
            }
        }
    }
}

pub fn aspect(framebuffer_size: (i32, i32)) -> f32 {
    framebuffer_size.0 as f32 / framebuffer_size.1 as f32
}
