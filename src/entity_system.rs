use crate::{
    camera::Camera,
    data_3d::Mesh,
    lighting::LightSource,
    linear::Transform,
    resources::ResourceManager,
    scripting::{ScriptObject, Scripting},
    serializable,
    utils::{self, FxHashMap32, Reallocated, TypelessVec},
};
use mlua::{FromLua, IntoLua, Lua, RegistryKey, Value};
use std::{
    collections::VecDeque,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
};
use strum::EnumCount;

#[derive(Default, Debug)]
pub struct SceneManager {
    entities: FxHashMap32<u32, Entity>,
    components: [TypelessVec; ComponentDataType::COUNT],
    mutated_transform: Vec<usize>,
    available_indecies: VecDeque<usize>,
    instance_counter: u32,
    // loaded_scenes: HashMap<SceneId, Vec<InstanceId>>
}

impl SceneManager {
    pub fn load_scene(
        &mut self,
        index: usize,
        resource_manager: &mut ResourceManager,
        scripting: &Scripting,
    ) {
        let scene = resource_manager.scenes().get(index).unwrap();
        let entities = scene.load_entities();
        let mut scene_manager = Self::default();
        scene_manager.components[ComponentDataType::Transform.usize()] =
            TypelessVec::init::<Transform>(entities.len());

        self.create_entities(None, entities, resource_manager, scripting);
    }

    fn create_entities(
        &mut self,
        parent_id: Option<u32>,
        entities: Vec<serializable::Entity>,
        resource_manager: &mut ResourceManager,
        scripting: &Scripting,
    ) {
        for entity in entities {
            let id = self.create_entity();

            let ent = self.entities.get_mut(&id).unwrap();
            ent.name = entity.name.clone();
            let transform: Transform = entity.transform.into();
            _ = self.components[ComponentDataType::Transform.usize()]
                .rewrite(ent.transform_index, transform);

            self.attach_components(id, utils::convert_vec::<_, Camera>(entity.cameras));
            self.attach_components(
                id,
                utils::convert_vec::<_, LightSource>(entity.light_sources),
            );
            self.attach_components(
                id,
                entity
                    .meshes
                    .iter()
                    .map(|item| resource_manager.get_mesh_lazily(&item.path))
                    .collect::<Vec<Mesh>>(),
            );
            self.attach_components(
                id,
                entity
                    .scripts
                    .iter()
                    .map(|item| scripting.create_script_object(id, item, resource_manager))
                    .collect::<Vec<ScriptObject>>(),
            );

            self.set_parent(id, parent_id);

            self.create_entities(Some(id), entity.children, resource_manager, scripting);
        }
    }

    pub fn create_entity(&mut self) -> u32 {
        let mut rewrite = true;
        let instance_id = self.instance_counter;
        let transform_index = self.available_indecies.pop_front().unwrap_or_else(|| {
            rewrite = false;
            self.instance_counter as usize
        });
        self.instance_counter = self.instance_counter.checked_add(1).unwrap(); // panic if overflows

        let entity = Entity::new(instance_id, transform_index);

        let transform = Transform::new();
        if rewrite {
            _ = self.components[ComponentDataType::Transform.usize()]
                .rewrite(transform_index, transform); // rewriting unused item
        } else {
            let reallocated = self.components[ComponentDataType::Transform.usize()].push(transform); // pushing new item
            if let Reallocated::Yes = reallocated {
                // update pointers
                self.update_transform_pointers_on_reallocation();
            }
        }

        instance_id
    }

    pub fn set_parent(&mut self, child_id: u32, parent_id: Option<u32>) {
        match parent_id {
            Some(parent_id) => {
                if !self.entities[&parent_id].children.contains(&child_id) {
                    self.set_parent(child_id, None);

                    let parent = self.entities.get_mut(&parent_id).unwrap();
                    parent.children.push(child_id);
                    let parent_transform = parent.transform_index;
                    let parent_transform = self.components[ComponentDataType::Transform.usize()]
                        .get::<Transform>(parent_transform)
                        as *const _;

                    let child = self.entities.get_mut(&child_id).unwrap();
                    child.parent = Some(parent_id);
                    let child_transform = child.transform_index;
                    self.components[ComponentDataType::Transform.usize()]
                        .get_mut::<Transform>(child_transform)
                        .parent = Some(parent_transform);
                }
            }
            None => {
                if self.entities[&child_id].parent.is_some() {
                    let parent_id = self.entities[&child_id].parent.unwrap();
                    let parent = self
                        .entities
                        .get_mut(&parent_id)
                        .unwrap()
                        .children
                        .retain(|item| *item != child_id);

                    let child = self.entities.get_mut(&child_id).unwrap();
                    child.parent = None;
                    let child_transform = child.transform_index;

                    self.components[ComponentDataType::Transform.usize()]
                        .get_mut::<Transform>(child_transform)
                        .parent = None;
                }
            }
        }
    }

