use crate::{camera::Camera, data3d::Model, lighting::LightSource};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

const ENTITIES_FILENAME: &str = "entities.json";
const TRANSFORMS_FILENAME: &str = "transforms.json";

pub struct Scene {
    path: String,
}

impl Scene {
    pub fn load(&self) {
        let scene_dir = Path::new(&self.path);
        let entities = Self::read_vec::<Vec<Entity>>(&scene_dir.with_file_name(ENTITIES_FILENAME));
        let transforms =
            Self::read_vec::<Vec<Transform>>(&scene_dir.with_file_name(TRANSFORMS_FILENAME));
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

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub id: u32,
    pub name: String,
}

#[derive(Deserialize)]
struct Transform {
    pub position: Vec3,
    pub orientation: Quat,
    pub scale: Vec3,
    pub parent_id: u32,
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
