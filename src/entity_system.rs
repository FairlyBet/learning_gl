use crate::{
    camera,
    lighting::LightSource,
    linear,
    resources::ResourceManager,
    scene::Scene,
    scripting::Script,
    serializable,
    util::{self, Reallocated, UntypedVec},
};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, ops::Range};
use strum::EnumCount;
use util::FxHashMap32;

pub type EntityId = u32;

#[derive(Default)]
pub struct SceneManager {
    entities: FxHashMap32<EntityId, Entity>,
    component_arrays: [UntypedVec; ComponentType::COUNT],
    free_ids: VecDeque<EntityId>,
    id_counter: EntityId,
}

impl SceneManager {
    pub fn init() -> Self {
        // assert_eq!(
        //     entities.len(),
        //     transforms.len(),
        //     "Amount of entitites and transforms must be equal"
        // );

        // let mut res: Self = Default::default();

        // let mut i = 0;
        // for mut entity in entities {
        //     assert_eq!(
        //         i, entity.id as usize,
        //         "Integrity of entities array is not satisfied"
        //     );
        //     res.entities.insert(entity.id, entity);
        //     i += 1;
        // }

        // res.component_arrays[ComponentType::Transform as usize] =
        //     UntypedVec::init::<linear::Transform>(transforms.len());
        // for transform in transforms {
        //     let transform: linear::Transform = transform.into();
        //     res.component_arrays[ComponentType::Transform as usize].push(transform);
        // }
        // res.id_counter = i as EntityId;

        // res

        todo!()
    }

    pub fn from_scene(scene: &Scene, asset_manager: &ResourceManager) -> Self {
        todo!()
        // let entities = scene.read_vec::<Entity>();
        // let transforms = scene.read_vec::<serializable::Transform>();

        // let mut self_ = Self::init(entities, transforms);

        // let mesh_components: Vec<MeshComponent> = scene
        //     .read_vec::<serializable::MeshComponent>()
        //     .iter()
        //     .map(|x| MeshComponent {
        //         model_index: asset_manager.get_meshes().get_index(&x.mesh_path),
        //         owner_id: x.owner_id,
        //     })
        //     .collect();
        // let camera_components: Vec<CameraComponent> =
        //     util::into_vec(scene.read_vec::<serializable::CameraComponent>());
        // let light_components: Vec<LightComponent> =
        //     util::into_vec(scene.read_vec::<serializable::LightComponent>());

        // self_.attach_components(mesh_components);
        // self_.attach_components(camera_components);
        // self_.attach_components(light_components);

        // self_
    }

    pub fn create_entity(&mut self) -> EntityId {
        let mut entity: Entity = Default::default();
        entity.id = self.free_ids.pop_front().unwrap_or_else(|| {
            let res = self.id_counter;
            self.id_counter += 1;
            res
        });

        let transform = linear::Transform::new();
        if (entity.id as usize)
            < self.component_arrays[ComponentType::Transform as usize].len::<linear::Transform>()
        {
            self.component_arrays[ComponentType::Transform as usize]
                .rewrite(transform, entity.id as usize); // rewrite existing item
        } else {
            let re = self.component_arrays[ComponentType::Transform as usize].push(transform); // push new item
            if let Reallocated::Yes = re {
                // update pointers
                self.update_transform_pointers_on_reallocation();
            }
        }

        let res = entity.id;
        assert!(self.entities.insert(entity.id, entity).is_none());
        res
    }

    /// Takes O(n), n - amount of entities.
    /// Only updates parent transform pointers
    fn update_transform_pointers_on_reallocation(&mut self) {
        todo!()
        // for entity in self.entities.values() {
        //     if let Some(parent_id) = entity.parent {
        //         let parent = self.entities.get(&parent_id).unwrap();
        //         let parent_transform_component = parent
        //             .components
        //             .iter()
        //             .find(|x| x.type_ == ComponentType::Transform)
        //             .unwrap();
        //         let parent_transform: &linear::Transform = self.component_arrays
        //             [ComponentType::Transform as usize]
        //             .get(parent_transform_component.array_index);

        //         let transform_component = entity
        //             .components
        //             .iter()
        //             .find(|x| x.type_ == ComponentType::Transform)
        //             .unwrap();
        //         let transform: &mut linear::Transform = self.component_arrays
        //             [ComponentType::Transform as usize]
        //             .get_mut(transform_component.array_index);
        //         // finally
        //         transform.parent = Some(parent_transform);
        //     }
        // }
    }

    pub fn attach_component<T>(&mut self, component: T)
    where
        T: Component,
    {
        let comp = ComponentRecord {
            array_index: self.component_arrays[T::component_type() as usize].len::<T>(),
            type_: T::component_type(),
        };
        let owner = self.entities.get_mut(&component.owner_id()).unwrap();
        owner.components.push(comp);
        self.component_arrays[T::component_type() as usize].push(component);
    }

    pub fn attach_components(&mut self, components: Vec<impl Component>) {
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

    pub fn component_slice<T>(&self) -> &[T]
    where
        T: Component,
    {
        self.component_arrays[T::component_type() as usize].slice::<T>()
    }

    pub fn component_slice_mut<T>(&mut self) -> &mut [T]
    where
        T: Component,
    {
        self.component_arrays[T::component_type() as usize].mut_slice::<T>()
    }

    /// This optimization requires entities ids to be a consequtive progression (0, 1, 2...)
    pub fn get_transfom(&self, entity_id: EntityId) -> &linear::Transform {
        self.component_arrays[ComponentType::Transform as usize].get(entity_id as usize)
    }

    pub fn get_transfom_mut(&mut self, entity_id: EntityId) -> &mut linear::Transform {
        self.component_arrays[ComponentType::Transform as usize].get_mut(entity_id as usize)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Entity {
    pub id: EntityId,
    pub name: String,
    pub components: Vec<ComponentRecord>,
    pub children: Vec<EntityId>,
    pub parent: Option<EntityId>,
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
    Light,
    Mesh,
    Script,
}

pub trait Component {
    fn component_type() -> ComponentType;
    fn owner_id(&self) -> EntityId;
}

pub struct MeshComponent {
    pub model_index: Range<usize>,
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

pub struct LightComponent {
    pub light_source: LightSource,
    pub owner_id: EntityId,
}

impl Component for LightComponent {
    fn component_type() -> ComponentType {
        ComponentType::Light
    }

    fn owner_id(&self) -> EntityId {
        self.owner_id
    }
}

pub struct ScriptComponent {
    pub owner_id: EntityId,
    pub script: Script,
}

impl Component for ScriptComponent {
    fn component_type() -> ComponentType {
        ComponentType::Script
    }

    fn owner_id(&self) -> EntityId {
        self.owner_id
    }
}
