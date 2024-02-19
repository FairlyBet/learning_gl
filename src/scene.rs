use crate::{
    lighting::LightType,
    resources::Resource,
    serializable::{self, Camera, Entity, LightSource, Mesh, Transform, Vec3},
};
use serde::Deserialize;
use std::{
    collections::HashSet,
    fs,
    hash::BuildHasher,
    path::{Path, PathBuf},
};

pub struct Scene {
    pub path: String,
}

impl Scene {
    pub fn new(path: &String) -> Self {
        Self { path: path.clone() }
    }

    pub fn get_entities(&self) -> Vec<Entity> {
        let json_str = fs::read_to_string(&self.path).unwrap();
        let entities = serde_json::from_str::<Vec<Entity>>(&json_str).unwrap();
        entities
    }
}

// impl Resource for Scene {
//     fn storage_name() -> &'static Path {
//         Path::new(SCENES_DIR)
//     }

//     fn acceptable_extensions() -> HashSet<String> {
//         todo!()
//     }
// }

// impl Resource for Entity {
//     fn storage_name() -> &'static Path {
//         Path::new(ENTITIES_FILENAME)
//     }

//     fn acceptable_extensions() -> HashSet<String> {
//         todo!()
//     }
// }

// impl Resource for Transform {
//     fn storage_name() -> &'static Path {
//         Path::new(TRANSFORMS_FILENAME)
//     }

//     fn acceptable_extensions() -> HashSet<String> {
//         todo!()
//     }
// }

// impl Resource for MeshComponent {
//     fn storage_name() -> &'static Path {
//         Path::new(MESHES_FILENAME)
//     }

//     fn acceptable_extensions() -> HashSet<String> {
//         todo!()
//     }
// }

// impl Resource for CameraComponent {
//     fn storage_name() -> &'static Path {
//         Path::new(CAMERAS_FILENAME)
//     }

//     fn acceptable_extensions() -> HashSet<String> {
//         todo!()
//     }
// }

// impl Resource for LightComponent {
//     fn storage_name() -> &'static Path {
//         Path::new(LIGHTS_FILENAME)
//     }

//     fn acceptable_extensions() -> HashSet<String> {
//         todo!()
//     }
// }

// pub fn generate_sample() {
//     let scene_name = "sample_scene";
//     let entities = vec![
//         Entity::default(),
//         Entity {
//             name: String::from("camera"),
//             children: Default::default(),
//             components: Default::default(),
//             parent: None,
//             id: 1,
//         },
//     ];
//     let mut camera_transform = Transform::default();
//     camera_transform.position = Vec3 {
//         x: 0.0,
//         y: 0.0,
//         z: 5.0,
//     };
//     let transforms = vec![Transform::default(), camera_transform];
//     let meshes = vec![MeshComponent {
//         owner_id: 0,
//         mesh_path: "assets\\meshes\\cube.fbx".to_string(),
//     }];
//     let camera = vec![CameraComponent {
//         projection: Default::default(),
//         owner_id: 1,
//     }];
//     let light = vec![LightComponent {
//         color: Vec3 {
//             x: 0.8,
//             y: 0.8,
//             z: 0.8,
//         },
//         type_: LightType::Spot,
//         position: Vec3 {
//             x: 0.0,
//             y: 0.0,
//             z: 0.0,
//         },
//         constant: 1.0,
//         direction: Vec3 {
//             x: 0.0,
//             y: 0.0,
//             z: 0.0,
//         },
//         linear: 0.01,
//         quadratic: 0.001,
//         inner_cutoff: 1.0,
//         outer_cutoff: 20.0,
//         owner_id: 1,
//     }];
//     let mut path_buf = Scene::storage_name().to_path_buf();
//     path_buf.push(scene_name);
//     fs::create_dir_all(path_buf.as_path());

//     let en_str = serde_json::to_string(&entities).unwrap();
//     let tr_str = serde_json::to_string(&transforms).unwrap();
//     let m_str = serde_json::to_string(&meshes).unwrap();
//     let cam_str = serde_json::to_string(&camera).unwrap();
//     let l_str = serde_json::to_string(&light).unwrap();

//     path_buf.push(Entity::storage_name());
//     fs::write(path_buf.as_path(), en_str);
//     path_buf.pop();
//     path_buf.push(Transform::storage_name());
//     fs::write(path_buf.as_path(), tr_str);
//     path_buf.pop();
//     path_buf.push(MeshComponent::storage_name());
//     fs::write(path_buf.as_path(), m_str);
//     path_buf.pop();
//     path_buf.push(CameraComponent::storage_name());
//     fs::write(path_buf.as_path(), cam_str);
//     path_buf.pop();
//     path_buf.push(LightComponent::storage_name());
//     fs::write(path_buf.as_path(), l_str);
// }
