use std::str::FromStr;

use libdof::Keyboard as _;
use libdof::prelude as libdof;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::combos::*;
use crate::keyboard::*;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Key {
    kind: String,
    value: String,
}

#[wasm_bindgen]
impl Key {
    #[wasm_bindgen(constructor)]
    pub fn new(key: &str) -> Self {
        libdof::Key::from_str(key).unwrap().into()
    }

    /// Check if the key is of type [`Key::Char`](crate::dofinitions::Key::Char) which outputs
    /// a single character.
    pub fn is_char(&self) -> bool {
        libdof::Key::from(self.clone()).is_char()
    }

    /// Check if the key is of type [`Key::Word`](crate::dofinitions::Key::Word) which outputs a specific
    /// string.
    pub fn is_word(&self) -> bool {
        libdof::Key::from(self.clone()).is_word()
    }

    /// Check if the key is of type [`Key::Empty`](crate::dofinitions::Key::Empty) which doesn't output
    /// anything.
    pub fn is_empty(&self) -> bool {
        libdof::Key::from(self.clone()).is_empty()
    }

    /// Check if the key is of type [`Key::Transparent`](crate::dofinitions::Key::Char) which outputs
    /// whatever it is the main layer outputs in that position.
    pub fn is_transparent(&self) -> bool {
        libdof::Key::from(self.clone()).is_transparent()
    }

    /// Check if the key is of type [`Key::Layer`](crate::dofinitions::Key::Layer), which holds the
    /// name of a particular layer on the layout
    pub fn is_layer(&self) -> bool {
        libdof::Key::from(self.clone()).is_layer()
    }

    /// Check if the key is of type [`Key::Magic`](crate::dofinitions::Key::Magic) which holds the
    /// name of a particular magic key on the layout
    pub fn is_magic(&self) -> bool {
        libdof::Key::from(self.clone()).is_magic()
    }

    /// Get the output if the key is of type [`Key::Char`](crate::dofinitions::Key::Char).
    pub fn char_output(&self) -> Option<char> {
        libdof::Key::from(self.clone()).char_output()
    }

    /// Get the output if the key is of type [`Key::Word`](crate::dofinitions::Key::Word).
    pub fn word_output(&self) -> Option<String> {
        libdof::Key::from(self.clone())
            .word_output()
            .map(ToString::to_string)
    }

    /// Get the layer label if the key is of type [`Key::Layer`](crate::dofinitions::Key::Layer).
    pub fn layer_label(&self) -> Option<String> {
        libdof::Key::from(self.clone())
            .layer_label()
            .map(ToString::to_string)
    }

    /// Get the magic key label if the key is of type [`Key::Magic`](crate::dofinitions::Key::Magic).
    pub fn magic_label(&self) -> Option<String> {
        libdof::Key::from(self.clone())
            .magic_label()
            .map(ToString::to_string)
    }
}

impl From<libdof::Key> for Key {
    fn from(key: libdof::Key) -> Self {
        use libdof::Key::*;

        let kind_s = match key {
            Empty => "Empty",
            Transparent => "Transparent",
            Char(_) => "Char",
            Word(_) => "Word",
            Special(_) => "Special",
            Layer { .. } => "Layer",
            Magic { .. } => "Magic",
        };

        let kind = kind_s.to_string();
        let value = key.to_string();

        Self { kind, value }
    }
}

impl From<Key> for libdof::Key {
    fn from(key: Key) -> Self {
        use libdof::Key::*;

        match key.kind.as_str() {
            "Empty" => Empty,
            "Transparent" => Transparent,
            "Char" | "Word" | "Special" | "Layer" | "Magic" => {
                libdof::Key::from_str(&key.value).unwrap()
            }
            _ => Empty,
        }
    }
}

#[wasm_bindgen]
pub struct DescriptiveKey {
    output: Key,
    layer: String,
    pos: Pos,
    finger: Finger,
    phys: PhysicalKey,
}

#[wasm_bindgen]
impl DescriptiveKey {
    /// Get the [`KeyPos`](crate::interaction::KeyPos) of a certain key, containing the layer name as well
    /// its row and column coordinates.
    pub fn keypos(&self) -> KeyPos {
        KeyPos::new(self.layer.clone(), self.pos)
    }

    /// Get the key's row and column.
    pub fn pos(&self) -> Pos {
        self.pos
    }

    /// Get the key's row.
    pub fn row(&self) -> usize {
        self.pos.row()
    }

    /// Get the key's column.
    pub fn col(&self) -> usize {
        self.pos.col()
    }

    /// Get the finger the key is supposed to be pressed with.
    pub fn finger(&self) -> Finger {
        self.finger
    }

    /// Get the key's output.
    pub fn output(&self) -> Key {
        self.output.clone()
    }

    /// Get the key's physical location
    pub fn physical_pos(&self) -> PhysicalKey {
        self.phys
    }

    /// Get the name of the layer of the key.
    pub fn layer_name(&self) -> String {
        self.layer.clone()
    }

