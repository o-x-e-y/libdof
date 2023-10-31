pub mod definitions;
pub mod interact_dof;
pub mod macros;

use interact_dof::{KeyPos, Pos};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, serde_conv, skip_serializing_none, DisplayFromStr};
use thiserror::Error;

use std::collections::BTreeMap;

use definitions::*;

#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "DofIntermediate", into = "DofIntermediate")]
pub struct Dof {
    name: String,
    authors: Option<Vec<String>>,
    #[serde_as(as = "DisplayFromStr")]
    board: KeyboardType,
    year: Option<u32>,
    description: Option<String>,
    link: Option<String>,
    layers: BTreeMap<String, Layer>,
    #[serde(default = "Anchor::default")]
    anchor: Anchor,
    // alt_fingerings: Option<Vec<String>>,
    // combos: Option<HashMap<String, String>>,
    fingering: Fingering,
    #[serde_as(as = "Option<DisplayFromStr>")]
    fingering_name: Option<NamedFingering>,
    has_generated_shift: bool,
    keys: Vec<DescriptiveKey>,
}

impl PartialEq for Dof {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.authors == other.authors
            && self.board == other.board
            && self.year == other.year
            && self.description == other.description
            && self.layers == other.layers
            && self.anchor == other.anchor
            && self.fingering == other.fingering
            && self.fingering_name == other.fingering_name
            && self.has_generated_shift == other.has_generated_shift
    }
}

impl Dof {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn authors(&self) -> Option<&Vec<String>> {
        self.authors.as_ref()
    }

    pub fn board(&self) -> &KeyboardType {
        &self.board
    }

    pub fn year(&self) -> Option<u32> {
        self.year
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn layers(&self) -> &BTreeMap<String, Layer> {
        &self.layers
    }

    pub fn anchor(&self) -> Anchor {
        self.anchor
    }

    pub fn fingering(&self) -> &Fingering {
        &self.fingering
    }

    pub fn fingering_name(&self) -> Option<&NamedFingering> {
        self.fingering_name.as_ref()
    }

    /// This function can be assumed to be infallible if you serialized into Dof as validation
    /// will have prevented you to create a Dof without a shift layer
    pub fn main_layer(&self) -> Option<&Layer> {
        self.layers.get("main")
    }

    /// This function can be assumed to be infallible if you serialized into Dof as validation
    /// will have prevented you to create a Dof without a shift layer
    pub fn shift_layer(&self) -> Option<&Layer> {
        self.layers.get("shift")
    }

    pub fn layer(&self, name: &str) -> Option<&Layer> {
        self.layers.get(name)
    }

    pub fn keys(&self) -> impl Iterator<Item = &DescriptiveKey> {
        self.keys.iter()
    }
}

impl TryFrom<DofIntermediate> for Dof {
    type Error = DofError;

