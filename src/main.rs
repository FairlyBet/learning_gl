#![deny(rust_2018_compatibility)]
#![allow(unused)]
// #![windows_subsystem = "windows"]

extern crate nalgebra_glm as glm;

mod camera;
mod data_3d;
mod entity_system;
mod gl_wrappers;
mod lighting;
mod linear;
mod meterial;
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
