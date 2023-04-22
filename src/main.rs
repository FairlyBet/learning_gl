// #![windows_subsystem = "windows"]

mod camera;
mod engine;
mod object;
mod renderer;
mod shader;
mod shader_program;
mod texture;
mod vertex_array_object;
mod vertex_buffer_object;

extern crate nalgebra_glm as glm;

use glfw::{
    Action, Context, CursorMode, Key, OpenGlProfileHint, SwapInterval, Window, WindowEvent,
    WindowHint, WindowMode,
};
use glm::{vec3, Mat4, Vec3};
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

const VERTICES: [f32; 24] = [
    0.5, 0.5, 0.5, //
    0.5, 0.5, -0.5, //
    -0.5, 0.5, -0.5, //
    -0.5, 0.5, 0.5, //
    // sdfgd
    0.5, -0.5, 0.5, //
    0.5, -0.5, -0.5, //
    -0.5, -0.5, -0.5, //
    -0.5, -0.5, 0.5, //
];

const ELEMENTS: [u32; 36] = [
    0, 1, 2, //
    0, 2, 3, //
    0, 1, 5, //
    0, 4, 5, //
    0, 3, 7, //
    0, 4, 7, //
    //asd
    6, 1, 5, //
    6, 1, 2, //
    6, 2, 3, //
    6, 3, 7, //
    6, 4, 5, //
    6, 4, 7, //
];

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::ContextVersion(3, 3));

    let (mut window, receiver) = glfw
        .create_window(WIDTH, HEIGHT, "", WindowMode::Windowed)
        .unwrap();

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_cursor_mode(CursorMode::Disabled);
    window.make_current();
    glfw.set_swap_interval(SwapInterval::Sync(1));

    gl_loader::init_gl();
    gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);

    unsafe {
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);
        gl::Enable(gl::DEPTH_TEST);
    }

    let vao = VertexArrayObject::new().unwrap();
    vao.bind();

    let vbo = VertexBufferObject::new(gl::ARRAY_BUFFER).unwrap();
    vbo.bind();
    vbo.buffer_data(
        size_of_val(&VERTICES),
        VERTICES.as_ptr().cast(),
        gl::STATIC_DRAW,
    );

    let ebo = VertexBufferObject::new(gl::ELEMENT_ARRAY_BUFFER).unwrap();
    ebo.bind();
    ebo.buffer_data(
        size_of_val(&ELEMENTS),
        ELEMENTS.as_ptr().cast(),
        gl::STATIC_DRAW,
    );

    let program = ShaderProgram::from_vert_frag(VERT_SHDR_SRC, FRAG_SHDR_SRC).unwrap();
    program.use_();
    ShaderProgram::configure_attribute(0, 3, gl::FLOAT, gl::FALSE, 0, 0 as *const _);
    ShaderProgram::enable_attribute(0);

    let mvp_location = program.get_uniform("MVP");
    let color_location = program.get_uniform("self_color");
    let light_location = program.get_uniform("light_color");

    let aspect = calculate_aspect(window.get_framebuffer_size());

    let target_transform = glm::translate(&Mat4::identity(), &vec3(1.0, 0.0, -2.0));
    let target_color = vec3(1.0, 0.5, 0.31);

    let light_transform = glm::translate(&Mat4::identity(), &vec3(0.0, 4.0, 0.0));
    let light_transform = glm::scale(&light_transform, &Vec3::from_element(0.3));
    let light_color = vec3(1.0, 1.0, 1.0);

    let mut camera = Camera::new();
    camera.translate(&vec3(0.0, 0.0, 2.0));

    let mut projection = glm::perspective(aspect, to_rad(45.0), 0.1, 100.0);
    let mut mvp: Mat4;

    let mut frame_time: f32 = 0.0;

    while !window.should_close() {
        glfw.set_time(0.0);
        window.set_cursor_pos(0.0, 0.0);
        glfw.poll_events();

        update_camera(&mut camera, &window, frame_time);

        handle_window_events(&receiver, &mut window, &mut projection);

        mvp = projection * camera.get_view() * target_transform;

        unsafe {
            // draw object
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::UniformMatrix4fv(mvp_location, 1, gl::FALSE, glm::value_ptr(&mvp).as_ptr());
            gl::Uniform3fv(color_location, 1, target_color.as_ptr());
            gl::Uniform3fv(light_location, 1, light_color.as_ptr());

            gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, 0 as *const _);

            // draw light source
            mvp = projection * camera.get_view() * light_transform;
            gl::UniformMatrix4fv(mvp_location, 1, gl::FALSE, glm::value_ptr(&mvp).as_ptr());
            gl::Uniform3fv(color_location, 1, vec3(1.0, 1.0, 1.0).as_ptr());
            gl::Uniform3fv(light_location, 1, light_color.as_ptr());

            gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, 0 as *const _);
        }

        window.swap_buffers();

        frame_time = glfw.get_time() as f32;
    }

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

fn to_deg(rad: f32) -> f32 {
    rad * DEG_TO_RAD
}

fn update_camera(camera: &mut Camera, window: &Window, frame_time: f32) {
    let sensitivity = 2.0;
    let pos = window.get_cursor_pos();
    let x = pos.0 as f32;
    let y = pos.1 as f32;
    let local_rotation = vec3(-y, 0.0, 0.0) * sensitivity * frame_time;
    let global_rotation = vec3(0.0, -x, 0.0) * sensitivity * frame_time;

    let mut delta = Vec3::zeros();
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
    if delta.magnitude() > 0.0 {
        delta = glm::normalize(&delta); // returning nan
    }
    delta *= velocity * frame_time;

    camera.rotate(&global_rotation);
    camera.rotate_local(&local_rotation);
    camera.move_local(&delta);
}
