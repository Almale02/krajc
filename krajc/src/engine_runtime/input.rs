use std::collections::{HashMap, HashSet};

use krajc_macros::EngineResource;
use winit::event::{ButtonId, ElementState, ModifiersState, VirtualKeyCode};

use crate::typed_addr::dupe;

#[derive(EngineResource, Debug)]
pub struct KeyboardInput {
    pub key_events: HashSet<(VirtualKeyCode, ElementState, ModifiersState)>,
    pub key_states: HashMap<VirtualKeyCode, ElementState>,
    prev_key_states: HashMap<VirtualKeyCode, ElementState>,
}
impl KeyboardInput {
    pub fn new() -> Self {
        let mut input = Self {
            key_events: HashSet::default(),
            key_states: HashMap::default(),
            prev_key_states: HashMap::default(),
        };
        input.fill_up_states();
        input.prev_key_states = input.key_states.clone();
        input
    }
    /// Fires once when pressed, matches only when no modifier is pressed
    pub fn is_pressed(&self, key: VirtualKeyCode) -> bool {
        self.key_events
            .contains(&(key, ElementState::Pressed, ModifiersState::default()))
            && self.prev_key_states.get(&key).unwrap() == &ElementState::Released
    }
    /// Fires once when pressed, uses custom modifiers, like control, shift, and alt
    pub fn is_pressed_mod(&self, key: VirtualKeyCode, modifier: ModifiersState) -> bool {
        self.key_events
            .contains(&(key, ElementState::Pressed, modifier))
            && self.prev_key_states.get(&key).unwrap() == &ElementState::Released
    }
    /// Fires once when pressed, matches only when no modifier is pressed
    pub fn is_released(&self, key: VirtualKeyCode) -> bool {
        self.key_events
            .contains(&(key, ElementState::Released, ModifiersState::default()))
            && self.prev_key_states.get(&key).unwrap() == &ElementState::Pressed
    }
    /// Fires once when released, uses custom modifiers, like control, shift, and alt
    pub fn is_released_mod(&self, key: VirtualKeyCode, modifier: ModifiersState) -> bool {
        self.key_events
            .contains(&(key, ElementState::Released, modifier))
            && self.prev_key_states.get(&key).unwrap() == &ElementState::Pressed
    }
    /// Constantly fires while the key is being pressed down, only fires when no modifier is being pressed down
    pub fn is_held_down(&self, key: VirtualKeyCode) -> bool {
        self.key_states.get(&key).unwrap() == &ElementState::Pressed
    }

