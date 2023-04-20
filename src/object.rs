use nalgebra_glm::Mat4;

use crate::renderer::Renderer;

pub struct Object<'a> {
    renderer: &'a Renderer<'a>,
    transform: Mat4,
}

impl<'a> Object<'_> {
    pub fn new(renderer: &'a Renderer) -> Object<'a> {
        Object {
            renderer,
            transform: Mat4::identity(),
        }
    }

    pub fn get_transform(&self) -> &Mat4 {
        &self.transform
    }

    pub fn make_transformation(&mut self) -> &mut Mat4 {
        &mut self.transform
    }

    pub fn get_renderer(&self) -> &Renderer {
        self.renderer
    }
}
