use fxhash::FxHasher32;
use rlua::{Error, Lua, RegistryKey, StdLib, Table};
use std::{collections::HashSet, fs, hash::BuildHasherDefault, path::Path};
use strum::EnumCount;

pub type CompiledChunk = Vec<u8>;

type FxHashSet32<T> = HashSet<T, BuildHasherDefault<FxHasher32>>;

pub struct Scripting {
    lua: Lua,
    updates: FxHashSet32<u32>,
    update_index: u32,
}

impl Scripting {
    pub fn new() -> Self {
        Self {
            lua: Lua::new_with(StdLib::ALL_NO_DEBUG),
            updates: Default::default(),
            update_index: 0,
        }
    }

    pub fn compile_file(&self, path: &Path) -> CompiledChunk {
        let src = fs::read_to_string(path).unwrap();
        self.lua.context(|context| {
            context
                .load(&src)
                .set_name(path.to_str().unwrap())
                .unwrap()
                .into_function()
                .unwrap()
                .dump()
                .unwrap()
        })
    }

    pub fn execute_file(&self, path: &Path) -> Result<Result<(), Error>, String> {
        let src = match fs::read_to_string(path) {
            Ok(value) => value,
            Err(err) => return Err(err.to_string()),
        };
        let res = self.lua.context(|context| context.load(&src).exec());
        Ok(res)
    }

    pub fn create_object(&mut self, src: &str) -> Result<Script, Error> {
        self.lua.context(|context| {
            let chunk = context.load(src);
            let res = chunk.eval::<Table>();
            match res {
                Ok(table) => {
                    let res = Ok(Script {
                        index: self.update_index,
                        key: context.create_registry_value(table).unwrap(),
                    });
                    self.updates.insert(self.update_index);
                    self.update_index += 1;
                    res
                }
                Err(error) => Err(error),
            }
        })
    }
}

pub struct Script {
    index: u32,
    key: RegistryKey,
}

#[derive(Clone, Copy, EnumCount)]
pub enum Callbacks {
    Start,
    Update,
}

impl Callbacks {
    const CALLBACK_NAMES: &'static [&'static str] = &["start", "update"];

    pub fn callback_name(&self) -> &'static str {
        Self::CALLBACK_NAMES[*self as usize]
    }
}
