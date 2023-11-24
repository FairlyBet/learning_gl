use crate::{
    linear, serializable,
    util::{ByteArray, Reallocated}, data3d::Mesh,
};
use fxhash::FxHasher32;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::BuildHasherDefault, str::FromStr as _};
use strum::EnumCount;

pub type EntityId = u32;
type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher32>>;

#[derive(Default)]
pub struct EntityComponentSys {
    entities: FxHashMap<EntityId, Entity>,
    component_arrays: [ByteArray; ComponentType::COUNT],
    free_ids: Vec<EntityId>,
    id_counter: EntityId,
}

impl EntityComponentSys {
    pub fn empty() -> Self {
        todo!();
    }

    pub fn init(entities: Vec<Entity>, transforms: Vec<serializable::Transform>) -> Self {
        assert_eq!(
            entities.len(),
            transforms.len(),
            "Amount of entitites and transforms must be equal"
        );

        let mut res: Self = Default::default();

        let mut i = 0;
        for mut entity in entities {
            entity.components.push(Component {
                array_index: i,
                type_: ComponentType::Transform,
            });
            res.entities.insert(entity.id, entity);
            i += 1;
        }

        res.component_arrays[ComponentType::Transform as usize] =
            ByteArray::init::<linear::Transform>(transforms.len());
        for transform in transforms {
            let transform = transform.into_actual();
            res.component_arrays[ComponentType::Transform as usize].write(transform);
        }

        res.id_counter = match res.entities.keys().max() {
            Some(max) => *max + 1,
            None => 0,
        };

        res
    }

    pub fn create_entity(&mut self) -> EntityId {
        let mut entity: Entity = Default::default();
        entity.id = self.id_counter;
        let result = entity.id;
        self.id_counter += 1;

        let transform_component = Component {
            array_index: self.component_arrays[ComponentType::Transform as usize]
                .len::<linear::Transform>(),
            type_: ComponentType::Transform,
        };
        entity.components.push(transform_component);

        let transform = linear::Transform::new();
        let re = self.component_arrays[ComponentType::Transform as usize].write(transform);
        if let Reallocated::Yes = re {
            // копец
            // update pointers
            self.update_transform_pointers_on_reallocation();
        }
        result
    }

    /// Takes O(n), n - amount of entities.
    /// Only updates parent transform pointers
    fn update_transform_pointers_on_reallocation(&mut self) {
        for entity in self.entities.values() {
            if let Some(parent_id) = entity.parent {
                let parent = self.entities.get(&parent_id).unwrap();
                let parent_transform_component = parent
                    .components
                    .iter()
                    .find(|x| x.type_ == ComponentType::Transform)
                    .unwrap();
                let parent_transform: &linear::Transform = self.component_arrays
                    [ComponentType::Transform as usize]
                    .get(parent_transform_component.array_index);

                let transform_component = entity
                    .components
                    .iter()
                    .find(|x| x.type_ == ComponentType::Transform)
                    .unwrap();
                let transform: &mut linear::Transform = self.component_arrays
                    [ComponentType::Transform as usize]
                    .get_mut(transform_component.array_index);
                // finally
                transform.parent = Some(parent_transform);
            }
        }
    }

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

    pub fn attach_component(&mut self, target: EntityId, type_: ComponentType) {
        match type_ {
            ComponentType::Transform => todo!(),
            ComponentType::StaticMesh => todo!(),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Entity {
    pub name: String,
    pub children: Vec<EntityId>,
    pub components: Vec<Component>,
    pub parent: Option<EntityId>,
    pub id: EntityId,
}

#[derive(Serialize, Deserialize)]
pub struct Component {
    pub array_index: usize,
    pub type_: ComponentType,
}

#[derive(Serialize, Deserialize, EnumCount, PartialEq)]
#[repr(u32)]
pub enum ComponentType {
    Transform,
    StaticMesh,
}

struct StaticMeshComponent {
    mesh: *const Mesh,
    transform: *const linear::Transform,
    owner_id: EntityId,
}
