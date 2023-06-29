// #![windows_subsystem = "windows"]

extern crate nalgebra_glm as glm;

use data_structures::{EngineApi, EventContainer, Transform, ViewObject, ViewType};
use gl_wrappers::{ShaderProgram, VertexArrayObject, VertexBufferObject};
use glfw::{Context, WindowEvent};
use std::{f32::consts, mem::size_of_val, sync::mpsc::Receiver};

mod data_structures;
mod gl_wrappers;
mod initializers;
mod updaters;

fn main() {
    let mut glfw = initializers::init_from_config(Default::default());
    let (mut window, receiver) = initializers::create_from_config(Default::default(), &mut glfw);
    let event_container = EventContainer::new_minimal();

    let projection = ViewType::Perspective(
        get_aspect(window.get_framebuffer_size()),
        to_rad(75.0),
        0.1,
        100.0,
    );
    let camera = ViewObject::new(projection, Transform::new());

    let cube_mesh = [
        -0.5_f32, -0.5, -0.5, 0.0, 0.0, 0.5, -0.5, -0.5, 1.0, 0.0, 0.5, 0.5, -0.5, 1.0, 1.0, 0.5,
        0.5, -0.5, 1.0, 1.0, -0.5, 0.5, -0.5, 0.0, 1.0, -0.5, -0.5, -0.5, 0.0, 0.0, -0.5, -0.5,
        0.5, 0.0, 0.0, 0.5, -0.5, 0.5, 1.0, 0.0, 0.5, 0.5, 0.5, 1.0, 1.0, 0.5, 0.5, 0.5, 1.0, 1.0,
        -0.5, 0.5, 0.5, 0.0, 1.0, -0.5, -0.5, 0.5, 0.0, 0.0, -0.5, 0.5, 0.5, 1.0, 0.0, -0.5, 0.5,
        -0.5, 1.0, 1.0, -0.5, -0.5, -0.5, 0.0, 1.0, -0.5, -0.5, -0.5, 0.0, 1.0, -0.5, -0.5, 0.5,
        0.0, 0.0, -0.5, 0.5, 0.5, 1.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.5, 0.5, -0.5, 1.0, 1.0, 0.5,
        -0.5, -0.5, 0.0, 1.0, 0.5, -0.5, -0.5, 0.0, 1.0, 0.5, -0.5, 0.5, 0.0, 0.0, 0.5, 0.5, 0.5,
        1.0, 0.0, -0.5, -0.5, -0.5, 0.0, 1.0, 0.5, -0.5, -0.5, 1.0, 1.0, 0.5, -0.5, 0.5, 1.0, 0.0,
        0.5, -0.5, 0.5, 1.0, 0.0, -0.5, -0.5, 0.5, 0.0, 0.0, -0.5, -0.5, -0.5, 0.0, 1.0, -0.5, 0.5,
        -0.5, 0.0, 1.0, 0.5, 0.5, -0.5, 1.0, 1.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0,
        -0.5, 0.5, 0.5, 0.0, 0.0, -0.5, 0.5, -0.5, 0.0, 1.0,
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
    ).unwrap();
    prog.use_();
    unsafe {
        
    }

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

    initializers::init_rendering();
    let mut frametime = 0.0_f32;
    while !window.should_close() {
        glfw.set_time(0.0);
        glfw.poll_events();

        let mut api = EngineApi::new(&window, frametime);

        // call updates from dynamic dll
        // а еще есть dyn trait

        handle_window_events(&receiver, &event_container, &mut api);
        if api.get_should_close() {
            window.set_should_close(true);
        }

        // rendering in separate place
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
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

const DEG_TO_RAD: f32 = 180.0 / consts::PI;

fn to_rad(deg: f32) -> f32 {
    deg / DEG_TO_RAD
}

// fn to_deg(rad: f32) -> f32 {
//     rad * DEG_TO_RAD
// }
