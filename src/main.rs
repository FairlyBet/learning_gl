// #![windows_subsystem = "windows"]
#![deny(rust_2018_compatibility)]
#![allow(unused)]

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

use fxhash::FxHashMap;
use glfw::{Key, Modifiers};
use rlua::{Chunk, Error, Function, Value};
use scripting::Scripting;
use std::{collections::HashMap, fs, io, mem::MaybeUninit, path::Path, rc::Rc};
use util::StaticVec;

struct S {
    n: i32,
}

impl Drop for S {
    fn drop(&mut self) {
        println!("{}", self.n);
    }
}

fn main() {
    // let s = Scripting::new();
    // let key = s
    //     .create_object(&fs::read_to_string("assets\\scripts\\CameraController.lua").unwrap())
    //     .unwrap();

    // scene::generate_sample();
    // let app = application::Application::new();
    // app.run();
}
