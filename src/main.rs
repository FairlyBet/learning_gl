#![deny(rust_2018_compatibility)]
// #![windows_subsystem = "windows"]
#![allow(unused)]

extern crate nalgebra_glm as glm;

mod camera;
mod data_3d;
mod entity_system;
mod gl_wrappers;
mod lighting;
mod linear;
mod rendering;
mod resources;
mod runtime;
mod scene;
mod scripting;
mod serializable;
mod shader;
mod utils;

fn main() {
    runtime::run();
}
