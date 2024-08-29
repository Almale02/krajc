use std::collections::{HashMap, HashSet};

use krajc_macros::EngineResource;
use winit::{
    event::{ButtonId, ElementState},
    keyboard::KeyCode,
};

use crate::typed_addr::dupe;
use shared_lib::prelude::*;

#[derive(EngineResource, Debug)]
pub struct KeyboardInput {
    pub key_states: HashMap<KeyCode, ElementState>,
    prev_key_states: HashMap<KeyCode, ElementState>,
}
impl KeyboardInput {
    pub fn new() -> Self {
        let mut input = Self {
            key_states: HashMap::default(),
            prev_key_states: HashMap::default(),
        };
        input.fill_up_states();
        input.prev_key_states = input.key_states.clone();
        input
    }
    /// Fires once when pressed, matches only when no modifier is pressed
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.key_states.get(&key).unwrap() == &ElementState::Pressed
            && self.prev_key_states.get(&key).unwrap() == &ElementState::Released
    }
    /// Fires once when pressed
    pub fn is_released(&self, key: KeyCode) -> bool {
        self.key_states.get(&key).unwrap() == &ElementState::Pressed
            && self.prev_key_states.get(&key).unwrap() == &ElementState::Released
    }
    /// Constantly fires while the key is being pressed down, only fires when no modifier is being pressed down
    pub fn is_held_down(&self, key: KeyCode) -> bool {
        self.key_states.get(&key).unwrap() == &ElementState::Pressed
    }

    pub fn reset_events(&mut self) {
        self.prev_key_states = self.key_states.clone();
    }
    pub fn register_input(&mut self, key: KeyCode, state: ElementState) {
        *self.key_states.get_mut(&key).unwrap() = state;
    }
    #[rustfmt::skip]
    fn fill_up_states(&mut self) {


        self.key_states.insert(KeyCode::Backquote, ElementState::Released);
        self.key_states.insert(KeyCode::Backslash, ElementState::Released);
        self.key_states.insert(KeyCode::BracketLeft, ElementState::Released);
        self.key_states.insert(KeyCode::BracketRight, ElementState::Released);
        self.key_states.insert(KeyCode::Comma, ElementState::Released);
        self.key_states.insert(KeyCode::Digit0, ElementState::Released);
        self.key_states.insert(KeyCode::Digit1, ElementState::Released);
        self.key_states.insert(KeyCode::Digit2, ElementState::Released);
        self.key_states.insert(KeyCode::Digit3, ElementState::Released);
        self.key_states.insert(KeyCode::Digit4, ElementState::Released);
        self.key_states.insert(KeyCode::Digit5, ElementState::Released);
        self.key_states.insert(KeyCode::Digit6, ElementState::Released);
        self.key_states.insert(KeyCode::Digit7, ElementState::Released);
        self.key_states.insert(KeyCode::Digit8, ElementState::Released);
        self.key_states.insert(KeyCode::Digit9, ElementState::Released);
        self.key_states.insert(KeyCode::Equal, ElementState::Released);
        self.key_states.insert(KeyCode::IntlBackslash, ElementState::Released);
        self.key_states.insert(KeyCode::IntlRo, ElementState::Released);
        self.key_states.insert(KeyCode::IntlYen, ElementState::Released);
        self.key_states.insert(KeyCode::KeyA, ElementState::Released);
        self.key_states.insert(KeyCode::KeyB, ElementState::Released);
        self.key_states.insert(KeyCode::KeyC, ElementState::Released);
        self.key_states.insert(KeyCode::KeyD, ElementState::Released);
        self.key_states.insert(KeyCode::KeyE, ElementState::Released);
        self.key_states.insert(KeyCode::KeyF, ElementState::Released);
        self.key_states.insert(KeyCode::KeyG, ElementState::Released);
        self.key_states.insert(KeyCode::KeyH, ElementState::Released);
        self.key_states.insert(KeyCode::KeyI, ElementState::Released);
        self.key_states.insert(KeyCode::KeyJ, ElementState::Released);
        self.key_states.insert(KeyCode::KeyK, ElementState::Released);
        self.key_states.insert(KeyCode::KeyL, ElementState::Released);
        self.key_states.insert(KeyCode::KeyM, ElementState::Released);
        self.key_states.insert(KeyCode::KeyN, ElementState::Released);
        self.key_states.insert(KeyCode::KeyO, ElementState::Released);
        self.key_states.insert(KeyCode::KeyP, ElementState::Released);
        self.key_states.insert(KeyCode::KeyQ, ElementState::Released);
        self.key_states.insert(KeyCode::KeyR, ElementState::Released);
        self.key_states.insert(KeyCode::KeyS, ElementState::Released);
        self.key_states.insert(KeyCode::KeyT, ElementState::Released);
        self.key_states.insert(KeyCode::KeyU, ElementState::Released);
        self.key_states.insert(KeyCode::KeyV, ElementState::Released);
        self.key_states.insert(KeyCode::KeyW, ElementState::Released);
        self.key_states.insert(KeyCode::KeyX, ElementState::Released);
        self.key_states.insert(KeyCode::KeyY, ElementState::Released);
        self.key_states.insert(KeyCode::KeyZ, ElementState::Released);
        self.key_states.insert(KeyCode::Minus, ElementState::Released);
        self.key_states.insert(KeyCode::Period, ElementState::Released);
        self.key_states.insert(KeyCode::Quote, ElementState::Released);
        self.key_states.insert(KeyCode::Semicolon, ElementState::Released);
        self.key_states.insert(KeyCode::Slash, ElementState::Released);
        self.key_states.insert(KeyCode::AltLeft, ElementState::Released);
        self.key_states.insert(KeyCode::AltRight, ElementState::Released);
        self.key_states.insert(KeyCode::Backspace, ElementState::Released);
        self.key_states.insert(KeyCode::CapsLock, ElementState::Released);
        self.key_states.insert(KeyCode::ContextMenu, ElementState::Released);
        self.key_states.insert(KeyCode::ControlLeft, ElementState::Released);
        self.key_states.insert(KeyCode::ControlRight, ElementState::Released);
        self.key_states.insert(KeyCode::Enter, ElementState::Released);
        self.key_states.insert(KeyCode::SuperLeft, ElementState::Released);
        self.key_states.insert(KeyCode::SuperRight, ElementState::Released);
        self.key_states.insert(KeyCode::ShiftLeft, ElementState::Released);
        self.key_states.insert(KeyCode::ShiftRight, ElementState::Released);
        self.key_states.insert(KeyCode::Space, ElementState::Released);
        self.key_states.insert(KeyCode::Tab, ElementState::Released);
        self.key_states.insert(KeyCode::Convert, ElementState::Released);
        self.key_states.insert(KeyCode::KanaMode, ElementState::Released);
        self.key_states.insert(KeyCode::Lang1, ElementState::Released);
        self.key_states.insert(KeyCode::Lang2, ElementState::Released);
        self.key_states.insert(KeyCode::Lang3, ElementState::Released);
        self.key_states.insert(KeyCode::Lang4, ElementState::Released);
        self.key_states.insert(KeyCode::Lang5, ElementState::Released);
        self.key_states.insert(KeyCode::NonConvert, ElementState::Released);
        self.key_states.insert(KeyCode::Delete, ElementState::Released);
        self.key_states.insert(KeyCode::End, ElementState::Released);
        self.key_states.insert(KeyCode::Help, ElementState::Released);
        self.key_states.insert(KeyCode::Home, ElementState::Released);
        self.key_states.insert(KeyCode::Insert, ElementState::Released);
        self.key_states.insert(KeyCode::PageDown, ElementState::Released);
        self.key_states.insert(KeyCode::PageUp, ElementState::Released);
        self.key_states.insert(KeyCode::ArrowDown, ElementState::Released);
        self.key_states.insert(KeyCode::ArrowLeft, ElementState::Released);
        self.key_states.insert(KeyCode::ArrowRight, ElementState::Released);
        self.key_states.insert(KeyCode::ArrowUp, ElementState::Released);
        self.key_states.insert(KeyCode::NumLock, ElementState::Released);
        self.key_states.insert(KeyCode::Numpad0, ElementState::Released);
        self.key_states.insert(KeyCode::Numpad1, ElementState::Released);
        self.key_states.insert(KeyCode::Numpad2, ElementState::Released);
        self.key_states.insert(KeyCode::Numpad3, ElementState::Released);
        self.key_states.insert(KeyCode::Numpad4, ElementState::Released);
        self.key_states.insert(KeyCode::Numpad5, ElementState::Released);
        self.key_states.insert(KeyCode::Numpad6, ElementState::Released);
        self.key_states.insert(KeyCode::Numpad7, ElementState::Released);
        self.key_states.insert(KeyCode::Numpad8, ElementState::Released);
        self.key_states.insert(KeyCode::Numpad9, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadAdd, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadBackspace, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadClear, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadClearEntry, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadComma, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadDecimal, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadDivide, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadEnter, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadEqual, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadHash, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadMemoryAdd, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadMemoryClear, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadMemoryRecall, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadMemoryStore, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadMemorySubtract, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadMultiply, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadParenLeft, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadParenRight, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadStar, ElementState::Released);
        self.key_states.insert(KeyCode::NumpadSubtract, ElementState::Released);
        self.key_states.insert(KeyCode::Escape, ElementState::Released);
        self.key_states.insert(KeyCode::Fn, ElementState::Released);
        self.key_states.insert(KeyCode::FnLock, ElementState::Released);
        self.key_states.insert(KeyCode::PrintScreen, ElementState::Released);
        self.key_states.insert(KeyCode::ScrollLock, ElementState::Released);
        self.key_states.insert(KeyCode::Pause, ElementState::Released);
        self.key_states.insert(KeyCode::BrowserBack, ElementState::Released);
        self.key_states.insert(KeyCode::BrowserFavorites, ElementState::Released);
        self.key_states.insert(KeyCode::BrowserForward, ElementState::Released);
        self.key_states.insert(KeyCode::BrowserHome, ElementState::Released);
        self.key_states.insert(KeyCode::BrowserRefresh, ElementState::Released);
        self.key_states.insert(KeyCode::BrowserSearch, ElementState::Released);
        self.key_states.insert(KeyCode::BrowserStop, ElementState::Released);
        self.key_states.insert(KeyCode::Eject, ElementState::Released);
        self.key_states.insert(KeyCode::LaunchApp1, ElementState::Released);
        self.key_states.insert(KeyCode::LaunchApp2, ElementState::Released);
        self.key_states.insert(KeyCode::LaunchMail, ElementState::Released);
        self.key_states.insert(KeyCode::MediaPlayPause, ElementState::Released);
        self.key_states.insert(KeyCode::MediaSelect, ElementState::Released);
        self.key_states.insert(KeyCode::MediaStop, ElementState::Released);
        self.key_states.insert(KeyCode::MediaTrackNext, ElementState::Released);
        self.key_states.insert(KeyCode::MediaTrackPrevious, ElementState::Released);
        self.key_states.insert(KeyCode::Power, ElementState::Released);
        self.key_states.insert(KeyCode::Sleep, ElementState::Released);
        self.key_states.insert(KeyCode::AudioVolumeDown, ElementState::Released);
        self.key_states.insert(KeyCode::AudioVolumeMute, ElementState::Released);
        self.key_states.insert(KeyCode::AudioVolumeUp, ElementState::Released);
        self.key_states.insert(KeyCode::WakeUp, ElementState::Released);
        // Legacy modifier key. Also called "Super" in certain places.
        self.key_states.insert(KeyCode::Meta, ElementState::Released);
        // Legacy modifier key.
        self.key_states.insert(KeyCode::Hyper, ElementState::Released);
        self.key_states.insert(KeyCode::Turbo, ElementState::Released);
        self.key_states.insert(KeyCode::Abort, ElementState::Released);
        self.key_states.insert(KeyCode::Resume, ElementState::Released);
        self.key_states.insert(KeyCode::Suspend, ElementState::Released);
        self.key_states.insert(KeyCode::Again, ElementState::Released);
        self.key_states.insert(KeyCode::Copy, ElementState::Released);
        self.key_states.insert(KeyCode::Cut, ElementState::Released);
        self.key_states.insert(KeyCode::Find, ElementState::Released);
        self.key_states.insert(KeyCode::Open, ElementState::Released);
        self.key_states.insert(KeyCode::Paste, ElementState::Released);
        self.key_states.insert(KeyCode::Props, ElementState::Released);
        self.key_states.insert(KeyCode::Select, ElementState::Released);
        self.key_states.insert(KeyCode::Undo, ElementState::Released);
        self.key_states.insert(KeyCode::Hiragana, ElementState::Released);
        self.key_states.insert(KeyCode::Katakana, ElementState::Released);
        self.key_states.insert(KeyCode::F1, ElementState::Released);
        self.key_states.insert(KeyCode::F2, ElementState::Released);
        self.key_states.insert(KeyCode::F3, ElementState::Released);
        self.key_states.insert(KeyCode::F4, ElementState::Released);
        self.key_states.insert(KeyCode::F5, ElementState::Released);
        self.key_states.insert(KeyCode::F6, ElementState::Released);
        self.key_states.insert(KeyCode::F7, ElementState::Released);
        self.key_states.insert(KeyCode::F8, ElementState::Released);
        self.key_states.insert(KeyCode::F9, ElementState::Released);
        self.key_states.insert(KeyCode::F10, ElementState::Released);
        self.key_states.insert(KeyCode::F11, ElementState::Released);
        self.key_states.insert(KeyCode::F12, ElementState::Released);
        self.key_states.insert(KeyCode::F13, ElementState::Released);
        self.key_states.insert(KeyCode::F14, ElementState::Released);
        self.key_states.insert(KeyCode::F15, ElementState::Released);
        self.key_states.insert(KeyCode::F16, ElementState::Released);
        self.key_states.insert(KeyCode::F17, ElementState::Released);
        self.key_states.insert(KeyCode::F18, ElementState::Released);
        self.key_states.insert(KeyCode::F19, ElementState::Released);
        self.key_states.insert(KeyCode::F20, ElementState::Released);
        self.key_states.insert(KeyCode::F21, ElementState::Released);
        self.key_states.insert(KeyCode::F22, ElementState::Released);
        self.key_states.insert(KeyCode::F23, ElementState::Released);
        self.key_states.insert(KeyCode::F24, ElementState::Released);
        self.key_states.insert(KeyCode::F25, ElementState::Released);
        self.key_states.insert(KeyCode::F26, ElementState::Released);
        self.key_states.insert(KeyCode::F27, ElementState::Released);
        self.key_states.insert(KeyCode::F28, ElementState::Released);
        self.key_states.insert(KeyCode::F29, ElementState::Released);
        self.key_states.insert(KeyCode::F30, ElementState::Released);
        self.key_states.insert(KeyCode::F31, ElementState::Released);
        self.key_states.insert(KeyCode::F32, ElementState::Released);
        self.key_states.insert(KeyCode::F33, ElementState::Released);
        self.key_states.insert(KeyCode::F34, ElementState::Released);
        self.key_states.insert(KeyCode::F35, ElementState::Released);
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
