// #![windows_subsystem = "windows"]
#![deny(rust_2018_compatibility)]
#![allow(unused)]

extern crate nalgebra_glm as glm;

mod application;
mod asset_loader;
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

use scripting::Scripting;
use std::path::Path;

fn main() {
    // scene::generate_sample();
    let app = application::Application::new();
    app.run();
}
