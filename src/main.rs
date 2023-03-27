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
    ffi::{c_void, CString},
    mem::size_of_val,
    sync::mpsc::Receiver,
};

use shader::{Shader, ShaderType};
use shader_program::ShaderProgram;
use shaders_src::*;
use vertex_array_object::VertexArrayObject;
use vertex_buffer_object::{BufferType, VertexBufferObject};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

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
        gl::ClearColor(0.1, 0.1, 0.1, 1.0);
    }

    let vertices: [f32; 12] = [
        1.0, 0.5, 0.0, 1.0, -0.5, 0.0, -1.0, -0.5, 0.0, -1.0, 0.5, 0.0,
    ];
    let indeces: [u32; 6] = [
        0, 1, 3, // first triangle
        1, 2, 3, // second triangle
    ];
    let program = ShaderProgram::new().unwrap();
    let vertex_shader = Shader::from_source(ShaderType::VertexShader, VERTEX_SHADER_SRC).unwrap();
    let fragment_shader =
        Shader::from_source(ShaderType::FragmentShader, MONO_COLOR_FRAG_SHDR_SRC).unwrap();

    vertex_shader.compile();
    fragment_shader.compile();

    program.attach_shader(&vertex_shader);
    program.attach_shader(&fragment_shader);
    program.link();
    vertex_shader.delete();
    fragment_shader.delete();

    let vbo = VertexBufferObject::new(BufferType::ArrayBuffer).unwrap();
    let ebo = VertexBufferObject::new(BufferType::ElementArrayBuffer).unwrap();
    vbo.bind();
    vbo.buffer_data(
        vertices.as_ptr().cast(),
        size_of_val(&vertices),
        gl::STATIC_DRAW,
    );
    ebo.bind();
    ebo.buffer_data(
        indeces.as_ptr().cast(),
        size_of_val(&vertices),
        gl::STATIC_DRAW,
    );
    let data = vec![&vbo, &ebo];
    let draw = || unsafe {
        gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
    };
    let attrib: fn() -> () = || unsafe {
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0 as i32, 0 as *const _);
        gl::EnableVertexAttribArray(0);
    };
    let object = Object::new(&data, &program, &attrib, draw);

    main_loop(&mut glfw, &mut window, &receiver, &object, &program);

    gl_loader::end_gl();
}

fn main_loop(
    glfw: &mut Glfw,
    window: &mut Window,
    receiver: &Receiver<(f64, WindowEvent)>,
    object: &Object, program : &ShaderProgram
) {
    while !window.should_close() {
        handle_window_events(receiver, window);
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        let time = glfw.get_time();
        let green_component = ((time / 2.0).sin() / 2.0 + 0.5) as f32;
        let input_proc= |green: Option<&f32>| {
            let location: i32;
            let name = CString::new("inputColor").unwrap();
            unsafe {
                location = gl::GetUniformLocation(program.get_id(), name.as_ptr());
            }
        };
        object.draw::<f32>(Some(&green_component), Some(&input_proc));
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

    pub fn draw<T>(
        &self,
        addtional_input: Option<&T>,
        input_processing: Option<&fn(Option<&T>) -> ()>,
    ) {
        match input_processing {
            Some(processing) => processing(addtional_input),
            _ => {}
        }
        (self.draw_fn)();
    }
}

// struct Rectangle {
//     vao: VertexArrayObject,
//     vbo: VertexBufferObject,
//     ebo: VertexBufferObject,
//     color_location: i32,
//     program: ShaderProgram,
// }

// impl Rectangle {
//     fn setup(offset: f32) -> Rectangle {
//         let vertex_shader =
//             Shader::from_source(ShaderType::VertexShader, VERTEX_SHADER_SRC).unwrap();
//         let fragment_shader =
//             Shader::from_source(ShaderType::FragmentShader, MONO_COLOR_FRAG_SHDR_SRC).unwrap();
//         let program = ShaderProgram::new().unwrap();
//         let location: i32;

//         vertex_shader.compile();
//         fragment_shader.compile();
//         program.attach_shader(&vertex_shader);
//         program.attach_shader(&fragment_shader);
//         program.link();
//         program.use_();

//         vertex_shader.delete();
//         fragment_shader.delete();

//         let name = CString::new("inputColor").unwrap();
//         unsafe {
//             location = gl::GetUniformLocation(program.get_id(), name.as_ptr());
//         }

//         let vertices: [f32; 12] = [
//             1.0,
//             0.5 + offset,
//             0.0,
//             1.0,
//             -0.5 + offset,
//             0.0,
//             -1.0,
//             -0.5 + offset,
//             0.0,
//             -1.0,
//             0.5 + offset,
//             0.0,
//         ];
//         let indeces: [u32; 6] = [
//             0, 1, 3, // first triangle
//             1, 2, 3, // second triangle
//         ];

//         let vao = VertexArrayObject::new().unwrap();
//         let vbo = VertexBufferObject::new().unwrap();
//         let ebo = VertexBufferObject::new().unwrap();
//         vao.bind();
//         vbo.bind(BufferType::ArrayBuffer);
//         unsafe {
//             gl::BufferData(
//                 gl::ARRAY_BUFFER,
//                 size_of_val(&vertices) as isize,
//                 vertices.as_ptr().cast(),
//                 gl::STATIC_DRAW,
//             );

//             ebo.bind(BufferType::ElementArrayBuffer);
//             gl::BufferData(
//                 gl::ELEMENT_ARRAY_BUFFER,
//                 size_of_val(&indeces) as isize,
//                 indeces.as_ptr().cast(),
//                 gl::STATIC_DRAW,
//             );

//             gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0 as i32, 0 as *const _);
//             gl::EnableVertexAttribArray(0);

//             gl::ClearColor(0.2_f32, 0.3_f32, 0.3_f32, 1_f32);
//         }
//         Rectangle {
//             vao,
//             vbo,
//             ebo,
//             color_location: location,
//             program,
//         }
//     }

//     fn draw(&self, color: (f32, f32, f32, f32)) {
//         unsafe {
//             gl::Uniform4f(self.color_location, color.0, color.1, color.2, color.3);
//             gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
//         }
//     }

//     fn end(self) {
//         self.ebo.delete();
//         self.vbo.delete();
//         self.vao.delete();
//         self.program.delete();
//     }
// }
