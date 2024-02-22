use crate::serializable::Entity;
use std::fs;

pub struct Scene {
    pub path: String,
}

impl Scene {
    pub fn new(path: &String) -> Self {
        Self { path: path.clone() }
    }

    pub fn load_entities(&self) -> Vec<Entity> {
        let json_str = fs::read_to_string(&self.path).unwrap();
        let entities = serde_json::from_str::<Vec<Entity>>(&json_str).unwrap();
        entities
    }
}
