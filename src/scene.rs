use crate::{camera::Camera, data3d::Model, lighting::LightSource, linear::Transform};

pub struct Scene {
    
}

impl Scene {
    pub fn load() {}
}

struct Entity {
    id: u32,
    name: String,
    transform: *const Transform,
}

struct Component {

}
