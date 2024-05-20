#![deny(rust_2018_compatibility)]
#![allow(unused)]
// #![windows_subsystem = "windows"]

extern crate nalgebra_glm as glm;

mod camera;
mod data3d;
mod entity_system;
mod gl_wrappers;
mod lighting;
mod linear;
mod material;
mod rendering;
mod resources;
mod runtime;
mod scene;
mod scripting;
mod serializable;
mod shader;
mod some_idea;
mod utils;

fn main() {
    let heap_obj = some_idea::HeapObject::new(1);
    // utils::StbImage::load("assets\\textures\\4px.png", false).extract_channel(0);
    // scene::Scene::sample();
    // runtime::run();
}
