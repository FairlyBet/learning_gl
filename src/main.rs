#![deny(rust_2018_compatibility)]
#![windows_subsystem = "windows"]
#![allow(unused)]

extern crate nalgebra_glm as glm;

mod application;
mod asset_loader;
mod camera;
mod data3d;
mod entity_sys;
mod gl_wrappers;
mod lighting;
mod linear;
mod rendering;
mod runtime;
mod scene;
mod serializable;
mod util;

use application::Application;

fn main() {
    // scene::generate_sample();
    let app = Application::new();
    app.run();
}

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