    fn try_from(mut inter: DofIntermediate) -> Result<Self, Self::Error> {
        let main_layer = inter.main_layer()?;

        inter.validate_layers(main_layer)?;
        inter.validate_layer_shapes(main_layer)?;

        let explicit_fingering = inter.explicit_fingering(main_layer)?;
        let implicit_fingering = match inter.fingering.clone() {
            ParsedFingering::Implicit(f) => Some(f),
            _ => None,
        };

        let has_generated_shift = if inter.layers.get("shift").is_none() {
            inter.layers.insert(
                "shift".into(),
                DofIntermediate::generate_shift_layer(main_layer),
            );
            true
        } else {
            false
        };

        if inter.board.is_custom() {
            inter.anchor = Anchor(0, 0);
        }

        let mut keys = Vec::<DescriptiveKey>::new();

        for (name, layer) in inter.layers.iter() {
            for (i, row) in layer.0.iter().enumerate() {
                for (j, key) in row.iter().enumerate() {
                    let finger = explicit_fingering.0[i][j];

                    let i = i + inter.anchor.0 as usize;
                    let j = j + inter.anchor.1 as usize;

                    let key = DescriptiveKey::new(key.clone(), name.into(), i, j, finger);

                    keys.push(key);
                }
            }
        }

        Ok(Self {
            name: inter.name,
            authors: inter.authors,
            board: inter.board,
            year: inter.year,
            description: inter.description,
            link: inter.link,
            layers: inter.layers,
            anchor: inter.anchor,
            fingering: explicit_fingering,
            fingering_name: implicit_fingering,
            has_generated_shift,
            keys,
        })
    }
}

impl Into<DofIntermediate> for Dof {
    fn into(mut self) -> DofIntermediate {
        if self.has_generated_shift {
            self.layers.remove("shift");
        }
        if let Some(fingering_name) = self.fingering_name {
            DofIntermediate {
                name: self.name,
                authors: self.authors,
                board: self.board,
                year: self.year,
                description: self.description,
                link: self.link,
                layers: self.layers,
                anchor: self.anchor,
                fingering: ParsedFingering::Implicit(fingering_name),
            }
        } else {
            DofIntermediate {
                name: self.name,
                authors: self.authors,
                board: self.board,
                year: self.year,
                description: self.description,
                link: self.link,
                layers: self.layers,
                anchor: self.anchor,
                fingering: ParsedFingering::Explicit(self.fingering),
            }
        }
    }
}

#[derive(Debug, Error, PartialEq)]
enum DofErrorInner {
    #[error("couldn't parse fingering")]
    DefinitionError(#[from] definitions::DefinitionError),
    #[error("The keyboard type '{0:?}' does not have an anchor at this time")]
    UnavailableKeyboardAnchor(KeyboardType),
    #[error("This layout is missing a main layer")]
    NoMainLayer,
    #[error("Found these layer keys '{0:?}' however these layers do not actually exist")]
    LayersNotFound(Vec<String>),
    #[error("The shape of these layers: '{0:?}' are not the same as the main layer")]
    IncompatibleLayerShapes(Vec<String>),
    #[error("The layer shapes do not match the fingering shape")]
    IncompatibleFingeringShape,
    #[error("The provided layout + anchor doesn't fit in the given fingering")]
    LayoutDoesntFit,
}

use DofErrorInner::*;

#[derive(Debug, Error, PartialEq)]
#[error("{0}")]
pub struct DofError(#[source] Box<DofErrorInner>);

impl From<DofErrorInner> for DofError {
    fn from(value: DofErrorInner) -> Self {
        Self(Box::new(value))
    }
}

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fingering(#[serde_as(as = "Vec<FingeringStrAsRow>")] Vec<Vec<Finger>>);

impl_keyboard!(Fingering, Finger, FingeringStrAsRow);

impl Fingering {
    pub fn resized_fingering(
        &self,
        Anchor(x, y): Anchor,
        desired_shape: Shape,
    ) -> Result<Fingering, DofError> {
        let (x, y) = (x as usize, y as usize);

        if y + desired_shape.row_count() < self.0.len() {
            let y_range = y..(y + desired_shape.row_count());

            self.0[y_range]
                .into_iter()
                .zip(desired_shape.into_inner())
                .map(|(row, len)| {
                    let row = &row[x..(x + len)];

                    match row.len() >= len {
                        true => Ok(row.to_vec()),
                        false => Err(LayoutDoesntFit.into()),
                    }
                })
                .collect::<Result<Vec<_>, DofError>>()
                .map(Into::into)
        } else {
            Err(LayoutDoesntFit.into())
        }
    }
}

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParsedFingering {
    Explicit(Fingering),
    Implicit(#[serde_as(as = "DisplayFromStr")] NamedFingering),
}

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer(#[serde_as(as = "Vec<LayerStrAsRow>")] Vec<Vec<Key>>);

impl_keyboard!(Layer, Key, LayerStrAsRow);

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Anchor(u8, u8);

impl Default for Anchor {
    fn default() -> Self {
        Self(1, 1)
    }
}

impl Anchor {
    pub fn new(x: u8, y: u8) -> Self {
        Anchor(x, y)
    }
}

impl TryFrom<KeyboardType> for Anchor {
    type Error = DofError;

    fn try_from(value: KeyboardType) -> Result<Self, Self::Error> {
        match value {
            KeyboardType::Ansi => Ok(Anchor::new(1, 1)),
            KeyboardType::Iso => Ok(Anchor::new(1, 1)),
            KeyboardType::Ortho => Ok(Anchor::new(0, 0)),
            KeyboardType::Colstag => Ok(Anchor::new(0, 0)),
            KeyboardType::Custom(_) => Err(UnavailableKeyboardAnchor(value).into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DescriptiveKey {
    output: Key,
    layer: String,
    row: usize,
    col: usize,
    finger: Finger,
}

impl DescriptiveKey {
    pub fn new(output: Key, layer: String, row: usize, col: usize, finger: Finger) -> Self {
        Self {
            output,
            layer,
            row,
            col,
            finger,
        }
    }

    pub fn is_left_hand(&self) -> bool {
        use Finger::*;

        matches!(self.finger, LP | LR | LM | LI | LT)
    }

    pub fn pos(&self) -> Pos {
        (self.row, self.col).into()
    }

    pub fn keypos<'a>(&'a self) -> KeyPos<'a> {
        (self.layer.as_str(), (self.row, self.col)).into()
    }
}

/// Main struct to use for parsing
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct DofIntermediate {
    name: String,
    authors: Option<Vec<String>>,
    #[serde_as(as = "DisplayFromStr")]
    board: KeyboardType,
    year: Option<u32>,
    description: Option<String>,
    link: Option<String>,
    layers: BTreeMap<String, Layer>,
    #[serde(default = "Anchor::default")]
    anchor: Anchor,
    // alt_fingerings: Option<Vec<String>>,
    // combos: Option<HashMap<String, String>>,
    fingering: ParsedFingering,
}

impl DofIntermediate {
    fn main_layer(&self) -> Result<&Layer, DofError> {
        self.layers.get("main").ok_or(NoMainLayer.into())
    }

    fn generate_shift_layer(main: &Layer) -> Layer {
        main.0
            .iter()
            .map(|row| row.into_iter().map(|k| k.shifted()).collect::<Vec<_>>())
            .collect::<Vec<_>>()
            .into()
    }

    fn validate_layers(&self, main: &Layer) -> Result<(), DofError> {
        let layers_dont_exist = main
            .keys()
            .map(|k| match k {
                Key::Layer { name: n } if !self.layers.contains_key(n) => Some(n.clone()),
                _ => None,
            })
            .flatten()
            .collect::<Vec<_>>();

        if layers_dont_exist.len() == 0 {
            Ok(())
        } else {
            Err(LayersNotFound(layers_dont_exist).into())
        }
    }

    fn validate_layer_shapes(&self, main: &Layer) -> Result<(), DofError> {
        let main_shape = main.shape();

        let incompatible_shapes = self
            .layers
            .iter()
            .map(|(name, l)| (name, l.shape()))
            .filter(|(_, shape)| shape != &main_shape)
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();

        if incompatible_shapes.len() == 0 {
            Ok(())
        } else {
            Err(IncompatibleLayerShapes(incompatible_shapes).into())
        }
    }

    fn explicit_fingering(&self, main: &Layer) -> Result<Fingering, DofError> {
        use ParsedFingering::*;

        match &self.fingering {
            Explicit(f) if f.shape() == main.shape() => Ok(f.clone()),
            Explicit(_) => Err(IncompatibleFingeringShape.into()),
            Implicit(named) => {
                let fingering = self
                    .board
                    .fingering(named)
                    .map_err(|e| DefinitionError(e))?;

                fingering
                    .resized_fingering(self.anchor, main.shape())
                    .map_err(|e| e.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_main_layer() {
        let minimal_test = DofIntermediate {
            name: "Qwerty".into(),
            authors: None,
            board: KeyboardType::Ansi,
            year: None,
            description: None,
            link: None,
            anchor: Anchor::default(),
            layers: BTreeMap::new(),
            fingering: { ParsedFingering::Implicit(NamedFingering::Angle) },
        };

        let v = Dof::try_from(minimal_test);

        assert_eq!(v, Err(DofError::from(NoMainLayer)));
    }

    #[test]
    fn parse_minimal() {
        let minimal_json = include_str!("../example_dofs/minimal_parsable.dof");

        let minimal_test = DofIntermediate {
            name: "Qwerty".into(),
            authors: None,
            board: KeyboardType::Ansi,
            year: None,
            description: None,
            link: None,
            anchor: Anchor::default(),
            layers: BTreeMap::new(),
            fingering: { ParsedFingering::Implicit(NamedFingering::Angle) },
        };

        let dof_minimal = serde_json::from_str::<DofIntermediate>(minimal_json)
            .expect("couldn't parse implicit json");

        assert_eq!(dof_minimal, minimal_test);
    }

    #[test]
    fn minimal_succesful_dof() {
        use Finger::*;
        use Key::*;

        let minimal_json = include_str!("../example_dofs/minimal_valid.dof");

        let d = serde_json::from_str::<Dof>(minimal_json).expect("Couldn't serialize as Dof");

        let d_manual = Dof {
            name: "Qwerty".into(),
            authors: None,
            board: KeyboardType::Ansi,
            year: None,
            description: None,
            link: None,
            anchor: Anchor::new(1, 1),
            layers: BTreeMap::from_iter([
                (
                    "main".into(),
                    vec![
                        vec![
                            Char('q'),
                            Char('w'),
                            Char('e'),
                            Char('r'),
                            Char('t'),
                            Char('y'),
                            Char('u'),
                            Char('i'),
                            Char('o'),
                            Char('p'),
                        ],
                        vec![
                            Char('a'),
                            Char('s'),
                            Char('d'),
                            Char('f'),
                            Char('g'),
                            Char('h'),
                            Char('j'),
                            Char('k'),
                            Char('l'),
                            Char(';'),
                            Char('\''),
                        ],
                        vec![
                            Char('z'),
                            Char('x'),
                            Char('c'),
                            Char('v'),
                            Char('b'),
                            Char('n'),
                            Char('m'),
                            Char(','),
                            Char('.'),
                            Char('/'),
                        ],
                    ]
                    .into(),
                ),
                (
                    "shift".into(),
                    Layer(vec![
                        vec![
                            Char('Q'),
                            Char('W'),
                            Char('E'),
                            Char('R'),
                            Char('T'),
                            Char('Y'),
                            Char('U'),
                            Char('I'),
                            Char('O'),
                            Char('P'),
                        ],
                        vec![
                            Char('A'),
                            Char('S'),
                            Char('D'),
                            Char('F'),
                            Char('G'),
                            Char('H'),
                            Char('J'),
                            Char('K'),
                            Char('L'),
                            Char(':'),
                            Char('\"'),
                        ],
                        vec![
                            Char('Z'),
                            Char('X'),
                            Char('C'),
                            Char('V'),
                            Char('B'),
                            Char('N'),
                            Char('M'),
                            Char('<'),
                            Char('>'),
                            Char('?'),
                        ],
                    ]),
                ),
            ]),
            fingering: {
                vec![
                    vec![LP, LR, LM, LI, LI, RI, RI, RM, RR, RP],
                    vec![LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP],
                    vec![LR, LM, LI, LI, LI, RI, RI, RM, RR, RP],
                ]
                .into()
            },
            fingering_name: Some(NamedFingering::Angle),
            has_generated_shift: true,
            keys: Vec::new(),
        };

        assert_eq!(d, d_manual);

        let reconvert_json =
            serde_json::to_string_pretty(&d).expect("Couldn't reconvert to json value");

        println!("{reconvert_json}")
    }

    #[test]
    fn maximal_succesful() {
        let maximal_json = include_str!("../example_dofs/minimal_valid.dof");

        serde_json::from_str::<Dof>(maximal_json).expect("Couldn't parse or validate Dof");
    }

    #[test]
    fn deserialize_minimal() {
        let minimal_test = DofIntermediate {
            name: "Qwerty".into(),
            authors: None,
            board: KeyboardType::Ansi,
            year: None,
            description: None,
            link: None,
            anchor: Anchor::default(),
            layers: BTreeMap::new(),
            fingering: { ParsedFingering::Implicit(NamedFingering::Angle) },
        };

        let s = serde_json::to_string_pretty(&minimal_test).unwrap();

        println!("{s}")
    }

    #[test]
    fn buggy() {
        let buggy_json = include_str!("../example_dofs/buggy.dof");

        let buggy = serde_json::from_str::<Dof>(buggy_json).expect("couldn't parse buggy json");

        assert_eq!(buggy.layers.len(), 4);
        assert_eq!(buggy.anchor, Anchor(0, 0));
    }

    #[test]
    fn parse_maximal() {
        use Finger::*;
        use Key::*;
        use SpecialKey::*;

        let maximal_json = include_str!("../example_dofs/maximal.dof");

        let maximal_test = DofIntermediate {
            name: "Qwerty".into(),
            authors: Some(vec!["Christopher Latham Sholes".into()]),
            board: KeyboardType::Ansi,
            year: Some(1878),
            description: Some("the OG. Without Qwerty, none of this would be necessary.".into()),
            link: Some("https://en.wikipedia.org/wiki/QWERTY".into()),
            anchor: Anchor::new(1, 1),
            layers: BTreeMap::from_iter([
                (
                    "main".into(),
                    crate::Layer::from(vec![
                        vec![
                            Char('`'),
                            Char('1'),
                            Char('2'),
                            Char('3'),
                            Char('4'),
                            Char('5'),
                            Char('6'),
                            Char('7'),
                            Char('8'),
                            Char('9'),
                            Char('0'),
                            Char('-'),
                            Char('='),
                            Special(Backspace),
                        ],
                        vec![
                            Special(Tab),
                            Char('q'),
                            Char('w'),
                            Char('e'),
                            Char('r'),
                            Char('t'),
                            Char('y'),
                            Char('u'),
                            Char('i'),
                            Char('o'),
                            Char('p'),
                            Char('['),
                            Char(']'),
                            Char('\\'),
                        ],
                        vec![
                            Special(Caps),
                            Char('a'),
                            Char('s'),
                            Char('d'),
                            Char('f'),
                            Char('g'),
                            Char('h'),
                            Char('j'),
                            Char('k'),
                            Char('l'),
                            Char(';'),
                            Char('\''),
                            Special(Enter),
                        ],
                        vec![
                            Special(Shift),
                            Char('z'),
                            Char('x'),
                            Char('c'),
                            Char('v'),
                            Char('b'),
                            Char('n'),
                            Char('m'),
                            Char(','),
                            Char('.'),
                            Char('/'),
                            Special(Shift),
                        ],
                        vec![
                            Empty,
                            Empty,
                            Empty,
                            Char('ß'),
                            Special(Space),
                            Layer {
                                name: "altgr".into(),
                            },
                            Empty,
                            Empty,
                        ],
                    ]),
                ),
                (
                    "shift".into(),
                    crate::Layer::from(vec![
                        vec![
                            Char('~'),
                            Char('!'),
                            Char('@'),
                            Char('#'),
                            Char('$'),
                            Char('%'),
                            Char('^'),
                            Char('&'),
                            Char('*'),
                            Char('('),
                            Char(')'),
                            Char('_'),
                            Char('+'),
                            Special(Backspace),
                        ],
                        vec![
                            Special(Tab),
                            Char('Q'),
                            Char('W'),
                            Char('E'),
                            Char('R'),
                            Char('T'),
                            Char('Y'),
                            Char('U'),
                            Char('I'),
                            Char('O'),
                            Char('P'),
                            Char('{'),
                            Char('}'),
                            Char('|'),
                        ],
                        vec![
                            Special(Caps),
                            Char('A'),
                            Char('S'),
                            Char('D'),
                            Char('F'),
                            Char('G'),
                            Char('H'),
                            Char('J'),
                            Char('K'),
                            Char('L'),
                            Char(':'),
                            Char('\"'),
                            Special(Enter),
                        ],
                        vec![
                            Special(Shift),
                            Char('Z'),
                            Char('X'),
                            Char('C'),
                            Char('V'),
                            Char('B'),
                            Char('N'),
                            Char('M'),
                            Char('<'),
                            Char('>'),
                            Char('?'),
                            Special(Shift),
                        ],
                        vec![
                            Empty,
                            Empty,
                            Empty,
                            Word("SS".into()),
                            Special(Space),
                            Word("altgr".into()),
                            Empty,
                            Empty,
                        ],
                    ]),
                ),
                (
                    "altgr".into(),
                    crate::Layer::from(vec![
                        vec![
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Special(Backspace),
                        ],
                        vec![
                            Special(Tab),
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Word("tb".into()),
                            Transparent,
                            Char('ü'),
                            Transparent,
                            Char('ö'),
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                        ],
                        vec![
                            Special(Caps),
                            Char('ä'),
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Special(Enter),
                        ],
                        vec![
                            Special(Shift),
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Transparent,
                            Special(Shift),
                        ],
                        vec![
                            Empty,
                            Empty,
                            Empty,
                            Empty,
                            Special(Space),
                            Transparent,
                            Empty,
                            Empty,
                        ],
                    ]),
                ),
            ]),
            fingering: {
                ParsedFingering::Explicit(Fingering::from(vec![
                    vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP, RP],
                    vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP, RP],
                    vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP],
                    vec![LP, LR, LM, LI, LI, LI, RI, RI, RM, RR, RP, RP],
                    vec![LP, LP, LT, LT, LT, RT, RT, RP],
                ]))
            },
        };

        let dof_maximal = serde_json::from_str::<DofIntermediate>(maximal_json)
            .expect("couldn't parse explicit json");

        assert_eq!(dof_maximal, maximal_test);
    }
}
