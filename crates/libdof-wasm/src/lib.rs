mod combos;
mod layer;
mod magic;
mod keyboard;

use std::collections::BTreeMap;
use std::str::FromStr;

use libdof::prelude as libdof;
use libdof::Keyboard as _;
use wasm_bindgen::prelude::*;

use layer::*;
use keyboard::*;
use combos::*;

#[wasm_bindgen]
pub struct Dof(libdof::Dof);

#[wasm_bindgen]
impl Dof {
    /// Parses a fully validated DOF JSON string into a `Dof` object.
    ///
    /// Equivalent to `serde_json::from_str::<Dof>(input)` in Rust. Throws on invalid input.
    #[wasm_bindgen(constructor)]
    pub fn new(input: &str) -> Result<Dof, JsError> {
        let dof = serde_json::from_str(input).map_err(|e| JsError::new(&e.to_string()))?;

        Ok(Dof(dof))
    }

    pub fn name(&self) -> String {
        self.0.name().to_owned()
    }

    pub fn authors(&self) -> Vec<String> {
        self.0.authors().to_vec()
    }

    pub fn year(&self) -> Option<u32> {
        self.0.year()
    }

    pub fn description(&self) -> Option<String> {
        self.0.description().map(str::to_owned)
    }

    pub fn link(&self) -> Option<String> {
        self.0.link().map(str::to_owned)
    }

    /// Returns the form factor as a `FormFactor` value.
    pub fn form_factor(&self) -> JsValue {
        self.0.form_factor().to_string().into()
    }

    /// Returns the named fingering if present, or `undefined`.
    pub fn fingering_name(&self) -> JsValue {
        match self.0.fingering_name() {
            Some(nf) => nf.to_string().into(),
            None => JsValue::undefined(),
        }
    }

    /// Returns the anchor as an `Anchor`.
    pub fn anchor(&self) -> JsValue {
        let anchor = self.0.anchor();
        serde_wasm_bindgen::to_value(&[anchor.x(), anchor.y()]).unwrap()
    }

    /// Returns the layout shape as `number[]` (keys per row).
    pub fn shape(&self) -> JsValue {
        let shape = self.0.shape().into_inner();
        serde_wasm_bindgen::to_value(&shape).unwrap()
    }

    /// Returns the languages as `Language[]`.
    pub fn languages(&self) -> JsValue {
        let langs = self.0.languages().to_vec();
        
        serde_wasm_bindgen::to_value(&langs).unwrap()
    }

    /// Returns the main layer as a `Layer` class instance.
    pub fn main_layer(&self) -> Layer {
        Layer::new(self.0.main_layer().clone())
    }

    /// Returns the shift layer as a `Layer` class instance.
    pub fn shift_layer(&self) -> Layer {
        Layer::new(self.0.shift_layer().clone())
    }

    /// Returns a specific layer by name as a `Layer`, or `undefined`.
    pub fn layer(&self, name: &str) -> Option<Layer> {
        self.0.layer(name).map(|l| Layer::new(l.clone()))
    }

