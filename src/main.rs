// #![windows_subsystem = "windows"]

extern crate nalgebra_glm as glm;

use data_structures::{EngineApi, EventContainer, Projection, Transform, ViewObject};
use gl_wrappers::ShaderProgram;
use glfw::{Context, WindowEvent};
use glm::vec3;
use std::{ffi::CStr, sync::mpsc::Receiver};
// use spin_sleep::LoopHelper;

mod data_structures;
mod gl_wrappers;
mod initializers;
mod updaters;

// const TARGET_FRAME_RATE: i32 = 45;

fn main() {
    let mut glfw = initializers::init_from_config(Default::default());
    let (mut window, receiver) = initializers::create_from_config(Default::default(), &mut glfw);
    let event_container = EventContainer::new_minimal();

    glfw.set_swap_interval(glfw::SwapInterval::Sync(2));
    window.set_cursor_mode(glfw::CursorMode::Disabled);
    window.set_raw_mouse_motion(true);

    let projection =
        Projection::Perspective(get_aspect(window.get_framebuffer_size()), 45.0, 0.1, 100.0);

    let prog = ShaderProgram::from_vert_frag_file(
        "src\\shaders\\trivial_shader.vert",
        "src\\shaders\\trivial_shader.frag",
    )
    .unwrap();
    prog.use_();
    let location = prog.get_uniform("mvp");
    let armor = data_structures::load_single_model("assets\\meshes\\Main.obj");
    let floor = data_structures::GlMesh::from_vertices(
        &data_structures::GlMesh::CUBE_VERTICES.to_vec(),
        gl::STATIC_DRAW,
    );
    initializers::init_rendering();

    let mut camera = ViewObject::new(projection);

    let mut floor_tr = Transform::new();
    floor_tr.scale = vec3(1000.0, 0.5, 1000.0);
    floor_tr.position = vec3(0.0, -0.5, 0.0);

    let mut armour1 = Transform::new();
    armour1.position = vec3(0.0, 0.0, -5.0);

    let mut armour2 = Transform::new();
    armour2.position = vec3(-1.0, 0.0, -5.0);
    armour2.scale = vec3(0.5, 0.5, 0.5);
    
    let mut armour3 = Transform::new();
    armour3.position = vec3(1.5, 0.0, -5.0);
    armour3.scale = vec3(2.0, 2.0, 2.0);

    // let mut loop_helper = LoopHelper::builder().build_with_target_rate(TARGET_FRAME_RATE);
    while !window.should_close() {
        // let frametime = loop_helper.loop_start_s() as f32;
        // while glfw.get_time() < (1.0 / TARGET_FRAME_RATE as f64) {
        //     std::thread::yield_now();
        // }
        let frametime = glfw.get_time() as f32;
        glfw.set_time(0.0);

        let cursor_pos_before = window.get_cursor_pos();
        glfw.poll_events();
        let cursor_pos_after = window.get_cursor_pos();
        let cursor_offset = (
            (cursor_pos_after.0 - cursor_pos_before.0) as f32,
            (cursor_pos_after.1 - cursor_pos_before.1) as f32,
        );
        let mut api = EngineApi::new(&window, frametime, cursor_offset);
        // call updates from dynamic dll
        // а еще есть dyn trait
        updaters::default_camera_controller(&mut camera, &api);
        armour1.rotate(&glm::vec3(0.0, 60.0 * frametime, 0.0));
        armour2.rotate(&glm::vec3(0.0, 60.0 * frametime, 0.0));
        armour3.rotate(&glm::vec3(0.0, 60.0 * frametime, 0.0));
        handle_window_events(&receiver, &event_container, &mut api);

        if api.get_should_close() {
            window.set_should_close(true);
        }
        // rendering in separate place
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::UniformMatrix4fv(
                location,
                1,
                gl::FALSE,
                glm::value_ptr(&(camera.get_projection() * camera.get_view() * armour1.get_model()))
                    .as_ptr()
                    .cast(),
            );
            armor.draw();
            gl::UniformMatrix4fv(
                location,
                1,
                gl::FALSE,
                glm::value_ptr(&(camera.get_projection() * camera.get_view() * armour2.get_model()))
                    .as_ptr()
                    .cast(),
            );
            armor.draw();
            gl::UniformMatrix4fv(
                location,
                1,
                gl::FALSE,
                glm::value_ptr(&(camera.get_projection() * camera.get_view() * armour3.get_model()))
                    .as_ptr()
                    .cast(),
            );
            armor.draw();
            gl::UniformMatrix4fv(
                location,
                1,
                gl::FALSE,
                glm::value_ptr(&(camera.get_projection() * camera.get_view() * floor_tr.get_model()))
                    .as_ptr()
                    .cast(),
            );
            floor.draw();
        }

        window.swap_buffers();
        // loop_helper.loop_sleep();
    }

    gl_loader::end_gl();
}

fn handle_window_events(
    receiver: &Receiver<(f64, WindowEvent)>,
    event_container: &EventContainer,
    api: &mut EngineApi,
) {
    for (_, event) in glfw::flush_messages(receiver) {
        match event {
            WindowEvent::Key(key, _, action, _) => {
                for item in event_container.on_key_pressed.iter() {
                    (item.callback)(key, action, api);
                }
            }
            WindowEvent::FramebufferSize(width, height) => {
                for item in event_container.on_framebuffer_size_changed.iter() {
                    (item.callback)(width, height);
                }
            }
            _ => {}
        }
    }
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