    /// Takes O(n), n - amount of entities.
    /// Only updates parent transform pointers
    fn update_transform_pointers_on_reallocation(&mut self) {
        todo!()
    }

    #[allow(private_bounds)]
    pub fn attach_component<T>(&mut self, target_id: u32, data: T)
    where
        T: ComponentData,
    {
        let entity = self.entities.get_mut(&target_id).unwrap();

        let component = Component::new(
            EntityRecord {
                transform_index: entity.transform_index,
                instance_id: target_id,
            },
            data,
        );
        self.components[T::data_type().usize()].push(component);

        let array_index = self.components[T::data_type().usize()].len::<Component<T>>() - 1;
        let component_record = ComponentRecord {
            array_index,
            data_type: T::data_type(),
        };
        entity.components.push(component_record);
    }

    #[allow(private_bounds)]
    pub fn attach_components<T>(&mut self, target: u32, data: Vec<T>)
    where
        T: ComponentData,
    {
        for item in data {
            self.attach_component(target, item);
        }
    }

    #[allow(private_bounds)]
    pub fn get_component<T>(&self, owner_id: u32) -> Option<&ComponentRecord>
    where
        T: ComponentData,
    {
        self.entities
            .get(&owner_id)?
            .components
            .iter()
            .find(|record| record.data_type == T::data_type())
    }

    #[allow(private_bounds)]
    pub fn get_components<T>(&self, owner_id: u32) -> impl Iterator<Item = &ComponentRecord>
    where
        T: ComponentData,
    {
        self.entities[&owner_id]
            .components
            .iter()
            .filter(|record| record.data_type == T::data_type())
    }

    #[allow(private_bounds)]
    pub fn component_slice<T>(&self) -> &[Component<T>]
    where
        T: ComponentData,
    {
        self.components[T::data_type().usize()].slice()
    }

    fn component_slice_mut<T>(&mut self) -> &mut [Component<T>]
    where
        T: ComponentData,
    {
        self.components[T::data_type().usize()].slice_mut()
    }

    pub fn get_transform(&self, record: &EntityRecord) -> &Transform {
        self.components[ComponentDataType::Transform.usize()].get(record.transform_index)
    }

    pub fn get_transform_mut(&mut self, record: &EntityRecord) -> &mut Transform {
        self.components[ComponentDataType::Transform.usize()].get_mut(record.transform_index)
    }

    #[allow(private_bounds)]
    pub fn delete_unmanaged_component<T>(&mut self, record: &ComponentRecord)
    where
        T: ComponentData + Unmanaged,
    {
        _ = self.delete_component::<T>(record);
    }

