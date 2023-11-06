//! Contains some structs and functions that are used when interacting with the layout, like swapping two keys.

use thiserror::Error;

use crate::{
    dofinitions::{Finger, Key},
    Dof,
};

/// Represents a (row, column) position on a keyboard. Can be created by `(num, num).into()`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pos {
    row: usize,
    col: usize,
}

impl From<(usize, usize)> for Pos {
    fn from((row, col): (usize, usize)) -> Self {
        Pos { row, col }
    }
}

/// Represents a layer name along with a row and column on a keyboard. Can also be created by `(name, Pos).into()`
/// or `(name, (row, col)).into()`.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyPos {
    pub layer: String,
    pub pos: Pos,
}

impl KeyPos {
    pub fn new(layer: &str, pos: Pos) -> Self {
        let layer = layer.into();
        Self { layer, pos }
    }
}

impl From<(&str, Pos)> for KeyPos {
    fn from((layer, pos): (&str, Pos)) -> Self {
        let layer = layer.into();
        KeyPos { layer, pos }
    }
}

impl From<(&str, (usize, usize))> for KeyPos {
    fn from((layer, pos): (&str, (usize, usize))) -> Self {
        KeyPos::new(layer, pos.into())
    }
}

/// Error subtype of the [`interaction`](crate::interaction) module. Can be converted into `DofError` seamlessly
/// using the `?` operator.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum DofInteractionError {
    #[error("the provided layer name '{0}' is invalid")]
    LayerDoesntExist(String),
    #[error("the given position ({0}, {1}) is not available on the keyboard")]
    InvalidPosition(u8, u8),
}

use DofInteractionError as DIErr;

impl Dof {
    /// Get every `KeyPos` that matches the given key. This can be multiple keys.
    pub fn get(&self, key: impl Into<Key>) -> Vec<KeyPos> {
        let key = key.into();

        self.keys()
            .into_iter()
            .filter(|dk| dk.output == key)
            .map(|dk| dk.keypos())
            .collect::<Vec<_>>()
    }

    /// Get all keys for a given `Pos`, one for each layer. 
    pub fn tower(&self, pos: impl Into<Pos>) -> Vec<Key> {
        let pos = pos.into();

        self.keys()
            .into_iter()
            .filter(|dk| dk.pos() == pos)
            .map(|dk| dk.output.clone())
            .collect::<Vec<_>>()
    }

    /// Get the finger a key on a certain `Pos` is pressed with.
    pub fn finger(&self, pos: impl Into<Pos>) -> Option<Finger> {
        let Pos { row, col } = pos.into();

        self.fingering()
            .rows()
            .nth(row)?
            .into_iter()
            .nth(col)
            .copied()
    }

