use serde::Deserialize;
use std::fs::File;

// use serde_json::

const GRAPH_FILE: &str = "scene1.json.scene";

pub fn read_graph() {
    let mut file = File::open(GRAPH_FILE).unwrap();
}

#[derive(Deserialize)]
pub struct Object {
    pub transform: Vec3,
}

#[derive(Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Deserialize)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
