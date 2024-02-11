use crate::entity_system::{EntityId, SceneChunk};
use fxhash::{FxHasher, FxHasher32};
use rlua::{Context, Error, Function, Lua, RegistryKey, Result, StdLib, Table, Variadic};
use std::{collections::HashMap, fs, hash::BuildHasherDefault, io::Write, str::FromStr, sync::Arc};
use strum::EnumCount;

pub struct CompiledChunk(Vec<u8>);

pub struct Scripting {
    lua: Lua,
}

impl Scripting {
    pub fn new() -> Self {
        let lua = Lua::new_with(StdLib::ALL_NO_DEBUG);
        Self { lua }
    }

    pub fn compile_chunk(&self, src: &str, chunk_name: &str) -> Result<CompiledChunk> {
        self.lua.context(|context| {
            let chunk = context.load(src);
            let chunk = chunk.set_name(chunk_name)?;
            let fucntion = chunk.into_function()?;
            let dumped = fucntion.dump()?;
            Ok(CompiledChunk(dumped))
        })
    }

    pub fn create_registry_object(&self, chunk: &CompiledChunk) -> Result<RegistryObject> {
        self.lua.context(|context| {
            let dumped = context.load(&chunk.0);
            let function = unsafe { dumped.into_function_allow_binary() }?;
            let object = function.call::<(), Table>(())?;
            let key = context.create_registry_value(object)?;
            Ok(RegistryObject { key })
        })
    }

    pub fn delete_registry_object(&self, object: RegistryObject) {
        self.lua.context(|context| {
            context.remove_registry_value(object.key);
        });
    }

    pub fn create_wrappers(&self, scene_chunk: &SceneChunk) {
        self.lua.context(|context| {
            let set_address = "
            return function(address, func)
                return function(args)
                    func(address, args)
                end
            end";
            let set_address = context.load(set_address).eval::<Function>().unwrap();

            let address = scene_chunk as *const _ as usize;

            let transform_move = context.create_function(Wrappers::transform_move).unwrap();
            let transform_move = set_address
                .call::<_, Function>((address, transform_move))
                .unwrap();
            let arg = context.create_table().unwrap();
            println!("{}", usize::MAX);
            transform_move.call::<_, ()>(arg);

            // transform_move.call::<_, ()>((10, 20));
            // let transform_move = set_address
            //     .call::<_, Function>((address, transform_move))
            //     .unwrap();

            // let wrapper_table = context.create_table().unwrap();
            // wrapper_table.set("transform_move", transform_move);
            // wrapper_table
            //     .get::<_, Function>("transform_move")
            //     .unwrap()
            //     .call::<_, ()>(());
        });
    }
}

struct Wrappers;

impl Wrappers {
    fn transform_move(_: Context, arg: (usize, Table)) -> Result<()> {
        // let address = arg.0 as usize; // direct conversion from Lua doesn't work with MAX value for some reason
        println!("{}", arg.0);
        // println!("{}", arg.1);
        // fs::File::create("transform move.txt").unwrap();

        // print!("rust transform move {}", address);

        // let id: EntityId = arg.get("id").unwrap();
        // let x: f32 = arg.get("x").unwrap();
        // let y: f32 = arg.get("y").unwrap();
        // let z: f32 = arg.get("z").unwrap();

        // let scn = unsafe { &mut *(address as *mut SceneChunk) };
        // scn.get_transfom_mut(id).move_(&glm::vec3(x, y, z));

        Ok(())
    }
}

pub fn execute_file(path: &str) {
    let src = fs::read_to_string(path).unwrap();
    let scr = Scripting::new();
    scr.lua.context(|context| {
        let chunk = context.load(&src);
        chunk.exec();
    });
}

pub struct RegistryObject {
    key: RegistryKey,
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
