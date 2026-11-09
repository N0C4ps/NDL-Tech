use std::collections::HashSet;

use super::keyboard::KeyCode;
use super::modifiers::Modifiers;
use super::mouse::MouseButton;

/// Snapshot acumulado do estado de input, construído a partir dos eventos
/// de [`crate::event`] que o código do jogo escolher repassar para ele.
///
/// `InputState` não observa o sistema operacional por conta própria — é uma
/// conveniência opcional: o jogo decide se quer manter esse snapshot ou
/// tratar os eventos diretamente.
#[derive(Debug, Clone, Default)]
pub struct InputState {
    pressed_keys: HashSet<KeyCode>,
    pressed_buttons: HashSet<MouseButton>,
    cursor_position: (f64, f64),
    modifiers: Modifiers,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn press_key(&mut self, key: KeyCode) {
        self.pressed_keys.insert(key);
    }

    pub fn release_key(&mut self, key: KeyCode) {
        self.pressed_keys.remove(&key);
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn press_button(&mut self, button: MouseButton) {
        self.pressed_buttons.insert(button);
    }

    pub fn release_button(&mut self, button: MouseButton) {
        self.pressed_buttons.remove(&button);
    }

    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    pub fn set_cursor_position(&mut self, position: (f64, f64)) {
        self.cursor_position = position;
    }

    pub fn cursor_position(&self) -> (f64, f64) {
        self.cursor_position
    }

    pub fn set_modifiers(&mut self, modifiers: Modifiers) {
        self.modifiers = modifiers;
    }

    pub fn modifiers(&self) -> Modifiers {
        self.modifiers
    }
}
