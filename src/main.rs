// #![windows_subsystem = "windows"]

extern crate nalgebra_glm as glm;

use data_structures::{EngineApi, EventContainer, Transform, ViewObject, Projection};
use gl_wrappers::{ShaderProgram, VertexArrayObject, VertexBufferObject};
use glfw::{Context, WindowEvent};
use glm::Mat4x4;
use std::{f32::consts, mem::size_of_val, sync::mpsc::Receiver};

mod data_structures;
mod gl_wrappers;
mod initializers;
mod updaters;

fn main() {
    let mut glfw = initializers::init_from_config(Default::default());
    let (mut window, receiver) = initializers::create_from_config(Default::default(), &mut glfw);
    let event_container = EventContainer::new_minimal();

    window.set_cursor_mode(glfw::CursorMode::Disabled);

    let projection = Projection::Perspective(
        get_aspect(window.get_framebuffer_size()),
        to_rad(35.0),
        0.1,
        100.0,
    );

    let mut camera = ViewObject::new(projection);

    let cube_mesh = [
        -0.5_f32, -0.5, -0.5, 0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5, -0.5, 0.5, -0.5,
        -0.5, -0.5, -0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5,
        0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5, -0.5,
        -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5,
        -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, -0.5, -0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, 0.5,
        0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5,
        0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5,
    ];

    let vao = VertexArrayObject::new().unwrap();
    vao.bind();

    let vbo = VertexBufferObject::new(gl::ARRAY_BUFFER).unwrap();
    vbo.bind();
    vbo.buffer_data(
        size_of_val(&cube_mesh),
        cube_mesh.as_ptr().cast(),
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
    let self_color = prog.get_uniform("self_color");

    let mut cube = Transform::new();
    cube.position = glm::vec3(0.0, 0.0, -2.0);

    initializers::init_rendering();

    let mut frametime = 0.0_f32;
    while !window.should_close() {
        glfw.set_time(0.0);

        window.set_cursor_pos(0.0, 0.0);
        glfw.poll_events();

        // let cursor_pos2 = window.get_cursor_pos();
        // let cursor_offset = (
        //     cursor_pos2.0 as f32 - cursor_pos1.0 as f32,
        //     cursor_pos2.1 as f32 - cursor_pos1.1 as f32,
        // );

        let mut api = EngineApi::new(&window, frametime);

        // api.
        // call updates from dynamic dll
        // а еще есть dyn trait

        // cube.rotate(&(glm::vec3(0.0, 90.0, 90.0) * frametime));
        // camera.transform.rotate(glm::vec3(0.0, 30.0 * frametime, 0.0));
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
            gl::Uniform3f(self_color, 0.5, 0.4, 0.3);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }

        window.swap_buffers();

        frametime = glfw.get_time() as f32;
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

fn to_rad(deg: f32) -> f32 {
    deg / (180.0 / consts::PI)
}
