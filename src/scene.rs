use crate::{
    camera::Camera,
    data3d::{Mesh, Model},
    lighting::LightSource,
    linear, serializable,
};
use core::panic;
use fxhash::FxHasher32;
use serde::{Deserialize, Serialize};
use std::{
    alloc::{self, Layout},
    collections::HashMap,
    fs,
    hash::BuildHasherDefault,
    mem::size_of,
    path::{Path, PathBuf},
    ptr,
    str::FromStr,
};

type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher32>>;
pub type EntityId = u32;

const ENTITIES_FILENAME: &str = "entities.json";
const TRANSFORMS_FILENAME: &str = "transforms.json";
const MESHES_FILENAME: &str = "meshes.json";

pub struct Scene {
    container: EntityComponentSys,
}

impl Scene {
    pub fn new(path: PathBuf) -> Self {
        Self {
            container: Default::default(),
        }
    }

    pub fn load(&mut self, path: &PathBuf) {
        let scene_dir = Path::new(path);

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
struct EntityComponentSys {
    entities: FxHashMap<EntityId, Entity>,
    components: [ByteArray; ComponentType::VariantCount as usize],
    transforms: Vec<linear::Transform>,
    free_ids: Vec<EntityId>,
    id_counter: EntityId,
}

impl EntityComponentSys {
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

    pub fn attach_component(&mut self, target: u32, type_: ComponentType) {
        match type_ {
            ComponentType::Transform => todo!(),
            ComponentType::StaticMesh => todo!(),
            ComponentType::VariantCount => unreachable!(),
        }
    }

    pub fn init(entities: Vec<Entity>, transforms: Vec<serializable::Transform>) {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Entity {
    pub id: EntityId,
    pub name: String,
    pub parent: Option<EntityId>,
    pub children: Vec<EntityId>,
    pub components: Vec<Component>,

    pub transform_index: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Component {
    pub index: usize,
    pub type_: ComponentType,
}

#[derive(Serialize, Deserialize)]
#[repr(u32)]
pub enum ComponentType {
    Transform,
    StaticMesh,

    VariantCount,
}

struct StaticMeshComponent {
    owner_id: u32,
    mesh: *const Mesh,
    transform: *const linear::Transform,
}

pub struct ByteArray {
    buf: *mut u8,
    size: usize,
    len: usize,
}

impl ByteArray {
    pub fn init<T>(n: usize) -> Self {
        let (buf, size) = Self::alloc_bytes(n * size_of::<T>());
        Self { buf, size, len: 0 }
    }

    fn uninited() -> Self {
        Self {
            buf: ptr::null_mut::<u8>(),
            size: Default::default(),
            len: Default::default(),
        }
    }

    pub fn write<T>(&mut self, value: T) {
        if self.len + size_of::<T>() > self.size {
            self.resize(self.size + size_of::<T>() * (self.size / size_of::<T>() + 1));
        }
        unsafe {
            self.buf
                .add(self.len)
                .copy_from(&value as *const T as *const u8, size_of::<T>());
        }
        self.len += size_of::<T>();
    }

    fn resize(&mut self, size: usize) {
        let (buf, size) = Self::alloc_bytes(size);
        unsafe {
            buf.copy_from(self.buf, self.len);
        }
        self.dealloc();
        self.buf = buf;
        self.size = size;
    }

    fn alloc_bytes(n_bytes: usize) -> (*mut u8, usize) {
        let layout = Layout::array::<u8>(n_bytes).unwrap();
        let buf = unsafe { alloc::alloc(layout) };
        if buf == ptr::null_mut::<u8>() {
            panic!("Can't allocate memory")
        }
        (buf, layout.size())
    }

    fn dealloc(&mut self) {
        let layout = Layout::array::<u8>(self.size).unwrap();
        unsafe {
            alloc::dealloc(self.buf, layout);
        }
    }

    pub fn len<T>(&self) -> usize {
        self.len / size_of::<T>()
    }

    pub fn get<T>(&self, index: usize) -> &T {
        if index >= self.len::<T>() {
            panic!("Index out of bounds")
        }
        unsafe {
            let ptr = self.buf.add(index * size_of::<T>()) as *const u8 as *const T;
            &(*ptr)
        }
    }
}

impl Drop for ByteArray {
    fn drop(&mut self) {
        self.dealloc();
    }
}

impl Default for ByteArray {
    fn default() -> Self {
        ByteArray::uninited()
    }
}
