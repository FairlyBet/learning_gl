use crate::serializable::{Camera, Entity, LightSource, MeshData, Script, Transform, Vec3};
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
            name: "Object".to_string(),
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
            name: "Camera".to_string(),
            transform: Transform::default(),
            children: vec![],
            meshes: vec![],
            cameras: vec![Camera::default()],
            light_sources: vec![],
            scripts: vec![Script {
                script_path: "assets\\scripts\\camera-controller.lua".to_string(),
            }],
        };
        let entity3 = Entity {
            name: "Light".to_string(),
            transform: Transform::default(),
            children: vec![],
            meshes: vec![],
            cameras: vec![],
            light_sources: vec![LightSource {
                color: Vec3 {
                    x: 1.0,
                    y: 1.0,
                    z: 1.0,
                },
                type_: crate::lighting::LightType::Directional,
                position: Vec3::default(),
                constant: 0.0,
                direction: Vec3::default(),
                linear: 0.0,
                quadratic: 0.0,
                inner_cutoff: 0.0,
                outer_cutoff: 0.0,
                shadow_distance: 100.0,
            }],
            scripts: vec![Script {
                script_path: "assets\\scripts\\sample.lua".to_string(),
            }],
        };

        let str_ = serde_json::to_string(&vec![entity1, entity2, entity3]).unwrap();
        fs::write("assets\\scenes\\sample.json", str_).unwrap();
    }
}
