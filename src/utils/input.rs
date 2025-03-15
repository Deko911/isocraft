use winit::dpi::PhysicalPosition;
use winit::event::MouseButton::{self, Left, Right};
use winit::keyboard::KeyCode::{self, *};
use winit_input_helper::WinitInputHelper;

use std::collections::HashMap;
use std::hash::Hash;

use InputType::*;

const KEYS: [(KeyCode, InputType); 15] = [
    (ArrowLeft, Held),
    (ArrowRight, Held),
    (ArrowUp, Held),
    (ArrowDown, Held),
    (KeyA, Held),
    (KeyD, Held),
    (KeyW, Held),
    (KeyS, Held),
    (Digit0, Pressed),
    (Digit1, Pressed),
    (Digit2, Pressed),
    (Digit3, Pressed),
    (Digit4, Pressed),
    (Digit5, Pressed),
    (Digit6, Pressed),
];

const MOUSE: [(MouseButton, InputType); 2] = [(Left, Pressed), (Right, Pressed)];

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum InputType {
    Pressed,
    Held,
}

#[derive(Clone)]
pub struct InputHandler {
    pub keys: HashMap<(KeyCode, InputType), bool>,
    pub close: bool,
    mouse: HashMap<(MouseButton, InputType), bool>,
    mouse_pos: PhysicalPosition<f32>,
}

impl InputHandler {
    pub fn new() -> Self {
        let mut keys = HashMap::new();
        let mut mouse = HashMap::new();
        for i in KEYS {
            keys.insert(i, false);
        }
        for i in MOUSE {
            mouse.insert(i, false);
        }
        Self {
            keys,
            close: false,
            mouse,
            mouse_pos: PhysicalPosition::new(0.0, 0.0),
        }
    }

    pub fn update(&mut self, input: &WinitInputHelper) {
        for ((key, typ), value) in self.keys.iter_mut() {
            match typ {
                Pressed => {
                    *value = input.key_pressed(*key);
                }
                Held => {
                    *value = input.key_held(*key);
                }
            }
        }

        for ((button, typ), value) in self.mouse.iter_mut() {
            match typ {
                Pressed => {
                    *value = input.mouse_pressed(*button);
                }
                Held => {
                    *value = input.mouse_held(*button);
                }
            }
        }

        self.mouse_pos = input.cursor().unwrap_or((0.0, 0.0)).into();
    }

    pub fn check_key(&self, key: KeyCode, typ: InputType) -> bool {
        match self.keys.get(&(key, typ)) {
            None => {
                panic!("Tecla no cubierta")
            }
            Some(result) => {
                return *result;
            }
        }
    }

    pub fn check_mouse(&self, button: MouseButton, typ: InputType) -> bool {
        match self.mouse.get(&(button, typ)) {
            None => {
                panic!("Entrada de mouse no cubierta")
            }
            Some(result) => {
                return *result;
            }
        }
    }

    pub fn mouse_pos(&self) -> PhysicalPosition<f32> {
        self.mouse_pos
    }
}
