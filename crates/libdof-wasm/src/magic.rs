use std::collections::BTreeMap;

use libdof::magic;
use serde::Serialize;
use wasm_bindgen::prelude::*;

// #[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, Tsify)]
// #[tsify(into_wasm_abi, from_wasm_abi)]
// pub enum Finger {
//     LP,
//     LR,
//     LM,
//     LI,
//     LT,
//     RT,
//     RI,
//     RM,
//     RR,
//     RP,
// }

#[wasm_bindgen]
pub struct Magic(pub(crate) magic::Magic);

#[wasm_bindgen]
impl Magic {
    /// Returns all magic key labels.
    pub fn labels(&self) -> Vec<String> {
        self.0.labels().map(str::to_owned).collect()
    }

    /// Returns a `MagicKey` for the given label, or `undefined`.
    pub fn key(&self, label: &str) -> Option<MagicKey> {
        self.0.key(label).map(|k| MagicKey(k.clone()))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns all magic keys as a `Record<string, { label: string, rules: Record<string, string> }>`.
    pub fn keys(&self) -> JsValue {
        #[derive(Serialize)]
        struct MagicKeyData<'a> {
            label: &'a str,
            rules: &'a BTreeMap<String, String>,
        }

        let map = self
            .0
            .keys()
            .iter()
            .map(|(label, key)| {
                (
                    label.clone(),
                    MagicKeyData {
                        label: key.label(),
                        rules: key.rules(),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();

        serde_wasm_bindgen::to_value(&map).unwrap()
    }
}

#[wasm_bindgen]
pub struct MagicKey(pub(crate) magic::MagicKey);

#[wasm_bindgen]
impl MagicKey {
    pub fn label(&self) -> String {
        self.0.label().to_owned()
    }

    /// Returns all rules as a `Record<string, string>`.
    pub fn rules(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.0.rules()).unwrap()
    }

    pub fn leading(&self) -> Vec<String> {
        self.0.leading().map(str::to_owned).collect()
    }

    pub fn outputs(&self) -> Vec<String> {
        self.0.outputs().map(str::to_owned).collect()
    }

    pub fn rule(&self, rule: &str) -> Option<String> {
        self.0.rule(rule).map(str::to_owned)
    }

    pub fn add_rule(&mut self, leading: &str, output: &str) {
        self.0.add_rule(leading, output)
    }

    pub fn remove_rule(&mut self, leading: &str) -> Option<String> {
        self.0.remove_rule(leading)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
