#![deny(rust_2018_compatibility)]
// #![windows_subsystem = "windows"]
#![allow(unused)]

extern crate nalgebra_glm as glm;

mod application;
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

use runtime::Runtime;
use scene::Scene;
use utils::UntypedVec;

fn main() {
    // Scene::sample();
    let r = Runtime::new();
    r.run();
}
