use fxhash::FxHashMap;
use glfw::fail_on_errors;
use std::{
    any::{self, TypeId},
    mem::ManuallyDrop, path::Path,
};

struct RawVec {
    vec: ManuallyDrop<Vec<()>>,
    drop: fn(&mut Self),
}

impl RawVec {
    fn new<T>() -> Self {
        let vec = ManuallyDrop::new(Vec::new());
        let drop = |self_: &mut Self| unsafe {
            let ptr = self_.vec.as_mut_ptr() as *mut T;
            let vec = Vec::from_raw_parts(ptr, self_.vec.len(), self_.vec.capacity());
            drop(vec);
        };
        Self { vec, drop }
    }

    fn cast<T>(&self) -> &Vec<T> {
        let ptr = &self.vec as *const _ as *const Vec<T>;
        unsafe { &*ptr }
    }

    fn cast_mut<T>(&mut self) -> &mut Vec<T> {
        let ptr = &mut self.vec as *mut _ as *mut Vec<T>;
        unsafe { &mut *ptr }
    }
}

impl Drop for RawVec {
    fn drop(&mut self) {
        (self.drop)(self);
    }
}

struct ComponentRegistry {
    components: FxHashMap<TypeId, RawVec>,
}

impl ComponentRegistry {
    fn new(builder: ComponentRegistryBuilder) -> Self {
        Self {
            components: builder.components,
        }
    }

    fn get_components<T>(&self) -> &Vec<T>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        self.components.get(&type_id).unwrap().cast::<T>()
    }

    fn get_components_mut<T>(&mut self) -> &mut Vec<T>
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        self.components.get_mut(&type_id).unwrap().cast_mut::<T>()
    }
}

struct ComponentRegistryBuilder {
    components: FxHashMap<TypeId, RawVec>,
    component_names: FxHashMap<TypeId, &'static str>,
}

impl ComponentRegistryBuilder {
    fn new() -> Self {
        Self {
            components: Default::default(),
            component_names: Default::default(),
        }
    }

    fn register_component<T>(&mut self)
    where
        T: 'static,
    {
        self.components
            .insert(TypeId::of::<T>(), RawVec::new::<T>());
        self.component_names
            .insert(TypeId::of::<T>(), any::type_name::<T>());
    }
}

struct Entity {
    id: EntityId,
    is_alive: bool,
}

struct EntityId {
    id: u32,
    gen: u32,
}

struct Engine;

impl Engine {
    fn run_ogl() {
        loop {}
    }
}

struct ResourceManager {}

struct Scene {
    path: Box<Path>,
}

struct SceneRunner {}

impl SceneRunner {
    fn play(scene: Scene) {
        loop {
            
        }
    }
}
