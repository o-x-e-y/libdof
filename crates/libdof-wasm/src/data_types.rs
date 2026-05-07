use libdof::prelude as libdof;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[wasm_bindgen]
pub struct Language(libdof::Language);

#[wasm_bindgen]
impl Language {
    #[wasm_bindgen(getter)]
    pub fn language(&self) -> String {
        self.0.language.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn weight(&self) -> usize {
        self.0.weight
    }
}

impl From<libdof::Language> for Language {
    fn from(language: libdof::Language) -> Self {
        Self(language)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Anchor(libdof::Anchor);

#[wasm_bindgen]
impl Anchor {
    #[wasm_bindgen(getter)]
    pub fn x(&self) -> usize {
        self.0.x()
    }

    #[wasm_bindgen(getter)]
    pub fn y(&self) -> usize {
        self.0.y()
    }
}

impl From<libdof::Anchor> for Anchor {
    fn from(anchor: libdof::Anchor) -> Self {
        Self(anchor)
    }
}