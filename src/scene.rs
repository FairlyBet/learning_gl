use crate::{camera::Camera, data3d::Model, lighting::LightSource, linear::Transform};

pub struct Scene {
    transform_buffer: Vec<Transform>
}

impl Scene {
    pub fn load() {
        // load from file
        // create entities
        // create components
    }
}

struct Entity {
    id: u32,
    name: String,
    transform: *const Transform,
}

struct Component {

}
