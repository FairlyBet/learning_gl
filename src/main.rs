// #![windows_subsystem = "windows"]

extern crate nalgebra_glm as glm;

use camera::Camera;
use glfw::{Action, Context, Key, WindowEvent};
use glm::Vec3;
use lighting::{LightObject, LightSource};
use linear::{Projection, Transform};
use rendering::{Canvas, Framebuffer, ModelRenderer, ScreenRenderer};
use russimp::scene::PostProcess;
use std::ffi::CStr;
use temp::Application;

mod camera;
mod data_3d;
mod gl_wrappers;
mod initializers;
mod lighting;
mod linear;
mod rendering;
mod temp;
mod updaters;

fn main() {
    // let p1 = projection.matrix() * glm::Vec4::new(0.0, 0.0, -0.3, 1.0);
    // let p2 = projection.matrix() * glm::Vec4::new(0.0, 0.0, -0.2, 1.0);
    // let p3 = projection.matrix() * glm::Vec4::new(0.0, 0.0, -0.1, 1.0);
    // println!("z {}", p1.z / p1.w);
    // println!("z {}", p2.z / p2.w);
    // println!("z {}", p3.z / p3.w);

    let mut glfw = initializers::init_from_config(Default::default());
    let (mut window, receiver) = initializers::create_from_config(Default::default(), &mut glfw);
    window.set_cursor_mode(glfw::CursorMode::Disabled);
    initializers::init_rendering();

    let projection = Projection::new_perspective(
        camera::aspect(window.get_framebuffer_size()),
        45.0,
        0.1,
        100.0,
    );
    let mut camera = Camera::new(projection);

    let model = data_3d::load_model(
        "assets\\meshes\\backpack.obj",
        vec![
            PostProcess::Triangulate,
            PostProcess::OptimizeGraph,
            PostProcess::OptimizeMeshes,
        ],
    );
    let mut model_transform = Transform::new();

    let light_source = LightSource::new_spot(Vec3::from_element(0.3), 0.5, 0.5, 0.01, 10.0, 15.0);
    let mut light_obj = LightObject::new(light_source);
    // light_obj.transform.move_(&Vec3::new(0.0, 0.0, 2.0));
    // LightSource::new_directional(
    //     glm::Vec3::from_element(0.7),
    //     glm::normalize(&glm::vec3(-1.0, -1.0, -1.0)),
    // );
    // LightSource::new_point(
    //     glm::Vec3::from_element(0.7),
    //     glm::vec3(1.0, 0.0, 1.0),
    //     1.0,
    //     0.07,
    //     0.017,
    // );

    let mut offscreen_framebuffer = Framebuffer::new(
        (
            window.get_framebuffer_size().0,
            window.get_framebuffer_size().1,
        ),
        gl::NEAREST,
        gl::NEAREST,
    );
    let canvas = Canvas::new();

    let model_renderer = ModelRenderer::new();
    let mut screen_renderer = ScreenRenderer::new();
    let mut is_rotating = false;

    while !window.should_close() {
        let frametime = glfw.get_time() as f32;
        glfw.set_time(0.0);

        let cursor_pos_before = window.get_cursor_pos();
        glfw.poll_events();
        let cursor_pos_after = window.get_cursor_pos();
        let cursor_offset = (
            (cursor_pos_after.0 - cursor_pos_before.0) as f32,
            (cursor_pos_after.1 - cursor_pos_before.1) as f32,
        );
        let api = Application::new(&window, frametime, cursor_offset);

        updaters::default_camera_controller(&mut camera, &api);
        light_obj.transform = camera.transform;
        model_transform.rotate(&(glm::vec3(0.0, 30.0, 0.0) * frametime * f32::from(is_rotating)));

        for (_, event) in glfw::flush_messages(&receiver) {
            match event {
                WindowEvent::Key(Key::R, _, Action::Press, _) => {
                    is_rotating = !is_rotating;
                }
                WindowEvent::Key(Key::Up, _, Action::Repeat | Action::Press, _) => {
                    screen_renderer.gamma += 0.1;
                }
                WindowEvent::Key(Key::Down, _, Action::Repeat | Action::Press, _) => {
                    screen_renderer.gamma -= 0.1;
                }
                WindowEvent::FramebufferSize(w, h) => {
                    camera.update_projection((w, h));
                    offscreen_framebuffer = Framebuffer::new((w, h), gl::NEAREST, gl::NEAREST);
                }
                _ => {}
            }
        }

        offscreen_framebuffer.bind();
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        model_renderer.draw(&camera, &model_transform, &model, &mut light_obj);

        Framebuffer::bind_default(window.get_framebuffer_size());
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        screen_renderer.draw_texture(&canvas, &offscreen_framebuffer.sampler_buffer);

        window.swap_buffers();
    }

    gl_loader::end_gl();
}

pub fn get_extensions() -> Vec<String> {
    unsafe {
        let mut amount = 0;
        gl::GetIntegerv(gl::NUM_EXTENSIONS, &mut amount);
        let mut result = Vec::<String>::with_capacity(amount as usize);
        for i in 0..amount {
            let name = CStr::from_ptr(gl::GetStringi(gl::EXTENSIONS, i as u32) as *const _);
            result.push(name.to_string_lossy().to_string());
        }
        result
    }
}
