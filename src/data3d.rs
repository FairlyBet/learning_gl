use crate::gl_wrappers::{self, BufferObject, VertexArrayObject};
use gl::types::GLenum;
use russimp::{
    scene::{PostProcess, Scene},
    Vector2D, Vector3D,
};
use std::{ffi::c_void, mem::size_of};

pub fn load_model(path: &str, post_pocess: Vec<PostProcess>) -> Model {
    let scene = Scene::from_file(path, post_pocess).unwrap();
    
    let mut meshes = Vec::<Mesh>::with_capacity(scene.meshes.len());
    for mesh in &scene.meshes {
        let vertex_count = mesh.vertices.len();
        let mut vertex_data = Vec::<VertexData>::with_capacity(vertex_count);
        
        for i in 0..vertex_count {
            let position = mesh.vertices[i];
            let normal = mesh.normals[i];
            let tex_coord: Vector2D;
            if let Some(tex_coords) = &(mesh.texture_coords[0]) {
                tex_coord = Vector2D {
                    x: tex_coords[i].x,
                    y: tex_coords[i].y,
                };
            } else {
                tex_coord = Default::default();
            }

            let vertex = VertexData {
                position,
                normal,
                tex_coord,
            };

            vertex_data.push(vertex);
        }

        let mut index_data = Vec::<u32>::with_capacity(mesh.faces.len() * 3);
        for face in &mesh.faces {
            for index in &face.0 {
                index_data.push(*index);
            }
        }

        let mesh = Mesh::from_vertex_index_data(&vertex_data, &index_data, gl::STATIC_DRAW);
        meshes.push(mesh);
    }
    Model::new(meshes)
}

#[repr(C)]
pub struct VertexData {
    pub position: Vector3D,
    pub normal: Vector3D,
    pub tex_coord: Vector2D,
}

#[derive(PartialEq)]
pub enum VertexAttribute {
    Position = 0,
    Normal = 1,
    TexCoord = 2,
}

pub struct Mesh {
    vao: VertexArrayObject,
    vbo: BufferObject,
    ebo: BufferObject,
    pub triangle_count: i32,
    pub index_count: i32,
}

impl Mesh {
    pub const CUBE_VERTICES_NORMALS: [f32; 216] = [
        -0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.5, 0.5, -0.5, 0.0,
        0.0, -1.0, 0.5, 0.5, -0.5, 0.0, 0.0, -1.0, -0.5, 0.5, -0.5, 0.0, 0.0, -1.0, -0.5, -0.5,
        -0.5, 0.0, 0.0, -1.0, -0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.5,
        0.5, 0.5, 0.0, 0.0, 1.0, 0.5, 0.5, 0.5, 0.0, 0.0, 1.0, -0.5, 0.5, 0.5, 0.0, 0.0, 1.0, -0.5,
        -0.5, 0.5, 0.0, 0.0, 1.0, -0.5, 0.5, 0.5, -1.0, 0.0, 0.0, -0.5, 0.5, -0.5, -1.0, 0.0, 0.0,
        -0.5, -0.5, -0.5, -1.0, 0.0, 0.0, -0.5, -0.5, -0.5, -1.0, 0.0, 0.0, -0.5, -0.5, 0.5, -1.0,
        0.0, 0.0, -0.5, 0.5, 0.5, -1.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.0, 0.5, 0.5, -0.5,
        1.0, 0.0, 0.0, 0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 0.5, -0.5,
        0.5, 1.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.0, -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 0.5,
        -0.5, -0.5, 0.0, -1.0, 0.0, 0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 0.5, -0.5, 0.5, 0.0, -1.0, 0.0,
        -0.5, -0.5, 0.5, 0.0, -1.0, 0.0, -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, -0.5, 0.5, -0.5, 0.0,
        1.0, 0.0, 0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 0.5, 0.5, 0.5, 0.0, 1.0, 0.0, 0.5, 0.5, 0.5, 0.0,
        1.0, 0.0, -0.5, 0.5, 0.5, 0.0, 1.0, 0.0, -0.5, 0.5, -0.5, 0.0, 1.0, 0.0,
    ];
    pub const QUAD_VERTICES_TEX_COORDS: [f32; 30] = [
        -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, -1.0, 1.0, 0.0, 0.0, 1.0, -1.0, -1.0,
        0.0, 0.0, 0.0, 1.0, -1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0,
    ];

