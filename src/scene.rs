use serde::Deserialize;
use std::{fs, path::PathBuf};

const ENTITIES_FILENAME: &str = "entities.json";
const TRANSFORMS_FILENAME: &str = "transforms.json";
const MESHES_FILENAME: &str = "meshes.json";

pub struct Scene {}

impl Scene {
    // pub fn load(&mut self, path: &PathBuf) {
    //     let scene_dir = Path::new(path);

    //     let mut entities = Self::read_vec::<Entity>(&scene_dir.with_file_name(ENTITIES_FILENAME));
    //     let transforms = Self::read_vec::<serializable::Transform>(
    //         &scene_dir.with_file_name(TRANSFORMS_FILENAME),
    //     );

    //     // fill transform container
    //     self.container.transforms = Vec::with_capacity(transforms.len());
    //     for (i, transform) in transforms.iter().enumerate() {
    //         self.container.transforms.push(transform.into_actual());
    //         self.container.transforms[i].owner_id = entities[i].id;
    //         entities[i].transform_index = i;
    //     }

    //     self.container.entities =
    //         FxHashMap::with_capacity_and_hasher(entities.len(), Default::default());
    //     for entity in entities {
    //         self.container.entities.insert(entity.id, entity);
    //     }

    //     // assign parenting pointers and never ever reallocate container
    //     self.container.update_parent_pointers();
    // }

    fn read_vec<T>(path: &PathBuf) -> Vec<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        let json_str = fs::read_to_string(path).unwrap();
        let values = serde_json::from_str::<Vec<T>>(&json_str).unwrap();
        values
    }
}
