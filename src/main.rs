#![windows_subsystem = "windows"]

mod object;
mod shader;
mod shader_program;
mod texture;
mod vertex_array_object;
mod vertex_buffer_object;

use glfw::{
    Action, Context, Glfw, Key, OpenGlProfileHint, SwapInterval, Window, WindowEvent, WindowHint,
    WindowMode,
};
use object::Object;
use stb::image::Channels;
use std::{
    fs::File,
    mem::{size_of, size_of_val},
    sync::mpsc::Receiver,
};

use shader_program::ShaderProgram;
use vertex_array_object::VertexArrayObject;
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

    let mut file = File::open("res\\container.jpg").unwrap();
    let texture_data = stb::image::stbi_load_from_reader(&mut file, Channels::Default).unwrap();
    let texture = texture::Texture::new(gl::TEXTURE_2D, texture_data);
    texture.parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
    texture.parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
    texture.parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
    texture.parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);
    texture.bind();

    let program = ShaderProgram::from_vert_frag(VERT_SHDR_SRC, FRAG_SHDR_SRC).unwrap();

    let vertices: [f32; 32] = [
        0.5, 0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, // top right
        0.5, -0.5, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, // bottom right
        -0.5, -0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, // bottom left
        -0.5, 0.5, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, // top left
    ];
    let elements: [u32; 6] = [0, 1, 3, 1, 2, 3];

    let vbo = VertexBufferObject::new(BufferType::ArrayBuffer).unwrap();
    vbo.buffer_data(
        vertices.as_ptr().cast(),
        size_of_val(&vertices),
        gl::STATIC_DRAW,
    );
    let ebo = VertexBufferObject::new(BufferType::ElementArrayBuffer).unwrap();
    ebo.buffer_data(
        elements.as_ptr().cast(),
        size_of_val(&elements),
        gl::STATIC_DRAW,
    );
    let data = vec![&vbo, &ebo];
    let draw = || unsafe {
        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
    };
    let attrib: fn() -> () = || unsafe {
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (8 * size_of::<f32>()) as i32,
            0 as *const _,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            (8 * size_of::<f32>()) as i32,
            (3 * size_of::<f32>()) as i32 as *const _,
        );
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            (8 * size_of::<f32>()) as i32,
            (6 * size_of::<f32>()) as i32 as *const _,
        );
        gl::EnableVertexAttribArray(2);
    };
    let object = Object::new(&data, &program, &attrib, draw);

    main_loop(&mut glfw, &mut window, &receiver, &object);

    vbo.delete();
    ebo.delete();
    object.delete();
    texture.delete();
    program.delete();
    gl_loader::end_gl();
}

fn main_loop(
    glfw: &mut Glfw,
    window: &mut Window,
    receiver: &Receiver<(f64, WindowEvent)>,
    object: &Object,
) {
    object.bind();
    while !window.should_close() {
        handle_window_events(receiver, window);

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        object.draw();

        window.swap_buffers();
        glfw.poll_events();
    }
}

fn handle_window_events(receiver: &Receiver<(f64, WindowEvent)>, window: &mut Window) {
    for (_, event) in glfw::flush_messages(receiver) {
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            WindowEvent::FramebufferSize(width, height) => unsafe {
                gl::Viewport(0, 0, width, height);
            },
            _ => {}
        }
    }
}
