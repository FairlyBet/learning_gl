use crate::{
    entity_system::Entity,
    lighting::LightType,
    resources::StorageName,
    serializable::{self, CameraComponent, LightComponent, MeshComponent, Transform, Vec3},
};
use serde::Deserialize;
use std::{
    collections::HashSet,
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
const LIGHTS_FILENAME: &str = "lights.json";
const SCRIPTS_FILENAME: &str = "scripts.json";

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
        let mut path = PathBuf::from(&self.path);
        // path.push(&self.path);
        path.push(T::storage_name());
        let json_str = fs::read_to_string(path).unwrap();
        let values = serde_json::from_str::<Vec<T>>(&json_str).unwrap();
        values
    }

    pub fn deserialize(&self) {
        let json_str = fs::read_to_string(&self.path).unwrap();
        let scene = serde_json::from_str::<Vec<serializable::Entity>>(&json_str).unwrap();
    }
}

impl StorageName for Scene {
    fn storage_name() -> &'static Path {
        Path::new(SCENES_DIR)
    }

    fn acceptable_extensions() -> HashSet<String> {
        todo!()
    }
}

impl StorageName for Entity {
    fn storage_name() -> &'static Path {
        Path::new(ENTITIES_FILENAME)
    }

    fn acceptable_extensions() -> HashSet<String> {
        todo!()
    }
}

impl StorageName for Transform {
    fn storage_name() -> &'static Path {
        Path::new(TRANSFORMS_FILENAME)
    }

    fn acceptable_extensions() -> HashSet<String> {
        todo!()
    }
}

impl StorageName for MeshComponent {
    fn storage_name() -> &'static Path {
        Path::new(MESHES_FILENAME)
    }

    fn acceptable_extensions() -> HashSet<String> {
        todo!()
    }
}

impl StorageName for CameraComponent {
    fn storage_name() -> &'static Path {
        Path::new(CAMERAS_FILENAME)
    }

    fn acceptable_extensions() -> HashSet<String> {
        todo!()
    }
}

impl StorageName for LightComponent {
    fn storage_name() -> &'static Path {
        Path::new(LIGHTS_FILENAME)
    }

    fn acceptable_extensions() -> HashSet<String> {
        todo!()
    }
}

pub fn generate_sample() {
    let scene_name = "sample_scene";
    let entities = vec![
        Entity::default(),
        Entity {
            name: String::from("camera"),
            children: Default::default(),
            components: Default::default(),
            parent: None,
            id: 1,
        },
    ];
    let mut camera_transform = Transform::default();
    camera_transform.position = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 5.0,
    };
    let transforms = vec![Transform::default(), camera_transform];
    let meshes = vec![MeshComponent {
        owner_id: 0,
        mesh_path: "assets\\meshes\\cube.fbx".to_string(),
    }];
    let camera = vec![CameraComponent {
        projection: Default::default(),
        owner_id: 1,
    }];
    let light = vec![LightComponent {
        color: Vec3 {
            x: 0.8,
            y: 0.8,
            z: 0.8,
        },
        type_: LightType::Spot,
        position: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        constant: 1.0,
        direction: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        linear: 0.01,
        quadratic: 0.001,
        inner_cutoff: 1.0,
        outer_cutoff: 20.0,
        owner_id: 1,
    }];
    let mut path_buf = Scene::storage_name().to_path_buf();
    path_buf.push(scene_name);
    fs::create_dir_all(path_buf.as_path());

    let en_str = serde_json::to_string(&entities).unwrap();
    let tr_str = serde_json::to_string(&transforms).unwrap();
    let m_str = serde_json::to_string(&meshes).unwrap();
    let cam_str = serde_json::to_string(&camera).unwrap();
    let l_str = serde_json::to_string(&light).unwrap();

    path_buf.push(Entity::storage_name());
    fs::write(path_buf.as_path(), en_str);
    path_buf.pop();
    path_buf.push(Transform::storage_name());
    fs::write(path_buf.as_path(), tr_str);
    path_buf.pop();
    path_buf.push(MeshComponent::storage_name());
    fs::write(path_buf.as_path(), m_str);
    path_buf.pop();
    path_buf.push(CameraComponent::storage_name());
    fs::write(path_buf.as_path(), cam_str);
    path_buf.pop();
    path_buf.push(LightComponent::storage_name());
    fs::write(path_buf.as_path(), l_str);
}
