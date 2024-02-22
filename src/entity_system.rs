use crate::data_3d::Mesh;
use crate::{
    camera::Camera,
    lighting::LightSource,
    linear::Transform,
    resources::ResourceManager,
    scripting::{ScriptFile, Scripting},
    serializable,
    utils::{FxHashMap32, Reallocated, UntypedVec},
};
use rlua::UserData;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use strum::EnumCount;

#[derive(PartialEq, Eq)]
pub struct EntityId(u32);

impl EntityId {
    fn clone(&self) -> Self {
        EntityId(self.0)
    }
}

impl Hash for EntityId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl UserData for EntityId {}

#[derive(Default)]
pub struct SceneManager {
    entities: FxHashMap32<EntityId, Entity>,
    component_arrays: [UntypedVec; ComponentDataType::COUNT],
    free_ids: VecDeque<EntityId>,
    id_counter: u32,
}

impl SceneManager {
    pub fn from_scene_index(
        index: usize,
        resource_manager: &mut ResourceManager,
        scripting: &Scripting,
    ) -> Option<Self> {
        let scene = resource_manager.get_scenes().get(index)?;
        let entities = scene.load_entities();
        let mut scene_manager = Self::default();
        scene_manager.component_arrays[ComponentDataType::Transform.usize()] =
            UntypedVec::init::<Transform>(entities.len());
        let mut current_id = 0;

        for entity in entities {
            let mut ent = Entity {
                id: EntityId(current_id),
                name: entity.name.clone(),
                components: vec![],
                children: vec![],
                parent: None,
            };

            current_id += 1;

            // let cameras = Component::from_vec(ent.id, utils::into_vec::<_, Camera>(entity.cameras));
            // let light_sources = Component::from_vec(
            //     ent.id,
            //     utils::into_vec::<_, LightSource>(entity.light_sources),
            // );
            // let meshes = Component::from_vec(
            //     ent.id,
            //     entity
            //         .meshes
            //         .iter()
            //         .map(|item| resource_manager.get_mesh_lazily(&item.path))
            //         .collect::<Vec<Mesh>>(),
            // );
            let scripts = entity.scripts;

            if !entity.children.is_empty() {
                Self::create_children(
                    ent.id.clone(),
                    &mut current_id,
                    entity.children,
                    &mut scene_manager,
                    resource_manager,
                    &scripting,
                );
            }

            scene_manager.entities.insert(ent.id.clone(), ent);
        }

        todo!()
    }

    fn create_children(
        parent_id: EntityId,
        current_id: &mut u32,
        children: Vec<serializable::Entity>,
        scene_manager: &mut SceneManager,
        resource_manager: &mut ResourceManager,
        scripting: &Scripting,
    ) -> Vec<Entity> {
        for child in children {
            let mut ent = Entity {
                id: EntityId(*current_id),
                name: child.name.clone(),
                components: vec![],
                children: vec![],
                parent: Some(parent_id.clone()),
            };

            *current_id += 1;
        }
        todo!()
    }

    pub fn create_entity(&mut self) -> EntityId {
        let id = self.free_ids.pop_front().unwrap_or_else(|| {
            let res = EntityId(self.id_counter);
            self.id_counter += 1;
            res
        });
        let mut entity = Entity::new(id.clone());

        assert!(
            self.entities.insert(entity.id.clone(), entity).is_none(),
            "Entity id wasn't free"
        );

        let transform = Transform::new();
        if (id.0 as usize)
            < self.component_arrays[ComponentDataType::Transform.usize()].len::<Transform>()
        {
            self.component_arrays[ComponentDataType::Transform.usize()]
                .rewrite(transform, id.0 as usize); // rewriting unused item
        } else {
            let re = self.component_arrays[ComponentDataType::Transform.usize()].push(transform); // pushing new item
            if let Reallocated::Yes = re {
                // update pointers
                self.update_transform_pointers_on_reallocation();
            }
        }

        id
    }

    /// Takes O(n), n - amount of entities.
    /// Only updates parent transform pointers
    fn update_transform_pointers_on_reallocation(&mut self) {
        todo!()
    }

