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
    ffi::CString,
    fs::File,
    mem::{size_of, size_of_val},
    sync::mpsc::Receiver,
};

use shader_program::ShaderProgram;
use vertex_buffer_object::{BufferType, VertexBufferObject};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

static VERT_SHDR_SRC: &str = include_str!("shaders\\vert_shdr.vert");
static FRAG_SHDR_SRC: &str = include_str!("shaders\\frag_shdr.frag");
// static WALL: &[u8; 256989] = include_bytes!("..\\res\\wall.jpg");
// static SMILE: &[u8; 59277] = include_bytes!("..\\res\\awesomeface.png");

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::ContextVersion(3, 3));

    let (mut window, receiver) = glfw
        .create_window(WIDTH, HEIGHT, "", WindowMode::Windowed)
        .unwrap();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_sticky_keys(true);
    window.make_current();
    glfw.set_swap_interval(SwapInterval::Sync(1));

    gl_loader::init_gl();
    gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);

    stb::image::stbi_set_flip_vertically_on_load(true);

    let mut file = File::open("res\\wall.jpg").unwrap();
    let texture_data = stb::image::stbi_load_from_reader(&mut file, Channels::Default).unwrap();
    // let texture_data = stb::image::stbi_load_from_memory(WALL, Channels::Default).unwrap();
    let texture1 = texture::Texture::new(gl::TEXTURE_2D, texture_data);
    texture1.bind();
    texture1.parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
    texture1.parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
    texture1.parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
    texture1.parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);

    let mut file = File::open("res\\awesomeface.png").unwrap();
    let texture_data = stb::image::stbi_load_from_reader(&mut file, Channels::Default).unwrap();
    // let texture_data = stb::image::stbi_load_from_memory(SMILE, Channels::Default).unwrap();
    let texture2 = texture::Texture::new(gl::TEXTURE_2D, texture_data);
    texture2.bind();
    texture2.parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
    texture2.parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
    texture2.parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
    texture2.parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);

    texture1.bind_to_unit(gl::TEXTURE0);
    texture2.bind_to_unit(gl::TEXTURE1);

    let program = ShaderProgram::from_vert_frag(VERT_SHDR_SRC, FRAG_SHDR_SRC).unwrap();
    unsafe {
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);
        program.use_();
        let name = CString::new("texture1").unwrap();
        gl::Uniform1i(gl::GetUniformLocation(program.get_id(), name.as_ptr()), 0);
        let name = CString::new("texture2").unwrap();
        gl::Uniform1i(gl::GetUniformLocation(program.get_id(), name.as_ptr()), 1);
    }

    let vertices: [f32; 32] = [
        0.75, 0.75, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, // top right
        0.75, -0.75, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, // bottom right
        -0.75, -0.75, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, // bottom left
        -0.75, 0.75, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, // top left
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

    let draw_fn: fn() -> () = || unsafe {
        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
    };
    let attrib_fn: fn() -> () = || unsafe {
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
    let object = Object::new(&data, &program, &attrib_fn, &draw_fn);

    main_loop(&mut glfw, &mut window, &receiver, &object);

    vbo.delete();
    ebo.delete();
    object.delete();
    texture1.delete();
    texture2.delete();
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
    let mut oppacity = 0.0;
    let fn_: fn(f32, &Object) -> () = |op, obj| unsafe {
        let name = CString::new("oppacity").unwrap();
        gl::Uniform1f(
            gl::GetUniformLocation(obj.get_program().get_id(), name.as_ptr()),
            op,
        );
    };
    while !window.should_close() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        object.draw_extra(oppacity, fn_);
        window.swap_buffers();

        let key_up = window.get_key(Key::Up);
        let key_down = window.get_key(Key::Down);
        if let Action::Repeat | Action::Press = key_up {
            oppacity += 0.01;
        }
        if let Action::Repeat | Action::Press = key_down {
            oppacity -= 0.01;
        }
        oppacity = oppacity.clamp(0.0, 1.0);

        glfw.poll_events();
        handle_window_events(receiver, window);
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
