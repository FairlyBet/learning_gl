#![windows_subsystem = "windows"]

mod shader;
mod shader_program;
mod shaders_src;
mod vertex_array_object;
mod vertex_buffer_object;

use glfw::{
    Action, Context, Glfw, Key, OpenGlProfileHint, SwapInterval, Window, WindowEvent, WindowHint,
    WindowMode,
};
use std::{
    mem::{size_of, size_of_val},
    sync::mpsc::Receiver,
};

use shader_program::ShaderProgram;
use vertex_array_object::VertexArrayObject;
use vertex_buffer_object::{BufferType, VertexBufferObject};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

static VERT_W_COLOR: &str = include_str!("shaders\\vert_with_color.vert");
static FRAG_MONO_COLOR: &str = include_str!("shaders\\frag_mono_color.frag");

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
    let program = ShaderProgram::from_vert_frag(VERT_W_COLOR, FRAG_MONO_COLOR).unwrap();
    let h = 3.0_f32.sqrt() * 0.5;
    let vertices: [f32; 18] = [
        // positions                // colors
        0.0 - h / 2.0,
        0.5,
        0.0,
        1.0,
        1.0,
        1.0, // bottom right
        0.0 - h / 2.0,
        -0.5,
        0.0,
        0.0,
        0.0,
        0.0, // bottom let
        h / 2.0,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0, // top
    ];

    let vbo = VertexBufferObject::new(BufferType::ArrayBuffer).unwrap();
    vbo.bind();
    vbo.buffer_data(
        vertices.as_ptr().cast(),
        size_of_val(&vertices),
        gl::STATIC_DRAW,
    );

    let data = vec![&vbo];
    let draw = || unsafe {
        gl::DrawArrays(gl::TRIANGLES, 0, 3);
    };
    let attrib: fn() -> () = || unsafe {
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (6 * size_of::<f32>()) as i32,
            0 as *const _,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            (6 * size_of::<f32>()) as i32,
            (3 * size_of::<f32>()) as i32 as *const _,
        );
        gl::EnableVertexAttribArray(1);
    };
    let object = Object::new(&data, &program, &attrib, draw);

    main_loop(&mut glfw, &mut window, &receiver, &object);

    vbo.delete();
    object.delete();
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

struct Object<'a> {
    vao: VertexArrayObject,
    program: &'a ShaderProgram,
    draw_fn: fn() -> (),
}

impl<'a> Object<'a> {
    pub fn new(
        data: &Vec<&VertexBufferObject>,
        program: &'a ShaderProgram,
        attribute_configurer: &fn() -> (),
        draw_fn: fn() -> (),
    ) -> Self {
        let vao = VertexArrayObject::new().unwrap();
        vao.bind();
        for buffer in data {
            buffer.bind();
        }
        attribute_configurer();
        VertexArrayObject::clear_binding();
        for buffer in data {
            buffer.clear_binding();
        }
        Object {
            vao,
            program,
            draw_fn,
        }
    }

    pub fn bind(&self) {
        self.vao.bind();
        self.program.use_();
    }

    pub fn draw(&self) {
        (self.draw_fn)();
    }

    pub fn delete(self) {
        self.vao.delete();
    }
}
