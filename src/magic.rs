//! Code to specify magic keys and magic rules

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Container for all magic functionality in a layout.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(from = "ParseMagic", into = "ParseMagic")]
pub struct Magic {
    pub(crate) keys: BTreeMap<String, MagicKey>,
}

impl Magic {
    /// Create a new Magic struct based on a list of [`MagicKey`](crate::magic::MagicKey)s.
    pub fn new(magic_keys: &[MagicKey]) -> Self {
        let keys = magic_keys
            .iter()
            .map(|k| (k.label().to_owned(), k.clone()))
            .collect();

        Self { keys }
    }

    /// Get the map of all magic keys on the layout
    pub fn keys(&self) -> &BTreeMap<String, MagicKey> {
        &self.keys
    }

    /// Get all labels on the layout
    pub fn labels(&self) -> impl Iterator<Item = &str> {
        self.keys.keys().map(String::as_str)
    }

    /// Get a particular key based on a label
    pub fn key(&self, label: &str) -> Option<&MagicKey> {
        self.keys.get(label)
    }

    /// Get the amount of keys on the layout.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Returns true if there are no magic keys associated with this layout.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

/// A particular magic key. It has a label, which is the name it has in the `.dof` configuration
/// file. It has a set of rules which dictate the
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct MagicKey {
    pub(crate) label: String,
    pub(crate) rules: BTreeMap<String, String>,
    pub(crate) max_leading_length: usize,
    pub(crate) max_output_length: usize,
}

impl MagicKey {
    /// Creates a new Magic key which has a certain label.
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            rules: Default::default(),
            max_leading_length: Default::default(),
            max_output_length: Default::default(),
        }
    }

    /// Get the amount of rules on the magic key.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Returns true if there are no rules associated with this magic key.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Get the magic key label
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Get the length of the longest leading sequence
    pub fn max_leading_length(&self) -> usize {
        self.max_leading_length
    }

    /// Get the length of the longest magic output sequence
    pub fn max_output_length(&self) -> usize {
        self.max_output_length
    }

    /// Get all rules on this key
    pub fn rules(&self) -> &BTreeMap<String, String> {
        &self.rules
    }

    /// Get all leading sequences
    pub fn leading(&self) -> impl Iterator<Item = &str> {
        self.rules.keys().map(String::as_str)
    }

    /// Get all output sequences
    pub fn outputs(&self) -> impl Iterator<Item = &str> {
        self.rules.values().map(String::as_str)
    }

    /// Get the output of a particular rule, if available
    pub fn rule(&self, rule: &str) -> Option<&str> {
        self.rules.get(rule).map(String::as_str)
    }

    /// Add a magic rule. The first argument specifies the keys leading up to the magic press. The
    /// second is the string that is output after the magic key is pressed.
    pub fn add_rule(&mut self, leading: &str, output: &str) {
        let leading_length = leading.chars().count();
        let output_length = output.chars().count();

        self.max_leading_length = self.max_leading_length.max(leading_length);
        self.max_output_length = self.max_output_length.max(output_length);

        self.rules.insert(leading.to_owned(), output.to_owned());
    }

    /// Remove a rule based on the leading sequence.
    pub fn remove_rule(&mut self, leading: &str) -> Option<String> {
        let rule = self.rules.remove(leading);

        if let Some(output) = &rule {
            if leading.len() == self.max_leading_length() {
                self.max_leading_length =
                    self.rules.keys().map(String::len).max().unwrap_or_default();
            }
            if output.len() == self.max_output_length() {
                self.max_output_length = self
                    .rules
                    .values()
                    .map(String::len)
                    .max()
                    .unwrap_or_default();
            }
        }

        rule
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParseMagicKey(BTreeMap<String, String>);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParseMagic(BTreeMap<String, ParseMagicKey>);

impl From<Magic> for ParseMagic {
    fn from(magic: Magic) -> Self {
        let keys = magic
            .keys
            .into_iter()
            .map(|(label, key)| (label, ParseMagicKey(key.rules)))
            .collect();

        ParseMagic(keys)
    }
}

impl From<ParseMagic> for Magic {
    fn from(pm: ParseMagic) -> Self {
        let keys =
            pm.0.into_iter()
                .map(|(label, pk)| {
                    let mut key = MagicKey::new(&label);

                    pk.0.iter()
                        .for_each(|(lead, output)| key.add_rule(lead, output));

                    (label, key)
                })
                .collect();

        Self { keys }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn parse_magic() {
        let json = r#"
            {
                "mgc": {
                    "a": "b",
                    "abc": "defghijklmnopqrstuvwxyz"
                },
                "mgc2": {
                    "more": " magic"
                }
            }
        "#;
        
        let magic = serde_json::from_str::<Magic>(json)
            .expect("Couldn't convert to magic from str");
        
        let reference = Magic {
            keys: BTreeMap::from([
                (
                    "mgc".into(),
                    MagicKey {
                        label: "mgc".into(),
                        rules: BTreeMap::from([
                            ("a".into(), "b".into()),
                            ("abc".into(), "defghijklmnopqrstuvwxyz".into()),
                        ]),
                        max_leading_length: 3,
                        max_output_length: 23,
                    },
                ),
                (
                    "mgc2".into(),
                    MagicKey {
                        label: "mgc2".into(),
                        rules: BTreeMap::from([("more".into(), " magic".into())]),
                        max_leading_length: 4,
                        max_output_length: 6,
                    },
                ),
            ]
        )};
        
        assert_eq!(magic, reference);
    }
}
