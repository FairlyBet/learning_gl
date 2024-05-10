use crate::{
    camera::Camera,
    data3d::Mesh,
    lighting::LightSource,
    linear::Transform,
    resources::ResourceManager,
    runtime::FramebufferSizeCallback,
    scripting::{ScriptObject, Scripting},
    serializable,
    utils::{self, Reallocated, TypelessVec},
};
use fxhash::FxHashMap;
use std::{collections::VecDeque, ops::Mul};
use strum::EnumCount;

#[derive(Default, Debug)]
pub struct SceneManager {
    entities: FxHashMap<usize, Entity>,
    components: [TypelessVec; ComponentDataType::COUNT],
    available_ids: VecDeque<usize>,
    id_counter: usize,
    // mutated_transforms: Vec<usize>,
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
        let entities = scene.read_entities();
        self.load_entities(None, entities, resource_manager, scripting);
    }

    fn load_entities(
        &mut self,
        parent_id: Option<usize>,
        entities: Vec<serializable::Entity>,
        resource_manager: &mut ResourceManager,
        scripting: &Scripting,
    ) {
        for entity in entities {
            let id = self.create_entity(&scripting);

            let ent = self.entities.get_mut(&id).unwrap();
            ent.name = entity.name.clone();
            let transform_index = ent.transform_index();
            let transform: Transform = entity.transform.into();
            _ = self.tranforms_mut().rewrite(transform_index, transform);

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
                    .map(|item| resource_manager.mesh_manager_mut().get_mesh_lazily(&item))
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

            self.load_entities(Some(id), entity.children, resource_manager, scripting);
        }
    }

    pub fn create_entity(&mut self, scripting: &Scripting) -> usize {
        let mut rewrite = true;
        let instance_id = self.available_ids.pop_front().unwrap_or_else(|| {
            rewrite = false;
            let id = self.id_counter;
            self.id_counter = self.id_counter.checked_add(1).unwrap(); // panic if overflows
            id
        });

        let entity = Entity::new(instance_id);

        let transform = Transform::new();
        if rewrite {
            _ = self
                .tranforms_mut()
                .rewrite(entity.transform_index(), transform); // rewriting unused item
        } else {
            let reallocated = self.tranforms_mut().push(transform); // pushing new item
            if let Reallocated::Yes = reallocated {
                self.update_transform_parent_pointers();
            }
        }
        assert!(self.entities.insert(instance_id, entity).is_none());
        scripting.register_entity(instance_id);
        instance_id
    }

    pub fn set_parent(&mut self, child_id: usize, parent_id: Option<usize>) {
        match parent_id {
            Some(parent_id) => {
                if !self.entities[&parent_id].children.contains(&child_id) {
                    self.set_parent(child_id, None);

                    let parent = self.entities.get_mut(&parent_id).unwrap();
                    parent.children.push(child_id);
                    let parent_transform = parent.transform_index();
                    let parent_transform = self.get_transform(parent_transform) as *const _;

                    let child = self.entities.get_mut(&child_id).unwrap();
                    child.parent = Some(parent_id);
                    let child_transform = child.transform_index();
                    self.get_transform_mut(child_transform).parent = Some(parent_transform);
                }
            }
            None => {
                if let Some(parent_id) = self.entities[&child_id].parent {
                    let parent = self.entities.get_mut(&parent_id).unwrap();
                    let index = parent
                        .children
                        .iter()
                        .position(|item| *item == child_id)
                        .unwrap();
                    parent.children.remove(index);

                    let child = self.entities.get_mut(&child_id).unwrap();
                    child.parent = None;
                    let child_transform_index = child.transform_index();

                    self.get_transform_mut(child_transform_index).parent = None;
                }
            }
        }
    }

    /// Takes O(n), n - amount of entities.
    /// Only updates parent transform pointers
    fn update_transform_parent_pointers(&mut self) {
        let len = self.tranforms().len::<Transform>();
        for i in 0..len {
            if self.get_transform(i).parent.is_some() {
                let entity = self.entities.get(&i).unwrap();
                let parent_id = entity.parent.unwrap();
                let parent_transform = self.get_transform(parent_id) as *const _;
                self.get_transform_mut(i).parent = Some(parent_transform);
            }
        }
    }

    #[allow(private_bounds)]
    pub fn attach_components<T>(&mut self, target_id: usize, data: Vec<T>)
    where
        T: ComponentData,
    {
        for item in data {
            self.attach_component(target_id, item);
        }
    }

    #[allow(private_bounds)]
    pub fn attach_component<T>(&mut self, target_id: usize, data: T)
    where
        T: ComponentData,
    {
        let component_record = ComponentRecord {
            array_index: self.components_mut::<T>().len::<Component<T>>(),
            data_type: T::data_type(),
        };
        let target = self.entities.get_mut(&target_id).unwrap();
        target.components.push(component_record);

        let component = Component::new(target_id, data);
        _ = self.components_mut::<T>().push(component);
    }

    #[allow(private_bounds)]
    pub fn get_components<T>(&self, owner_id: usize) -> impl Iterator<Item = &ComponentRecord>
    where
        T: ComponentData,
    {
        self.entities[&owner_id]
            .components
            .iter()
            .filter(|record| record.data_type == T::data_type())
    }

    #[allow(private_bounds)]
    pub fn get_component<T>(&self, owner_id: usize) -> Option<&ComponentRecord>
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
    pub fn component_slice<T>(&self) -> &[Component<T>]
    where
        T: ComponentData,
    {
        self.components::<T>().slice()
    }

    fn component_slice_mut<T>(&mut self) -> &mut [Component<T>]
    where
        T: ComponentData,
    {
        self.components_mut::<T>().slice_mut()
    }

    fn components<T>(&self) -> &TypelessVec
    where
        T: ComponentData,
    {
        &self.components[T::data_type().usize()]
    }

    fn components_mut<T>(&mut self) -> &mut TypelessVec
    where
        T: ComponentData,
    {
        &mut self.components[T::data_type().usize()]
    }

    pub fn get_transform(&self, owner_id: usize) -> &Transform {
        self.tranforms().get(owner_id)
    }

    pub fn get_transform_mut(&mut self, owner_id: usize) -> &mut Transform {
        self.tranforms_mut().get_mut(owner_id)
    }

    fn tranforms(&self) -> &TypelessVec {
        &self.components[ComponentDataType::Transform.usize()]
    }

    fn tranforms_mut(&mut self) -> &mut TypelessVec {
        &mut self.components[ComponentDataType::Transform.usize()]
    }

    pub fn delete_entity(&mut self, target_id: usize, scripting: &Scripting) {
        let children = self.entities[&target_id].children.clone();
        children
            .iter()
            .for_each(|child| self.delete_entity(*child, scripting));

        let records = self.entities[&target_id]
            .components
            .iter()
            .map(|item| item.copy())
            .collect::<Vec<ComponentRecord>>();

        for record in records {
            match record.data_type {
                ComponentDataType::Camera => {
                    _ = self.delete_component::<Camera>(target_id, record);
                }
                ComponentDataType::LightSource => {
                    _ = self.delete_component::<LightSource>(target_id, record);
                }
                ComponentDataType::Mesh => _ = self.delete_component::<Mesh>(target_id, record),
                ComponentDataType::ScriptObject => {
                    let data = self.delete_component::<ScriptObject>(target_id, record);
                    Self::delete_managed_stuff(data);
                }
                ComponentDataType::Transform => unreachable!(),
            }
        }

        self.get_transform_mut(target_id).parent = None; // Important!

        scripting.expire_entity(target_id);

        _ = self.entities.remove(&target_id).unwrap();
        self.available_ids.push_back(target_id);
    }

    #[allow(private_bounds)]
    pub fn delete_unmanaged_component<T>(&mut self, target_id: usize, index: usize)
    where
        T: Unmanaged,
    {
        let opt = self.find_by_index::<T>(target_id, index);
        if let Some(record) = opt {
            _ = self.delete_component::<T>(target_id, record.copy());
        }
    }

    #[allow(private_bounds)]
    pub fn delete_managed_component<T>(
        &mut self,
        target_id: usize,
        index: usize,
        scripting: &Scripting,
    ) where
        T: Managed,
    {
        let opt = self.find_by_index::<T>(target_id, index);
        if let Some(record) = opt {
            let data = self.delete_component::<T>(target_id, record.copy());
            Self::delete_managed_stuff(data);
        }
    }

    fn delete_managed_stuff<T>(data: T)
    where
        T: Managed,
    {
        todo!()
    }

    fn find_by_index<T>(&self, target_id: usize, index: usize) -> Option<&ComponentRecord>
    where
        T: ComponentData,
    {
        self.entities[&target_id]
            .components
            .iter()
            .filter(|item| item.data_type == T::data_type())
            .nth(index)
    }

    fn delete_component<T>(&mut self, owner_id: usize, record: ComponentRecord) -> T
    where
        T: ComponentData,
    {
        let index_of_deleting = record.array_index;

        let owner = self.entities.get_mut(&owner_id).unwrap();
        let index = owner
            .components
            .iter()
            .position(|item| *item == record)
            .unwrap();
        owner.components.remove(index);

        let index_of_last = self.components::<T>().len::<Component<T>>() - 1;
        let owner_id_of_last = self.component_slice::<T>().last().unwrap().owner_id;
        let owner_of_last = self.entities.get_mut(&owner_id_of_last).unwrap();
        let index = owner_of_last
            .components
            .iter()
            .position(|item| item.data_type == T::data_type() && item.array_index == index_of_last)
            .unwrap();
        owner_of_last.components[index].array_index = index_of_deleting;

        self.components_mut::<T>()
            .swap_take::<Component<T>>(index_of_deleting)
            .data
    }
}