    pub fn attach_component<T>(&mut self, target: EntityId, data: T)
    where
        T: ComponentData,
    {
        assert!(
            self.entities.contains_key(&target),
            "Attempt to attach component to non-existent entity"
        );
        let entity = self.entities.get_mut(&target).unwrap();
        let component = Component::new(target, data);
        let array_index = self.component_arrays[T::type_index().usize()].len::<Component<T>>();
        let component_record = ComponentRecord {
            array_index,
            type_index: T::type_index(),
        };
        entity.components.push(component_record);
        self.component_arrays[T::type_index().usize()].push(component);
    }

    pub fn attach_components<T>(&mut self, target: EntityId, data: Vec<T>)
    where
        T: ComponentData,
    {
        for item in data {
            self.attach_component(target.clone(), item);
        }
    }

    pub fn get_component<T>(&self, entity_id: EntityId) -> Option<&Component<T>>
    where
        T: ComponentData,
    {
        let entity = match self.entities.get(&entity_id) {
            Some(val) => val,
            None => {
                println!("Entity with invalid id is detected: {}", entity_id.0);
                return None;
            }
        };
        let component_record = entity
            .components
            .iter()
            .find(|x| x.type_index == T::type_index())?;

        Some(
            self.component_arrays[component_record.type_index.usize()]
                .get::<Component<T>>(component_record.array_index),
        )
    }

    pub fn component_slice<T>(&self) -> &[Component<T>]
    where
        T: ComponentData,
    {
        self.component_arrays[T::type_index().usize()].slice::<Component<T>>()
    }

    pub fn component_slice_mut<T>(&mut self) -> &mut [Component<T>]
    where
        T: ComponentData,
    {
        self.component_arrays[T::type_index().usize()].mut_slice::<Component<T>>()
    }

    // This optimization requires entities' ids to directly
    // correlate with their transforms' positions in the array
    pub fn get_transform(&self, entity_id: EntityId) -> &Transform {
        self.component_arrays[ComponentDataType::Transform.usize()].get(entity_id.0 as usize)
    }

    pub fn get_transform_mute(&mut self, entity_id: EntityId) -> &mut Transform {
        self.component_arrays[ComponentDataType::Transform.usize()].get_mut(entity_id.0 as usize)
    }
}

pub struct Entity {
    pub id: EntityId,
    pub name: String,
    pub components: Vec<ComponentRecord>,
    pub children: Vec<EntityId>,
    pub parent: Option<EntityId>,
}

impl Entity {
    fn new(id: EntityId) -> Self {
        Self {
            id,
            name: "".to_string(),
            components: vec![],
            children: vec![],
            parent: None,
        }
    }
}

pub struct ComponentRecord {
    pub array_index: usize,
    pub type_index: ComponentDataType,
}

impl ComponentRecord {
    pub fn new(array_index: usize, type_index: ComponentDataType) -> Self {
        Self {
            array_index,
            type_index,
        }
    }
}

#[derive(EnumCount, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
enum ComponentDataType {
    Transform,
    Camera,
    Light,
    Mesh,
    Script,
}

impl ComponentDataType {
    fn usize(&self) -> usize {
        // todo!();
        *self as usize // TEST IF CONVERTING A REFERENCE
    }
}

trait ComponentData {
    fn type_index() -> ComponentDataType;
}

impl ComponentData for Mesh {
    fn type_index() -> ComponentDataType {
        ComponentDataType::Mesh
    }
}

impl ComponentData for Camera {
    fn type_index() -> ComponentDataType {
        ComponentDataType::Camera
    }
}

impl ComponentData for LightSource {
    fn type_index() -> ComponentDataType {
        ComponentDataType::Light
    }
}

impl ComponentData for ScriptFile {
    fn type_index() -> ComponentDataType {
        ComponentDataType::Script
    }
}

pub struct Component<T: ComponentData> {
    owner_id: EntityId,
    pub data: T,
}

impl<T: ComponentData> Component<T> {
    fn type_index() -> ComponentDataType {
        T::type_index()
    }

    pub fn new(owner_id: EntityId, data: T) -> Self {
        Self { owner_id, data }
    }

    pub fn owner_id(&self) -> &EntityId {
        &self.owner_id
    }
}
