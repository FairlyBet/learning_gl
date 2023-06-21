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
use std::{
    f32::consts,
    mem::{size_of, size_of_val},
    sync::mpsc::Receiver,
};

use camera::Camera;
use shader_program::ShaderProgram;
use vertex_array_object::VertexArrayObject;
use vertex_buffer_object::VertexBufferObject;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const VERTICES: [f32; 216] = [
    -0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.5, 0.5, -0.5, 0.0, 0.0,
    -1.0, 0.5, 0.5, -0.5, 0.0, 0.0, -1.0, -0.5, 0.5, -0.5, 0.0, 0.0, -1.0, -0.5, -0.5, -0.5, 0.0,
    0.0, -1.0, -0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.5, 0.5, 0.5, 0.0,
    0.0, 1.0, 0.5, 0.5, 0.5, 0.0, 0.0, 1.0, -0.5, 0.5, 0.5, 0.0, 0.0, 1.0, -0.5, -0.5, 0.5, 0.0,
    0.0, 1.0, -0.5, 0.5, 0.5, -1.0, 0.0, 0.0, -0.5, 0.5, -0.5, -1.0, 0.0, 0.0, -0.5, -0.5, -0.5,
    -1.0, 0.0, 0.0, -0.5, -0.5, -0.5, -1.0, 0.0, 0.0, -0.5, -0.5, 0.5, -1.0, 0.0, 0.0, -0.5, 0.5,
    0.5, -1.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.0, 0.5, 0.5, -0.5, 1.0, 0.0, 0.0, 0.5, -0.5,
    -0.5, 1.0, 0.0, 0.0, 0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 0.5, -0.5, 0.5, 1.0, 0.0, 0.0, 0.5, 0.5,
    0.5, 1.0, 0.0, 0.0, -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 0.5,
    -0.5, 0.5, 0.0, -1.0, 0.0, 0.5, -0.5, 0.5, 0.0, -1.0, 0.0, -0.5, -0.5, 0.5, 0.0, -1.0, 0.0,
    -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, -0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 0.5, 0.5, -0.5, 0.0, 1.0,
    0.0, 0.5, 0.5, 0.5, 0.0, 1.0, 0.0, 0.5, 0.5, 0.5, 0.0, 1.0, 0.0, -0.5, 0.5, 0.5, 0.0, 1.0, 0.0,
    -0.5, 0.5, -0.5, 0.0, 1.0, 0.0,
];