impl FramebufferSizeCallback for SceneManager {
    fn framebuffer_size(&mut self, size: (i32, i32)) {
        for camera in self.component_slice_mut::<Camera>() {
            camera.data.update_aspect(size);
        }
    }
}

#[derive(Debug)]
pub struct Entity {
    instance_id: usize,
    pub name: String,
    pub components: Vec<ComponentRecord>,
    children: Vec<usize>,
    parent: Option<usize>,
    // loaded_from_scene: u32
    // modified_transforms: Vec<usize>
}

impl Entity {
    fn new(instance_id: usize) -> Self {
        Self {
            instance_id,
            name: "".to_string(),
            components: vec![],
            children: vec![],
            parent: None,
        }
    }

    fn transform_index(&self) -> usize {
        self.instance_id
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

    fn copy(&self) -> Self {
        Self {
            array_index: self.array_index,
            data_type: self.data_type,
        }
    }
}

#[repr(u32)]
#[derive(EnumCount, PartialEq, Eq, Clone, Copy, Debug)]
pub enum ComponentDataType {
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

// Consider moving responsibility for Script managing to Scripting system
impl ComponentData for ScriptObject {
    fn data_type() -> ComponentDataType {
        ComponentDataType::ScriptObject
    }
}

trait Unmanaged: ComponentData {}

impl Unmanaged for Mesh {}
impl Unmanaged for Camera {}
impl Unmanaged for LightSource {}

trait Managed: ComponentData {}

impl Managed for ScriptObject {}

#[allow(private_bounds)]
pub struct Component<T: ComponentData> {
    owner_id: usize,
    pub data: T,
}

#[allow(private_bounds)]
impl<T: ComponentData> Component<T> {
    pub fn new(owner_id: usize, data: T) -> Self {
        Self { owner_id, data }
    }

    pub fn owner_id(&self) -> usize {
        self.owner_id
    }
}