    pub fn reset_events(&mut self) {
        self.prev_key_states = self.key_states.clone();
        self.key_events.clear();
    }
    pub fn register_input(
        &mut self,
        key: VirtualKeyCode,
        state: ElementState,
        modifier: ModifiersState,
    ) {
        self.key_events.insert((key, state, modifier));

        *self.key_states.get_mut(&key).unwrap() = state;
    }
    #[rustfmt::skip]
    fn fill_up_states(&mut self) {
        self.key_states.insert(VirtualKeyCode::Key1, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Key2, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Key3, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Key4, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Key5, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Key6, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Key7, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Key8, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Key9, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Key0, ElementState::Released);

        self.key_states.insert(VirtualKeyCode::A, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::B, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::C, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::D, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::E, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::G, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::H, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::I, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::J, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::K, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::L, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::M, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::N, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::O, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::P, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Q, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::R, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::S, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::T, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::U, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::V, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::W, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::X, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Y, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Z, ElementState::Released);

        self.key_states.insert(VirtualKeyCode::Escape, ElementState::Released);

        self.key_states.insert(VirtualKeyCode::F1, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F2, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F3, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F4, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F5, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F6, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F7, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F8, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F9, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F10, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F11, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F12, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F13, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F14, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F15, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F16, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F17, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F18, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F19, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F20, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F21, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F22, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F23, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::F24, ElementState::Released);

        self.key_states.insert(VirtualKeyCode::Snapshot, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Scroll, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Pause, ElementState::Released);

        self.key_states.insert(VirtualKeyCode::Insert, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Home, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Delete, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::End, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::PageDown, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::PageUp, ElementState::Released);

        self.key_states.insert(VirtualKeyCode::Left, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Up, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Right, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Down, ElementState::Released);

        self.key_states.insert(VirtualKeyCode::Back, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Return, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Space, ElementState::Released);

        self.key_states.insert(VirtualKeyCode::Compose, ElementState::Released);

        self.key_states.insert(VirtualKeyCode::Caret, ElementState::Released);

        self.key_states.insert(VirtualKeyCode::Numlock, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Numpad0, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Numpad1, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Numpad2, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Numpad3, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Numpad4, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Numpad5, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Numpad6, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Numpad7, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Numpad8, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Numpad9, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::NumpadAdd, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::NumpadDivide, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::NumpadDecimal, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::NumpadComma, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::NumpadEnter, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::NumpadEquals, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::NumpadMultiply, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::NumpadSubtract, ElementState::Released);

        self.key_states.insert(VirtualKeyCode::AbntC1, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::AbntC2, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Apostrophe, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Apps, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Asterisk, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::At, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Ax, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Backslash, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Calculator, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Capital, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Colon, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Comma, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Convert, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Equals, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Grave, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Kana, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Kanji, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::LAlt, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::LBracket, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::LControl, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::LShift, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::LWin, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Mail, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::MediaSelect, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::MediaStop, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Minus, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Mute, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::MyComputer, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::NavigateForward, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::NavigateBackward, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::NextTrack, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::NoConvert, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::OEM102, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Period, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::PlayPause, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Plus, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Power, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::PrevTrack, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::RAlt, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::RBracket, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::RControl, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::RShift, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::RWin, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Semicolon, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Slash, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Sleep, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Stop, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Sysrq, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Tab, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Underline, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Unlabeled, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::VolumeDown, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::VolumeUp, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Wake, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::WebBack, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::WebFavorites, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::WebForward, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::WebHome, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::WebRefresh, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::WebSearch, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::WebStop, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Yen, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Copy, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Paste, ElementState::Released);
        self.key_states.insert(VirtualKeyCode::Cut, ElementState::Released);
    }
}

impl Default for KeyboardInput {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(EngineResource, Debug)]
pub struct MouseInput {
    pub button_events: HashSet<(ButtonId, ElementState)>,
    pub button_states: HashMap<ButtonId, ElementState>,
    pub mouse_motion: (f32, f32),
    prev_button_states: HashMap<ButtonId, ElementState>,
}
impl MouseInput {
    pub fn new() -> Self {
        let mut input = Self {
            button_events: HashSet::default(),
            button_states: HashMap::default(),
            prev_button_states: HashMap::default(),
            mouse_motion: Default::default(),
        };

        input.button_states.insert(0, ElementState::Released);
        input.button_states.insert(1, ElementState::Released);
        input.button_states.insert(2, ElementState::Released);
        input.button_states.insert(3, ElementState::Released);

        //input.fill_up_states();
        input.prev_button_states = input.button_states.clone();
        input
    }
    /// Fires once when pressed, matches only when no modifier is pressed
    pub fn is_pressed(&self, button: ButtonId) -> bool {
        self.button_events
            .contains(&(button, ElementState::Pressed))
            && self.prev_button_states.get(&button).unwrap() == &ElementState::Released
    }
    /// Fires once when pressed, matches only when no modifier is pressed
    pub fn is_released(&self, button: ButtonId) -> bool {
        self.button_events
            .contains(&(button, ElementState::Released))
            && self.prev_button_states.get(&button).unwrap() == &ElementState::Pressed
    }
    /// Constantly fires while the button is being pressed down, only fires when no modifier is being pressed down
    pub fn is_held_down(&self, button: ButtonId) -> bool {
        self.button_states.get(&button).unwrap() == &ElementState::Pressed
    }
    /// Returns the motion of the mouse compared to the previous frame with X and Y motion
    pub fn get_mouse_motion(&self) -> (f32, f32) {
        self.mouse_motion
    }

    pub fn reset_events(&mut self) {
        self.prev_button_states = self.button_states.clone();
        self.button_events.clear();
        self.mouse_motion = (0., 0.);
    }
    pub fn register_input(&mut self, button: ButtonId, state: ElementState) {
        self.button_events.insert((button, state));

        *dupe(self).button_states.get_mut(&button).unwrap() = state;
    }
}

impl Default for MouseInput {
    fn default() -> Self {
        Self::new()
    }
}
