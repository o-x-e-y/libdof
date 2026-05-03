use libdof::dofinitions;
use libdof::prelude as dof;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[wasm_bindgen]
pub enum Finger {
    /// Left Pinky
    LP,
    /// Left Ring
    LR,
    /// Left Middle
    LM,
    /// Left Index
    LI,
    /// Left Thumb
    LT,
    /// Right Thumb
    RT,
    /// Right Index
    RI,
    /// Right Middle
    RM,
    /// Right Ring
    RR,
    /// Right Pinky
    RP,
}

impl From<dof::Finger> for Finger {
    fn from(f: dof::Finger) -> Self {
        match f {
            dof::Finger::LP => Finger::LP,
            dof::Finger::LR => Finger::LR,
            dof::Finger::LM => Finger::LM,
            dof::Finger::LI => Finger::LI,
            dof::Finger::LT => Finger::LT,
            dof::Finger::RT => Finger::RT,
            dof::Finger::RI => Finger::RI,
            dof::Finger::RM => Finger::RM,
            dof::Finger::RR => Finger::RR,
            dof::Finger::RP => Finger::RP,
        }
    }
}

impl From<Finger> for dof::Finger {
    fn from(f: Finger) -> Self {
        match f {
            Finger::LP => dof::Finger::LP,
            Finger::LR => dof::Finger::LR,
            Finger::LM => dof::Finger::LM,
            Finger::LI => dof::Finger::LI,
            Finger::LT => dof::Finger::LT,
            Finger::RT => dof::Finger::RT,
            Finger::RI => dof::Finger::RI,
            Finger::RM => dof::Finger::RM,
            Finger::RR => dof::Finger::RR,
            Finger::RP => dof::Finger::RP,
        }
    }
}

/// Enum to specify both hands. Used in combination with [`Finger`](libdof::dofinitions::Finger).
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[wasm_bindgen]
pub enum Hand {
    /// Left hand
    Left,
    /// Right hand
    Right,
}

impl From<dofinitions::Hand> for Hand {
    fn from(h: dofinitions::Hand) -> Self {
        match h {
            dofinitions::Hand::Left => Hand::Left,
            dofinitions::Hand::Right => Hand::Right,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct PhysicalKey {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl From<dof::PhysicalKey> for PhysicalKey {
    fn from(key: dof::PhysicalKey) -> Self {
        Self {
            x: key.x(),
            y: key.y(),
            width: key.width(),
            height: key.height(),
        }
    }
}
