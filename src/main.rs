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
mod static_runtime;
mod utils;

// use runtime::Runtime;
// use scene::Scene;
// use utils::TypelessVec;

fn main() {
    // let lua = mlua::Lua::new();
    // lua.load(
    //     "
    //     local object = {}
    //     print(tostring(object))
    //     local table = {}
    //     object[table] = 12

    //     for key, value in pairs(object) do
    //         print(\"Key: \" .. tostring(key) .. \"\t\tValue: \" .. tostring(value))
    //     end

    // ",
    // )
    // .exec()
    // .unwrap();

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

    // let r = Runtime::new();
    // r.run();
    static_runtime::start();
}
