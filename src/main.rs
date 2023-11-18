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

    // let camera_tr = Transform::new();
    // let projection = Projection::new_perspective(
    //     camera::aspect(window.get_framebuffer_size()),
    //     45.0,
    //     0.1,
    //     100.0,
    // );

    // while !window.should_close() {
    //     let frametime = glfw.get_time() as f32;
    //     glfw.set_time(0.0);
    //     let cursor_pos_before = window.get_cursor_pos();
    //     glfw.poll_events();
    //     let cursor_pos_after = window.get_cursor_pos();
    //     let cursor_offset = (
    //         (cursor_pos_after.0 - cursor_pos_before.0) as f32,
    //         (cursor_pos_after.1 - cursor_pos_before.1) as f32,
    //     );
    //     let api = Application::new(&window, frametime, cursor_offset);
    //     updaters::default_camera_controller(&mut camera, &api);
    //     light_obj.transform = camera.transform;
    //     model_transform.rotate(&(glm::vec3(0.0, 30.0, 0.0) * frametime * f32::from(is_rotating)));

    //     offscreen_framebuffer.bind();
    //     unsafe {
    //         gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    //     }
    //     model_renderer.draw(&camera, &model_transform, &model, &mut light_obj);
    //     Framebuffer::bind_default(window.get_framebuffer_size());
    //     unsafe {
    //         gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    //     }
    //     screen_renderer.draw_texture(&canvas, &offscreen_framebuffer.sampler_buffer);
    //     window.swap_buffers();
    // }
}