    /// Returns all layers as a plain `Record<string, Key[][]>` for map iteration.
    ///
    /// For typed `Layer` instances, use `main_layer()`, `shift_layer()`, or `layer(name)`.
    pub fn layers(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self.0.layers()).unwrap()
    }
    
    /// Returns the physical keyboard layout as a `PhysicalKey[][]`.
    pub fn board(&self) -> JsValue {
        let rows = self
            .0
            .board()
            .rows()
            .cloned()
            .map(|row| row.into_iter().map(PhysicalKey::from).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        
        serde_wasm_bindgen::to_value(&rows).unwrap()
    }

    /// Returns the fingering as a `Finger[][]` (2D array of finger strings like `"LP"`).
    pub fn fingering(&self) -> JsValue {
        let rows = self
            .0
            .fingering()
            .rows()
            .map(|row| row.iter().map(|f| Finger::from(*f)).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        
        serde_wasm_bindgen::to_value(&rows).unwrap()
    }

    /// Returns the magic keys as a `Magic` class instance.
    pub fn magic(&self) -> magic::Magic {
        magic::Magic(self.0.magic().clone())
    }

    /// Returns the combos as a `Record<string, ComboEntry[]>`.
    pub fn combos(&self) -> JsValue {
        let combos = self.0.combos();
        
        let map = combos
            .0
            .iter()
            .map(|(layer, entries)| {
                let wasm_entries = entries
                    .iter()
                    .map(|(positions, output)| ComboEntry {
                        positions: positions.iter().copied().map(Pos::from).collect(),
                        output: Key::from(output.clone()),
                    })
                    .collect::<Vec<_>>();
                (layer.clone(), wasm_entries)
            })
            .collect::<BTreeMap<_, _>>();
        
        serde_wasm_bindgen::to_value(&map).unwrap()
    }

    /// Returns all keys with metadata as an array of `DescriptiveKey` class instances.
    pub fn keys(&self) -> Vec<DescriptiveKey> {
        self.0.keys()
            .into_iter()
            .map(DescriptiveKey::from)
            .collect()
        
    }

    /// Finds all positions of a given key string (e.g. `"a"`, `"spc"`, `"@sym"`).
    /// Returns a `KeyPos[]`.
    pub fn get(&self, key: &str) -> Vec<KeyPos> {
        let key = libdof::Key::from_str(key).unwrap();
        
        self
            .0
            .get(key)
            .into_iter()
            .map(KeyPos::from)
            .collect()
    }

    /// Returns all keys at position `(row, col)` across all layers as a `Key[]`.
    pub fn tower(&self, row: usize, col: usize) -> JsValue {
        let keys: Vec<Key> = self
            .0
            .tower((row, col))
            .iter()
            .cloned()
            .map(Key::from)
            .collect();
        
        serde_wasm_bindgen::to_value(&keys).unwrap()
    }

    /// Returns the finger assigned to `(row, col)` as a `Finger` string, or `undefined`.
    pub fn finger(&self, row: usize, col: usize) -> Option<Finger> {
        self.0.finger((row, col)).map(Finger::from)
    }

    /// Swaps two keys in place.
    ///
    /// `dof.swap("main", 0, 0, "main", 0, 1)` swaps row 0 col 0 with row 0 col 1 on the main layer.
    pub fn swap(
        &mut self,
        layer1: &str,
        row1: usize,
        col1: usize,
        layer2: &str,
        row2: usize,
        col2: usize,
    ) -> Result<(), JsError> {
        self.0
            .swap(
                (layer1, libdof::Pos::new(row1, col1)),
                (layer2, libdof::Pos::new(row2, col2)),
            )
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Serializes the layout back to a DOF JSON string.
    pub fn serialize(&self) -> Result<String, JsValue> {
        let res = serde_json::to_string(&self.0).map_err(|e| e.to_string())?;

        Ok(res)
    }
}

#[wasm_bindgen]
pub struct DofIntermediate(libdof::DofIntermediate);

#[wasm_bindgen]
impl DofIntermediate {
    /// Parses a fully validated DOF JSON string into a `Dof` object.
    ///
    /// Equivalent to `serde_json::from_str::<Dof>(input)` in Rust. Throws on invalid input.
    #[wasm_bindgen(constructor)]
    pub fn new(input: &str) -> Result<DofIntermediate, JsError> {
        let dof = serde_json::from_str(input)
            .map_err(|e| JsError::new(&e.to_string()))?;

        Ok(DofIntermediate(dof))
    }

    pub fn serialize(&self) -> Result<String, JsValue> {
        let res = serde_json::to_string(&self.0).map_err(|e| e.to_string())?;

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_VALID: &str = include_str!("../../../example_dofs/minimal_valid.dof");
    const MINIMAL_PARSABLE: &str = include_str!("../../../example_dofs/minimal_parsable.dof");
    const MAXIMAL: &str = include_str!("../../../example_dofs/maximal.dof");

    #[test]
    fn parse_minimal_valid() {
        let result = Dof::new(MINIMAL_VALID).unwrap();
        let json = result.serialize().unwrap();
        let obj: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(obj.get("name").is_some());
        assert!(obj.get("fingering").is_some());
    }

    #[test]
    fn parse_maximal() {
        assert!(Dof::new(MAXIMAL).is_ok());
    }

    #[test]
    fn round_trip() {
        let first = Dof::new(MINIMAL_VALID).unwrap();
        let second = Dof::new(&first.serialize().unwrap()).unwrap();
        assert_eq!(first.0, second.0);
    }

    #[test]
    fn parse_intermediate_accepts_minimal_parsable() {
        assert!(DofIntermediate::new(MINIMAL_PARSABLE).is_ok());
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(Dof::new("not json").is_err());
        assert!(DofIntermediate::new("not json").is_err());
    }

    #[test]
    fn rejects_incomplete_dof() {
        assert!(Dof::new(r#"{"name":"test"}"#).is_err());
    }
}
