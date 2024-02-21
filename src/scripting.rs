use crate::{
    entity_system::{EntityId, SceneManager},
    runtime::WindowEvents,
};
use glfw::{Action, Key, Modifiers, Window};
use nalgebra_glm::Vec3;
use rlua::{
    Context, Error, Function, LightUserData, Lua, RegistryKey, Result, StdLib, Table, Value,
};
use std::{ffi::c_void, fs, sync::Arc};

pub struct CompiledChunk(Vec<u8>);

pub struct Script {
    chunk: CompiledChunk,
}

pub struct RegistryObject {
    key: RegistryKey,
}

pub struct Scripting {
    pub lua: Lua,
    object_table_key: RegistryKey,
}

impl Scripting {
    pub fn new() -> Self {
        let lua = Lua::new_with(StdLib::ALL_NO_DEBUG);
        let object_table_key = lua.context(|context| {
            let table = context.create_table().unwrap();
            context.create_registry_value(table).unwrap()
        });
        Self {
            lua,
            object_table_key,
        }
    }

    pub fn register_object_id<'lua>(
        &self,
        context: &Context<'lua>,
        object: Table<'lua>, /* Probably replace with Value*/
        id: EntityId,
    ) {
        let object_table = context
            .registry_value::<Table>(&self.object_table_key)
            .unwrap();
        object_table.set(object, id).unwrap();
    }

    pub fn compile_chunk(&self, src: &str, chunk_name: &str) -> Result<CompiledChunk> {
        self.lua.context(|context| {
            let chunk = context.load(src);
            let chunk = chunk.set_name(chunk_name)?;
            let function = chunk.into_function()?;
            let dumped = function.dump()?;
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
            context.remove_registry_value(object.key).unwrap();
        });
    }

    pub fn create_wrappers(
        &self,
        scene_manager: &SceneManager,
        events: &WindowEvents,
        window: &Window,
        frame_time: &f64,
    ) {
        self.lua.context(|context| {
            let scene_manager   = LightUserData(scene_manager             as *const _ as *mut c_void);
            let window_events   = LightUserData(events                    as *const _ as *mut c_void);
            let window          = LightUserData(window                    as *const _ as *mut c_void);
            let frame_time      = LightUserData(frame_time                as *const _ as *mut c_void);
            let object_table    = LightUserData(&self.object_table_key    as *const _ as *mut c_void);

            let transform_move = context
                .create_function(TransformWrappers::transform_move)
                .unwrap()
                .bind((object_table, scene_manager))
                .unwrap();
            let transform_move_local = context
                .create_function(TransformWrappers::transform_move_local)
                .unwrap()
                .bind((object_table, scene_manager))
                .unwrap();
            let transform_get_position = context
                .create_function(TransformWrappers::transform_get_position)
                .unwrap()
                .bind((object_table, scene_manager))
                .unwrap();

            let transform = context.create_table().unwrap();
            transform.set("move", transform_move).unwrap();
            transform.set("moveLocal", transform_move_local).unwrap();
            transform
                .set("getPosition", transform_get_position)
                .unwrap();
            context.globals().set("Transform", transform).unwrap();

            let get_key = context
                .create_function(InputWrappers::get_key)
                .unwrap()
                .bind(window_events)
                .unwrap();
            let get_key_held = context
                .create_function(InputWrappers::get_key_held)
                .unwrap()
                .bind(window)
                .unwrap();
            let input = context.create_table().unwrap();
            input.set("getKey", get_key).unwrap();
            input.set("getKeyHeld", get_key_held).unwrap();
            context.globals().set("Input", input).unwrap();

            let frame_time = context
                .create_function(ApplicationWrappers::frame_time)
                .unwrap()
                .bind(frame_time)
                .unwrap();
            context.globals().set("frameTime", frame_time).unwrap();

            InputWrappers::create_keys_table(&context);
        });
    }

    pub fn execute_chunk(&self, chunk: &CompiledChunk) {
        self.lua.context(|context| unsafe {
            context
                .load(&chunk.0)
                .into_function_allow_binary()
                .unwrap()
                .call::<_, ()>(())
                .unwrap();
        });
    }
}

struct TransformWrappers;

