use fxhash::FxHasher32;
use rlua::{Error, Lua, RegistryKey, StdLib, Table};
use std::{collections::HashSet, fs, hash::BuildHasherDefault, path::Path};
use strum::EnumCount;

pub struct CompiledChunk(Vec<u8>);

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

    pub fn execute_updates() {

    }
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
