use crate::{
    camera::Camera,
    data_3d::Mesh,
    lighting::LightSource,
    linear::Transform,
    resources::ResourceManager,
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
    instance_counter: usize,
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
        let entities = scene.load_entities();
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
            let id = self.create_entity();

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

            self.load_entities(Some(id), entity.children, resource_manager, scripting);
        }
    }

    pub fn create_entity(&mut self) -> usize {
        let mut rewrite = true;
        let instance_id = self.available_ids.pop_front().unwrap_or_else(|| {
            rewrite = false;
            let id = self.instance_counter;
            self.instance_counter = self.instance_counter.checked_add(1).unwrap(); // panic if overflows
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
        let entity = self.entities.get_mut(&target_id).unwrap();
        entity.components.push(component_record);

        let component = Component::new(target_id, data);
        self.components_mut::<T>().push(component);
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
        self.components[T::data_type().usize()].slice()
    }

    fn component_slice_mut<T>(&mut self) -> &mut [Component<T>]
    where
        T: ComponentData,
    {
        self.components[T::data_type().usize()].slice_mut()
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

    #[allow(private_bounds)]
    pub fn delete_unmanaged_component<T>(&mut self, target_id: usize, index: usize)
    where
        T: ComponentData + Unmanaged,
    {
        let f = self.entities[&target_id]
            .components
            .iter()
            .filter(|item| item.data_type == T::data_type());
    }

    #[allow(private_bounds)]
    pub fn delete_managed_component<T>(&mut self, index: usize, scripting: &Scripting)
    where
        T: ComponentData + Managed,
    {
        match T::data_type() {
            ComponentDataType::ScriptObject => {}
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
        let owner_id = self.component_slice::<T>()[record.array_index].owner_id;

        self.entities
            .get_mut(&owner_id)
            .unwrap()
            .components
            .retain(|item| item != record);

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
        todo!()
    }

    pub fn delete_entity(&mut self, target_id: usize, scripting: &Scripting) {
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

    fn clone(&self) -> Self {
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
