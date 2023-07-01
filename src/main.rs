// #![windows_subsystem = "windows"]

extern crate nalgebra_glm as glm;

use data_structures::{EngineApi, EventContainer, Projection, Transform, ViewObject};
use gl_wrappers::{ShaderProgram, VertexArrayObject, VertexBufferObject};
use glfw::{Context, WindowEvent};
use spin_sleep::LoopHelper;
use std::{mem::size_of_val, sync::mpsc::Receiver};

mod data_structures;
mod gl_wrappers;
mod initializers;
mod updaters;

const TARGET_FRAME_RATE: i32 = 60;
const CUBE_MESH: [f32; 108] = [
    -0.5, -0.5, -0.5, 0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5,
    -0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5,
    -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5,
    0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, 0.5,
    -0.5, 0.5, 0.5, 0.5, 0.5, -0.5, -0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5,
    -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
    0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5,
];

fn main() {
    let mut glfw = initializers::init_from_config(Default::default());
    let (mut window, receiver) = initializers::create_from_config(Default::default(), &mut glfw);
    let event_container = EventContainer::new_minimal();

    glfw.set_swap_interval(glfw::SwapInterval::None);
    window.set_cursor_mode(glfw::CursorMode::Disabled);
    window.set_raw_mouse_motion(true);
    let projection =
        Projection::Perspective(get_aspect(window.get_framebuffer_size()), 45.0, 0.1, 100.0);

    let vao = VertexArrayObject::new().unwrap();
    vao.bind();

    let vbo = VertexBufferObject::new(gl::ARRAY_BUFFER).unwrap();
    vbo.bind();
    vbo.buffer_data(
        size_of_val(&CUBE_MESH),
        CUBE_MESH.as_ptr().cast(),
        gl::STATIC_DRAW,
    );

    let prog = ShaderProgram::from_vert_frag_file(
        "src\\shaders\\trivial_shader.vert",
        "src\\shaders\\trivial_shader.frag",
    )
    .unwrap();
    prog.use_();
    ShaderProgram::configure_attribute(0, 3, gl::FLOAT, gl::FALSE, 0, 0 as *const _);
    ShaderProgram::enable_attribute(0);
    let location = prog.get_uniform("mvp");

    initializers::init_rendering();

    let mut camera = ViewObject::new(projection);

    let mut cube = Transform::new();
    cube.position = glm::vec3(0.0, 0.0, -5.0);

    let mut loop_helper = LoopHelper::builder().build_with_target_rate(TARGET_FRAME_RATE);
    while !window.should_close() {
        let frametime = loop_helper.loop_start_s() as f32;

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
                glm::value_ptr(&(camera.get_projection() * camera.get_view() * cube.get_model()))
                    .as_ptr()
                    .cast(),
            );
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }

        window.swap_buffers();
        loop_helper.loop_sleep();
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
