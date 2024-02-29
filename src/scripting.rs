use crate::{
    entity_system::{EntityId, RefEntityId, SceneManager},
    resources::ResourceManager,
    runtime::WindowEvents,
    serializable,
};
use glfw::{Action, Key, Modifiers, PWindow};
use glm::Vec3;
use mlua::{
    prelude::{LuaUserDataFields, LuaUserDataMethods},
    Error, FromLua, Function, LightUserData, Lua, RegistryKey, Result, Table, UserData, Value,
};
use std::{ffi::c_void, ops::Deref};

#[derive(Debug)]
pub struct CompiledScript(Vec<u8>);

#[derive(Debug)]
pub struct ScriptObject(RegistryKey);

impl ScriptObject {
    fn as_table<'lua>(&self, lua: &'lua Lua) -> Result<Table<'lua>> {
        lua.registry_value::<Table>(&self.0)
    }
}

#[derive(Debug)]
pub struct Scripting {
    pub lua: Lua,
    object_owners: ScriptObject,
    updates: ScriptObject,
    collect_time: f64,
}

impl Scripting {
    pub fn new() -> Self {
        let lua = Lua::new();

        let table = lua.create_table().unwrap();
        let metatable = lua.create_table().unwrap();
        metatable.set("__mode", "k").unwrap();
        table.set_metatable(Some(metatable));
        let object_owners_key = lua.create_registry_value(table).unwrap();

        let table = lua.create_table().unwrap();
        let metatable = lua.create_table().unwrap();
        metatable.set("__mode", "kv").unwrap();
        table.set_metatable(Some(metatable));
        let updates_key = lua.create_registry_value(table).unwrap();

        Self {
            lua,
            object_owners: ScriptObject(object_owners_key),
            updates: ScriptObject(updates_key),
            collect_time: 0.0,
        }
    }

    pub fn create_script_object(
        &self,
        owner_id: &EntityId,
        script: &serializable::Script,
        resource_manager: &ResourceManager,
    ) -> ScriptObject {
        let object = self
            .lua
            .load(&resource_manager.compiled_scripts()[&script.script_path].0)
            .eval::<Table>()
            .unwrap();

        if let Ok(fun) = object.get::<_, Function>("update") {
            let updates = self.updates.as_table(&self.lua).unwrap();
            updates.set(object.clone(), fun).unwrap();
        }

        self.object_owners
            .as_table(&self.lua)
            .unwrap()
            .set(object.clone(), owner_id)
            .unwrap();

        let key = self.lua.create_registry_value(object).unwrap();

        ScriptObject(key)
    }

    pub fn run_updates(&self ) {
        let updates = self.updates.as_table(&self.lua).unwrap();
        updates
            .for_each(|k: Table, v: Function| v.call::<_, ()>(k))
            .unwrap();
    }

    fn gc_collect(&self) {
        self.lua.expire_registry_values();
        self.lua.gc_collect();
    }

    pub fn compile_script(&self, src: &str, name: &str) -> Result<CompiledScript> {
        let chunk = self.lua.load(src).set_name(name);
        let dumped = chunk.into_function()?.dump(false);
        Ok(CompiledScript(dumped))
    }

