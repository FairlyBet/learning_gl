#![windows_subsystem = "windows"]

mod shader;
mod shader_program;
mod vertex_array_object;
mod vertex_buffer_object;

use glfw::{
    Action, Context, Glfw, Key, OpenGlProfileHint, SwapInterval, Window, WindowEvent, WindowHint,
    WindowMode,
};
use std::{mem::size_of_val, sync::mpsc::Receiver};

use shader::{Shader, ShaderType};
use shader_program::ShaderProgram;
use vertex_array_object::VertexArray;
use vertex_buffer_object::{Buffer, BufferType};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const VERTEX_SHADER_SRC: &str = "#version 330 core
layout (location = 0) in vec3 aPos;
void main()
{
    gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
}";
const FRAGMENT_SHADER_YELLOW_SRC: &str = "#version 330 core
out vec4 FragColor;
void main()
{
    FragColor = vec4(1.0f, .843f, 0.0f, 1.0f);
}";
const FRAGMENT_SHADER_BLUE_SRC: &str = "#version 330 core
out vec4 FragColor;
void main()
{
    FragColor = vec4(.0f, .341f, .721f, 1.0f);
}";

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::ContextVersion(3, 3));

    let (mut window, receiver) = glfw
        .create_window(WIDTH, HEIGHT, "Слава Україні!", WindowMode::Windowed)
        .unwrap();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.make_current();
    glfw.set_swap_interval(SwapInterval::Sync(0));

    gl_loader::init_gl();
    gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);

    let blue_rectangle = RectangleRenderingContext::setup(FRAGMENT_SHADER_BLUE_SRC, 0.5);
    blue_rectangle.draw();
    let yellow_rectangle = RectangleRenderingContext::setup(FRAGMENT_SHADER_YELLOW_SRC, -0.5);
    yellow_rectangle.draw();
    window.swap_buffers();
    main_loop::<RectangleRenderingContext>(
        &mut window,
        &receiver,
        &mut glfw,
        &vec![blue_rectangle, yellow_rectangle],
    );

    gl_loader::end_gl();
}

fn main_loop<T>(
    window: &mut Window,
    receiver: &Receiver<(f64, WindowEvent)>,
    glfw: &mut Glfw,
    render_context: &Vec<RectangleRenderingContext>,
) where
    T: RenderContext,
{
    while !window.should_close() {
        handle_window_events(receiver, window, render_context);

        glfw.wait_events();
    }
}

fn handle_window_events(
    receiver: &Receiver<(f64, WindowEvent)>,
    window: &mut Window,
    render_context: &Vec<RectangleRenderingContext>,
) {
    for (_, event) in glfw::flush_messages(receiver) {
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            WindowEvent::FramebufferSize(width, height) => unsafe {
                gl::Viewport(0, 0, width, height);
                gl::Clear(gl::COLOR_BUFFER_BIT);
                for r in render_context.iter() {
                    r.bind();
                    r.draw();
                }
                window.swap_buffers();
            },
            _ => {}
        }
    }
}

trait RenderContext {
    fn bind(&self);
    fn draw(&self);
    fn end(self);
}

struct RectangleRenderingContext {
    vao: VertexArray,
    vbo: Buffer,
    ebo: Buffer,
    program: ShaderProgram,
}

impl RectangleRenderingContext {
    fn setup(fragment_shader_src: &str, offset: f32) -> RectangleRenderingContext {
        let vertex_shader =
            Shader::from_source(ShaderType::VertexShader, VERTEX_SHADER_SRC).unwrap();
        let fragment_shader =
            Shader::from_source(ShaderType::FragmentShader, fragment_shader_src).unwrap();
        let program = ShaderProgram::new().unwrap();
        vertex_shader.compile();
        fragment_shader.compile();
        program.attach_shader(&vertex_shader);
        program.attach_shader(&fragment_shader);
        program.link();
        program.use_();
        vertex_shader.delete();
        fragment_shader.delete();

        let vertices: [f32; 12] = [
            1.0,
            0.5 + offset,
            0.0,
            1.0,
            -0.5 + offset,
            0.0,
            -1.0,
            -0.5 + offset,
            0.0,
            -1.0,
            0.5 + offset,
            0.0,
        ];
        let indeces: [u32; 6] = [
            0, 1, 3, // first triangle
            1, 2, 3, // second triangle
        ];
        let vao = VertexArray::new().unwrap();
        let vbo = Buffer::new().unwrap();
        let ebo = Buffer::new().unwrap();
        vao.bind();
        vbo.bind(BufferType::ArrayBuffer);
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(&vertices) as isize,
                vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            ebo.bind(BufferType::ElementArrayBuffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                size_of_val(&indeces) as isize,
                indeces.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0 as i32, 0 as *const _);
            gl::EnableVertexAttribArray(0);

            gl::ClearColor(0.2_f32, 0.3_f32, 0.3_f32, 1_f32);
        }
        RectangleRenderingContext {
            vao,
            vbo,
            ebo,
            program,
        }
    }
}

impl RenderContext for RectangleRenderingContext {
    fn draw(&self) {
        unsafe {
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const _);
        }
    }

    fn end(self) {
        self.ebo.delete();
        self.vbo.delete();
        self.vao.delete();
        self.program.delete();
    }

    fn bind(&self) {
        self.vao.bind();
        self.program.use_();
    }
}
