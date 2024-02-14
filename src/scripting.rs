use crate::{
    entity_system::{EntityId, SceneChunk},
    runtime::WindowEvents,
};
use glfw::{Action, Key, Modifiers, Window};
use rlua::{
    Context, Error, Function, LightUserData, Lua, RegistryKey, Result, StdLib, Table, Value,
};
use std::{ffi::c_void, fs, sync::Arc};

pub struct CompiledChunk(Vec<u8>);

pub enum Script {
    Object(RegistryObject),
    File(RegistryObject)
}

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

            InputWrappers::create_keys_table(&context);
        });
    }

    pub fn execute_chunk(&self, chunk: &CompiledChunk) {
        self.lua.context(|context| unsafe {
            context
                .load(&chunk.0)
                .into_function_allow_binary()
                .unwrap()
                .call::<_, ()>(());
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

    pub fn key_from_i32(value: i32) -> Option<Key> {
        match value {
            32 => Some(Key::Space),
            39 => Some(Key::Apostrophe),
            44 => Some(Key::Comma),
            45 => Some(Key::Minus),
            46 => Some(Key::Period),
            47 => Some(Key::Slash),
            48 => Some(Key::Num0),
            49 => Some(Key::Num1),
            50 => Some(Key::Num2),
            51 => Some(Key::Num3),
            52 => Some(Key::Num4),
            53 => Some(Key::Num5),
            54 => Some(Key::Num6),
            55 => Some(Key::Num7),
            56 => Some(Key::Num8),
            57 => Some(Key::Num9),
            59 => Some(Key::Semicolon),
            61 => Some(Key::Equal),
            65 => Some(Key::A),
            66 => Some(Key::B),
            67 => Some(Key::C),
            68 => Some(Key::D),
            69 => Some(Key::E),
            70 => Some(Key::F),
            71 => Some(Key::G),
            72 => Some(Key::H),
            73 => Some(Key::I),
            74 => Some(Key::J),
            75 => Some(Key::K),
            76 => Some(Key::L),
            77 => Some(Key::M),
            78 => Some(Key::N),
            79 => Some(Key::O),
            80 => Some(Key::P),
            81 => Some(Key::Q),
            82 => Some(Key::R),
            83 => Some(Key::S),
            84 => Some(Key::T),
            85 => Some(Key::U),
            86 => Some(Key::V),
            87 => Some(Key::W),
            88 => Some(Key::X),
            89 => Some(Key::Y),
            90 => Some(Key::Z),
            91 => Some(Key::LeftBracket),
            92 => Some(Key::Backslash),
            93 => Some(Key::RightBracket),
            96 => Some(Key::GraveAccent),
            161 => Some(Key::World1),
            162 => Some(Key::World2),
            256 => Some(Key::Escape),
            257 => Some(Key::Enter),
            258 => Some(Key::Tab),
            259 => Some(Key::Backspace),
            260 => Some(Key::Insert),
            261 => Some(Key::Delete),
            262 => Some(Key::Right),
            263 => Some(Key::Left),
            264 => Some(Key::Down),
            265 => Some(Key::Up),
            266 => Some(Key::PageUp),
            267 => Some(Key::PageDown),
            268 => Some(Key::Home),
            269 => Some(Key::End),
            280 => Some(Key::CapsLock),
            281 => Some(Key::ScrollLock),
            282 => Some(Key::NumLock),
            283 => Some(Key::PrintScreen),
            284 => Some(Key::Pause),
            290 => Some(Key::F1),
            291 => Some(Key::F2),
            292 => Some(Key::F3),
            293 => Some(Key::F4),
            294 => Some(Key::F5),
            295 => Some(Key::F6),
            296 => Some(Key::F7),
            297 => Some(Key::F8),
            298 => Some(Key::F9),
            299 => Some(Key::F10),
            300 => Some(Key::F11),
            301 => Some(Key::F12),
            302 => Some(Key::F13),
            303 => Some(Key::F14),
            304 => Some(Key::F15),
            305 => Some(Key::F16),
            306 => Some(Key::F17),
            307 => Some(Key::F18),
            308 => Some(Key::F19),
            309 => Some(Key::F20),
            310 => Some(Key::F21),
            311 => Some(Key::F22),
            312 => Some(Key::F23),
            313 => Some(Key::F24),
            314 => Some(Key::F25),
            320 => Some(Key::Kp0),
            321 => Some(Key::Kp1),
            322 => Some(Key::Kp2),
            323 => Some(Key::Kp3),
            324 => Some(Key::Kp4),
            325 => Some(Key::Kp5),
            326 => Some(Key::Kp6),
            327 => Some(Key::Kp7),
            328 => Some(Key::Kp8),
            329 => Some(Key::Kp9),
            330 => Some(Key::KpDecimal),
            331 => Some(Key::KpDivide),
            332 => Some(Key::KpMultiply),
            333 => Some(Key::KpSubtract),
            334 => Some(Key::KpAdd),
            335 => Some(Key::KpEnter),
            336 => Some(Key::KpEqual),
            340 => Some(Key::LeftShift),
            341 => Some(Key::LeftControl),
            342 => Some(Key::LeftAlt),
            343 => Some(Key::LeftSuper),
            344 => Some(Key::RightShift),
            345 => Some(Key::RightControl),
            346 => Some(Key::RightAlt),
            347 => Some(Key::RightSuper),
            348 => Some(Key::Menu),
            -1 => Some(Key::Unknown),
            _ => None,
        }
    }

    pub fn action_from_i32(value: i32) -> Option<Action> {
        match value {
            0 => Some(Action::Release),
            1 => Some(Action::Press),
            _ => None,
        }
    }

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

        let key = match Self::key_from_i32(key) {
            Some(value) => value,
            None => {
                return Err(Error::ExternalError(Arc::new(CustomError(
                    "Invalid key code".to_string(),
                ))))
            }
        };
        let action = match Self::action_from_i32(action) {
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
        let key = match Self::key_from_i32(key) {
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
        let mut s = String::from(
            "
Actions = {}
function Actions.Release() return 0 end
function Actions.Press() return 1 end

Modifiers = {}
function Modifiers.Shift() return 1 end
function Modifiers.Control() return 2 end
function Modifiers.Alt() return 4 end
function Modifiers.Super() return 8 end
function Modifiers.CapsLock() return 16 end
function Modifiers.NumLock() return 32 end

Keys = {}\n",
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
