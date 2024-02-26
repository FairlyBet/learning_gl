use crate::serializable::{Camera, Entity, MeshData, Script, Transform};
use serde::Serialize;
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

    pub fn sample() {
        let entity1 = Entity {
            name: "object1".to_string(),
            transform: Transform::default(),
            children: vec![],
            meshes: vec![MeshData {
                path: "assets\\meshes\\cube.fbx".to_string(),
            }],
            cameras: vec![],
            light_sources: vec![],
            scripts: vec![],
        };
        let entity2 = Entity {
            name: "object2".to_string(),
            transform: Transform::default(),
            children: vec![],
            meshes: vec![],
            cameras: vec![Camera::default()],
            light_sources: vec![],
            scripts: vec![Script {
                script_path: "assets\\scripts\\camera-controller.lua".to_string(),
            }],
        };

        let str_ = serde_json::to_string(&vec![entity1, entity2]).unwrap();
        fs::write("assets\\scenes\\sample.json", str_).unwrap();
    }
}
