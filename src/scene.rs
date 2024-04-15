use crate::{
    lighting::LightType,
    serializable::{Camera, Entity, LightSource, Mesh, PBRTextures, ScriptObject, Transform, Vec3},
};
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
            meshes: vec![Mesh {
                path: "assets\\meshes\\vange_well.glb".to_string(),
                material: crate::serializable::Material::default(),
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
            scripts: vec![ScriptObject {
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
                type_: LightType::Directional,
                shadow_distance: 100.0,
                inner: 0.0,
                outer: 0.0,
            }],
            scripts: vec![],
        };

        let str_ = serde_json::to_string(&vec![entity1, entity2, entity3]).unwrap();
        fs::write("assets\\scenes\\sample.json", str_).unwrap();
    }
}
