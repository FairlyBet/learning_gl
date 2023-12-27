use rlua::{Error, Lua, StdLib};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

pub type CompiledChunk = Vec<u8>;

pub struct Script;

pub struct Scripting {
    pub lua: Lua,
}

impl Scripting {
    pub fn new() -> Self {
        Self {
            lua: Lua::new_with(StdLib::ALL_NO_DEBUG),
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

        let res = self.lua.context(|context| {
            let chunk = context.load(&src);
            chunk.exec()
        });

        Ok(res)
    }
}