impl TransformWrappers {
    fn transform_move<'lua>(
        context: Context<'lua>,
        args: (LightUserData, LightUserData, Table<'lua>, Table<'lua>),
    ) -> Result<()> {
        let object_table = unsafe {
            context
                .registry_value::<Table>(&*(args.0 .0 as *const RegistryKey))
                .unwrap()
        };
        let scene_manager = unsafe { &mut *(args.1 .0 as *mut SceneManager) }; // seems quite unsafe
        let object = args.2;
        let vector_lua = args.3;
        let vector = vec_from_lua(&vector_lua)?;
        let id = object_table.get::<_, EntityId>(object)?;

        scene_manager.get_transform_mute(id).move_(&vector);

        Ok(())
    }

    fn transform_move_local<'lua>(
        context: Context<'lua>,
        args: (LightUserData, LightUserData, Table<'lua>, Table<'lua>),
    ) -> Result<()> {
        let object_table = unsafe {
            context
                .registry_value::<Table>(&*(args.0 .0 as *const RegistryKey))
                .unwrap()
        };
        let scene_manager = unsafe { &mut *(args.1 .0 as *mut SceneManager) }; // seems quite unsafe
        let object = args.2;
        let id = object_table.get::<_, EntityId>(object)?;
        let vector_lua = args.3;
        let vector = vec_from_lua(&vector_lua)?;

        scene_manager.get_transform_mute(id).move_local(&vector);

        Ok(())
    }

    fn transform_get_position<'lua>(
        context: Context<'lua>,
        args: (LightUserData, LightUserData, Table<'lua>),
    ) -> Result<Table<'lua>> {
        let object_table = unsafe {
            context
                .registry_value::<Table>(&*(args.0 .0 as *const RegistryKey))
                .unwrap()
        };
        let scene_manager = unsafe { &mut *(args.1 .0 as *mut SceneManager) }; // seems quite unsafe
        let object = args.2;
        let id = object_table.get::<_, EntityId>(object)?;
        let position = &scene_manager.get_transform(id).position;
        let vector_lua = vec_to_lua(&context, position);

        Ok(vector_lua)
    }
}

struct InputWrappers;

impl InputWrappers {
    pub const KEY_VALUES: [Key; 121] = [
        Key::Space,
        Key::Apostrophe,
        Key::Comma,
        Key::Minus,
        Key::Period,
        Key::Slash,
        Key::Num0,
        Key::Num1,
        Key::Num2,
        Key::Num3,
        Key::Num4,
        Key::Num5,
        Key::Num6,
        Key::Num7,
        Key::Num8,
        Key::Num9,
        Key::Semicolon,
        Key::Equal,
        Key::A,
        Key::B,
        Key::C,
        Key::D,
        Key::E,
        Key::F,
        Key::G,
        Key::H,
        Key::I,
        Key::J,
        Key::K,
        Key::L,
        Key::M,
        Key::N,
        Key::O,
        Key::P,
        Key::Q,
        Key::R,
        Key::S,
        Key::T,
        Key::U,
        Key::V,
        Key::W,
        Key::X,
        Key::Y,
        Key::Z,
        Key::LeftBracket,
        Key::Backslash,
        Key::RightBracket,
        Key::GraveAccent,
        Key::World1,
        Key::World2,
        Key::Escape,
        Key::Enter,
        Key::Tab,
        Key::Backspace,
        Key::Insert,
        Key::Delete,
        Key::Right,
        Key::Left,
        Key::Down,
        Key::Up,
        Key::PageUp,
        Key::PageDown,
        Key::Home,
        Key::End,
        Key::CapsLock,
        Key::ScrollLock,
        Key::NumLock,
        Key::PrintScreen,
        Key::Pause,
        Key::F1,
        Key::F2,
        Key::F3,
        Key::F4,
        Key::F5,
        Key::F6,
        Key::F7,
        Key::F8,
        Key::F9,
        Key::F10,
        Key::F11,
        Key::F12,
        Key::F13,
        Key::F14,
        Key::F15,
        Key::F16,
        Key::F17,
        Key::F18,
        Key::F19,
        Key::F20,
        Key::F21,
        Key::F22,
        Key::F23,
        Key::F24,
        Key::F25,
        Key::Kp0,
        Key::Kp1,
        Key::Kp2,
        Key::Kp3,
        Key::Kp4,
        Key::Kp5,
        Key::Kp6,
        Key::Kp7,
        Key::Kp8,
        Key::Kp9,
        Key::KpDecimal,
        Key::KpDivide,
        Key::KpMultiply,
        Key::KpSubtract,
        Key::KpAdd,
        Key::KpEnter,
        Key::KpEqual,
        Key::LeftShift,
        Key::LeftControl,
        Key::LeftAlt,
        Key::LeftSuper,
        Key::RightShift,
        Key::RightControl,
        Key::RightAlt,
        Key::RightSuper,
        Key::Menu,
        Key::Unknown,
    ];

