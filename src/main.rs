// #![windows_subsystem = "windows"]

extern crate nalgebra_glm as glm;

use std::{sync::mpsc::Receiver, f32::consts};
use glfw::{Context, WindowEvent, Window};
use crate::initializers::{GlfwInit, WindowCreator};

mod initializers;
mod data_structures;
mod gl_wrappers;

fn main() {
    let mut glfw = GlfwInit::init_from_config(_);
    let (mut window, receiver) = WindowCreator::create_from_config(_, &mut glfw);
    // let aspect = calculate_aspect(window.get_framebuffer_size());

    // let cube_transform = glm::translate(&Mat4::identity(), &vec3(0.0, 0.0, 0.0));

    // let lamp_position = vec3(1.0, 0.0, -2.0);
    // let lamp_scale = Vec3::from_element(0.5);
    // let mut lamp_transform = Mat4::identity();
    // lamp_transform = glm::translate(&lamp_transform, &lamp_position);
    // lamp_transform = glm::scale(&lamp_transform, &lamp_scale);
    // let lamp_color = vec3(0.6, 0.45, 0.3);

    // let mut camera = Camera::new();
    // camera.translate(&vec3(0.0, 0.0, 3.0));

    // let mut projection = glm::perspective(aspect, to_rad(45.0), 0.1, 100.0);

    let mut frame_time = 0.0_f32;

    while !window.should_close() {
        glfw.set_time(0.0);
        glfw.poll_events();
        
        // window.set_cursor_pos(0.0, 0.0);
        // update_camera(&mut camera, &window, frame_time);

        handle_window_events(&receiver, &mut window);

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        window.swap_buffers();

        frame_time = glfw.get_time() as f32;
    }

    gl_loader::end_gl();
}

fn handle_window_events(
    receiver: &Receiver<(f64, WindowEvent)>,
    window: &mut Window,
) {
    for (_, event) in glfw::flush_messages(receiver) {
        // match event {
        //     WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        //     WindowEvent::FramebufferSize(width, height) => unsafe {
        //         let aspect = calculate_aspect((width, height));
        //         *projection = glm::perspective(aspect, to_rad(45.0), 0.1, 100.0);
        //         gl::Viewport(0, 0, width, height);
        //     },
        //     _ => {}
        // }
    }
}

fn calculate_aspect(framebuffer_size: (i32, i32)) -> f32 {
    framebuffer_size.0 as f32 / framebuffer_size.1 as f32
}

const DEG_TO_RAD: f32 = 180.0 / consts::PI;

fn to_rad(deg: f32) -> f32 {
    deg / DEG_TO_RAD
}

// fn to_deg(rad: f32) -> f32 {
//     rad * DEG_TO_RAD
// }

// fn update_camera(camera: &mut Camera, window: &Window, frame_time: f32) {
//     let sensitivity = 2.0;
//     let pos = window.get_cursor_pos();
//     let x = pos.0 as f32;
//     let y = pos.1 as f32;
//     let local_rotation = vec3(-y, 0.0, 0.0) * sensitivity * frame_time;
//     let global_rotation = vec3(0.0, -x, 0.0) * sensitivity * frame_time;

//     let mut delta = Vec3::zeros();
//     let velocity = 5.0;
//     if let Action::Press | Action::Repeat = window.get_key(Key::W) {
//         delta.z += 1.0;
//     }
//     if let Action::Press | Action::Repeat = window.get_key(Key::A) {
//         delta.x -= 1.0;
//     }
//     if let Action::Press | Action::Repeat = window.get_key(Key::S) {
//         delta.z -= 1.0;
//     }
//     if let Action::Press | Action::Repeat = window.get_key(Key::D) {
//         delta.x += 1.0;
//     }
//     if delta.magnitude() > 0.0 {
//         delta = glm::normalize(&delta); // returning nan
//     }
//     delta *= velocity * frame_time;

//     camera.rotate(&global_rotation);
//     camera.rotate_local(&local_rotation);
//     camera.move_local(&delta);
// }