    pub fn create_wrappers<'a>(
        &'a self,
        scene_manager: &'a SceneManager,
        events: &'a WindowEvents,
        window: &'a PWindow,
        frame_time: &'a f64,
    ) {
        let scene_manager = LightUserData(scene_manager as *const _ as *mut c_void);
        let window_events = LightUserData(events as *const _ as *mut c_void);
        let window = LightUserData(window as *const _ as *mut c_void);
        let frame_time = LightUserData(frame_time as *const _ as *mut c_void);
        let scripting = LightUserData(self as *const _ as *mut c_void);
        let object_owners = LightUserData(&self.object_owners.0 as *const _ as *mut c_void);
        let updates = LightUserData(&self.updates.0 as *const _ as *mut c_void);

        InputApi::create_wrappers(&self.lua, window_events, window);
        TransformApi::create_wrappers(&self.lua, object_owners, scene_manager);

        let frame_time = self
            .lua
            .create_function(ApplicationApi::frame_time)
            .unwrap()
            .bind(frame_time)
            .unwrap();
        self.lua.globals().set("FrameTime", frame_time).unwrap();

        let delete_script = self
            .lua
            .create_function(ScriptingApi::delete_script)
            .unwrap()
            .bind((scripting, scene_manager))
            .unwrap();
        self.lua.globals().set("DeleteScript", delete_script);

        // let delete_entity = self.lua.create_function()
    }

    pub fn delete_script_object(&self, script_object: ScriptObject) {
        let object = self.lua.registry_value::<Table>(&script_object.0).unwrap();
        self.object_owners
            .as_table(&self.lua)
            .unwrap()
            .set(object.clone(), Value::Nil);
        self.updates
            .as_table(&self.lua)
            .unwrap()
            .set(object.clone(), Value::Nil);
        self.lua.remove_registry_value(script_object.0);
    }
}

struct TransformApi;

impl TransformApi {
    fn create_wrappers(lua: &Lua, object_owners: LightUserData, scene_manager: LightUserData) {
        let transform_move = lua
            .create_function(TransformApi::transform_move)
            .unwrap()
            .bind((object_owners, scene_manager))
            .unwrap();
        let transform_move_local = lua
            .create_function(TransformApi::transform_move_local)
            .unwrap()
            .bind((object_owners, scene_manager))
            .unwrap();
        let transform_rotate = lua
            .create_function(TransformApi::transform_rotate)
            .unwrap()
            .bind((object_owners, scene_manager))
            .unwrap();
        let transform_rotate_local = lua
            .create_function(TransformApi::transform_rotate_local)
            .unwrap()
            .bind((object_owners, scene_manager))
            .unwrap();
        let transform_get_position = lua
            .create_function(TransformApi::transform_get_position)
            .unwrap()
            .bind((object_owners, scene_manager))
            .unwrap();
        let transform_get_global_position = lua
            .create_function(TransformApi::transform_get_global_position)
            .unwrap()
            .bind((object_owners, scene_manager))
            .unwrap();
        let transform_get_orientation = lua
            .create_function(TransformApi::transform_get_orientation)
            .unwrap()
            .bind((object_owners, scene_manager))
            .unwrap();
        let transform_set_position = lua
            .create_function(TransformApi::transform_set_position)
            .unwrap()
            .bind((object_owners, scene_manager))
            .unwrap();
        let transform_set_orientation = lua
            .create_function(TransformApi::transform_set_orientation)
            .unwrap()
            .bind((object_owners, scene_manager))
            .unwrap();

        let transform = lua.create_table().unwrap();
        transform.set("move", transform_move).unwrap();
        transform.set("moveLocal", transform_move_local).unwrap();
        transform.set("rotate", transform_rotate).unwrap();
        transform
            .set("rotateLocal", transform_rotate_local)
            .unwrap();
        transform
            .set("getPosition", transform_get_position)
            .unwrap();
        transform
            .set("getGlobalPosition", transform_get_global_position)
            .unwrap();
        transform
            .set("getOrientation", transform_get_orientation)
            .unwrap();
        transform
            .set("setPosition", transform_set_position)
            .unwrap();
        transform
            .set("setOrientation", transform_set_orientation)
            .unwrap();
        lua.globals().set("Transform", transform).unwrap();

        let vec3_type = lua.create_proxy::<LuaVec3>().unwrap();
        lua.globals().set("Vec3", vec3_type).unwrap();
    }

    fn transform_move(
        lua: &Lua,
        args: (LightUserData, LightUserData, Table, LuaVec3),
    ) -> Result<()> {
        let (mut scene_manager, vector, id) = Self::extract_args_vec(lua, args)?;
        scene_manager.get_transform_mut(&id).move_(&vector);
        Ok(())
    }

