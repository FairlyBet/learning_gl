use crate::{
    gl_wrappers::{self, BufferObject, VertexArrayObject},
    resources::RangeIndex, serializable::Material,
};
use gl::types::GLenum;
use russimp::{Vector2D, Vector3D};
use std::{ffi::c_void, mem::size_of};

pub const QUAD_VERTICES_TEX_COORDS: &[f32] = &[
    -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, -1.0, 1.0, 0.0, 0.0, 1.0, -1.0, -1.0, 0.0,
    0.0, 0.0, 1.0, -1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0,
];

#[repr(C)]
pub struct Vertex {
    pub position: Vector3D,
    pub normal: Vector3D,
    pub tex_coord: Vector2D,
}

#[derive(PartialEq)]
pub enum VertexAttribute {
    Position,
    Normal,
    TexCoord,
}

#[derive(Debug)]
pub struct MeshData {
    vao: VertexArrayObject,
    vbo: BufferObject,
    ebo: BufferObject,
    pub vertex_count: i32,
    pub index_count: i32,
}

impl MeshData {
    pub fn new(
        vertex_count: i32,
        vertex_data_size: usize,
        vertex_data: *const c_void,
        vertex_attributes: Vec<VertexAttribute>,
        index_count: i32,
        index_data: *const c_void,
        usage: GLenum,
    ) -> Self {
        let vao = VertexArrayObject::new().unwrap();
        vao.bind();

        let vertex_buffer = BufferObject::new(gl::ARRAY_BUFFER).unwrap();
        vertex_buffer.bind();
        vertex_buffer.buffer_data(vertex_data_size, vertex_data, usage);

        let element_buffer = BufferObject::new(gl::ELEMENT_ARRAY_BUFFER).unwrap();
        element_buffer.bind();
        element_buffer.buffer_data(index_count as usize * size_of::<u32>(), index_data, usage);

        MeshData::configure_vertex_attributes(vertex_attributes);

        MeshData {
            vao,
            vbo: vertex_buffer,
            ebo: element_buffer,
            vertex_count,
            index_count,
        }
    }

    pub fn from_vertex_index_data(
        vertex_data: &Vec<Vertex>,
        index_data: &Vec<u32>,
        usage: GLenum,
    ) -> MeshData {
        let attributes = vec![
            VertexAttribute::Position,
            VertexAttribute::Normal,
            VertexAttribute::TexCoord,
        ];
        let vertex_count = vertex_data.len() as i32;
        let index_count = index_data.len() as i32;
        MeshData::new(
            vertex_count,
            vertex_data.len() * size_of::<Vertex>(),
            vertex_data.as_ptr().cast(),
            attributes,
            index_count,
            index_data.as_ptr().cast(),
            usage,
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
        VertexArrayObject::unbind(); // anti-pattern
    }
}

pub struct Mesh {
    pub mesh_index: RangeIndex,
    pub material_index: RangeIndex,
}