const PHONG_VERT_SRC: &str = include_str!("shaders\\phong_shader.vert");
const PHONG_FRAG_SRC: &str = include_str!("shaders\\phong_shader.frag");
const TRIVIAL_VERT_SRC: &str = include_str!("shaders\\trivial_shader.vert");
const TRIVIAL_FRAG_SRC: &str = include_str!("shaders\\trivial_shader.frag");

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

    let array_buffer = VertexBufferObject::new(gl::ARRAY_BUFFER).unwrap();
    array_buffer.bind();
    array_buffer.buffer_data(
        size_of_val(&VERTICES),
        VERTICES.as_ptr().cast(),
        gl::STATIC_DRAW,
    );
    array_buffer.unbind();

    let lamp_vao = VertexArrayObject::new().unwrap();
    lamp_vao.bind();

    array_buffer.bind();

    let trivial_program =
        ShaderProgram::from_vert_frag(TRIVIAL_VERT_SRC, TRIVIAL_FRAG_SRC).unwrap();
    trivial_program.use_();
    ShaderProgram::configure_attribute(
        0,
        3,
        gl::FLOAT,
        gl::FALSE,
        size_of::<f32>() * 6,
        0 as *const _,
    );
    ShaderProgram::enable_attribute(0);

    VertexArrayObject::unbind();

    let cube_vao = VertexArrayObject::new().unwrap();
    cube_vao.bind();

    array_buffer.bind();

    let phong_program = ShaderProgram::from_vert_frag(PHONG_VERT_SRC, PHONG_FRAG_SRC).unwrap();
    phong_program.use_();
    ShaderProgram::configure_attribute(
        0,
        3,
        gl::FLOAT,
        gl::FALSE,
        size_of::<f32>() * 6,
        0 as *const _,
    );
    ShaderProgram::configure_attribute(
        1,
        3,
        gl::FLOAT,
        gl::FALSE,
        size_of::<f32>() * 6,
        (size_of::<f32>() * 3) as *const _,
    );
    ShaderProgram::enable_attribute(0);
    ShaderProgram::enable_attribute(1);

    VertexArrayObject::unbind();

    let aspect = calculate_aspect(window.get_framebuffer_size());

    let cube_transform = glm::translate(&Mat4::identity(), &vec3(0.0, 0.0, 0.0));

    let lamp_position = vec3(1.0, 0.0, -2.0);
    let lamp_scale = Vec3::from_element(0.5);
    let mut lamp_transform = Mat4::identity();
    lamp_transform = glm::translate(&lamp_transform, &lamp_position);
    lamp_transform = glm::scale(&lamp_transform, &lamp_scale);
    let lamp_color = vec3(1.0, 1.0, 1.0);

    let mut camera = Camera::new();
    camera.translate(&vec3(0.0, 0.0, 3.0));

    let mut projection = glm::perspective(aspect, to_rad(45.0), 0.1, 100.0);

    let mut frame_time = 0.0_f32;

    while !window.should_close() {
        glfw.set_time(0.0);
        window.set_cursor_pos(0.0, 0.0);
        glfw.poll_events();

        update_camera(&mut camera, &window, frame_time);

        handle_window_events(&receiver, &mut window, &mut projection);

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            trivial_program.use_();
            lamp_vao.bind();

            let model_location = trivial_program.get_uniform("model");
            let view_location = trivial_program.get_uniform("view");
            let projection_location = trivial_program.get_uniform("projection");
            let color_location = trivial_program.get_uniform("self_color");

            gl::UniformMatrix4fv(
                model_location,
                1,
                gl::FALSE,
                glm::value_ptr(&lamp_transform).as_ptr(),
            );
            gl::UniformMatrix4fv(
                view_location,
                1,
                gl::FALSE,
                glm::value_ptr(&camera.get_view()).as_ptr(),
            );
            gl::UniformMatrix4fv(
                projection_location,
                1,
                gl::FALSE,
                glm::value_ptr(&projection).as_ptr(),
            );
            gl::Uniform3fv(color_location, 1, glm::value_ptr(&lamp_color).as_ptr());

            gl::DrawArrays(gl::TRIANGLES, 0, 36);

            phong_program.use_();
            cube_vao.bind();

            let model_location = phong_program.get_uniform("model");
            let view_location = phong_program.get_uniform("view");
            let projection_location = phong_program.get_uniform("projection");

            let ambient = phong_program.get_uniform("material.ambient");
            let diffuse = phong_program.get_uniform("material.diffuse");
            let specular = phong_program.get_uniform("material.specular");
            let shininess = phong_program.get_uniform("material.shininess");

            let light_color_location = phong_program.get_uniform("light_color");
            let light_position_location = phong_program.get_uniform("light_position");
            let view_position_location = phong_program.get_uniform("view_position");

            gl::UniformMatrix4fv(
                model_location,
                1,
                gl::FALSE,
                glm::value_ptr(&cube_transform).as_ptr(),
            );
            gl::UniformMatrix4fv(
                view_location,
                1,
                gl::FALSE,
                glm::value_ptr(&camera.get_view()).as_ptr(),
            );
            gl::UniformMatrix4fv(
                projection_location,
                1,
                gl::FALSE,
                glm::value_ptr(&projection).as_ptr(),
            );
            gl::Uniform3fv(
                light_color_location,
                1,
                glm::value_ptr(&lamp_color).as_ptr(),
            );
            gl::Uniform3fv(
                light_position_location,
                1,
                glm::value_ptr(&lamp_position).as_ptr(),
            );
            gl::Uniform3fv(
                view_position_location,
                1,
                glm::value_ptr(&camera.get_position()).as_ptr(),
            );
            gl::Uniform3f(ambient, 0.2125, 0.1275, 0.054);
            gl::Uniform3f(diffuse, 0.714, 0.4284, 0.18144);
            gl::Uniform3f(specular, 0.393548, 0.271906, 0.166721);
            gl::Uniform1f(shininess, 0.2);

            gl::DrawArrays(gl::TRIANGLES, 0, 36);
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