    fn transform_move_local(
        lua: &Lua,
        args: (LightUserData, LightUserData, Table, LuaVec3),
    ) -> Result<()> {
        let (mut scene_manager, vector, id) = Self::extract_args_vec(lua, args)?;
        scene_manager.get_transform_mut(&id).move_local(&vector);
        Ok(())
    }

    fn transform_rotate(
        lua: &Lua,
        args: (LightUserData, LightUserData, Table, LuaVec3),
    ) -> Result<()> {
        let (mut scene_manager, vector, id) = Self::extract_args_vec(lua, args)?;
        scene_manager.get_transform_mut(&id).rotate(&vector);
        Ok(())
    }

    fn transform_rotate_local(
        lua: &Lua,
        args: (LightUserData, LightUserData, Table, LuaVec3),
    ) -> Result<()> {
        let (mut scene_manager, vector, id) = Self::extract_args_vec(lua, args)?;
        scene_manager.get_transform_mut(&id).rotate_local(&vector);
        Ok(())
    }

    fn transform_get_position<'lua>(
        lua: &'lua Lua,
        args: (LightUserData, LightUserData, Table<'lua>),
    ) -> Result<LuaVec3> {
        let (scene_manager, id) = Self::extract_args_no_vec(lua, args)?;
        Ok(LuaVec3(scene_manager.get_transform(&id).position))
    }

    fn transform_get_global_position<'lua>(
        lua: &'lua Lua,
        args: (LightUserData, LightUserData, Table<'lua>),
    ) -> Result<LuaVec3> {
        let (scene_manager, id) = Self::extract_args_no_vec(lua, args)?;
        Ok(LuaVec3(scene_manager.get_transform(&id).global_position()))
    }

    fn transform_get_orientation<'lua>(
        lua: &'lua Lua,
        args: (LightUserData, LightUserData, Table<'lua>),
    ) -> Result<LuaVec3> {
        let (scene_manager, id) = Self::extract_args_no_vec(lua, args)?;
        Ok(LuaVec3(glm::degrees(&glm::quat_euler_angles(
            &scene_manager.get_transform(&id).orientation,
        ))))
    }

    fn transform_set_position(
        lua: &Lua,
        args: (LightUserData, LightUserData, Table, LuaVec3),
    ) -> Result<()> {
        let (mut scene_manager, vector, id) = Self::extract_args_vec(lua, args)?;
        scene_manager.get_transform_mut(&id).position = *vector;
        Ok(())
    }

    fn transform_set_orientation(
        lua: &Lua,
        args: (LightUserData, LightUserData, Table, LuaVec3),
    ) -> Result<()> {
        let (mut scene_manager, vector, id) = Self::extract_args_vec(lua, args)?;
        scene_manager
            .get_transform_mut(&id)
            .set_orientation(&vector);
        Ok(())
    }

    fn extract_args_vec<'lua>(
        lua: &'lua Lua,
        args: (LightUserData, LightUserData, Table<'lua>, LuaVec3),
    ) -> Result<(&'lua mut SceneManager, LuaVec3, RefEntityId<'lua>)> {
        let object_owners = unsafe {
            lua.registry_value::<Table>(&*args.0 .0.cast::<RegistryKey>())
                .unwrap()
        };
        let scene_manager = unsafe { &mut *args.1 .0.cast::<SceneManager>() };
        let object = args.2;
        let vector = args.3;
        let id = object_owners.get::<_, RefEntityId>(object)?;
        Ok((scene_manager, vector, id))
    }

