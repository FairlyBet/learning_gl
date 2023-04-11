use glfw::{Action, Key, Window};

pub struct Input<'a> {
    window: &'a Window,
}

impl<'a> Input<'_> {
    pub fn new(window: &'a Window) -> Input {
        Input { window }
    }

    pub fn get_key(&self, key: Key) -> Action {
        self.window.get_key(key)
    }

    pub fn get_key_with_action(&self, key: Key, action: Action) -> bool {
        let actual = self.window.get_key(key);
        action == actual
    }
}
