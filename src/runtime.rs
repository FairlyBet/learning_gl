use crate::{
    application::Application,
    asset_manager::ResourceManager,
    entity_system::SceneChunk,
    gl_wrappers,
    rendering::{DefaultRenderer, Screen},
    scene,
    scripting::Scripting,
};
use glfw::{Action, Context as _, Key, Modifiers, MouseButton, WindowEvent};
use std::{fs::File, io::Write as _};

pub struct Runtime;

impl Runtime {
    pub fn new() -> Self {
        Self {}
    }

    fn update_events(
        app: &mut Application,
        events: &mut WindowEvents,
        framebuffer_size_callbacks: &mut Vec<&mut dyn FramebufferSizeCallback>,
    ) {
        app.glfw.poll_events();
        events.clear_events();
        for (_, event) in glfw::flush_messages(&app.receiver) {
            match event {
                WindowEvent::Key(Key::Enter, _, Action::Repeat, _) => {
                    events.char_input.push('\n');
                }
                WindowEvent::Key(key, _, action @ (Action::Press | Action::Release), modifiers) => {
                    println!(
                        "{} {} {}",
                        WindowEvents::key_to_str(key),
                        action as i32,
                        modifiers.bits()
                    );
                    events.key_input.push((key, action, modifiers));
                    if key == Key::V && action == Action::Press && modifiers == Modifiers::Control {
                        if let Some(string) = app.window.get_clipboard_string() {
                            events.char_input.push_str(&string);
                        }
                    }
                    if key == Key::Enter && action == Action::Press {
                        events.char_input.push('\n');
                    }
                }
                WindowEvent::Char(char_) => {
                    events.char_input.push(char_);
                }
                WindowEvent::CursorPos(x, y) => {
                    events.update_cursor_pos((x, y));
                }
                WindowEvent::MouseButton(button, action, modifiers) => {
                    events.mouse_button_input.push((button, action, modifiers));
                }
                WindowEvent::Scroll(x, y) => {}
                WindowEvent::FramebufferSize(w, h) => {
                    if (w == 0 || h == 0) {
                        continue;
                    }
                    for callback in framebuffer_size_callbacks.iter_mut() {
                        callback.framebuffer_size((w, h));
                    }
                }
                WindowEvent::Focus(focused) => {}
                WindowEvent::FileDrop(paths) => {}
                _ => {}
            }
        }
    }

    fn render_iteration(app: &mut Application) {
        gl_wrappers::clear(gl::COLOR_BUFFER_BIT);
        app.window.swap_buffers();
    }

    pub fn run(self, mut app: Application) {
        let scenes = scene::get_scenes();

        let start = match scenes.first() {
            Some(scene) => scene,
            None => return,
        };

        let mut resource_manager = ResourceManager::new();
        resource_manager.load(&start);

        let mut chunk = SceneChunk::from_scene(&start, &resource_manager);

        let scripting = Scripting::new();

        let mut renderer = DefaultRenderer::new(
            app.window.get_framebuffer_size(),
            app.window.get_context_version(),
        );

        let mut screen = Screen::new(
            app.window.get_framebuffer_size(),
            app.window.get_context_version(),
        );

        let mut input = WindowEvents::default();

        app.window.focus();
        let mut file = File::create("input.txt").unwrap();

        let mut frame_time = 0.0;

        while !app.window.should_close() {
            app.glfw.set_time(0.0);

            Self::update_events(&mut app, &mut input, &mut vec![&mut renderer, &mut screen]);
            file.write(&input.char_input.as_bytes());
            Self::render_iteration(&mut app);

            frame_time = app.glfw.get_time();
        }
    }
}

#[derive(Default)]
pub struct WindowEvents {
    key_input: Vec<(Key, Action, Modifiers)>,
    char_input: String,
    mouse_button_input: Vec<(MouseButton, Action, Modifiers)>,
    cursor_offset: (f64, f64),
    cursor_pos: (f64, f64),
}

impl WindowEvents {
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

    fn update_cursor_pos(&mut self, cursor_pos: (f64, f64)) {
        self.cursor_offset.0 = self.cursor_pos.0 - cursor_pos.0;
        self.cursor_offset.1 = self.cursor_pos.1 - cursor_pos.1;
        self.cursor_pos = cursor_pos;
    }

    pub fn get_key(&self, key: (Key, Action, Modifiers)) -> bool {
        self.key_input.contains(&key)
    }

    pub fn get_mouse_button(&self, button: (MouseButton, Action, Modifiers)) -> bool {
        self.mouse_button_input.contains(&button)
    }

    pub fn clear_events(&mut self) {
        self.key_input.clear();
        self.char_input.clear();
        self.mouse_button_input.clear();
    }
}

pub trait FramebufferSizeCallback {
    fn framebuffer_size(&mut self, size: (i32, i32));
}
