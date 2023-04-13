#![windows_subsystem = "windows"]

mod camera;
mod shader;
mod shader_program;
mod texture;
mod vertex_array_object;
mod vertex_buffer_object;

extern crate nalgebra_glm as glm;

use gl::types::GLsizei;
use glfw::{
    Action, Context, Key, OpenGlProfileHint, SwapInterval, Window, WindowEvent, WindowHint,
    WindowMode,
};
use glm::{vec3, Mat4};
use stb::image::Channels;
use std::{
    f32::consts,
    fs::File,
    mem::{size_of, size_of_val},
    sync::mpsc::Receiver,
};

use camera::Camera;
use shader_program::ShaderProgram;
use vertex_array_object::VertexArrayObject;
use vertex_buffer_object::VertexBufferObject;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const VERT_SHDR_SRC: &str = include_str!("shaders\\vert_shdr.vert");
const FRAG_SHDR_SRC: &str = include_str!("shaders\\frag_shdr.frag");

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::ContextVersion(3, 3));

    let (mut window, receiver) = glfw
        .create_window(WIDTH, HEIGHT, "", WindowMode::Windowed)
        .unwrap();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.make_current();
    glfw.set_swap_interval(SwapInterval::Sync(1));

    gl_loader::init_gl();
    gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);
    unsafe {
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);
        gl::Enable(gl::DEPTH_TEST);
    }

    stb::image::stbi_set_flip_vertically_on_load(true);

    let mut file = File::open("res\\container.jpg").unwrap();
    let texture_data = stb::image::stbi_load_from_reader(&mut file, Channels::Default).unwrap();
    let texture = texture::Texture::new(gl::TEXTURE_2D, texture_data).unwrap();
    texture.bind();
    texture.parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
    texture.parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
    texture.parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
    texture.parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);

    let vertices = [
        -0.5f32, -0.5, -0.5, 0.0, 0.0, 0.5, -0.5, -0.5, 1.0, 0.0, 0.5, 0.5, -0.5, 1.0, 1.0, 0.5,
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
    let vbo = VertexBufferObject::new(gl::ARRAY_BUFFER).unwrap();
    vao.bind();
    vbo.bind();
    vbo.buffer_data(
        size_of_val(&vertices),
        vertices.as_ptr().cast(),
        gl::STATIC_DRAW,
    );

    let program = ShaderProgram::from_vert_frag(VERT_SHDR_SRC, FRAG_SHDR_SRC).unwrap();
    program.use_();
    ShaderProgram::configure_attribute(
        0,
        3,
        gl::FLOAT,
        gl::FALSE,
        5 * size_of::<f32>() as GLsizei,
        0 as *const _,
    );
    ShaderProgram::configure_attribute(
        1,
        2,
        gl::FLOAT,
        gl::FALSE,
        5 * size_of::<f32>() as GLsizei,
        (3 * size_of::<f32>()) as *const _,
    );
    ShaderProgram::enable_attribute(0);
    ShaderProgram::enable_attribute(1);
    let location = program.get_uniform("MVP");
    let aspect = calculate_aspect(window.get_framebuffer_size());

    let model = Mat4::identity();

    let mut camera = Camera::new();
    camera.move_(&vec3(0.0, 0.0, 2.0));
    let mut projection = glm::perspective(aspect, to_rad(45.0), 0.1, 100.0);
    let mut mvp: Mat4;
    let mut frame_time: f32 = 0.0;
    while !window.should_close() {
        glfw.set_time(0.0);

        glfw.poll_events();
        update_camera(&mut camera, &window, frame_time);
        handle_window_events(&receiver, &mut window, &mut projection);

        mvp = projection * camera.get_view() * model;

        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, glm::value_ptr(&mvp).as_ptr());
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::DrawArrays(gl::TRIANGLES, 0, 36);
        }

        window.swap_buffers();

        frame_time = glfw.get_time() as f32;
    }

    vbo.delete();
    vao.delete();
    texture.delete();
    program.delete();
    gl_loader::end_gl();
}

fn handle_window_events(
    receiver: &Receiver<(f64, WindowEvent)>,
    window: &mut Window,
    projection: &mut Mat4,
) {
    for (_, event) in glfw::flush_messages(receiver) {
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            WindowEvent::FramebufferSize(width, height) => unsafe {
                let aspect = calculate_aspect((width, height));
                *projection = glm::perspective(aspect, to_rad(45.0), 0.1, 100.0);
                gl::Viewport(0, 0, width, height);
            },
            _ => {}
        }
    }
}

fn calculate_aspect(framebuffer_size: (i32, i32)) -> f32 {
    framebuffer_size.0 as f32 / framebuffer_size.1 as f32
}

const DEG_TO_RAD: f32 = 180.0 / consts::PI;

fn to_rad(deg: f32) -> f32 {
    deg / DEG_TO_RAD
}

fn update_camera(camera: &mut Camera, window: &Window, frame_time: f32) {
    let mut delta = vec3(0.0, 0.0, 0.0);
    let velocity = 5.0;
    if let Action::Press | Action::Repeat = window.get_key(Key::W) {
        delta.z += 1.0;
    }
    if let Action::Press | Action::Repeat = window.get_key(Key::A) {
        delta.x -= 1.0;
    }
    if let Action::Press | Action::Repeat = window.get_key(Key::S) {
        delta.z -= 1.0;
    }
    if let Action::Press | Action::Repeat = window.get_key(Key::D) {
        delta.x += 1.0;
    }
    if let Action::Press | Action::Repeat = window.get_key(Key::LeftAlt) {
        delta.y += 1.0;
    }
    if let Action::Press | Action::Repeat = window.get_key(Key::LeftControl) {
        delta.y -= 1.0;
    }
    if delta.magnitude() > 0.0 {
        delta = glm::normalize(&delta); // returning nan
    }
    delta *= velocity * frame_time;
    camera.move_local(&delta);
}
