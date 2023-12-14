use crate::data3d::{Mesh, ModelContainer, VertexData};
use russimp::{
    scene::{PostProcess, Scene},
    Vector2D,
};
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr as _,
};

const ASSETS_DIR: &str = "assets";
const MESHES_FOLDER: &str = "meshes";
const TEXTURE_FOLDER: &str = "textures";

const MESH_EXT: &str = "fbx";

pub fn get_paths<T>(target_ext: Option<&str>) -> Vec<String>
where
    T: StorageName,
{
    let result = fs::create_dir_all(ASSETS_DIR);
    result.unwrap();
    let mut path = PathBuf::from_str(ASSETS_DIR).unwrap();
    path.push(T::storage_name());

    let mut result = vec![];
    let entries = fs::read_dir(path).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let extension;
        if entry.file_type().unwrap().is_file() {
            match entry.path().extension() {
                Some(ext) => extension = ext.to_owned().into_string().unwrap_or_default(),
                None => extension = Default::default(),
            }
            if let Some(target) = target_ext {
                if extension == target {
                    result.push(entry.path().into_os_string().into_string().unwrap());
                }
            }
        }
    }
    result
}

pub fn load_all_models() -> ModelContainer {
    let mut result = ModelContainer::new();
    let mesh_paths = get_paths::<ModelContainer>(Some(MESH_EXT));
    for mesh_path in mesh_paths {
        let meshes = load_model(
            &mesh_path,
            vec![
                PostProcess::Triangulate,
                PostProcess::OptimizeMeshes,
                PostProcess::OptimizeGraph,
            ],
        );
        result.push(mesh_path, meshes);
    }
    result
}

pub fn load_model(path: &String, post_pocess: Vec<PostProcess>) -> Vec<Mesh> {
    let model = Scene::from_file(path, post_pocess).unwrap();
    let mut meshes = Vec::<Mesh>::with_capacity(model.meshes.len());
    for mesh in &model.meshes {
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
        // 1 face contains 3 indexes
        let mut index_data = Vec::<u32>::with_capacity(mesh.faces.len() * 3);
        for face in &mesh.faces {
            for index in &face.0 {
                index_data.push(*index);
            }
        }
        let mesh = Mesh::from_vertex_index_data(&vertex_data, &index_data, gl::STATIC_DRAW);
        meshes.push(mesh);
    }
    meshes
}

pub trait StorageName {
    fn storage_name() -> &'static Path;
}

impl StorageName for ModelContainer {
    fn storage_name() -> &'static Path {
        Path::new(MESHES_FOLDER)
    }
}