    /// Swaps two keys on a layout, provided the `KeyPos`es provided are valid. Useful for what it does,
    /// but using this where performance is even remotely important is _strongly discouraged_.
    pub fn swap(
        &mut self,
        keypos1: impl Into<KeyPos>,
        keypos2: impl Into<KeyPos>,
    ) -> Result<(), DofInteractionError> {
        let KeyPos {
            layer: layer_name1,
            pos: pos1,
        } = keypos1.into();

        let KeyPos {
            layer: layer_name2,
            pos: pos2,
        } = keypos2.into();

        if layer_name1 == layer_name2 {
            if pos1 == pos2 {
                return Ok(());
            }

            let layer = self
                .layers
                .remove(&layer_name1)
                .ok_or(DIErr::LayerDoesntExist(layer_name1.clone()))?;

            let char1 = layer
                .0
                .get(pos1.row)
                .ok_or(DIErr::InvalidPosition(pos1.row as u8, pos1.col as u8))?
                .get(pos1.col)
                .ok_or(DIErr::InvalidPosition(pos1.row as u8, pos1.col as u8))?;

            let char2 = layer
                .0
                .get(pos2.row)
                .ok_or(DIErr::InvalidPosition(pos2.row as u8, pos2.col as u8))?
                .get(pos2.col)
                .ok_or(DIErr::InvalidPosition(pos2.row as u8, pos2.col as u8))?;

            let char1 = char1 as *const _ as *mut Key;
            let char2 = char2 as *const _ as *mut Key;

            unsafe {
                std::ptr::swap(char1, char2);
            }

            self.layers.insert(layer_name1.clone(), layer);
        } else {
            let mut layer1 = self
                .layers
                .remove(&layer_name1)
                .ok_or(DIErr::LayerDoesntExist(layer_name1.clone()))?;

            let mut layer2 = self
                .layers
                .remove(&layer_name2)
                .ok_or(DIErr::LayerDoesntExist(layer_name2.clone()))?;

            let char1 = layer1
                .0
                .get_mut(pos1.row)
                .ok_or(DIErr::InvalidPosition(pos1.row as u8, pos1.col as u8))?
                .get_mut(pos1.col)
                .ok_or(DIErr::InvalidPosition(pos1.row as u8, pos1.col as u8))?;

            let char2 = layer2
                .0
                .get_mut(pos2.row)
                .ok_or(DIErr::InvalidPosition(pos2.row as u8, pos2.col as u8))?
                .get_mut(pos2.col)
                .ok_or(DIErr::InvalidPosition(pos2.row as u8, pos2.col as u8))?;

            std::mem::swap(char1, char2);

            self.layers.insert(layer_name1, layer1);
            self.layers.insert(layer_name2, layer2);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static MINIMAL: &str = include_str!("../example_dofs/minimal_valid.dof");

    #[test]
    fn get() {
        let buggy = include_str!("../example_dofs/buggy.dof");
        let buggy_json = serde_json::from_str::<Dof>(buggy).expect("couldn't parse json");

        assert_eq!(buggy_json.get(Key::Char('a')), [("main", (1, 5)).into()]);
        assert_eq!(
            buggy_json.get(Key::Transparent),
            [
                ("l2", (2, 0)).into(),
                ("l2s", (2, 0)).into(),
                ("l2s", (2, 2)).into(),
                ("shift", (2, 2)).into()
            ]
        );
    }

    #[test]
    fn swap_main_layer_same_row() {
        let minimal_json = serde_json::from_str::<Dof>(MINIMAL).expect("couldn't parse json");

        let mut minimal_clone = minimal_json.clone();

        let swap1 = KeyPos::new("main", (0, 0).into());
        let swap2 = KeyPos::new("main", (0, 9).into());

        minimal_clone
            .swap(swap1.clone(), swap2.clone())
            .expect("couldn't swap because");

        minimal_clone
            .swap(swap1, swap2)
            .expect("couldn't swap because");

        assert_eq!(minimal_json, minimal_clone);
    }

    #[test]
    fn swap_main_layer() {
        let minimal_json = serde_json::from_str::<Dof>(MINIMAL).expect("couldn't parse json");

        let mut minimal_clone = minimal_json.clone();

        let swap1 = KeyPos::new("main", (2, 0).into());
        let swap2 = KeyPos::new("main", (1, 10).into());

        minimal_clone
            .swap(swap1.clone(), swap2.clone())
            .expect("couldn't swap because");

        minimal_clone
            .swap(swap1, swap2)
            .expect("couldn't swap because");

        assert_eq!(minimal_json, minimal_clone);
    }

    #[test]
    fn swap_different_layers() {
        let minimal_json = serde_json::from_str::<Dof>(MINIMAL).expect("couldn't parse json");

        let mut minimal_clone = minimal_json.clone();

        let swap1 = KeyPos::new("shift", (2, 0).into());
        let swap2 = KeyPos::new("main", (1, 10).into());

        minimal_clone
            .swap(swap1.clone(), swap2.clone())
            .expect("couldn't swap because");

        minimal_clone
            .swap(swap1, swap2)
            .expect("couldn't swap because");

        assert_eq!(minimal_json, minimal_clone);
    }
}
