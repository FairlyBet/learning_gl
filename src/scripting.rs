use crate::{
    entity_system::SceneManager, resources::ResourceManager, serializable, runtime::WindowEvents,
};
use glfw::{Action, Key, Modifiers, MouseButton, PWindow};
use glm::Vec3;
use mlua::{
    prelude::{LuaUserDataFields, LuaUserDataMethods},
    Error, FromLua, Function, LightUserData, Lua, RegistryKey, Result, Table, UserData, Value,
};
use std::{
    cell::{RefCell, RefMut},
    ops::Deref,
};

#[derive(Debug)]
pub struct CompiledScript(Vec<u8>);

#[derive(Debug)]
pub struct ScriptObject(RegistryKey);

#[derive(Debug)]
pub struct Scripting {
    lua: Lua,
    creation_functions: RegistryKey,
    object_handlers: RegistryKey,
    starts: RegistryKey,
    updates: RegistryKey,
}

impl Scripting {
    pub fn new() -> Self {
        let lua = Lua::new();

        let creation_functions = Self::create_table(&lua, None);
        let object_handlers = Self::create_table(&lua, None);
        let starts = Self::create_table(&lua, Some("kv"));
        let updates = Self::create_table(&lua, Some("kv"));

        Self {
            lua,
            creation_functions,
            object_handlers,
            starts,
            updates,
        }
    }

    fn create_table(lua: &Lua, ref_mode: Option<&str>) -> RegistryKey {
        let table = lua.create_table().unwrap();
        let metatable = lua.create_table().unwrap();
        if let Some(mode) = ref_mode {
            metatable.set("__mode", mode).unwrap();
        }
        table.set_metatable(Some(metatable));
        lua.create_registry_value(table).unwrap()
    }

    pub fn create_script_object(
        &self,
        owner_id: usize,
        script: &serializable::Script,
        resource_manager: &ResourceManager,
    ) -> ScriptObject {
        let creation_functions = self
            .lua
            .registry_value::<Table>(&self.creation_functions)
            .unwrap();
        let function = creation_functions
            .get::<&str, Function>(&script.script_path)
            .unwrap();

        let object = function.call::<_, Table>(()).unwrap();
        // entity_table[id] = 0
        // weakref
        todo!()
    }

    pub fn load_api(&self, scene_manager: &mut SceneManager, events: &WindowEvents) {
        TransformApi::create_wrappers(&self.lua, scene_manager);
    }

    pub fn run_updates(&self) {
        let updates = self.lua.registry_value::<Table>(&self.updates).unwrap();
        updates
            .for_each(|k: Table, v: Function| v.call::<_, ()>(k))
            .unwrap();
    }

    pub fn compile_script(&self, src: &str, name: &str) -> Result<CompiledScript> {
        let chunk = self.lua.load(src).set_name(name);
        let dumped = chunk.into_function()?.dump(false);
        Ok(CompiledScript(dumped))
    }

    pub fn load_script(&self, src: &str, name: &str) {
        let function = self.lua.load(src).eval::<Function>().unwrap();
        let creation_functions = self
            .lua
            .registry_value::<Table>(&self.creation_functions)
            .unwrap();
        creation_functions.set(name, function).unwrap();
    }

    // pub fn delete_script_object(&self, script_object: ScriptObject) {
    //     let object = self.lua.registry_value::<Table>(&script_object.0).unwrap();
    //     self.object_owners
    //         .as_table(&self.lua)
    //         .unwrap()
    //         .set(object.clone(), Value::Nil);
    //     self.updates
    //         .as_table(&self.lua)
    //         .unwrap()
    //         .set(object.clone(), Value::Nil);
    //     self.lua.remove_registry_value(script_object.0);
    // }
}

struct TransformApi;

impl TransformApi {
    fn create_wrappers(lua: &Lua, scene_manager: &mut SceneManager) {
        let vec = lua.create_proxy::<LuaVec3>().unwrap();
        lua.globals().set("Vec3", vec).unwrap();

        let position = lua.create_function(Self::position(scene_manager)).unwrap();
        let global_position = lua
            .create_function(Self::global_position(scene_manager))
            .unwrap();
        let set_position = lua
            .create_function(Self::set_position(scene_manager))
            .unwrap();
        let move_ = lua.create_function(Self::move_(scene_manager)).unwrap();
        let move_local = lua
            .create_function(Self::move_local(scene_manager))
            .unwrap();
        let orientation = lua
            .create_function(Self::orientation(scene_manager))
            .unwrap();
        let set_orientation = lua
            .create_function(Self::set_orientation(scene_manager))
            .unwrap();
        let rotate = lua.create_function(Self::rotate(scene_manager)).unwrap();
        let rotate_local = lua
            .create_function(Self::rotate_local(scene_manager))
            .unwrap();

        let transform = lua.create_table().unwrap();
        transform.set("getPosition", position).unwrap();
        transform.set("getGlobalPosition", global_position).unwrap();
        transform.set("setPosition", set_position).unwrap();
        transform.set("move", move_).unwrap();
        transform.set("moveLocal", move_local).unwrap();
        transform.set("getOrientation", orientation).unwrap();
        transform.set("setOrientation", set_orientation).unwrap();
        transform.set("rotate", rotate).unwrap();
        transform.set("rotateLocal", rotate_local).unwrap();
        lua.globals().set("Transform", transform).unwrap();
    }

