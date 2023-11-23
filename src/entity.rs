use crate::{linear, serializable, util::ByteArray};
use fxhash::FxHasher32;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::BuildHasherDefault, str::FromStr as _};
use strum::EnumCount;

pub type EntityId = u32;
type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher32>>;

#[derive(Default)]
pub struct EntityComponentSys {
    entities: FxHashMap<EntityId, Entity>,
    components: [ByteArray; ComponentType::COUNT],
    free_ids: Vec<EntityId>,
    id_counter: EntityId,
}

impl EntityComponentSys {
    pub fn empty() -> Self {
        todo!();
    }

    pub fn init(entities: Vec<Entity>, transforms: Vec<serializable::Transform>) -> Self {
        let mut res: Self = Default::default();
        for entity in entities {
            res.entities.insert(entity.id, entity);
        }
        for transform in transforms {
            let transform = transform.into_actual();
            res.components[ComponentType::Transform as usize].write(transform);
        }
        let mut i = 0;
        for entity in res.entities.values_mut() {
            entity.components.push(Component {
                index: i,
                type_: ComponentType::Transform,
            });
            i += 1;
        }
        res.id_counter = match res.entities.keys().max() {
            Some(max) => *max + 1,
            None => 0,
        };
        res
    }

    /// Takes O(n), n - amount of entities
    // fn update_parent_pointers(&mut self) {
    //     for entity in self.entities.values() {
    //         if let Some(parent_id) = entity.parent {
    //             let parent_transform_index = self.entities.get(&parent_id).unwrap().transform_index;
    //             self.transforms[entity.transform_index].parent =
    //                 Some(&self.transforms[parent_transform_index]);
    //         }
    //     }
    // }

    // pub fn insert_entity(&mut self, mut entity: Entity, mut transform: linear::Transform) {
    //     transform.owner_id = entity.id;
    //     entity.transform_index = self.transforms.len();
    //     self.entities.insert(entity.id, entity);
    //     let reallocating = self.transforms.len() == self.transforms.capacity();
    //     self.transforms.push(transform);
    //     if reallocating {
    //         self.update_parent_pointers();
    //     }
    // }

    /// Returns id of created entity
    // pub fn create_entity(&mut self) -> u32 {
    //     let mut entity: Entity = Default::default();
    //     entity.name = String::from_str("New entity").unwrap();
    //     self.id_counter += 1;
    //     entity.id = self.id_counter;
    //     let result = entity.id;
    //     let transform = linear::Transform::new();
    //     self.insert_entity(entity, transform);
    //     result
    // }

    pub fn attach_component(&mut self, target: u32, type_: ComponentType) {
        match type_ {
            ComponentType::Transform => todo!(),
            ComponentType::StaticMesh => todo!(),
        }
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

#[derive(Serialize, Deserialize, EnumCount)]
#[repr(u32)]
pub enum ComponentType {
    Transform,
    StaticMesh,
}

// struct StaticMeshComponent {
//     owner_id: u32,
//     mesh: *const Mesh,
//     transform: *const linear::Transform,
// }