    fn extract_args_no_vec<'lua>(
        lua: &'lua Lua,
        args: (LightUserData, LightUserData, Table<'lua>),
    ) -> Result<(&'lua mut SceneManager, RefEntityId<'lua>)> {
        let object_owners = unsafe {
            lua.registry_value::<Table>(&*args.0 .0.cast::<RegistryKey>())
                .unwrap()
        };
        let scene_manager = unsafe { &mut *args.1 .0.cast::<SceneManager>() };
        let object = args.2;
        let id = object_owners.get::<_, RefEntityId>(object)?;
        Ok((scene_manager, id))
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

    pub fn create_wrappers(lua: &Lua, window_events: LightUserData, window: LightUserData) {
        let get_key = lua
            .create_function(Self::get_key)
            .unwrap()
            .bind(window_events)
            .unwrap();
        let get_key_held = lua
            .create_function(Self::get_key_held)
            .unwrap()
            .bind(window)
            .unwrap();
        let input = lua.create_table().unwrap();
        input.set("getKey", get_key).unwrap();
        input.set("getKeyHeld", get_key_held).unwrap();
        lua.globals().set("Input", input).unwrap();

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

    fn get_key(
        _: &Lua,
        args: (LightUserData, LuaKey, LuaAction, Option<LuaModifiers>),
    ) -> Result<bool> {
        let events = unsafe { &*args.0 .0.cast::<WindowEvents>() };
        let modifiers = args.3.unwrap_or(LuaModifiers(Modifiers::empty()));
        Ok(events.get_key((args.1 .0, args.2 .0, modifiers.0)))
    }

    fn get_key_held(_: &Lua, args: (LightUserData, LuaKey)) -> Result<bool> {
        let window = unsafe { &*args.0 .0.cast::<PWindow>() };
        Ok(window.get_key(args.1 .0) == Action::Press)
    }
}

struct ApplicationApi;

impl ApplicationApi {
    fn frame_time(_: &Lua, args: LightUserData) -> Result<f64> {
        unsafe { Ok(*args.0.cast::<f64>()) }
    }
}

struct ScriptingApi;

impl ScriptingApi {
    fn delete_script(lua: &Lua, args: (LightUserData, LightUserData, Table)) -> Result<()> {
        let scripting = unsafe { &*args.0 .0.cast::<Scripting>() };
        let scene_manager = unsafe { &mut *(args.1 .0.cast::<SceneManager>()) };
        let object = args.2;

        let object_owners = scripting.object_owners.as_table(lua).unwrap();
        let updates = scripting.updates.as_table(lua).unwrap();
        let owner_id = object_owners.get::<_, RefEntityId>(object.clone())?;
        
        object_owners.set(object.clone(), Value::Nil).unwrap();
        updates.set(object.clone(), Value::Nil).unwrap();

        let target = scene_manager
            .get_components::<ScriptObject>(&owner_id)
            .find(|item| {
                let key = &scene_manager.component_slice::<ScriptObject>()[item.array_index()]
                    .data
                    .0;
                let o = lua.registry_value::<Table>(key).unwrap();
                o == object
            })
            .unwrap();

        // scene_manager.delete_managed_component::<ScriptObject>(target, scripting);
        println!("Object is removed");

        Ok(())
    }

    fn delete_entity(
        lua: &Lua,
        args: (LightUserData, LightUserData, LightUserData, Table),
    ) -> Result<()> {
        let object_owners = unsafe { &*args.0 .0.cast::<RegistryKey>() };
        let updates = unsafe { &*args.1 .0.cast::<RegistryKey>() };
        let scene_manager = unsafe { &mut *(args.2 .0.cast::<SceneManager>()) };
        let object = args.3;

        let object_owners = lua.registry_value::<Table>(&object_owners).unwrap();
        let updates = lua.registry_value::<Table>(updates).unwrap();

        let owner_id = object_owners.get::<_, RefEntityId>(object.clone())?;

        let (index, _) = &scene_manager
            .component_slice::<ScriptObject>()
            .iter()
            .enumerate()
            .find(|(_, item)| {
                let o = lua.registry_value::<Table>(&item.data.0).unwrap();
                o == object
            })
            .unwrap();

        object_owners.set(object.clone(), Value::Nil).unwrap();
        updates.set(object, Value::Nil).unwrap();
        // scene_manager.delete_script(&owner_id, *index);
        println!("Object is removed");

        Ok(())
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
                to: "LuaAction",
                message: Some("Invalid argument".to_string()),
            })
        }
    }
}