    #[allow(private_bounds)]
    pub fn delete_managed_component<T>(&mut self, record: &ComponentRecord, scripting: &Scripting)
    where
        T: ComponentData + Managed,
    {
        match T::data_type() {
            ComponentDataType::ScriptObject => {
                let script_object = self.delete_component::<ScriptObject>(record);
                scripting.delete_script_object(script_object)
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn delete_component<T>(&mut self, record: &ComponentRecord) -> T
    where
        T: ComponentData,
    {
        todo!()
        // assert_eq!(T::data_type(), record.data_type);

        // let index_of_deleting = record.array_index;
        // let owner_id = self.component_slice::<T>()[record.array_index]
        //     .owner_record.instance_id;

        // self.entities
        //     .get_mut(&owner_id)
        //     .unwrap()
        //     .components
        //     .retain(|item| item != record);

        // let data = self.components[T::data_type().usize()]
        //     .take_at::<Component<T>>(index_of_deleting)
        //     .data;

        // let len = self.component_slice::<T>().len();
        // if index_of_deleting < len {
        //     let affected_entities = (&self.component_slice::<T>()[index_of_deleting..len])
        //         .iter()
        //         .map(|component| component.owner_record.clone())
        //         .collect::<Vec<EntityId>>();

        //     for id in affected_entities {
        //         let entity = self.entities.get_mut(&id).unwrap();
        //         entity
        //             .components
        //             .iter_mut()
        //             .filter(|record| {
        //                 record.data_type == T::data_type() && record.array_index > index_of_deleting
        //             })
        //             .for_each(|record| record.array_index -= 1);
        //     }
        // }
        // data
    }

    pub fn delete_entity(&mut self, id: u32, scripting: &Scripting) {
        // let records = self.entities[id]
        //     .components
        //     .iter()
        //     .map(|item| item.clone())
        //     .collect::<Vec<ComponentRecord>>();

        // for record in records {
        //     match record.data_type {
        //         ComponentDataType::Camera => self.delete_unmanaged_component::<Camera>(&record),
        //         ComponentDataType::LightSource => {
        //             self.delete_unmanaged_component::<LightSource>(&record)
        //         }
        //         ComponentDataType::Mesh => self.delete_unmanaged_component::<Mesh>(&record),
        //         ComponentDataType::ScriptObject => {
        //             self.delete_managed_component::<ScriptObject>(&record, scripting)
        //         }
        //         _ => unreachable!(),
        //     }
        // }

        // _ = self.entities.remove(&id).unwrap();
        // self.available_indecies.push_back(id.clone());
    }
}

#[derive(Debug)]
pub struct EntityRecord {
    transform_index: usize,
    instance_id: u32,
}

impl EntityRecord {
    pub fn transform_index(&self) -> usize {
        self.transform_index
    }

    pub fn instance_id(&self) -> u32 {
        self.instance_id
    }
}

#[derive(Debug)]
pub struct Entity {
    instance_id: usize,
    transform_index: usize,
    pub name: String,
    pub components: Vec<ComponentRecord>,
    children: Vec<usize>,
    parent: Option<usize>,
    // loaded_from_scene: u32
    // modified_transforms: Vec<usize>
}

impl Entity {
    fn new(instance_id: u32, transform_index: usize) -> Self {
        Self {
            instance_id,
            transform_index,
            name: "".to_string(),
            components: vec![],
            children: vec![],
            parent: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ComponentRecord {
    array_index: usize,
    data_type: ComponentDataType,
}

impl ComponentRecord {
    fn new(array_index: usize, type_index: ComponentDataType) -> Self {
        Self {
            array_index,
            data_type: type_index,
        }
    }

    pub fn array_index(&self) -> usize {
        self.array_index
    }

    fn clone(&self) -> Self {
        Self {
            array_index: self.array_index,
            data_type: self.data_type,
        }
    }
}

#[repr(u32)]
#[derive(EnumCount, PartialEq, Eq, Clone, Copy, Debug)]
enum ComponentDataType {
    Transform,
    Camera,
    LightSource,
    Mesh,
    ScriptObject,
}

impl ComponentDataType {
    const fn usize(&self) -> usize {
        *self as usize
    }
}

#[allow(private_bounds)]
trait ComponentData {
    fn data_type() -> ComponentDataType;
}

impl ComponentData for Mesh {
    fn data_type() -> ComponentDataType {
        ComponentDataType::Mesh
    }
}

impl ComponentData for Camera {
    fn data_type() -> ComponentDataType {
        ComponentDataType::Camera
    }
}

impl ComponentData for LightSource {
    fn data_type() -> ComponentDataType {
        ComponentDataType::LightSource
    }
}

impl ComponentData for ScriptObject {
    fn data_type() -> ComponentDataType {
        ComponentDataType::ScriptObject
    }
}

trait Unmanaged {}

impl Unmanaged for Mesh {}
impl Unmanaged for Camera {}
impl Unmanaged for LightSource {}

trait Managed {}

impl Managed for ScriptObject {}

#[allow(private_bounds)]
pub struct Component<T: ComponentData> {
    owner_record: EntityRecord,
    pub data: T,
}

#[allow(private_bounds)]
impl<T: ComponentData> Component<T> {
    pub fn new(owner_record: EntityRecord, data: T) -> Self {
        Self { owner_record, data }
    }

    pub fn owner_record(&self) -> &EntityRecord {
        &self.owner_record
    }
}
