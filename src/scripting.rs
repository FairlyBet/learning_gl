use crate::entity_system::{EntityId, SceneChunk};
use fxhash::FxHasher32;
use rlua::{Context, Error, Lua, RegistryKey, StdLib, Table, Variadic};
use std::{collections::HashMap, fs, hash::BuildHasherDefault, str::FromStr, sync::Arc};
use strum::EnumCount;

pub struct CompiledChunk(Vec<u8>);

type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher32>>;

pub struct Scripting {
    lua: Lua,
    start_callbacks: RegistryKey,
    update_callbacks: RegistryKey,
}

impl Scripting {
    pub fn new() -> Result<Self, Error> {
        let lua = Lua::new_with(StdLib::ALL_NO_DEBUG);

        let (start_callbacks, update_callbacks) = lua.context(|context| {
            let table = context.create_table()?;
            let start_callbacks = context.create_registry_value(table)?;
            let table = context.create_table()?;
            let update_callbacks = context.create_registry_value(table)?;
            Ok((start_callbacks, update_callbacks))
        })?;

        Ok(Self {
            lua,
            start_callbacks,
            update_callbacks,
        })
    }

    pub fn compile_chunk(&self, src: &str, chunk_name: &str) -> Result<CompiledChunk, Error> {
        self.lua.context(|context| {
            let chunk = context.load(src);
            let chunk = chunk.set_name(chunk_name)?;
            let fucntion = chunk.into_function()?;
            let dumped = fucntion.dump()?;
            Ok(CompiledChunk(dumped))
        })
    }

    pub fn create_function(&self) {
        self.lua.context(|context| {
            context.create_function(
                |_, params: FxHashMap<String, usize>| -> rlua::Result<FxHashMap<String, f32>> {
                    let address = match params.get("address") {
                        Some(value) => *value,
                        None => {
                            return Err(Error::ExternalError(Arc::new(CustomError::new(
                                "Address parameter is missing",
                            ))))
                        }
                    };

                    todo!()
                    // Ok(())
                },
            );
        });
    }

    pub fn create_object(&mut self, chunk: &CompiledChunk) -> Result<ScriptObject, Error> {
        self.lua.context(|context| {
            let chunk = context.load(&chunk.0);
            unsafe {
                let function = chunk.into_function_allow_binary()?;
                let object: Table = function.call(())?;
                let has_start = object.contains_key(Callbacks::Start.name())?;
                if has_start {}
                let has_update = object.contains_key(Callbacks::Update.name())?;
                todo!()
            }
        })
    }

    pub fn modify_transform(&self, scene_chunk: &mut SceneChunk) {
        self.lua.context(|context| {
            
        });
    }

    pub fn execute_updates() {}
}

pub struct ScriptObject {
    key: RegistryKey,
}

#[derive(Clone, Copy, EnumCount)]
pub enum Callbacks {
    Start,
    Update,
}

impl Callbacks {
    const CALLBACK_NAMES: &'static [&'static str] = &["start", "update"];

    pub fn name(&self) -> &'static str {
        Self::CALLBACK_NAMES[*self as usize]
    }
}

pub fn execute_file(path: &str) {
    let src = fs::read_to_string(path).unwrap();
    let scr = Scripting::new().unwrap();
    scr.lua.context(|context| {
        let chunk = context.load(&src);
        chunk.exec();
    });
}

pub fn get_transform(context: Context, address: usize, id: EntityId) {
    let ptr = address as *const usize as *const SceneChunk;
    unsafe {
        let chunk = &(*ptr);
        chunk.get_transfom(id);
    }
}

pub fn chunk_ffi_access(ptr: u64) {
    let ptr = ptr as *mut u64 as *mut SceneChunk;
    unsafe {
        let chunk = &mut (*ptr);
        let entity = chunk.create_entity();
    }
}

pub struct CustomError {
    message: String,
}

impl CustomError {
    pub fn new(message: &str) -> Self {
        Self {
            message: String::from_str(message).unwrap(),
        }
    }
}

impl std::error::Error for CustomError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.message)
    }
}

impl std::fmt::Debug for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.message)
    }
}
