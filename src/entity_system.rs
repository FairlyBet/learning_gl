use crate::{
    camera::Camera,
    data_3d::Mesh,
    lighting::LightSource,
    linear::Transform,
    resources::ResourceManager,
    scripting::{ScriptObject, Scripting},
    serializable,
    utils::{self, FxHashMap32, Reallocated, UntypedVec},
};
use mlua::{FromLua, IntoLua, Lua, RegistryKey, Value};
use std::{
    collections::VecDeque,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
};
use strum::EnumCount;

#[derive(PartialEq, Eq, Debug)]
pub struct EntityId(u32);

impl EntityId {
    fn clone(&self) -> Self {
        EntityId(self.0)
    }

    pub fn id(&self) -> u32 {
        self.0
    }
}

impl Hash for EntityId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<'lua> IntoLua<'lua> for &EntityId {
    fn into_lua(self, lua: &'lua Lua) -> mlua::prelude::LuaResult<Value<'lua>> {
        lua.create_any_userdata(self.clone()).unwrap().into_lua(lua)
    }
}

#[derive(Debug)]
pub struct RefEntityId<'lua>(EntityId, PhantomData<&'lua ()>);

impl<'lua> PartialEq for RefEntityId<'lua> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'lua> Eq for RefEntityId<'lua> {}

impl<'lua> Deref for RefEntityId<'lua> {
    type Target = EntityId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'lua> FromLua<'lua> for RefEntityId<'lua> {
    fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::prelude::LuaResult<Self> {
        let userdata = value.as_userdata().unwrap();
        Ok(RefEntityId(
            userdata.borrow::<EntityId>()?.clone(),
            PhantomData::default(),
        ))
    }
}

#[derive(Default, Debug)]
pub struct SceneManager {
    entities: FxHashMap32<EntityId, Entity>,
    components: [UntypedVec; ComponentDataType::COUNT],
    free_ids: VecDeque<EntityId>,
    id_counter: u32,
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
        scene_manager.components[ComponentDataType::Transform.usize()] =
            UntypedVec::init::<Transform>(entities.len());

        Self::create_entities(
            None,
            entities,
            &mut scene_manager,
            resource_manager,
            scripting,
        );