    pub fn new(
        size: usize,
        vertex_data: *const c_void,
        index_data: *const c_void,
        attributes: Vec<VertexAttribute>,
        usage: GLenum,
        triangle_count: i32,
        index_count: i32,
    ) -> Self {
        let vao = VertexArrayObject::new().unwrap();
        vao.bind();

        let vertex_buffer = BufferObject::new(gl::ARRAY_BUFFER).unwrap();
        vertex_buffer.bind();
        vertex_buffer.buffer_data(size, vertex_data, usage);

        let element_buffer = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER).unwrap();
        element_buffer.bind();
        element_buffer.buffer_data(index_count as usize * size_of::<u32>(), index_data, usage);

        Mesh::configure_vertex_attributes(attributes);

        Mesh {
            vao,
            vbo: vertex_buffer,
            ebo: element_buffer,
            triangle_count,
            index_count,
        }
    }

    pub fn from_vertex_index_data(
        vertex_data: &Vec<VertexData>,
        index_data: &Vec<u32>,
        usage: GLenum,
    ) -> Mesh {
        let attributes = vec![
            VertexAttribute::Position,
            VertexAttribute::Normal,
            VertexAttribute::TexCoord,
        ];
        let triangle_count = vertex_data.len() as i32 / 3;
        let index_count = index_data.len() as i32;
        Mesh::new(
            vertex_data.len() * size_of::<VertexData>(),
            vertex_data.as_ptr().cast(),
            index_data.as_ptr().cast(),
            attributes,
            usage,
            triangle_count,
            index_count,
        )
    }

    fn configure_vertex_attributes(attributes: Vec<VertexAttribute>) {
        let mut stride = 0;

        let position_ptr = 0;
        let mut normal_ptr = 0;
        let mut tex_coord_ptr = 0;

        if attributes.contains(&VertexAttribute::Position) {
            stride += 3;
            normal_ptr += 3;
            tex_coord_ptr += 3;
        }
        if attributes.contains(&VertexAttribute::Normal) {
            stride += 3;
            tex_coord_ptr += 3;
        }
        if attributes.contains(&VertexAttribute::TexCoord) {
            stride += 2;
        }

        stride *= size_of::<f32>();
        normal_ptr *= size_of::<f32>();
        tex_coord_ptr *= size_of::<f32>();

        if attributes.contains(&VertexAttribute::Position) {
            gl_wrappers::configure_attribute(
                VertexAttribute::Position as u32,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                position_ptr as *const _,
            );
            gl_wrappers::enable_attribute(VertexAttribute::Position as u32);
        }
        if attributes.contains(&VertexAttribute::Normal) {
            gl_wrappers::configure_attribute(
                VertexAttribute::Normal as u32,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                normal_ptr as *const _,
            );
            gl_wrappers::enable_attribute(VertexAttribute::Normal as u32);
        }
        if attributes.contains(&VertexAttribute::TexCoord) {
            gl_wrappers::configure_attribute(
                VertexAttribute::TexCoord as u32,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                tex_coord_ptr as *const _,
            );
            gl_wrappers::enable_attribute(VertexAttribute::TexCoord as u32);
        }
    }

    pub fn bind(&self) {
        self.vao.bind();
    }

    pub fn unbind(&self) {
        VertexArrayObject::unbind(); // antipattern
    }
}

pub struct Model {
    meshes: Vec<Mesh>,
}

impl Model {
    pub fn new(meshes: Vec<Mesh>) -> Self {
        Self { meshes }
    }

    pub fn get_meshes(&self) -> &Vec<Mesh> {
        &self.meshes
    }
}
