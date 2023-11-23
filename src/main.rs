#![deny(rust_2018_compatibility)]
#![allow(unused)]
// #![windows_subsystem = "windows"]

extern crate nalgebra_glm as glm;

mod application;
mod camera;
mod data3d;
mod gl_wrappers;
mod lighting;
mod linear;
mod rendering;
mod scene;
mod serializable;
mod entity;
mod util;
mod runtime;

use application::Application;

fn main() {
    let app = Application::new();
    app.run();
    // let lua = Lua::new();
    // let src = fs::read_to_string("src\\scripts\\load-scene.lua").unwrap();
    // let result = lua.context(|context| {
    //     let chunk = context.load(&src);
    //     chunk.exec()
    // });
    // match result {
    //     Ok(_) => {
    //         println!("Lua script executed successfully.");
    //     }
    //     Err(err) => {
    //         eprintln!("Error: {:?}", err);
    //     }
    // }
}
