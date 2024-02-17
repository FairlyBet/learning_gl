// #![windows_subsystem = "windows"]
#![deny(rust_2018_compatibility)]
#![allow(unused)]

use entity_system::{Entity, SceneManager};
use scripting::Scripting;
use std::{fmt::Debug, fs};

extern crate nalgebra_glm as glm;

mod application;
mod resources;
mod camera;
mod data3d;
mod entity_system;
mod gl_wrappers;
mod lighting;
mod linear;
mod rendering;
mod runtime;
mod scene;
mod scripting;
mod serializable;
mod shader;
mod util;

fn main() {

    // scripting::execute_file("assets\\scripts\\Entity.lua");
    // let entity = Entity::default();
    // let transform = serializable::Transform::default();
    // let mut chunk = SceneChunk::init(vec![entity], vec![transform]);

    // let scripting = Scripting::new();
    // scripting.create_wrappers(&mut chunk);

    // scripting.();
    // let key = s
    //     .create_object(&fs::read_to_string("assets\\scripts\\CameraController.lua").unwrap())
    //     .unwrap();

    // scene::generate_sample();
    let app = application::Application::new();
    app.run();
}
