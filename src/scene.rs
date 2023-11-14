use crate::{camera::Camera, data3d::Model, lighting::LightSource, linear::Transform};
use serde::Deserialize;
use std::fs;

const ENTITIES_FILENAME: &str = "entities.json";
const TRANSFORMS_FILENAME: &str = "transforms.json";

pub struct Scene {}

impl Scene {
    pub fn load_from_file(path: &str) {
        let scene_dir = std::path::Path::new(path);

        let path = scene_dir.with_file_name(ENTITIES_FILENAME);

        let path = scene_dir.with_file_name(TRANSFORMS_FILENAME);
        let transforms_str = fs::read_to_string(path).unwrap();
        // load from file
        // create entities
        // create components
    }

    fn read_vec<'a, T>(path: &str) -> Vec<T>
    where
        T: Deserialize<'a>,
    {
        // let values_str = ;
        let result = serde_json::from_::<Vec<T>>(&fs::read_to_string(path).unwrap()).unwrap();
        result
    }
} 

#[derive(Deserialize)]
struct Entity {
    id: u32,
    name: String,
}

struct Component {}