    /// Check if the key is on a certain finger.
    pub fn is_on_finger(&self, finger: Finger) -> bool {
        (self.finger as u8) == (finger as u8)
    }

    /// Check if the key is on any of the provided fingers.
    pub fn is_on_fingers(&self, fingers: Vec<Finger>) -> bool {
        fingers.contains(&self.finger)
    }

    /// Check if the key is on left hand, including left thumb.
    pub fn is_on_left_hand(&self) -> bool {
        libdof::Finger::from(self.finger).is_on_left_hand()
    }

    /// Check if the key is on left hand, including left thumb.
    pub fn is_on_right_hand(&self) -> bool {
        libdof::Finger::from(self.finger).is_on_right_hand()
    }

    /// Check if the key is on a specific layer.
    pub fn is_on_layer(&self, layer: &str) -> bool {
        self.layer == layer
    }

    /// Check if the key is of type [`Key::Char`](crate::dofinitions::Key::Char) which outputs
    /// a single character.
    pub fn is_char_key(&self) -> bool {
        self.output.is_char()
    }

    /// Check if the key is of type [`Key::Word`](crate::dofinitions::Key::Word) which outputs a specific
    /// string.
    pub fn is_word_key(&self) -> bool {
        self.output.is_word()
    }

    /// Check if the key is of type [`Key::Empty`](crate::dofinitions::Key::Empty) which doesn't output
    /// anything.
    pub fn is_empty_key(&self) -> bool {
        self.output.is_empty()
    }

    /// Check if the key is of type [`Key::Transparent`](crate::dofinitions::Key::Char) which outputs
    /// whatever it is the main layer outputs in that position.
    pub fn is_transparent_key(&self) -> bool {
        self.output.is_transparent()
    }

    /// Check if the key is of type [`Key::Layer`](crate::dofinitions::Key::Layer) which holds the name.
    /// of a layer on the layout
    pub fn is_layer_key(&self) -> bool {
        self.output.is_layer()
    }

    /// Get the output if the key is of type [`Key::Char`](crate::dofinitions::Key::Char).
    pub fn char_output(&self) -> Option<char> {
        self.output.char_output()
    }

    /// Get the output if the key is of type [`Key::Word`](crate::dofinitions::Key::Word).
    pub fn word_output(&self) -> Option<String> {
        self.output.word_output()
    }

    /// Get the layer name if the key is of type [`Key::Layer`](crate::dofinitions::Key::Layer).
    pub fn layer_output(&self) -> Option<String> {
        self.output.layer_label()
    }

    /// Get the magic key label if the key is of type
    /// [`Key::Magic`](crate::dofinitions::Key::Magic).
    pub fn magic_label(&self) -> Option<String> {
        self.output.magic_label()
    }
}

impl From<libdof::DescriptiveKey<'_>> for DescriptiveKey {
    fn from(key: libdof::DescriptiveKey) -> Self {
        let output = key.output().clone().into();
        let layer = key.layer_name().to_string();
        let pos = key.pos().into();
        let finger = key.finger().into();
        let phys = key.physical_pos().clone().into();

        Self {
            output,
            layer,
            pos,
            finger,
            phys,
        }
    }
}

#[wasm_bindgen]
pub struct Layer(libdof::Layer);

impl Layer {
    pub fn new(layer: libdof::Layer) -> Self {
        Self(layer)
    }
}

#[wasm_bindgen]
impl Layer {
    /// Returns all rows as a `Key[][]`.
    pub fn rows(&self) -> JsValue {
        let rows = self
            .0
            .rows()
            .cloned()
            .map(|row| row.into_iter().map(Key::from).collect::<Vec<_>>())
            .collect::<Vec<_>>();

        serde_wasm_bindgen::to_value(&rows).unwrap()
    }

    /// Returns all keys in row-major order as a `Key[]`.
    pub fn keys(&self) -> Vec<Key> {
        self.0.keys().cloned().map(Key::from).collect::<Vec<_>>()
    }

    /// Returns the number of keys per row as an array of integers.
    pub fn shape(&self) -> Vec<usize> {
        self.0.shape().into_inner()
    }

    pub fn row_count(&self) -> usize {
        self.0.row_count()
    }

    /// Returns the key at `(row, col)` as a `Key`, or `undefined` if out of bounds.
    pub fn get_key(&self, row: usize, col: usize) -> JsValue {
        match self.0.get_key(libdof::Pos::new(row, col)) {
            Ok(key) => serde_wasm_bindgen::to_value(&Key::from(key.clone())).unwrap(),
            Err(_) => JsValue::undefined(),
        }
    }

    /// Sets the key at `(row, col)`. Parses `key` using the same rules as the `.dof` format.
    pub fn set_key(&mut self, row: usize, col: usize, key: &str) -> Result<(), JsError> {
        let k = libdof::Key::from(key);
        self.0
            .set_key(libdof::Pos::new(row, col), k)
            .map_err(|e| JsError::new(&e.to_string()))
    }
}
