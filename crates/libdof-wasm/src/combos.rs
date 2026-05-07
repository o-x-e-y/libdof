use libdof::prelude as dof;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Represents a (row, column) position on a keyboard. Can be created by `(num, num).into()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Pos {
    pub row: usize,
    pub col: usize,
}

#[wasm_bindgen]
impl Pos {
    /// Create a new position based on a layer's row and column indices.
    #[wasm_bindgen(constructor)]
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

impl From<dof::Pos> for Pos {
    fn from(pos: dof::Pos) -> Self {
        Self {
            row: pos.row(),
            col: pos.col(),
        }
    }
}

/// Represents a layer name along with a row and column on a keyboard. Can also be created by `(name, Pos).into()`
/// or `(name, (row, col)).into()`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct KeyPos {
    /// Name of the layer the keypos refers to
    layer: String,
    /// Row, col position of the key on the keyboard.
    pos: Pos,
}

#[wasm_bindgen]
impl KeyPos {
    #[wasm_bindgen(constructor)]
    pub fn new(layer: String, pos: Pos) -> Self {
        Self { layer, pos }
    }

    pub fn layer(&self) -> String {
        self.layer.clone()
    }

    pub fn pos(&self) -> Pos {
        self.pos.clone()
    }
}

impl From<dof::KeyPos> for KeyPos {
    fn from(kp: dof::KeyPos) -> Self {
        Self {
            layer: kp.layer.into(),
            pos: kp.pos.into(),
        }
    }
}

use crate::layer::Key;

#[derive(Clone, Debug, Serialize)]
pub struct ComboEntry {
    pub positions: Vec<Pos>,
    pub output: Key,
}
