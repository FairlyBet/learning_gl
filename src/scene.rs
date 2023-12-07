use crate::{
    asset_loader::StorageName,
    entity_sys::Entity,
    serializable::{Mesh, Transform},
};
use serde::Deserialize;
use std::{
    fs,
    hash::BuildHasher,
    path::{Path, PathBuf},
};

const SCENES_DIR: &str = "scenes";
const COMPONENTS_FOLDER: &str = "components";
const ENTITIES_FILENAME: &str = "entities.json";
const TRANSFORMS_FILENAME: &str = "transforms.json";
const MESHES_FILENAME: &str = "meshes.json";
const CAMERAS_FILENAME: &str = "cameras.json";

pub fn get_scenes() -> Vec<Scene> {
    fs::create_dir_all(Scene::storage_name());
    let entries = fs::read_dir(Scene::storage_name()).unwrap();

    let mut scenes = vec![];
    for entry in entries {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            scenes.push(Scene::new(
                entry.path().into_os_string().into_string().unwrap(),
            ));
        }
    }
    scenes
}

pub struct Scene {
    pub path: String,
}

impl Scene {
    fn new(path: String) -> Self {
        Self { path }
    }

    pub fn read_vec<T>(&self) -> Vec<T>
    where
        T: for<'a> Deserialize<'a> + StorageName,
    {
        let mut path = PathBuf::new();
        path.push(&self.path);
        path.push(T::storage_name());
        let json_str = fs::read_to_string(path).unwrap();
        let values = serde_json::from_str::<Vec<T>>(&json_str).unwrap();
        values
    }
}

impl StorageName for Scene {
    fn storage_name() -> &'static Path {
        Path::new(SCENES_DIR)
    }
}

impl StorageName for Entity {
    fn storage_name() -> &'static Path {
        Path::new(ENTITIES_FILENAME)
    }
}

impl StorageName for Transform {
    fn storage_name() -> &'static Path {
        Path::new(TRANSFORMS_FILENAME)
    }
}

impl StorageName for Mesh {
    fn storage_name() -> &'static Path {
        Path::new(MESHES_FILENAME)
    }
}

pub fn generate_sample() {
    let scene_name = "sample_scene";
    let entities = vec![Entity::default()];
    let transforms = vec![Transform::default()];
    let meshes = vec![Mesh {
        owner_id: 0,
        mesh_path: "assets\\meshes\\sample.fbx".to_owned(),
    }];

    let mut path_buf = Scene::storage_name().to_path_buf();
    path_buf.push(scene_name);
    fs::create_dir_all(path_buf.as_path());

    let en_str = serde_json::to_string(&entities).unwrap();
    let tr_str = serde_json::to_string(&transforms).unwrap();
    let m_str = serde_json::to_string(&meshes).unwrap();

    path_buf.push(Entity::storage_name());
    fs::write(path_buf.as_path(), en_str);
    path_buf.pop();
    path_buf.push(Transform::storage_name());
    fs::write(path_buf.as_path(), tr_str);
    path_buf.pop();
    path_buf.push(Mesh::storage_name());
    fs::write(path_buf.as_path(), m_str);
}
