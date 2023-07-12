// #![windows_subsystem = "windows"]

extern crate nalgebra_glm as glm;

use data_structures::{
    DirectionalLight, EngineApi, Projection, ShaderProgram, Transform, ViewObject,
};
use glfw::{Context, WindowEvent};
use glm::{vec3, Vec3};
use std::ffi::CStr;

mod data_structures;
mod gl_wrappers;
mod initializers;
mod updaters;

fn main() {
    let mut glfw = initializers::init_from_config(Default::default());
    let (mut window, receiver) = initializers::create_from_config(Default::default(), &mut glfw);

    window.set_cursor_mode(glfw::CursorMode::Disabled);
    window.set_raw_mouse_motion(true);

    let projection =
        Projection::Perspective(get_aspect(window.get_framebuffer_size()), 45.0, 0.1, 100.0);
    let mut camera = ViewObject::new(projection);

    initializers::init_rendering();
    let program = ShaderProgram::new();
    program.use_();

    let model3d = data_structures::load_as_single_model("assets\\meshes\\backpack.obj");
    let model_transform = Transform::new();
    let light = DirectionalLight {
        direction: glm::normalize(&vec3(1.0, -1.0, -1.0)),
        color: Vec3::from_element(1.0),
    };

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
                    updaters::update_viewport(w, h);
                    camera.projection_matrix = updaters::update_perspective(w, h);
                }
                _ => {}
            }
        }

        if api.get_should_close() {
            window.set_should_close(true);
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }
        program.draw(&model_transform, &model3d, &camera, &light);
        window.swap_buffers();
    }

    gl_loader::end_gl();
}

fn get_aspect(framebuffer_size: (i32, i32)) -> f32 {
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
