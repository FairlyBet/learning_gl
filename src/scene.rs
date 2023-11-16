use crate::{camera::Camera, data3d::Model, lighting::LightSource, linear};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

const ENTITIES_FILENAME: &str = "entities.json";
const TRANSFORMS_FILENAME: &str = "transforms.json";

pub struct Scene {
    path: String,
    transforms: Vec<linear::Transform>,
}

impl Scene {
    pub fn load(&mut self) {
        let scene_dir = Path::new(&self.path);

        let entities = Self::read_vec::<Entity>(&scene_dir.with_file_name(ENTITIES_FILENAME));
        let transforms =
            Self::read_vec::<Transform>(&scene_dir.with_file_name(TRANSFORMS_FILENAME));

        self.transforms = Vec::with_capacity(transforms.len());
        for (i, transform) in transforms.iter().enumerate() {
            self.transforms[i] = transforms[i].into_actual();
        }

        for (i, entity) in entities.iter().enumerate() {
            if let Some(parent_id) = entity.parent {
                self.transforms[i].parent = Some(&self.transforms[parent_id as usize]);
            }
        }
    }

    fn read_vec<T>(path: &PathBuf) -> Vec<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        let json_str = fs::read_to_string(path).unwrap();
        let values = serde_json::from_str::<Vec<T>>(&json_str).unwrap();
        values
    }
}

struct EntityContainer {
    
}

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub id: u32,
    pub name: String,
    pub parent: Option<u32>,
    pub children: Option<Vec<u32>>,
}

#[derive(Deserialize)]
struct Transform {
    pub position: Vec3,
    pub orientation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn into_actual(&self) -> linear::Transform {
        let mut result = linear::Transform::new();

        result.position.x = self.position.x;
        result.position.y = self.position.y;
        result.position.z = self.position.z;

        result.orientation.coords.x = self.orientation.x;
        result.orientation.coords.y = self.orientation.y;
        result.orientation.coords.z = self.orientation.z;
        result.orientation.coords.w = self.orientation.w;

        result.scale.x = self.scale.x;
        result.scale.y = self.scale.y;
        result.scale.z = self.scale.z;

        result
    }
}

#[derive(Deserialize)]
struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Deserialize)]
struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

struct Component {}
