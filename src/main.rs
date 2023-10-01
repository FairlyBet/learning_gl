// #![windows_subsystem = "windows"]

extern crate nalgebra_glm as glm;

use data_structures::{
    Canvas, EngineApi, LightSource, Projection, ScreenBuffer, Transform, ViewObject,
};
use glfw::{Context, WindowEvent};
use renderers::{ModelRenderer, ScreenRenderer};
use russimp::scene::PostProcess;
use std::ffi::CStr;

mod data_structures;
mod gl_wrappers;
mod initializers;
mod renderers;
mod updaters;

fn main() {
    let mut glfw = initializers::init_from_config(Default::default());
    let (mut window, receiver) = initializers::create_from_config(Default::default(), &mut glfw);

    window.set_cursor_mode(glfw::CursorMode::Disabled);
    // window.set_raw_mouse_motion(true);

    let projection =
        Projection::Perspective(get_aspect(window.get_framebuffer_size()), 45.0, 0.1, 100.0);
    let mut camera = ViewObject::new(projection);

    initializers::init_rendering();
    let model_renderer = ModelRenderer::new();
    let model = data_structures::load_model(
        "assets\\meshes\\backpack.obj",
        vec![
            PostProcess::Triangulate,
            PostProcess::OptimizeGraph,
            PostProcess::OptimizeMeshes,
        ],
    );
    let model_transform = Transform::new();
    // model_transform.set_rotation(&glm::vec3(-90.0, -90.0, 0.0));
    let light_source = LightSource::new_spot(
        glm::vec3(0.9, 0.9, 0.9),
        glm::vec3(0.0, 0.0, 1.0),
        glm::vec3(0.0, 0.0, -1.0),
        1.0,
        0.7,
        0.017,
        15.0,
        45.0,
    );
    // LightSource::new_point(
    //     glm::vec3(0.9, 0.9, 0.9),
    //     glm::vec3(1.0, 2.0, 1.0),
    //     1.0,
    //     0.07,
    //     0.017,
    // );
    let mut screen_buffer = ScreenBuffer::new(
        (
            window.get_framebuffer_size().0 / 2,
            window.get_framebuffer_size().1 / 2,
        ),
        gl::NEAREST,
        gl::NEAREST,
    );
    screen_buffer.color_buffer.bind();

    let canvas = Canvas::new();
    let screen_renderer = ScreenRenderer::new();
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
        let api = EngineApi::new(&window, frametime, cursor_offset);

        updaters::default_camera_controller(&mut camera, &api);
        // model_transform.rotate(&(vec3(0.0, 60.0, 0.0) * frametime));

        for (_, event) in glfw::flush_messages(&receiver) {
            match event {
                WindowEvent::FramebufferSize(w, h) => {
                    camera.projection = updaters::update_perspective(w, h);
                    screen_buffer = ScreenBuffer::new((w / 2, h / 2), gl::NEAREST, gl::NEAREST);
                }
                _ => {}
            }
        }

        screen_buffer.bind();
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        model_renderer.draw(&camera, &model_transform, &model, &light_source);

        ScreenBuffer::bind_default(window.get_framebuffer_size());
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        screen_renderer.draw_texture(&canvas, &screen_buffer.color_buffer);

        window.swap_buffers();
    }

    gl_loader::end_gl();
}

pub fn get_aspect(framebuffer_size: (i32, i32)) -> f32 {
    framebuffer_size.0 as f32 / framebuffer_size.1 as f32
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