    const fn position(
        scene_manager: *const SceneManager,
    ) -> impl Fn(&Lua, Table<'_>) -> Result<LuaVec3> {
        move |lua: &Lua, weak_ref: Table| {
            let scene_manager = unsafe { &*scene_manager };
            let metatable = Self::get_metatable(weak_ref)?;
            let entity = metatable.get::<_, Table>("__index")?;
            let id = entity.get::<_, IdWrapper>(entity.clone())?.0;

            Ok(LuaVec3(scene_manager.get_transform(id).position))
        }
    }

    const fn global_position(
        scene_manager: *const SceneManager,
    ) -> impl Fn(&Lua, Table<'_>) -> Result<LuaVec3> {
        move |lua: &Lua, weak_ref: Table| {
            let scene_manager = unsafe { &*scene_manager };
            let metatable = Self::get_metatable(weak_ref)?;
            let entity = metatable.get::<_, Table>("__index")?;
            let id = entity.get::<_, IdWrapper>(entity.clone())?.0;

            Ok(LuaVec3(scene_manager.get_transform(id).global_position()))
        }
    }

    const fn set_position(
        scene_manager: *mut SceneManager,
    ) -> impl Fn(&Lua, (Table<'_>, LuaVec3)) -> Result<()> {
        move |lua: &Lua, args: (Table, LuaVec3)| {
            let scene_manager = unsafe { &mut *scene_manager };
            let metatable = Self::get_metatable(args.0)?;
            let entity = metatable.get::<_, Table>("__index")?;
            let id = entity.get::<_, IdWrapper>(entity.clone())?.0;
            scene_manager.get_transform_mut(id).position = args.1 .0;

            Ok(())
        }
    }

    const fn move_(
        scene_manager: *mut SceneManager,
    ) -> impl Fn(&Lua, (Table<'_>, LuaVec3)) -> Result<()> {
        move |lua: &Lua, args: (Table, LuaVec3)| {
            let scene_manager = unsafe { &mut *scene_manager };
            let metatable = Self::get_metatable(args.0)?;
            let entity = metatable.get::<_, Table>("__index")?;
            let id = entity.get::<_, IdWrapper>(entity.clone())?.0;
            scene_manager.get_transform_mut(id).move_(&args.1 .0);

            Ok(())
        }
    }

    const fn move_local(
        scene_manager: *mut SceneManager,
    ) -> impl Fn(&Lua, (Table<'_>, LuaVec3)) -> Result<()> {
        move |lua: &Lua, args: (Table, LuaVec3)| {
            let scene_manager = unsafe { &mut *scene_manager };
            let metatable = Self::get_metatable(args.0)?;
            let entity = metatable.get::<_, Table>("__index")?;
            let id = entity.get::<_, IdWrapper>(entity.clone())?.0;
            scene_manager.get_transform_mut(id).move_local(&args.1 .0);

            Ok(())
        }
    }

    const fn orientation(
        scene_manager: *const SceneManager,
    ) -> impl Fn(&Lua, Table<'_>) -> Result<LuaVec3> {
        move |lua: &Lua, weak_ref: Table| {
            let scene_manager = unsafe { &*scene_manager };
            let metatable = Self::get_metatable(weak_ref)?;
            let entity = metatable.get::<_, Table>("__index")?;
            let id = entity.get::<_, IdWrapper>(entity.clone())?.0;

            Ok(LuaVec3(glm::quat_euler_angles(
                &scene_manager.get_transform(id).orientation,
            )))
        }
    }

    const fn set_orientation(
        scene_manager: *mut SceneManager,
    ) -> impl Fn(&Lua, (Table<'_>, LuaVec3)) -> Result<()> {
        move |lua: &Lua, args: (Table, LuaVec3)| {
            let scene_manager = unsafe { &mut *scene_manager };
            let metatable = Self::get_metatable(args.0)?;
            let entity = metatable.get::<_, Table>("__index")?;
            let id = entity.get::<_, IdWrapper>(entity.clone())?.0;
            scene_manager
                .get_transform_mut(id)
                .set_orientation(&args.1 .0);

            Ok(())
        }
    }

    const fn rotate(
        scene_manager: *mut SceneManager,
    ) -> impl Fn(&Lua, (Table<'_>, LuaVec3)) -> Result<()> {
        move |lua: &Lua, args: (Table, LuaVec3)| {
            let scene_manager = unsafe { &mut *scene_manager };
            let metatable = Self::get_metatable(args.0)?;
            let entity = metatable.get::<_, Table>("__index")?;
            let id = entity.get::<_, IdWrapper>(entity.clone())?.0;
            scene_manager.get_transform_mut(id).rotate(&args.1 .0);

            Ok(())
        }
    }

    const fn rotate_local(
        scene_manager: *mut SceneManager,
    ) -> impl Fn(&Lua, (Table<'_>, LuaVec3)) -> Result<()> {
        move |lua: &Lua, args: (Table, LuaVec3)| {
            let scene_manager = unsafe { &mut *scene_manager };
            let metatable = Self::get_metatable(args.0)?;
            let entity = metatable.get::<_, Table>("__index")?;
            let id = entity.get::<_, IdWrapper>(entity.clone())?.0;
            scene_manager.get_transform_mut(id).rotate_local(&args.1 .0);

            Ok(())
        }
    }

    fn get_metatable<'lua>(table: Table<'lua>) -> Result<Table<'lua>> {
        match table.get_metatable() {
            Some(table) => Ok(table),
            None => Err(Error::external(CustomError("Invalid argument".to_string()))),
        }
    }
}

struct InputApi;

impl InputApi {
    pub const KEYS: &'static [Key] = &[
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

    pub const ACTIONS: &'static [Action] = &[Action::Press, Action::Release, Action::Repeat];

    pub const MODIFIERS: &'static [Modifiers] = &[
        Modifiers::Alt,
        Modifiers::CapsLock,
        Modifiers::Control,
        Modifiers::NumLock,
        Modifiers::Shift,
        Modifiers::Super,
    ];

    pub fn create_wrappers(lua: &Lua, events: &WindowEvents, window: &PWindow) {
        let get_key = lua.create_function(Self::get_key(events)).unwrap();
        let get_key_held = lua.create_function(Self::get_key_held(window)).unwrap();
        let get_mouse_button = lua
            .create_function_mut(Self::get_mouse_button(events))
            .unwrap();

        let input = lua.create_table().unwrap();
        input.set("getKey", get_key).unwrap();
        input.set("getKeyHeld", get_key_held).unwrap();
        input.set("getMouseButton", get_mouse_button).unwrap();
        lua.globals().set("Input", input).unwrap();

        // Test later
        // let key = lua.create_proxy::<LuaKey>().unwrap();
        // let action = lua.create_proxy::<LuaAction>().unwrap();
        // let modifiers = lua.create_proxy::<LuaModifiers>().unwrap();
        // lua.globals().set("Keys", key).unwrap();
        // lua.globals().set("Actions", action).unwrap();
        // lua.globals().set("Modifiers", modifiers).unwrap();

        let keys = Self::KEYS
            .iter()
            .map(|item| (Self::key_to_str(*item), LuaKey(*item)));
        let keys = lua.create_table_from(keys).unwrap();
        lua.globals().set("Keys", keys);

        let action = Self::ACTIONS
            .iter()
            .map(|item| (Self::action_to_str(*item), LuaAction(*item)));
        let action = lua.create_table_from(action).unwrap();
        lua.globals().set("Actions", action);

        let modifiers = Self::MODIFIERS
            .iter()
            .map(|item| (Self::modifier_to_str(*item), LuaModifiers(*item)));
        let modifiers = lua.create_table_from(modifiers).unwrap();
        lua.globals().set("Modifiers", modifiers);
    }

    const fn get_key(
        events: *const WindowEvents,
    ) -> impl Fn(&Lua, (LuaKey, LuaAction, Option<LuaModifiers>)) -> Result<bool> {
        move |_: &Lua, args: (LuaKey, LuaAction, Option<LuaModifiers>)| {
            let events = unsafe { &*events };
            let modifiers = args.2.unwrap_or(LuaModifiers(Modifiers::empty()));
            Ok(events.get_key((args.0 .0, args.1 .0, modifiers.0)))
        }
    }

    const fn get_key_held(window: *const PWindow) -> impl Fn(&Lua, LuaKey) -> Result<bool> {
        move |_: &Lua, key: LuaKey| {
            let window = unsafe { &*window };
            Ok(window.get_key(key.0) == Action::Press)
        }
    }

    const fn get_mouse_button(
        events: *const WindowEvents,
    ) -> impl Fn(&Lua, (LuaMouseButton, LuaAction, Option<LuaModifiers>)) -> Result<bool> {
        move |_: &Lua, args: (LuaMouseButton, LuaAction, Option<LuaModifiers>)| {
            let events = unsafe { &*events };
            let modifiers = args.2.unwrap_or(LuaModifiers(Modifiers::empty()));
            Ok(events.get_mouse_button((args.0 .0, args.1 .0, modifiers.0)))
        }
    }

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

    pub fn action_to_str(action: Action) -> &'static str {
        match action {
            Action::Release => "Release",
            Action::Press => "Press",
            Action::Repeat => "Repeat",
        }
    }

    pub fn modifier_to_str(modifier: Modifiers) -> &'static str {
        match modifier {
            Modifiers::Alt => "Alt",
            Modifiers::CapsLock => "CapsLock",
            Modifiers::Control => "Control",
            Modifiers::NumLock => "NumLock",
            Modifiers::Shift => "Shift",
            Modifiers::Super => "Super",
            _ => panic!(),
        }
    }
}

struct ApplicationApi;

impl ApplicationApi {
    fn frame_time(_: &Lua, args: LightUserData) -> Result<f64> {
        unsafe { Ok(*args.0.cast::<f64>()) }
    }
}

struct ScriptingApi;

// impl ScriptingApi {
//     fn delete_script(lua: &Lua, args: (LightUserData, LightUserData, Table)) -> Result<()> {
//         let scripting = unsafe { &*args.0 .0.cast::<Scripting>() };
//         let scene_manager = unsafe { &mut *(args.1 .0.cast::<SceneManager>()) };
//         let object = args.2;
//         let object_owners = scripting.object_owners.as_table(lua).unwrap();
//         let updates = scripting.updates.as_table(lua).unwrap();
//         let owner_id = object_owners.get::<_, RefEntityId>(object.clone())?;
//         object_owners.set(object.clone(), Value::Nil).unwrap();
//         updates.set(object.clone(), Value::Nil).unwrap();
//         let target = scene_manager
//             .get_components::<ScriptObject>(&owner_id)
//             .find(|item| {
//                 let key = &scene_manager.component_slice::<ScriptObject>()[item.array_index()]
//                     .data
//                     .0;
//                 let o = lua.registry_value::<Table>(key).unwrap();
//                 o == object
//             })
//             .unwrap();
//         // scene_manager.delete_managed_component::<ScriptObject>(target, scripting);
//         println!("Object is removed");
//         Ok(())
//     }
//     fn delete_entity(
//         lua: &Lua,
//         args: (LightUserData, LightUserData, LightUserData, Table),
//     ) -> Result<()> {
//         let object_owners = unsafe { &*args.0 .0.cast::<RegistryKey>() };
//         let updates = unsafe { &*args.1 .0.cast::<RegistryKey>() };
//         let scene_manager = unsafe { &mut *(args.2 .0.cast::<SceneManager>()) };
//         let object = args.3;
//         let object_owners = lua.registry_value::<Table>(&object_owners).unwrap();
//         let updates = lua.registry_value::<Table>(updates).unwrap();
//         let owner_id = object_owners.get::<_, RefEntityId>(object.clone())?;
//         let (index, _) = &scene_manager
//             .component_slice::<ScriptObject>()
//             .iter()
//             .enumerate()
//             .find(|(_, item)| {
//                 let o = lua.registry_value::<Table>(&item.data.0).unwrap();
//                 o == object
//             })
//             .unwrap();
//         object_owners.set(object.clone(), Value::Nil).unwrap();
//         updates.set(object, Value::Nil).unwrap();
//         // scene_manager.delete_script(&owner_id, *index);
//         println!("Object is removed");
//         Ok(())
//     }
// }

struct IdWrapper(usize);

impl UserData for IdWrapper {}

impl<'lua> FromLua<'lua> for IdWrapper {
    fn from_lua(value: Value<'lua>, lua: &'lua Lua) -> Result<Self> {
        match value.as_userdata() {
            Some(userdata) => Ok(IdWrapper(userdata.borrow::<IdWrapper>()?.0)),
            None => Err(Error::FromLuaConversionError {
                from: "Value",
                to: "IdWrapper",
                message: None,
            }),
        }
    }
}

#[derive(Debug)]
struct LuaVec3(Vec3);

impl<'lua> FromLua<'lua> for LuaVec3 {
    fn from_lua(value: Value<'lua>, lua: &'lua Lua) -> Result<Self> {
        if let Some(data) = value.as_userdata() {
            Ok(LuaVec3(data.borrow::<LuaVec3>()?.0))
        } else {
            Err(Error::FromLuaConversionError {
                from: "Value",
                to: "LuaVec3",
                message: Some("Invalid argument".to_string()),
            })
        }
    }
}

impl UserData for LuaVec3 {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, this| Ok(this.0.x));
        fields.add_field_method_get("y", |_, this| Ok(this.0.y));
        fields.add_field_method_get("z", |_, this| Ok(this.0.z));

