use crate::{
    entity_system::{EntityId, SceneChunk},
    runtime::WindowEvents,
};
use glfw::{Action, Modifiers, Window};
use rlua::{
    Context, Error, Function, LightUserData, Lua, RegistryKey, Result, StdLib, Table, Value,
};
use std::{ffi::c_void, fs, sync::Arc};

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

    pub fn create_wrappers(
        &self,
        scene_chunk: &mut SceneChunk,
        events: &WindowEvents,
        window: &Window,
    ) {
        self.lua.context(|context| {
            // let bind_ptr = "
            // return function(ptr, func)
            //     return function(args)
            //         func(ptr, args)
            //     end
            // end";

            // let set_address = context.load(bind_ptr).eval::<Function>().unwrap();

            let scene_chunk = LightUserData(scene_chunk as *const _ as *mut c_void);
            let window_events = LightUserData(events as *const _ as *mut c_void);
            let window = LightUserData(window as *const _ as *mut c_void);

            let transform_move = context
                .create_function(TransformWrappers::transform_move)
                .unwrap()
                .bind(scene_chunk.clone())
                .unwrap();

            let get_key = context
                .create_function(InputWrappers::get_key)
                .unwrap()
                .bind(window_events.clone())
                .unwrap();

            let get_key_holded = context
                .create_function(InputWrappers::get_key_holded)
                .unwrap()
                .bind(window.clone())
                .unwrap();

            let input = context.create_table().unwrap();
            input.set("getKey", get_key);
            input.set("getKeyHolded", get_key_holded);
            context.globals().set("Input", input);
        });
    }
}

struct TransformWrappers;

impl TransformWrappers {
    fn transform_move(_: Context, args: (LightUserData, EntityId, Table)) -> Result<()> {
        let x: f32 = match args.2.get("x") {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        let y: f32 = match args.2.get("y") {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        let z: f32 = match args.2.get("z") {
            Ok(value) => value,
            Err(err) => return Err(err),
        };
        let id = args.1;

        let chunk = unsafe { &mut *(args.0 .0 as *mut SceneChunk) }; // seems quite unsafe
        chunk.get_transfom_mut(id).move_(&glm::vec3(x, y, z));

        Ok(())
    }
}

struct InputWrappers;

impl InputWrappers {
    fn get_key(_: Context, args: (LightUserData, Function, Function, Value)) -> Result<bool> {
        let key = args.1.call::<_, i32>(())?;
        let action = args.2.call::<_, i32>(())?;
        let modifiers = match args.3 {
            Value::Nil => 0,
            Value::Integer(int) => int as i32,
            _ => {
                return Err(Error::ExternalError(Arc::new(CustomError(
                    "Invalid argument type [modifiers]".to_string(),
                ))))
            }
        };

        let key = match WindowEvents::key_from_i32(key) {
            Some(value) => value,
            None => {
                return Err(Error::ExternalError(Arc::new(CustomError(
                    "Invalid key code".to_string(),
                ))))
            }
        };
        let action = match WindowEvents::action_from_i32(action) {
            Some(value) => value,
            None => {
                return Err(Error::ExternalError(Arc::new(
                    (CustomError("Invalid action code".to_string())),
                )))
            }
        };
        let modifiers = match Modifiers::from_bits(modifiers) {
            Some(value) => value,
            None => {
                return Err(Error::ExternalError(Arc::new(
                    (CustomError("Invalid modifiers".to_string())),
                )))
            }
        };

        let events = unsafe { &*(args.0 .0 as *const WindowEvents) };
        Ok(events.get_key((key, action, modifiers)))
    }

    fn get_key_holded(_: Context, args: (LightUserData, Function)) -> Result<bool> {
        // Test if works with window recreation

        let key = args.1.call::<_, i32>(())?;
        let key = match WindowEvents::key_from_i32(key) {
            Some(value) => value,
            None => {
                return Err(Error::ExternalError(Arc::new(CustomError(
                    "Invalid key code".to_string(),
                ))))
            }
        };
        let window = unsafe { &*(args.0 .0 as *const Window) };
        Ok(window.get_key(key) == Action::Press)
    }

    fn create_keys_table(context: &Context) {
        let mut s = String::from("Keys = {}\n");
        for key in WindowEvents::KEY_VALUES {
            s.push_str(&format!(
                "function Keys.{}() return {} end\n",
                WindowEvents::key_to_str(key),
                key as i32
            ));
        }
        println!("{}", s);
        // let chunk = context.load(&s);
        // chunk.exec();
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

#[derive(Debug)]
pub struct CustomError(String);

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
        write!(f, "{}", self.0)
    }
}
