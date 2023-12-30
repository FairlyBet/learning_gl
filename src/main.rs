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

use fxhash::FxHashMap;
use glfw::Key;
use rlua::{Chunk, Error, Function, Value};
use scripting::Scripting;
use std::{collections::HashMap, fs, io, path::Path};

fn main() {
    let s = Scripting::new();
    loop {
        _ = s.execute_file(Path::new(r#"assets\scripts\sample.lua"#));
        _ = io::stdin().read_line(&mut String::new());
    }

    // scene::generate_sample();
    // let app = application::Application::new();
    // app.run();
}
