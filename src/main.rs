// #![windows_subsystem = "windows"]
#![deny(rust_2018_compatibility)]
#![allow(unused)]

use std::fs;

use entity_system::SceneChunk;
use scripting::Scripting;

extern crate nalgebra_glm as glm;

mod application;
mod asset_manager;
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
    let mut chunk = SceneChunk::default();
    chunk.create_entity();

    let scripting = Scripting::new();
    scripting.create_wrappers(&mut chunk);
    // scripting.();
    // let key = s
    //     .create_object(&fs::read_to_string("assets\\scripts\\CameraController.lua").unwrap())
    //     .unwrap();

    // scene::generate_sample();
    // let app = application::Application::new();
    // app.run();
}
