use crate::{
    data3d::{Mesh, Model, ModelContainer, VertexData},
    scripting::Scripting,
};
use russimp::{
    scene::{PostProcess, PostProcessSteps, Scene},
    Vector2D,
};
use std::{
    cell::RefCell,
    collections::{hash_set, HashSet},
    fs::{self, FileType},
    path::{Path, PathBuf},
    str::FromStr as _,
};

const ASSETS_DIR: &str = "assets";
const MESHES_FOLDER: &str = "meshes";
const TEXTURE_FOLDER: &str = "textures";
const SCRIPT_FOLDER: &str = "scripts";
const FBX_EXT: &str = "fbx";
const LUA_EXT: &str = "lua";

pub trait StorageName {
    fn storage_name() -> &'static Path;
    fn acceptable_extensions() -> HashSet<String>;
}

pub fn get_paths<T>() -> Vec<String>
where
    T: StorageName,
{
    let result = fs::create_dir_all(ASSETS_DIR);
    result.unwrap();
    let mut path = PathBuf::from_str(ASSETS_DIR).unwrap();
    path.push(T::storage_name());

    fn search<T>(path: &Path) -> Vec<String>
    where
        T: StorageName,
    {
        let mut result = vec![];
        let entries = fs::read_dir(path).unwrap();

        for entry in entries {
            let entry = entry.unwrap();
            if let Some(extension) = entry.path().extension() {
                if T::acceptable_extensions().contains(extension.to_str().unwrap()) {
                    result.push(entry.path().into_os_string().into_string().unwrap());
                } else if entry.file_type().unwrap().is_dir() {
                    result.append(&mut search::<T>(&path));
                }
            }
        }
        result
    }
    
    search::<T>(&path)
}

impl StorageName for ModelContainer {
    fn storage_name() -> &'static Path {
        Path::new(MESHES_FOLDER)
    }

    fn acceptable_extensions() -> HashSet<String> {
        HashSet::<String>::from([FBX_EXT.to_string()])
    }
}

impl StorageName for Scripting {
    fn storage_name() -> &'static Path {
        Path::new(SCRIPT_FOLDER)
    }

    fn acceptable_extensions() -> HashSet<String> {
        HashSet::<String>::from([LUA_EXT.to_string()])
    }
}

pub const DEFAULT_POSTPROCESS: [PostProcess; 5] = [
    PostProcess::Triangulate,
    PostProcess::OptimizeMeshes,
    PostProcess::OptimizeGraph,
    PostProcess::JoinIdenticalVertices,
    PostProcess::ImproveCacheLocality,
];

pub fn load_model(path: &String, post_pocess: PostProcessSteps) -> Model {
    let model = Scene::from_file(path, post_pocess).unwrap();
    let mut meshes = Model::with_capacity(model.meshes.len());
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
