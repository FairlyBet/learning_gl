use crate::{
    camera::Camera,
    data3d::{Mesh, Model},
    lighting::LightSource,
    linear, serializable,
};
use fxhash::FxHasher32;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    hash::BuildHasherDefault,
    mem::size_of,
    path::{Path, PathBuf},
    str::FromStr,
};

type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher32>>;

const ENTITIES_FILENAME: &str = "entities.json";
const TRANSFORMS_FILENAME: &str = "transforms.json";
const MESHES_FILENAME: &str = "meshes.json";

pub struct Scene {
    path: PathBuf,
    container: EntityComponentContainer,
}

impl Scene {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            container: Default::default(),
        }
    }

    pub fn load(&mut self) {
        let scene_dir = Path::new(&self.path);

        let mut entities = Self::read_vec::<Entity>(&scene_dir.with_file_name(ENTITIES_FILENAME));
        let transforms = Self::read_vec::<serializable::Transform>(
            &scene_dir.with_file_name(TRANSFORMS_FILENAME),
        );

        // fill transform container
        self.container.transforms = Vec::with_capacity(transforms.len());
        for (i, transform) in transforms.iter().enumerate() {
            self.container.transforms.push(transform.into_actual());
            self.container.transforms[i].owner_id = entities[i].id;
            entities[i].transform_index = i;
        }

        self.container.entities =
            FxHashMap::with_capacity_and_hasher(entities.len(), Default::default());
        for entity in entities {
            self.container.entities.insert(entity.id, entity);
        }

        // assign parenting pointers and never ever reallocate container
        self.container.update_parent_pointers();
    }

    fn read_vec<T>(path: &PathBuf) -> Vec<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        let json_str = fs::read_to_string(path).unwrap();
        let values = serde_json::from_str::<Vec<T>>(&json_str).unwrap();
        values
    }
}

#[derive(Default)]
struct EntityComponentContainer {
    entities: FxHashMap<u32, Entity>,
    transforms: Vec<linear::Transform>,
    static_meshes: Vec<Mesh>,
    id_counter: u32,
}

impl EntityComponentContainer {
    /// Takes O(n), n - amount of entities
    fn update_parent_pointers(&mut self) {
        for entity in self.entities.values() {
            if let Some(parent_id) = entity.parent {
                let parent_transform_index = self.entities.get(&parent_id).unwrap().transform_index;
                self.transforms[entity.transform_index].parent =
                    Some(&self.transforms[parent_transform_index]);
            }
        }
    }

    pub fn insert_entity(&mut self, mut entity: Entity, mut transform: linear::Transform) {
        transform.owner_id = entity.id;
        entity.transform_index = self.transforms.len();
        self.entities.insert(entity.id, entity);
        let reallocating = self.transforms.len() == self.transforms.capacity();
        self.transforms.push(transform);
        if reallocating {
            self.update_parent_pointers();
        }
    }

    /// Returns id of created entity
    pub fn create_entity(&mut self) -> u32 {
        let mut entity: Entity = Default::default();
        entity.name = String::from_str("New entity").unwrap();
        self.id_counter += 1;
        entity.id = self.id_counter;
        let result = entity.id;
        let transform = linear::Transform::new();
        self.insert_entity(entity, transform);
        result
    }

    pub fn init(entities: Vec<Entity>, transforms: Vec<serializable::Transform>) {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Entity {
    pub id: u32,
    pub name: String,
    pub parent: Option<u32>,
    pub children: Vec<u32>,
    pub transform_index: usize,
}

struct StaticMeshComponent {
    owner_id: u32,
    mesh: *const Mesh,
}