        fields.add_field_method_set("x", |_, self_, value: f32| {
            self_.0.x = value;
            Ok(())
        });
        fields.add_field_method_set("y", |_, self_, value: f32| {
            self_.0.y = value;
            Ok(())
        });
        fields.add_field_method_set("z", |_, self_, value: f32| {
            self_.0.z = value;
            Ok(())
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method("__tostring", |_, self_, _: Value| {
            Ok(format!("[{}; {}; {}]", self_.0.x, self_.0.y, self_.0.z))
        });
        methods.add_meta_method("__add", |_, self_, vec: LuaVec3| {
            Ok(LuaVec3(self_.0 + vec.0))
        });
        methods.add_meta_method("__sub", |_, self_, vec: LuaVec3| {
            Ok(LuaVec3(self_.0 - vec.0))
        });
        methods.add_meta_method("__unm", |_, self_, vec: LuaVec3| Ok(LuaVec3(-self_.0)));
        methods.add_meta_method("__mul", |_, self_, num: f32| Ok(LuaVec3(self_.0 * num)));

        methods.add_function("new", |_, args: (f32, f32, f32)| {
            Ok(LuaVec3(glm::vec3(args.0, args.1, args.2)))
        });
        methods.add_function("zeros", |_, args: ()| Ok(LuaVec3(glm::Vec3::zeros())));
    }
}

impl Deref for LuaVec3 {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
struct LuaKey(Key);

impl UserData for LuaKey {}

impl<'lua> FromLua<'lua> for LuaKey {
    fn from_lua(value: Value<'lua>, lua: &'lua Lua) -> Result<Self> {
        if let Some(data) = value.as_userdata() {
            Ok(LuaKey((data.borrow::<LuaKey>()?.0)))
        } else {
            Err(Error::FromLuaConversionError {
                from: "Value",
                to: "LuaKey",
                message: Some("Invalid argument".to_string()),
            })
        }
    }
}

#[derive(Debug)]
struct LuaAction(Action);

impl UserData for LuaAction {}

impl<'lua> FromLua<'lua> for LuaAction {
    fn from_lua(value: Value<'lua>, lua: &'lua Lua) -> Result<Self> {
        if let Some(data) = value.as_userdata() {
            Ok(LuaAction((data.borrow::<LuaAction>()?.0)))
        } else {
            Err(Error::FromLuaConversionError {
                from: "Value",
                to: "LuaAction",
                message: Some("Invalid argument".to_string()),
            })
        }
    }
}

struct LuaModifiers(Modifiers);

impl UserData for LuaModifiers {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_function("__bor", |_, args: (LuaModifiers, LuaModifiers)| {
            Ok(LuaModifiers(args.0 .0 | args.1 .0))
        });
    }
}

impl<'lua> FromLua<'lua> for LuaModifiers {
    fn from_lua(value: Value<'lua>, lua: &'lua Lua) -> Result<Self> {
        if let Some(data) = value.as_userdata() {
            Ok(LuaModifiers((data.borrow::<LuaModifiers>()?.0)))
        } else {
            Err(Error::FromLuaConversionError {
                from: "Value",
                to: "LuaModifiers",
                message: Some("Invalid argument".to_string()),
            })
        }
    }
}

struct LuaMouseButton(MouseButton);

impl UserData for LuaMouseButton {}

impl<'lua> FromLua<'lua> for LuaMouseButton {
    fn from_lua(value: Value<'lua>, lua: &'lua Lua) -> Result<Self> {
        if let Some(data) = value.as_userdata() {
            Ok(LuaMouseButton((data.borrow::<LuaMouseButton>()?.0)))
        } else {
            Err(Error::FromLuaConversionError {
                from: "Value",
                to: "LuaMouseButton",
                message: Some("Invalid argument".to_string()),
            })
        }
    }
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
