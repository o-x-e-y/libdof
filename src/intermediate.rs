use serde::{Deserialize, Serialize};
use serde_with::{serde_as, serde_conv, DisplayFromStr};
use thiserror::Error;

use std::collections::BTreeMap;

use crate::definitions::{self, *};

#[derive(Debug, Error, PartialEq)]
pub enum IntermediateDofErrorInner {
    #[error("couldn't parse fingering")]
    DefinitionError(#[from] definitions::DefinitionError),
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

use IntermediateDofErrorInner::*;

#[derive(Debug, Error, PartialEq)]
pub struct DofError(#[source] Box<IntermediateDofErrorInner>);

impl std::fmt::Display for DofError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl From<IntermediateDofErrorInner> for DofError {
    fn from(value: IntermediateDofErrorInner) -> Self {
        Self(Box::new(value))
    }
}

macro_rules! impl_keyboard {
    ($type:ty, $ret:ty, $alias:ident) => {
        impl $type {
            pub fn rows(&self) -> impl Iterator<Item = &Vec<$ret>> {
                self.0.iter()
            }
            pub fn keys(&self) -> impl Iterator<Item = &$ret> {
                self.rows().flatten()
            }
            pub fn shape(&self) -> Shape {
                self.rows().map(|r| r.len()).collect::<Vec<_>>().into()
            }
        }

        impl From<Vec<Vec<$ret>>> for $type {
            fn from(f: Vec<Vec<$ret>>) -> Self {
                Self(f)
            }
        }

        serde_conv!(
            $alias,
            Vec<$ret>,
            |row: &Vec<$ret>| {
                if row.len() == 0 {
                    String::new()
                } else {
                    row.into_iter()
                        .take(row.len() - 1)
                        .map(|e| format!("{e} "))
                        .chain([row.last().unwrap().to_string()])
                        .collect::<String>()
                }
            },
            |line: String| {
                line.split_whitespace()
                    .map(|s| s.parse::<$ret>())
                    .collect::<Result<Vec<_>, crate::definitions::DefinitionError>>()
            }
        );
    };
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

impl_keyboard!(Layer, Key, LayerStrAsRow);

#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer(#[serde_as(as = "Vec<LayerStrAsRow>")] Vec<Vec<Key>>);

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

/// Main struct to use for parsing
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DofIntermediate {
    pub(crate) name: String,
    pub(crate) authors: Option<Vec<String>>,
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) board: KeyboardType,
    pub(crate) year: Option<u32>,
    pub(crate) notes: Option<String>,
    pub(crate) layers: BTreeMap<String, Layer>,
    #[serde(default = "Anchor::default")]
    pub(crate) anchor: Anchor,
    // alt_fingerings: Option<Vec<String>>,
    // combos: Option<HashMap<String, String>>,
    pub(crate) fingering: ParsedFingering,
}

impl DofIntermediate {
    pub fn validate(&self) -> Result<(), DofError> {
        let main_layer = self.main_layer()?;

        self.validate_layers(main_layer)?;
        self.validate_layer_shapes(main_layer)?;
        self.validate_fingering(main_layer)?;

        Ok(())
    }

    pub fn main_layer(&self) -> Result<&Layer, DofError> {
        self.layers.get("main").ok_or(NoMainLayer.into())
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

    fn validate_fingering(&self, main: &Layer) -> Result<(), DofError> {
        self.explicit_fingering(main).map(|_| ())
    }

    pub fn explicit_fingering(&self, main: &Layer) -> Result<Fingering, DofError> {
        use ParsedFingering::*;

        match &self.fingering {
            Explicit(f) if f.shape() == main.shape() => Ok(f.clone()),
            Explicit(_) => Err(IncompatibleFingeringShape.into()),
            Implicit(named) => {
                let fingering = self
                    .board
                    .fingering(named)
                    .map_err(|e| DefinitionError(e))?;
                let reshaped = fingering.resized_fingering(self.anchor, main.shape())?;

                Ok(reshaped)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn parse_minimal() {
        let minimal_json = json!({
            "name": "Qwerty",
            "board": "ansi",
            "layers": {},
            "fingering": "angle"
        });

        let minimal_test = DofIntermediate {
            name: "Qwerty".into(),
            authors: None,
            board: KeyboardType::Ansi,
            year: None,
            notes: None,
            anchor: Anchor::default(),
            layers: BTreeMap::new(),
            fingering: { ParsedFingering::Implicit(NamedFingering::Angle) },
        };

        let dof_minimal = serde_json::from_value::<DofIntermediate>(minimal_json)
            .expect("couldn't parse implicit json");

        assert_eq!(dof_minimal, minimal_test);
    }

    #[test]
    fn parse_maximal() {
        use Finger::*;
        use Key::*;
        use SpecialKey::*;

        let maximal_json = json!({
            "name": "Qwerty",
            "authors": ["Christopher Latham Sholes"],
            "board": "ansi",
            "year": 1878,
            "notes": "the OG. Without Qwerty, none of this would be necessary.",
            "anchor": [1, 2],
            "layers": {
                "main": [
                    "` 1 2 3 4 5  6 7 8 9 0 - = bsp",
                    "tb q w e r t  y u i o p [ ] \\",
                    "cps a s d f g  h j k l ; ' ret",
                    "shft z x c v b  n m , . / shft",
                    "~ ~ ~ ~       spc       altgr ~ ~"
                ],
                "shift": [
                    "\\~ ! @ # $ %  ^ & \\* ( ) _ + bsp",
                    "tab  Q W E R T  Y U   I O P { } |",
                    "caps  A S D F G  H J   K L : \" ent",
                    "*      Z X C V B  N M   < > ? shft",
                    "~ ~ ~ ~         spc    saltgr ~ ~"
                ]
            },
            "fingering": [
                "0  0  1  2  3  3   6  6  7  8  9  9  9  9  9",
                "LP LP LR LM LI LI  RI RI RM RR RP RP RP RP",
                "LP LP LR LM LI LI  RI RI RM RR RP RP RP",
                "LP LR LM LI LI LI  RI RI RM RR RP RP",
                "LP  LP  LT  LT    LT    RT  RT  RP"
            ]
        });

        let maximal_test = DofIntermediate {
            name: "Qwerty".into(),
            authors: Some(vec!["Christopher Latham Sholes".into()]),
            board: KeyboardType::Ansi,
            year: Some(1878),
            notes: Some("the OG. Without Qwerty, none of this would be necessary.".into()),
            anchor: Anchor(1, 2),
            layers: BTreeMap::from_iter([
                (
                    "main".into(),
                    Layer(vec![
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
                            Empty,
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
                    Layer(vec![
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
                            Transparent,
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
                            Empty,
                            Special(Space),
                            Layer {
                                name: "saltgr".into(),
                            },
                            Empty,
                            Empty,
                        ],
                    ]),
                ),
            ]),
            fingering: {
                ParsedFingering::Explicit(Fingering(vec![
                    vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP, RP, RP],
                    vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP, RP],
                    vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP],
                    vec![LP, LR, LM, LI, LI, LI, RI, RI, RM, RR, RP, RP],
                    vec![LP, LP, LT, LT, LT, RT, RT, RP],
                ]))
            },
        };

        let dof_maximal = serde_json::from_value::<DofIntermediate>(maximal_json)
            .expect("couldn't parse explicit json");

        assert_eq!(dof_maximal, maximal_test);

        let maximal_shape = dof_maximal.layers.get("main").unwrap().shape();
        let maximal_anchor = dof_maximal.anchor;
        let thirtykey = Shape::from(vec![10, 10, 10]);
        let thirtykey_anchor = Anchor(0, 1);
        // let compat = thirtykey.fits_in(thirtykey_anchor, &maximal_shape, maximal_anchor);

        // assert!(compat);
    }
}
