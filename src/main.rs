#![deny(rust_2018_compatibility)]
// #![windows_subsystem = "windows"]
#![allow(unused)]

use scripting::CompiledScript;

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
    // let mut v = UntypedVec::default();
    // v.push(10);
    // v.push(20);
    // v.push(30);
    // println!("{}", v.len::<i32>());
    // println!("{}", v.capacity::<i32>());
    // v.take_at::<i32>(0);
    // v.take_at::<i32>(0);
    // v.take_at::<i32>(0);
    // println!("{}", v.len::<i32>());
    // println!("{}", v.capacity::<i32>());
    // v.slice::<i32>().iter().for_each(|item| println!("Item {}", *item));
    // v.push(12);
    // println!("{}", v.len::<i32>());
    // v.slice::<i32>().iter().for_each(|item| println!("Item {}", *item));
    // Scene::sample();

    // let src = std::fs::read_to_string("assets\\scripts\\api\\gameobject.lua").unwrap();
    // let lua = mlua::Lua::new();
    // lua.load(&src).exec().unwrap();
    // lua.gc_collect().unwrap();
    // lua.load("
    // print \"After GC\"
    // print(WeakRefCopy.value)").exec().unwrap();

    // for item in resources::get_paths::<CompiledScript>() {
    //     println!("{}", item);
    // }
    runtime::run();
}