    pub fn key_to_str(key: Key) -> &'static str {
        match key {
            Key::Space => "Space",
            Key::Apostrophe => "Apostrophe",
            Key::Comma => "Comma",
            Key::Minus => "Minus",
            Key::Period => "Period",
            Key::Slash => "Slash",
            Key::Num0 => "Num0",
            Key::Num1 => "Num1",
            Key::Num2 => "Num2",
            Key::Num3 => "Num3",
            Key::Num4 => "Num4",
            Key::Num5 => "Num5",
            Key::Num6 => "Num6",
            Key::Num7 => "Num7",
            Key::Num8 => "Num8",
            Key::Num9 => "Num9",
            Key::Semicolon => "Semicolon",
            Key::Equal => "Equal",
            Key::A => "A",
            Key::B => "B",
            Key::C => "C",
            Key::D => "D",
            Key::E => "E",
            Key::F => "F",
            Key::G => "G",
            Key::H => "H",
            Key::I => "I",
            Key::J => "J",
            Key::K => "K",
            Key::L => "L",
            Key::M => "M",
            Key::N => "N",
            Key::O => "O",
            Key::P => "P",
            Key::Q => "Q",
            Key::R => "R",
            Key::S => "S",
            Key::T => "T",
            Key::U => "U",
            Key::V => "V",
            Key::W => "W",
            Key::X => "X",
            Key::Y => "Y",
            Key::Z => "Z",
            Key::LeftBracket => "LeftBracket",
            Key::Backslash => "Backslash",
            Key::RightBracket => "RightBracket",
            Key::GraveAccent => "GraveAccent",
            Key::World1 => "World1",
            Key::World2 => "World2",
            Key::Escape => "Escape",
            Key::Enter => "Enter",
            Key::Tab => "Tab",
            Key::Backspace => "Backspace",
            Key::Insert => "Insert",
            Key::Delete => "Delete",
            Key::Right => "Right",
            Key::Left => "Left",
            Key::Down => "Down",
            Key::Up => "Up",
            Key::PageUp => "PageUp",
            Key::PageDown => "PageDown",
            Key::Home => "Home",
            Key::End => "End",
            Key::CapsLock => "CapsLock",
            Key::ScrollLock => "ScrollLock",
            Key::NumLock => "NumLock",
            Key::PrintScreen => "PrintScreen",
            Key::Pause => "Pause",
            Key::F1 => "F1",
            Key::F2 => "F2",
            Key::F3 => "F3",
            Key::F4 => "F4",
            Key::F5 => "F5",
            Key::F6 => "F6",
            Key::F7 => "F7",
            Key::F8 => "F8",
            Key::F9 => "F9",
            Key::F10 => "F10",
            Key::F11 => "F11",
            Key::F12 => "F12",
            Key::F13 => "F13",
            Key::F14 => "F14",
            Key::F15 => "F15",
            Key::F16 => "F16",
            Key::F17 => "F17",
            Key::F18 => "F18",
            Key::F19 => "F19",
            Key::F20 => "F20",
            Key::F21 => "F21",
            Key::F22 => "F22",
            Key::F23 => "F23",
            Key::F24 => "F24",
            Key::F25 => "F25",
            Key::Kp0 => "Kp0",
            Key::Kp1 => "Kp1",
            Key::Kp2 => "Kp2",
            Key::Kp3 => "Kp3",
            Key::Kp4 => "Kp4",
            Key::Kp5 => "Kp5",
            Key::Kp6 => "Kp6",
            Key::Kp7 => "Kp7",
            Key::Kp8 => "Kp8",
            Key::Kp9 => "Kp9",
            Key::KpDecimal => "KpDecimal",
            Key::KpDivide => "KpDivide",
            Key::KpMultiply => "KpMultiply",
            Key::KpSubtract => "KpSubtract",
            Key::KpAdd => "KpAdd",
            Key::KpEnter => "KpEnter",
            Key::KpEqual => "KpEqual",
            Key::LeftShift => "LeftShift",
            Key::LeftControl => "LeftControl",
            Key::LeftAlt => "LeftAlt",
            Key::LeftSuper => "LeftSuper",
            Key::RightShift => "RightShift",
            Key::RightControl => "RightControl",
            Key::RightAlt => "RightAlt",
            Key::RightSuper => "RightSuper",
            Key::Menu => "Menu",
            Key::Unknown => "Unknown",
        }
    }

    pub fn key_from_i32(value: i32) -> Result<Key> {
        match value {
            32 => Ok(Key::Space),
            39 => Ok(Key::Apostrophe),
            44 => Ok(Key::Comma),
            45 => Ok(Key::Minus),
            46 => Ok(Key::Period),
            47 => Ok(Key::Slash),
            48 => Ok(Key::Num0),
            49 => Ok(Key::Num1),
            50 => Ok(Key::Num2),
            51 => Ok(Key::Num3),
            52 => Ok(Key::Num4),
            53 => Ok(Key::Num5),
            54 => Ok(Key::Num6),
            55 => Ok(Key::Num7),
            56 => Ok(Key::Num8),
            57 => Ok(Key::Num9),
            59 => Ok(Key::Semicolon),
            61 => Ok(Key::Equal),
            65 => Ok(Key::A),
            66 => Ok(Key::B),
            67 => Ok(Key::C),
            68 => Ok(Key::D),
            69 => Ok(Key::E),
            70 => Ok(Key::F),
            71 => Ok(Key::G),
            72 => Ok(Key::H),
            73 => Ok(Key::I),
            74 => Ok(Key::J),
            75 => Ok(Key::K),
            76 => Ok(Key::L),
            77 => Ok(Key::M),
            78 => Ok(Key::N),
            79 => Ok(Key::O),
            80 => Ok(Key::P),
            81 => Ok(Key::Q),
            82 => Ok(Key::R),
            83 => Ok(Key::S),
            84 => Ok(Key::T),
            85 => Ok(Key::U),
            86 => Ok(Key::V),
            87 => Ok(Key::W),
            88 => Ok(Key::X),
            89 => Ok(Key::Y),
            90 => Ok(Key::Z),
            91 => Ok(Key::LeftBracket),
            92 => Ok(Key::Backslash),
            93 => Ok(Key::RightBracket),
            96 => Ok(Key::GraveAccent),
            161 => Ok(Key::World1),
            162 => Ok(Key::World2),
            256 => Ok(Key::Escape),
            257 => Ok(Key::Enter),
            258 => Ok(Key::Tab),
            259 => Ok(Key::Backspace),
            260 => Ok(Key::Insert),
            261 => Ok(Key::Delete),
            262 => Ok(Key::Right),
            263 => Ok(Key::Left),
            264 => Ok(Key::Down),
            265 => Ok(Key::Up),
            266 => Ok(Key::PageUp),
            267 => Ok(Key::PageDown),
            268 => Ok(Key::Home),
            269 => Ok(Key::End),
            280 => Ok(Key::CapsLock),
            281 => Ok(Key::ScrollLock),
            282 => Ok(Key::NumLock),
            283 => Ok(Key::PrintScreen),
            284 => Ok(Key::Pause),
            290 => Ok(Key::F1),
            291 => Ok(Key::F2),
            292 => Ok(Key::F3),
            293 => Ok(Key::F4),
            294 => Ok(Key::F5),
            295 => Ok(Key::F6),
            296 => Ok(Key::F7),
            297 => Ok(Key::F8),
            298 => Ok(Key::F9),
            299 => Ok(Key::F10),
            300 => Ok(Key::F11),
            301 => Ok(Key::F12),
            302 => Ok(Key::F13),
            303 => Ok(Key::F14),
            304 => Ok(Key::F15),
            305 => Ok(Key::F16),
            306 => Ok(Key::F17),
            307 => Ok(Key::F18),
            308 => Ok(Key::F19),
            309 => Ok(Key::F20),
            310 => Ok(Key::F21),
            311 => Ok(Key::F22),
            312 => Ok(Key::F23),
            313 => Ok(Key::F24),
            314 => Ok(Key::F25),
            320 => Ok(Key::Kp0),
            321 => Ok(Key::Kp1),
            322 => Ok(Key::Kp2),
            323 => Ok(Key::Kp3),
            324 => Ok(Key::Kp4),
            325 => Ok(Key::Kp5),
            326 => Ok(Key::Kp6),
            327 => Ok(Key::Kp7),
            328 => Ok(Key::Kp8),
            329 => Ok(Key::Kp9),
            330 => Ok(Key::KpDecimal),
            331 => Ok(Key::KpDivide),
            332 => Ok(Key::KpMultiply),
            333 => Ok(Key::KpSubtract),
            334 => Ok(Key::KpAdd),
            335 => Ok(Key::KpEnter),
            336 => Ok(Key::KpEqual),
            340 => Ok(Key::LeftShift),
            341 => Ok(Key::LeftControl),
            342 => Ok(Key::LeftAlt),
            343 => Ok(Key::LeftSuper),
            344 => Ok(Key::RightShift),
            345 => Ok(Key::RightControl),
            346 => Ok(Key::RightAlt),
            347 => Ok(Key::RightSuper),
            348 => Ok(Key::Menu),
            -1 => Ok(Key::Unknown),
            _ => Err(Error::ExternalError(Arc::new(CustomError(
                "Invalid key code".to_string(),
            )))),
        }
    }

    pub fn action_from_i32(value: i32) -> Result<Action> {
        match value {
            0 => Ok(Action::Release),
            1 => Ok(Action::Press),
            _ => Err(Error::ExternalError(Arc::new(CustomError(
                "Invalid action code".to_string(),
            )))),
        }
    }

    pub fn value_to_modifiers(value: Value) -> Result<Modifiers> {
        match value {
            Value::Nil => Ok(Modifiers::empty()),
            Value::Integer(int) => match Modifiers::from_bits(int as i32) {
                Some(val) => Ok(val),
                None => Err(Error::ExternalError(Arc::new(CustomError(
                    "Invalid argument value [modifiers]".to_string(),
                )))),
            },
            _ => Err(Error::ExternalError(Arc::new(CustomError(
                "Invalid argument type [modifiers]".to_string(),
            )))),
        }
    }

    fn get_key(_: Context, args: (LightUserData, Function, Function, Value)) -> Result<bool> {
        let key = args.1.call::<_, i32>(())?;
        let action = args.2.call::<_, i32>(())?;

        let key = Self::key_from_i32(key)?;
        let action = Self::action_from_i32(action)?;
        let modifiers = Self::value_to_modifiers(args.3)?;

        let events = unsafe { &*(args.0 .0 as *const WindowEvents) };
        Ok(events.get_key((key, action, modifiers)))
    }

    fn get_key_held(_: Context, args: (LightUserData, Function)) -> Result<bool> {
        // Test if works with window recreation

        let key = args.1.call::<_, i32>(())?;
        let key = Self::key_from_i32(key)?;
        let window = unsafe { &*(args.0 .0 as *const Window) };
        Ok(window.get_key(key) == Action::Press)
    }

    fn create_keys_table(context: &Context) {
        let mut s = format!(
            "
Actions = {{}}
function Actions.Release() return {} end
function Actions.Press() return {} end

Modifiers = {{}}
function Modifiers.Shift() return {} end
function Modifiers.Control() return {} end
function Modifiers.Alt() return {} end
function Modifiers.Super() return {} end
function Modifiers.CapsLock() return {} end
function Modifiers.NumLock() return {} end

Keys = {{}}\n",
            Action::Release as i32,
            Action::Press as i32,
            Modifiers::Shift.bits(),
            Modifiers::Control.bits(),
            Modifiers::Alt.bits(),
            Modifiers::Super.bits(),
            Modifiers::CapsLock.bits(),
            Modifiers::NumLock.bits()
        );
        for key in Self::KEY_VALUES {
            s.push_str(&format!(
                "function Keys.{}() return {} end\n",
                Self::key_to_str(key),
                key as i32
            ));
        }
        // println!("{}", s);
        let chunk = context.load(&s);
        chunk.exec().unwrap();
    }
}

struct ApplicationWrappers;

impl ApplicationWrappers {
    fn frame_time(_: Context, args: LightUserData) -> Result<f64> {
        unsafe { Ok(*(args.0 as *const f64)) }
    }
}

pub fn execute_file(path: &str) {
    let src = fs::read_to_string(path).unwrap();
    let scr = Scripting::new();
    scr.lua.context(|context| {
        let chunk = context.load(&src);
        chunk.exec().unwrap();
    });
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

fn vec_to_lua<'lua>(context: &Context<'lua>, vector: &Vec3) -> Table<'lua> {
    let vector_lua = context.create_table().unwrap();
    vector_lua.set("x", vector.x).unwrap();
    vector_lua.set("y", vector.y).unwrap();
    vector_lua.set("z", vector.z).unwrap();
    match context.globals().get::<_, Table>("Vector") {
        Ok(metatable) => vector_lua.set_metatable(Some(metatable)),
        _ => {}
    }
    vector_lua
}

fn vec_from_lua(vector_lua: &Table) -> Result<Vec3> {
    let x: f32 = vector_lua.get("x")?;
    let y: f32 = vector_lua.get("y")?;
    let z: f32 = vector_lua.get("z")?;
    Ok(glm::vec3(x, y, z))
}
