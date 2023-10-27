use std::{sync::Arc, rc::Rc};

use pyo3::{prelude::*, types::PyList};

use crate::{Dof, definitions::KeyboardType};

// #[pymethods]
impl Dof {
    pub fn name(&self) -> &str {
        &self.name
    }

    // pub fn authors(&self) -> Option<PyList> {
    //     self.authors.map(|s| PyList::from(s))
    // }

    // pub fn board(&self) -> PyRef<KeyboardType> {
    //     PyRef::from(self.board)
    // }

    // pub fn year(&self) -> Option<u32> {
    //     self.year
    // }

    // pub fn notes(&self) -> Option<&str> {
    //     self.notes.as_deref()
    // }

    // pub fn layers(&self) -> &BTreeMap<String, Layer> {
    //     &self.layers
    // }

    // pub fn anchor(&self) -> Anchor {
    //     self.anchor
    // }

    // pub fn fingering(&self) -> &Fingering {
    //     &self.fingering
    // }

    // pub fn fingering_name(&self) -> Option<&NamedFingering> {
    //     self.fingering_name.as_ref()
    // }

    // /// This function can be assumed to be infallible if you serialized into Dof as validation
    // /// will have prevented you to create a Dof without a shift layer
    // pub fn main_layer(&self) -> Option<&Layer> {
    //     self.layers.get("main")
    // }

    // /// This function can be assumed to be infallible if you serialized into Dof as validation
    // /// will have prevented you to create a Dof without a shift layer
    // pub fn shift_layer(&self) -> Option<&Layer> {
    //     self.layers.get("shift")
    // }

    // pub fn layer(&self, name: &str) -> Option<&Layer> {
    //     self.layers.get(name)
    // }
}