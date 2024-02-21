use crate::resources::RangedIndex;
use crate::{
    camera::Camera,
    lighting::LightSource,
    linear::Transform,
    resources::ResourceManager,
    scripting::{Script, Scripting},
    serializable,
    utils::{self, FxHashMap32, Reallocated, UntypedVec},
};
use std::{collections::VecDeque, ops::Range};
use strum::EnumCount;

pub type EntityId = u32;

#[derive(Default)]
pub struct SceneManager {
    entities: FxHashMap32<EntityId, Entity>,
    component_arrays: [UntypedVec; ComponentType::COUNT],
    free_ids: VecDeque<EntityId>,
    id_counter: EntityId,
}

impl SceneManager {
    pub fn from_scene_index(
        index: usize,
        resource_manager: &mut ResourceManager,
        scripting: &Scripting,
    ) -> Option<Self> {
        let scene = resource_manager.scenes().get(index)?;
        let entities = scene.load_entities();
        let mut scene_manager = Self::default();
        scene_manager.component_arrays[ComponentType::Transform as usize] =
            UntypedVec::init::<Transform>(entities.len());
        let mut current_id = 0;

        for entity in entities {
            let mut ent = Entity {
                id: current_id,
                name: entity.name.clone(),
                components: vec![],
                children: vec![],
                parent: None,
            };

            current_id += 1;

            let cameras = utils::into_vec::<_, Camera>(entity.cameras);
            let light_sources = utils::into_vec::<_, LightSource>(entity.light_sources);
            let meshes = entity
                .meshes
                .iter()
                .map(|item| resource_manager.get_mesh_lazily(&item.path))
                .collect::<Vec<Range<usize>>>();
            let scripts = entity.scripts;

            if !entity.children.is_empty() {
                Self::create_children(
                    ent.id,
                    &mut current_id,
                    entity.children,
                    &mut scene_manager,
                    resource_manager,
                    &scripting,
                );
            }

            scene_manager.entities.insert(ent.id, ent);
        }

        todo!()
    }

    fn create_children(
        parent_id: EntityId,
        current_id: &mut EntityId,
        children: Vec<serializable::Entity>,
        scene_manager: &mut SceneManager,
        resource_manager: &mut ResourceManager,
        scripting: &Scripting,
    ) -> Vec<Entity> {
        for child in children {
            let mut ent = Entity {
                id: *current_id,
                name: child.name.clone(),
                components: vec![],
                children: vec![],
                parent: Some(parent_id),
            };

            *current_id += 1;
        }
        todo!()
    }

    pub fn create_entity(&mut self) -> EntityId {
        let mut entity: Entity = Default::default();
        entity.id = self.free_ids.pop_front().unwrap_or_else(|| {
            let res = self.id_counter;
            self.id_counter += 1;
            res
        });

        let transform = Transform::new();
        if (entity.id as usize)
            < self.component_arrays[ComponentType::Transform as usize].len::<Transform>()
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
    }

    pub fn get_component<T>(&self, entity_id: EntityId) -> Option<&T>
    where
        T: Component,
    {
        let entity = match self.entities.get(&entity_id) {
            Some(val) => val,
            None => {
                println!("Entity with invalid id is detected: {}", entity_id);
                return None;
            }
        };
        let component = entity
            .components
            .iter()
            .find(|x| x.type_ == T::component_type())?;

        Some(self.component_arrays[component.type_ as usize].get::<T>(component.array_index))
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

    // This optimization requires entities' ids to directly
    // correlate with their transforms' positions in the array
    pub fn get_transform(&self, entity_id: EntityId) -> &Transform {
        self.component_arrays[ComponentType::Transform as usize].get(entity_id as usize)
    }

    pub fn get_transform_mute(&mut self, entity_id: EntityId) -> &mut Transform {
        self.component_arrays[ComponentType::Transform as usize].get_mut(entity_id as usize)
    }
}

#[derive(Default)]
pub struct Entity {
    pub id: EntityId,
    pub name: String,
    pub components: Vec<ComponentRecord>,
    pub children: Vec<EntityId>,
    pub parent: Option<EntityId>,
}

pub struct ComponentRecord {
    pub array_index: usize,
    pub type_: ComponentType,
}

#[derive(EnumCount, PartialEq, Debug, Clone, Copy)]
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
    pub model_index: RangedIndex,
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
    pub camera: Camera,
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
    pub script: Script,
    pub owner_id: EntityId,
}

impl Component for ScriptComponent {
    fn component_type() -> ComponentType {
        ComponentType::Script
    }

    fn owner_id(&self) -> EntityId {
        self.owner_id
    }
}

pub trait ToComponent<Component: self::Component> {
    fn convert(self, owner_id: EntityId) -> Component;
}

impl ToComponent<CameraComponent> for Camera {
    fn convert(self, owner_id: EntityId) -> CameraComponent {
        CameraComponent {
            camera: self,
            owner_id,
        }
    }
}

impl ToComponent<LightComponent> for LightSource {
    fn convert(self, owner_id: EntityId) -> LightComponent {
        LightComponent {
            light_source: self,
            owner_id,
        }
    }
}

impl ToComponent<MeshComponent> for RangedIndex {
    fn convert(self, owner_id: EntityId) -> MeshComponent {
        MeshComponent {
            model_index: self,
            owner_id,
        }
    }
}

pub struct Comp<T: Component> {
    owner_id: EntityId,
    val: T
}

impl<T: Component> Comp<T> {
    
}