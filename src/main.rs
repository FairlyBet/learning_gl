#![allow(unused)]
// #![windows_subsystem = "windows"]

use std::io;

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
mod idea2;

fn main() {
    let mm = some_idea::MemoryManager::new().unwrap();
    io::stdin().read_line(&mut String::new());
    // scene::Scene::sample();
    // runtime::run();
    // v.iter().for_each(|item| item());
    // println!("{}", size_of::<fn()>());

    // println!("{}", std::any::type_name::<i32>());
    // println!("{:?}", std::any::TypeId::of::<camera::Camera>());
    // println!("{:?}", std::any::TypeId::of::<scene::Scene>());
    // println!("{:?}", std::any::TypeId::of::<S>());
}