        Some(scene_manager)
    }

    fn create_entities(
        parent_id: Option<&EntityId>,
        entities: Vec<serializable::Entity>,
        scene_manager: &mut SceneManager,
        resource_manager: &mut ResourceManager,
        scripting: &Scripting,
    ) {
        for entity in entities {
            let id = scene_manager.create_entity().clone();

            scene_manager.entities.get_mut(&id).unwrap().name = entity.name.clone();

            let transform: Transform = entity.transform.into();
            _ = scene_manager.components[ComponentDataType::Transform.usize()]
                .rewrite(id.0 as usize, transform);

            scene_manager.attach_components(&id, utils::into_vec::<_, Camera>(entity.cameras));
            scene_manager
                .attach_components(&id, utils::into_vec::<_, LightSource>(entity.light_sources));
            scene_manager.attach_components(
                &id,
                entity
                    .meshes
                    .iter()
                    .map(|item| resource_manager.get_mesh_lazily(&item.path))
                    .collect::<Vec<Mesh>>(),
            );
            scene_manager.attach_components(
                &id,
                entity
                    .scripts
                    .iter()
                    .map(|item| scripting.create_script_object(&id, item, resource_manager))
                    .collect::<Vec<ScriptObject>>(),
            );

            scene_manager.set_parent(&id, parent_id);

            Self::create_entities(
                Some(&id),
                entity.children,
                scene_manager,
                resource_manager,
                scripting,
            );
        }
    }

    pub fn create_entity(&mut self) -> &EntityId {
        let id = self.free_ids.pop_front().unwrap_or_else(|| {
            let res = EntityId(self.id_counter);
            self.id_counter += 1;
            res
        });
        let entity = Entity::new(id.clone());

        assert!(
            self.entities.insert(entity.id.clone(), entity).is_none(),
            "Entity id wasn't free"
        );

        let transform = Transform::new();
        if (id.0 as usize)
            < self.components[ComponentDataType::Transform.usize()].len::<Transform>()
        {
            _ = self.components[ComponentDataType::Transform.usize()]
                .rewrite(id.0 as usize, transform); // rewriting unused item
        } else {
            let re = self.components[ComponentDataType::Transform.usize()].push(transform); // pushing new item
            if let Reallocated::Yes = re {
                // update pointers
                self.update_transform_pointers_on_reallocation();
            }
        }

        &self.entities[&id].id
    }

    pub fn set_parent(&mut self, child: &EntityId, parent: Option<&EntityId>) {
        match parent {
            Some(parent) => {
                if !self.entities[parent].children.contains(&child) {
                    self.set_parent(child, None);
                    self.entities
                        .get_mut(parent)
                        .unwrap()
                        .children
                        .push(child.clone());
                    self.entities.get_mut(child).unwrap().parent = Some(parent.clone());

                    let parent_transform = self.components[ComponentDataType::Transform.usize()]
                        .get::<Transform>(parent.0 as usize)
                        as *const _;

                    self.components[ComponentDataType::Transform.usize()]
                        .get_mut::<Transform>(child.0 as usize)
                        .parent = Some(parent_transform);
                }
            }
            None => {
                if self.entities[child].parent.is_some() {
                    let parent = self.entities[child].parent.as_ref().unwrap().clone();
                    self.entities
                        .get_mut(&parent)
                        .unwrap()
                        .children
                        .retain(|item| item != child);
                    self.entities.get_mut(child).unwrap().parent = None;

                    self.components[ComponentDataType::Transform.usize()]
                        .get_mut::<Transform>(child.0 as usize)
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

    pub fn attach_component<T>(&mut self, target: &EntityId, data: T)
    where
        T: ComponentData,
    {
        assert!(
            self.entities.contains_key(&target),
            "Attempt to attach component to non-existent entity"
        );
        let entity = self.entities.get_mut(&target).unwrap();
        let component = Component::new(target.clone(), data);
        let array_index = self.components[T::data_type().usize()].len::<Component<T>>();
        let component_record = ComponentRecord {
            array_index,
            data_type: T::data_type(),
        };
        entity.components.push(component_record);
        self.components[T::data_type().usize()].push(component);
    }

    pub fn attach_components<T>(&mut self, target: &EntityId, data: Vec<T>)
    where
        T: ComponentData,
    {
        for item in data {
            self.attach_component(target, item);
        }
    }

    pub fn get_component<T>(&self, owner_id: &EntityId) -> Option<&ComponentRecord>
    where
        T: ComponentData,
    {
        self.entities[owner_id]
            .components
            .iter()
            .find(|record| record.data_type == T::data_type())
    }

    pub fn get_components<T>(&self, owner_id: &EntityId) -> impl Iterator<Item = &ComponentRecord>
    where
        T: ComponentData,
    {
        self.entities[owner_id]
            .components
            .iter()
            .filter(|record| record.data_type == T::data_type())
    }

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
        self.components[T::data_type().usize()].mut_slice()
    }

    // This optimization requires entities' ids to directly
    // correlate with their transforms' positions in the array
    pub fn get_transform(&self, entity_id: &EntityId) -> &Transform {
        self.components[ComponentDataType::Transform.usize()].get(entity_id.0 as usize)
    }

    pub fn get_transform_mut(&mut self, entity_id: &EntityId) -> &mut Transform {
        self.components[ComponentDataType::Transform.usize()].get_mut(entity_id.0 as usize)
    }

    pub fn delete_unmanaged_component<T>(&mut self, record: &ComponentRecord)
    where
        T: ComponentData + Unmanaged,
    {
        _ = self.delete_component::<T>(record);
    }

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
        assert_eq!(T::data_type(), record.data_type);

        let index_of_deleting = record.array_index;
        let owner_id = self.component_slice::<T>()[record.array_index]
            .owner_id
            .clone();

        self.entities
            .get_mut(&owner_id)
            .unwrap()
            .components
            .retain(|item| item != record);

        let data = self.components[T::data_type().usize()]
            .take_at::<Component<T>>(index_of_deleting)
            .data;

        let len = self.component_slice::<T>().len();
        if index_of_deleting < len {
            let affected_entities = (&self.component_slice::<T>()[index_of_deleting..len])
                .iter()
                .map(|component| component.owner_id.clone())
                .collect::<Vec<EntityId>>();

            for id in affected_entities {
                let entity = self.entities.get_mut(&id).unwrap();
                entity
                    .components
                    .iter_mut()
                    .filter(|record| {
                        record.data_type == T::data_type() && record.array_index > index_of_deleting
                    })
                    .for_each(|record| record.array_index -= 1);
            }
        }
        data
    }

    pub fn delete_entity(&mut self, id: &EntityId, scripting: &Scripting) {
        let records = self.entities[id]
            .components
            .iter()
            .map(|item| item.clone())
            .collect::<Vec<ComponentRecord>>();

        for record in records {
            match record.data_type {
                ComponentDataType::Camera => self.delete_unmanaged_component::<Camera>(&record),
                ComponentDataType::LightSource => {
                    self.delete_unmanaged_component::<LightSource>(&record)
                }
                ComponentDataType::Mesh => self.delete_unmanaged_component::<Mesh>(&record),
                ComponentDataType::ScriptObject => {
                    self.delete_managed_component::<ScriptObject>(&record, scripting)
                }
                _ => unreachable!(),
            }
        }

        _ = self.entities.remove(&id).unwrap();
        self.free_ids.push_back(id.clone());
    }
}

#[derive(Debug)]
pub struct Entity {
    pub id: EntityId,
    pub name: String,
    pub components: Vec<ComponentRecord>,
    children: Vec<EntityId>,
    parent: Option<EntityId>,
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

#[derive(EnumCount, PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u32)]
enum ComponentDataType {
    Transform,
    Camera,
    LightSource,
    Mesh,
    ScriptObject,
}

impl ComponentDataType {
    fn usize(&self) -> usize {
        // todo!();
        *self as usize // TEST IF CONVERTING A REFERENCE
    }
}

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

pub struct Component<T: ComponentData> {
    owner_id: EntityId,
    pub data: T,
}

impl<T: ComponentData> Component<T> {
    pub fn new(owner_id: EntityId, data: T) -> Self {
        Self { owner_id, data }
    }

    pub fn owner_id(&self) -> &EntityId {
        &self.owner_id
    }
}
