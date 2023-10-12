use glfw::{Action, CursorMode, Key, OpenGlProfileHint, Window, WindowMode};

pub struct GlfwConfig {
    pub profile: OpenGlProfileHint,
    pub version: (u32, u32),
}

impl Default for GlfwConfig {
    fn default() -> Self {
        Self {
            profile: OpenGlProfileHint::Core,
            version: (4, 2),
        }
    }
}

pub struct WindowConfig<'a> {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub mode: WindowMode<'a>,
    pub cursor_mode: CursorMode,
    pub vsync: bool,
}

impl Default for WindowConfig<'_> {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            title: Default::default(),
            mode: WindowMode::Windowed,
            cursor_mode: CursorMode::Normal,
            vsync: true,
        }
    }
}

pub struct Application<'a> {
    window: &'a Window,
    frametime: f32,
    should_close: bool,
    cursor_offset: (f32, f32),
}

impl<'a> Application<'a> {
    pub fn new(window: &'a Window, frametime: f32, cursor_offset: (f32, f32)) -> Self {
        Application {
            window,
            frametime,
            should_close: false,
            cursor_offset,
        }
    }

    pub fn get_key(&self, key: Key) -> Action {
        self.window.get_key(key)
    }

    pub fn get_cursor_pos(&self) -> (f32, f32) {
        let pos = self.window.get_cursor_pos();
        (pos.0 as f32, pos.1 as f32)
    }

    pub fn get_cursor_offset(&self) -> (f32, f32) {
        self.cursor_offset
    }

    pub fn get_frametime(&self) -> f32 {
        self.frametime
    }

    pub fn set_should_close_true(&mut self) {
        self.should_close = true;
    }

    pub fn get_should_close(&self) -> bool {
        self.should_close
    }
}
