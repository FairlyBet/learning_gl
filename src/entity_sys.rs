use crate::{
    camera,
    data3d::ModelIndex,
    linear,
    scene::Scene,
    serializable,
    util::{ByteArray, Reallocated},
};
use fxhash::FxHasher32;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::BuildHasherDefault};
use strum::EnumCount;

pub type EntityId = u32;
type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher32>>;

#[derive(Default)]
pub struct EntitySystem {
    entities: FxHashMap<EntityId, Entity>,
    component_arrays: [ByteArray; ComponentType::COUNT],
    free_ids: Vec<EntityId>,
    id_counter: EntityId,
}

impl EntitySystem {
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
            assert_eq!(
                i, entity.id as usize,
                "Integrity of entities set is not satisfied"
            );
            entity.components.push(ComponentRecord {
                array_index: i,
                type_: ComponentType::Transform,
            });
            res.entities.insert(entity.id, entity);
            i += 1;
        }

        res.component_arrays[ComponentType::Transform as usize] =
            ByteArray::init::<linear::Transform>(transforms.len());
        for transform in transforms {
            let transform: linear::Transform = transform.into();
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

        let transform_component = ComponentRecord {
            array_index: self.component_arrays[ComponentType::Transform as usize]
                .len::<linear::Transform>(),
            type_: ComponentType::Transform,
        };
        entity.components.push(transform_component);

        let transform = linear::Transform::new();
        let re = self.component_arrays[ComponentType::Transform as usize].write(transform);
        if let Reallocated::Yes = re {
            // noooooo...
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

    pub fn attach_component<T>(&mut self, component: T)
    where
        T: Component,
    {
        assert_ne!(
            T::component_type(),
            ComponentType::Transform,
            "Transform may not be attached manually"
        );

        let comp = ComponentRecord {
            array_index: self.component_arrays[T::component_type() as usize].len::<T>(),
            type_: T::component_type(),
        };
        let owner = self.entities.get_mut(&component.owner_id()).unwrap();
        owner.components.push(comp);
        self.component_arrays[T::component_type() as usize].write(component);
    }

    pub fn attach_components<T>(&mut self, components: Vec<T>)
    where
        T: Component,
    {
        for component in components {
            self.attach_component(component);
        }
    }

    pub fn get_component<T>(&self, entity_id: EntityId) -> Option<&T>
    where
        T: Component,
    {
        let entity = self.entities.get(&entity_id).unwrap();
        let component = entity
            .components
            .iter()
            .find(|x| x.type_ == T::component_type());
        match component {
            Some(val) => Some(self.component_arrays[val.type_ as usize].get::<T>(val.array_index)),
            None => None,
        }
    }

    pub fn get_transfom(&self, entity_id: EntityId) -> &linear::Transform {
        self.component_arrays[ComponentType::Transform as usize].get(entity_id as usize)
    }

    pub fn from_scene(scene: &Scene) -> Self {
        let entities = scene.read_vec::<Entity>();
        let transforms = scene.read_vec::<serializable::Transform>();

        Self::init(entities, transforms)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Entity {
    pub name: String,
    // #[serde(skip_serializing)]
    pub children: Vec<EntityId>,
    // #[serde(skip_serializing)]
    pub components: Vec<ComponentRecord>,
    pub parent: Option<EntityId>,
    pub id: EntityId,
}

#[derive(Serialize, Deserialize)]
pub struct ComponentRecord {
    pub array_index: usize,
    pub type_: ComponentType,
}

#[derive(Serialize, Deserialize, EnumCount, PartialEq, Debug, Clone, Copy)]
#[repr(u32)]
pub enum ComponentType {
    Transform,
    Camera,
    Mesh,
}

pub trait Component {
    fn component_type() -> ComponentType;
    fn owner_id(&self) -> EntityId;
}

pub struct MeshComponent {
    pub model_index: ModelIndex,
    pub owner_id: EntityId,
}

impl Component for MeshComponent {
    fn component_type() -> ComponentType {
        ComponentType::Mesh
    }

    fn owner_id(&self) -> EntityId {
        self.owner_id
    }
}

pub struct CameraComponent {
    pub camera: camera::Camera,
    pub owner_id: EntityId,
}

impl Component for CameraComponent {
    fn component_type() -> ComponentType {
        ComponentType::Camera
    }

    fn owner_id(&self) -> EntityId {
        self.owner_id
    }
}
