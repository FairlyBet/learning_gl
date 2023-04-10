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
    Action, Context, Glfw, Key, OpenGlProfileHint, SwapInterval, Window, WindowEvent, WindowHint,
    WindowMode,
};
use glm::{vec3, Vec3};
use nalgebra_glm::Mat4;
use stb::image::Channels;
use std::{
    f32::consts,
    fs::File,
    mem::{size_of, size_of_val},
    sync::mpsc::Receiver,
};
use vertex_array_object::VertexArrayObject;

use shader_program::ShaderProgram;
use vertex_buffer_object::{BufferType, VertexBufferObject};

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

static VERT_SHDR_SRC: &str = include_str!("shaders\\vert_shdr.vert");
static FRAG_SHDR_SRC: &str = include_str!("shaders\\frag_shdr.frag");

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
    }

    stb::image::stbi_set_flip_vertically_on_load(true);

    let mut file = File::open("res\\awesomeface.png").unwrap();
    let texture_data = stb::image::stbi_load_from_reader(&mut file, Channels::Default).unwrap();
    let texture = texture::Texture::new(gl::TEXTURE_2D, texture_data);
    texture.bind();
    texture.parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
    texture.parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
    texture.parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
    texture.parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);

    let vertices = [
        1.0f32, 1.0, 0.0, 1.0, 1.0, //
        1.0, -1.0, 0.0, 1.0, 0.0, //
        -1.0, -1.0, 0.0, 0.0, 0.0, //
        -1.0, 1.0, 0.0, 0.0, 1.0, //
    ];
    let elements = [0u32, 1, 3, 1, 2, 3];

    let vao = VertexArrayObject::new().unwrap();
    vao.bind();
    let vbo = VertexBufferObject::new(BufferType::ArrayBuffer).unwrap();
    vbo.bind();
    vbo.buffer_data(
        vertices.as_ptr().cast(),
        size_of_val(&vertices),
        gl::STATIC_DRAW,
    );
    let ebo = VertexBufferObject::new(BufferType::ElementArrayBuffer).unwrap();
    ebo.bind();
    ebo.buffer_data(
        elements.as_ptr().cast(),
        size_of_val(&elements),
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

    main_loop(&mut glfw, &mut window, &receiver, &program);

    vbo.delete();
    ebo.delete();
    vao.delete();
    texture.delete();
    program.delete();
    gl_loader::end_gl();
}

fn main_loop(
    glfw: &mut Glfw,
    window: &mut Window,
    receiver: &Receiver<(f64, WindowEvent)>,
    program: &ShaderProgram,
) {
    let location = program.get_uniform("MVP");
    let aspect = calculate_aspect(window.get_framebuffer_size());
    let mut model: Mat4 = glm::identity();
    let view: Mat4 = glm::identity();
    let mut projection = glm::perspective(aspect, glm::pi::<f32>() / 4.0, 0.1, 100.0);
    let mut mvp: Mat4;

    model = glm::translate(&model, &vec3(0.0, 0.0, -5.0));
    model = glm::rotate(&model, -to_rad(80.0), &Vec3::x_axis());

    while !window.should_close() {
        mvp = projection * view * model;

        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, glm::value_ptr(&mvp).as_ptr());
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
        }

        window.swap_buffers();
        glfw.poll_events();
        handle_window_events(receiver, window, &mut projection);
    }
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
