//! Contains the `Keyboard` struct and helpers which can be used to describe physical keyboards.

use std::str::FromStr;
use std::{cmp::Ordering, num::ParseFloatError};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    keyboard_conv, Anchor, DofError, DofErrorInner as DE, Fingering, Keyboard, KeyboardType,
    NamedFingering, Result,
};

/// Representation of a physical key on a keyboard, where `(x, y)` are the top left and the width and
/// height go right and down respectively.
#[derive(Clone, Debug, PartialEq)]
pub struct PhysicalKey {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl std::fmt::Display for PhysicalKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let omit = |v: f64| v >= 0.999999 || v <= 1.000001;

        match (omit(self.width), omit(self.height)) {
            (true, true) => write!(f, "{} {}", self.x, self.y),
            (false, true) => write!(f, "{} {} {}", self.x, self.y, self.width),
            (_, false) => write!(f, "{} {} {} {}", self.x, self.y, self.width, self.height),
        }
    }
}

impl FromStr for PhysicalKey {
    type Err = DofError;

    fn from_str(s: &str) -> Result<Self> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err(DE::EmptyPhysKey.into());
        }

        let vals = trimmed
            .split_whitespace()
            .map(|s| s.parse::<f64>())
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|_| DE::KeyParseError(trimmed.into()))?;

        match vals.as_slice() {
            &[x, y] => Ok(Self::xy(x, y)),
            &[x, y, width] => Ok(Self::xyw(x, y, width)),
            &[x, y, width, height] => Ok(Self::xywh(x, y, width, height)),
            sl => Err(DE::ValueAmountError(sl.len(), trimmed.into()).into()),
        }
    }
}

/// Representation of a physical keyboard, based on a configuration of physical keys.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PhysicalKeyboard(#[serde_as(as = "Vec<Vec<DisplayFromStr>>")] Vec<Vec<PhysicalKey>>);

impl Keyboard for PhysicalKeyboard {
    type K = PhysicalKey;

    fn inner(&self) -> &[Vec<Self::K>] {
        &self.0
    }

    fn into_inner(self) -> Vec<Vec<Self::K>> {
        self.0
    }
}

impl From<Vec<Vec<PhysicalKey>>> for PhysicalKeyboard {
    fn from(v: Vec<Vec<PhysicalKey>>) -> Self {
        Self(v)
    }
}

/// Key representation with a width only. Useful for boards that have no vertical stagger where
/// each key is next to each other.
#[derive(Clone, Debug, PartialEq)]
pub struct RelativeKey {
    pub(crate) width: f64,
    pub(crate) has_key: bool,
}

impl std::fmt::Display for RelativeKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.width {
            w if w == 1.0 => write!(f, "k"),
            w if w.fract() == 0.0 => write!(f, "{}k", w as u64),
            w => write!(f, "{w}k"),
        }
    }
}

impl FromStr for RelativeKey {
    type Err = DofError;

    fn from_str(s: &str) -> Result<Self> {
        match s.strip_suffix('k') {
            Some(s) if !s.is_empty() => Ok(Self {
                width: s.parse::<f64>()?,
                has_key: true,
            }),
            Some(_) => Ok(Self {
                width: 1.0,
                has_key: true,
            }),
            None => Ok(Self {
                width: s.parse::<f64>()?,
                has_key: false,
            }),
        }
    }
}

/// Representation of a physical keyboard where each row is built of
/// [`RelativeKey`](crate::keyboard::RelativeKey)s as a shorthand for defining each key individually.
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RelativeKeyboard(#[serde_as(as = "Vec<RelativeKeyboardRow>")] Vec<Vec<RelativeKey>>);

impl Keyboard for RelativeKeyboard {
    type K = RelativeKey;

    fn inner(&self) -> &[Vec<Self::K>] {
        &self.0
    }

    fn into_inner(self) -> Vec<Vec<Self::K>> {
        self.0
    }
}

impl From<Vec<Vec<RelativeKey>>> for RelativeKeyboard {
    fn from(v: Vec<Vec<RelativeKey>>) -> Self {
        Self(v)
    }
}

keyboard_conv!(RelativeKeyboard, RelativeKey, RelativeKeyboardRow);

/// Representation of a physical keyboard using a keyboard type and an optional anchor. If these are
/// known defaults, it can be converted to a physical keyboard directly.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamedKeyboard {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "type")]
    pub(crate) board: KeyboardType,
    pub(crate) anchor: Option<Anchor>,
}

impl TryFrom<KeyboardType> for PhysicalKeyboard {
    type Error = DofError;

    fn try_from(board: KeyboardType) -> Result<Self> {
        let kb = match board {
            KeyboardType::Ansi => vec![
                phys_row(&[(1.0, 1), (1.0, 12), (2.0, 1)], 0.0, 0.0),
                phys_row(&[(1.5, 1), (1.0, 12), (1.5, 1)], 0.0, 1.0),
                phys_row(&[(1.75, 1), (1.0, 11), (2.25, 1)], 0.0, 2.0),
                phys_row(&[(2.25, 1), (1.0, 10), (2.75, 1)], 0.0, 3.0),
                phys_row(&[(1.25, 3), (6.25, 1), (1.25, 4)], 0.0, 4.0),
            ],
            KeyboardType::Iso => {
                let mut iso = vec![
                    phys_row(&[(1.0, 1), (1.0, 12), (2.0, 1)], 0.0, 0.0),
                    phys_row(&[(1.5, 1), (1.0, 12) /* iso */], 0.0, 1.0),
                    phys_row(&[(1.75, 1), (1.0, 12) /*enter*/], 0.0, 2.0),
                    phys_row(&[(1.25, 1), (1.0, 11), (2.75, 1)], 0.0, 3.0),
                    phys_row(&[(1.25, 3), (6.25, 1), (1.25, 4)], 0.0, 4.0),
                ];

                // ISO enter. This is an approximation because in reality it is not actually a rectangle.
                iso[1].push(PhysicalKey::xywh(13.75, 2.0, 1.5, 2.0));

                iso
            }
            KeyboardType::Ortho => vec![
                phys_row(&[(1.0, 10)], 0.0, 0.0),
                phys_row(&[(1.0, 10)], 0.0, 1.0),
                phys_row(&[(1.0, 10)], 0.0, 2.0),
                phys_row(&[(1.0, 6)], 3.0, 3.0),
            ],
            KeyboardType::Colstag => vec![
                vec![
                    PhysicalKey::xy(0.0, 0.45),
                    PhysicalKey::xy(1.0, 0.15),
                    PhysicalKey::xy(2.0, 0.0),
                    PhysicalKey::xy(3.0, 0.15),
                    PhysicalKey::xy(4.0, 0.30),
                    PhysicalKey::xy(7.0, 0.30),
                    PhysicalKey::xy(8.0, 0.15),
                    PhysicalKey::xy(9.0, 0.0),
                    PhysicalKey::xy(10.0, 0.15),
                    PhysicalKey::xy(11.0, 0.45),
                ],
                vec![
                    PhysicalKey::xy(0.0, 1.45),
                    PhysicalKey::xy(1.0, 1.15),
                    PhysicalKey::xy(2.0, 1.0),
                    PhysicalKey::xy(3.0, 1.15),
                    PhysicalKey::xy(4.0, 1.30),
                    PhysicalKey::xy(7.0, 1.30),
                    PhysicalKey::xy(8.0, 1.15),
                    PhysicalKey::xy(9.0, 1.0),
                    PhysicalKey::xy(10.0, 1.15),
                    PhysicalKey::xy(11.0, 1.45),
                ],
                vec![
                    PhysicalKey::xy(0.0, 2.45),
                    PhysicalKey::xy(1.0, 2.15),
                    PhysicalKey::xy(2.0, 2.0),
                    PhysicalKey::xy(3.0, 2.15),
                    PhysicalKey::xy(4.0, 2.30),
                    PhysicalKey::xy(7.0, 2.30),
                    PhysicalKey::xy(8.0, 2.15),
                    PhysicalKey::xy(9.0, 2.0),
                    PhysicalKey::xy(10.0, 2.15),
                    PhysicalKey::xy(11.0, 2.45),
                ],
                vec![
                    PhysicalKey::xy(2.4, 3.3),
                    PhysicalKey::xy(3.5, 3.5),
                    PhysicalKey::xy(4.7, 3.8),
                    PhysicalKey::xy(6.3, 3.8),
                    PhysicalKey::xy(7.5, 3.5),
                    PhysicalKey::xy(8.6, 3.3),
                ],
            ],
            c @ KeyboardType::Custom(_) => return Err(DE::UnknownKeyboardType(c).into()),
        };

        Ok(kb.into())
    }
}

impl From<RelativeKeyboard> for PhysicalKeyboard {
    fn from(rkb: RelativeKeyboard) -> Self {
        rkb.into_inner()
            .into_iter()
            .enumerate()
            .map(|(y, r)| {
                let mut x = 0.0;

                r.into_iter()
                    .filter_map(|rk| {
                        let r = PhysicalKey::xyw(x, y as f64, rk.width);
                        x += rk.width;
                        rk.has_key.then_some(r)
                    })
                    .collect()
            })
            .collect::<Vec<_>>()
            .into()
    }
}

impl From<PhysicalKeyboard> for ParseKeyboard {
    fn from(board: PhysicalKeyboard) -> Self {
        if board.inner().is_empty() {
            return Self::Full(Default::default());
        }

        let relative_board = board
            .rows()
            .enumerate()
            .map(|(row_i, r)| match r.split_first() {
                None => Some(vec![]),
                Some((f, r)) if f.y == row_i as f64 => {
                    let mut res = match f.x == 0.0 {
                        true => vec![RelativeKey {
                            width: f.width,
                            has_key: true,
                        }],
                        _ => vec![
                            RelativeKey {
                                width: f.x,
                                has_key: false,
                            },
                            RelativeKey {
                                width: f.width,
                                has_key: true,
                            },
                        ],
                    };

                    let mut last_x = f.width + f.x;

                    for key in r {
                        if key.y != row_i as f64 {
                            return None;
                        }
                        match key.x.total_cmp(&last_x) {
                            Ordering::Less => return None,
                            Ordering::Equal => res.push(RelativeKey {
                                width: key.width,
                                has_key: true,
                            }),
                            Ordering::Greater => {
                                res.push(RelativeKey {
                                    width: key.x - last_x,
                                    has_key: false,
                                });
                                res.push(RelativeKey {
                                    width: key.width,
                                    has_key: true,
                                });
                            }
                        }

                        last_x = key.x + key.width;
                    }

                    Some(res)
                }
                _ => None,
            })
            .collect::<Option<Vec<Vec<RelativeKey>>>>();

        match relative_board {
            Some(b) => Self::Relative(b.into()),
            None => Self::Full(board),
        }
    }
}

/// Abstraction over the way an actual .dof file is allowed to represent a physical keyboard for a
/// specific layout. Has three different ways of doing so.
/// * `Named`: a [`KeyboardType`](crate::KeyboardType) name. If a custom name is provided,
/// the `Dof` can likely not be parsed.
/// * `Relative`: a [`RelativeKeyboard`](crate::keyboard::RelativeKeyboard),
/// * `Full`: a [`PhysicalKeyboard`](crate::keyboard::PhysicalKeyboard), which is what is converted
/// to when converting to `Dof`.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParseKeyboard {
    /// * `Named`: a [`KeyboardType`](crate::KeyboardType) name. If a custom name is provided,
    /// the `Dof` can likely not be parsed.
    Named(#[serde_as(as = "DisplayFromStr")] KeyboardType),
    // NamedAnchor(NamedKeyboard),
    /// * `Relative`: a [`RelativeKeyboard`](crate::keyboard::RelativeKeyboard),
    Relative(RelativeKeyboard),
    /// * `Full`: a [`PhysicalKeyboard`](crate::keyboard::PhysicalKeyboard), which is what is converted
    /// to when converting to `Dof`.
    Full(PhysicalKeyboard),
}

impl ParseKeyboard {
    /// Get the default anchor for a parsed kebyoard. This is (0, 0) for anything custom, (1, 1) for
    /// `Ansi` and `Iso` boards (as the vast majority of keyboard layouts doesn't remap the number
    /// row or special keys on the left) and (0, 0) for `Ortho` and `Colstag`.
    pub fn anchor(&self) -> Anchor {
        match self {
            ParseKeyboard::Named(n) => n.anchor(),
            _ => Anchor(0, 0),
        }
    }

    /// Given a known fingering from `NamedFingering`, provide a `Fingering` object with all keys on a board
    /// like that specified. Will return an error if any combination is provided that isn't valid. This
    /// is the case for any custom physical boards, but also for `KeyboardType::Ortho` and
    /// `NamedFingering::Angle` for example.
    pub fn fingering(&self, named_fingering: &NamedFingering) -> Result<Fingering> {
        match self {
            ParseKeyboard::Named(n) => n.fingering(named_fingering),
            _ => Err(DE::FingeringForCustomKeyboard.into()),
        }
    }
}

impl TryFrom<ParseKeyboard> for PhysicalKeyboard {
    type Error = DofError;

    fn try_from(value: ParseKeyboard) -> Result<Self> {
        match value {
            ParseKeyboard::Named(board) => board.try_into(),
            ParseKeyboard::Relative(r) => Ok(r.into()),
            ParseKeyboard::Full(f) => Ok(f),
        }
    }
}

pub(crate) fn phys_row(widths: &[(f64, usize)], x_offset: f64, y_offset: f64) -> Vec<PhysicalKey> {
    let mut x = x_offset;

    widths
        .iter()
        .copied()
        .flat_map(|(width, count)| std::iter::repeat(width).take(count))
        .map(|w| {
            let pk = PhysicalKey::xyw(x, y_offset, w);
            x += w;
            pk
        })
        .collect()
}

impl PhysicalKey {
    /// Get the `x` coordinate.
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Get the `y` coordinate.
    pub fn y(&self) -> f64 {
        self.y
    }

    /// Get the width of the key, which is the space from `x` to the right side of the key.
    pub fn width(&self) -> f64 {
        self.width
    }

    /// Get the height of the key, which is the space from `y` to the bottom of the key.
    pub fn height(&self) -> f64 {
        self.height
    }

    /// Create a new key with x, y coordinates where the width and height are set to 1.0.
    pub const fn xy(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            width: 1.0,
            height: 1.0,
        }
    }

    /// Create a new key with x, y coordinates and a width, with a set height of 1.0.
    pub const fn xyw(x: f64, y: f64, width: f64) -> Self {
        Self {
            x,
            y,
            width,
            height: 1.0,
        }
    }

    /// Create a new key with x, y coordinates, a width and a height.
    pub const fn xywh(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

impl From<ParseFloatError> for DofError {
    fn from(value: ParseFloatError) -> Self {
        DE::ParseFloatError(value).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn parse_physical_key() {
        let k1 = "0.0 0.0".parse::<PhysicalKey>();
        let k2 = "1 2 3".parse::<PhysicalKey>();
        let k3 = "0 0 4 2".parse::<PhysicalKey>();
        let k4 = "0.1".parse::<PhysicalKey>();
        let k5 = "0.1 0.2".parse::<PhysicalKey>();
        let k6 = "".parse::<PhysicalKey>();

        assert_eq!(k1, Ok(PhysicalKey::xy(0.0, 0.0)));
        assert_eq!(k2, Ok(PhysicalKey::xywh(1.0, 2.0, 3.0, 1.0)));
        assert_eq!(k3, Ok(PhysicalKey::xywh(0.0, 0.0, 4.0, 2.0)));
        assert_eq!(
            k4,
            Err(DofError::from(DE::ValueAmountError(1, "0.1".into())))
        );
        assert_eq!(k5, Ok(PhysicalKey::xy(0.1, 0.2)));
        assert_eq!(k6, Err(DofError::from(DE::EmptyPhysKey)));
    }

    #[test]
    fn parse_physical_key_board() {
        let board_str = r#"[
                [
                    "1.8125 0.5 2 3"
                ]
            ]
        "#;

        let kb = serde_json::from_str::<PhysicalKeyboard>(board_str)
            .expect("parsing of Keyboard failed");

        let cmp = PhysicalKeyboard(vec![vec![PhysicalKey {
            x: 1.8125,
            y: 0.5,
            width: 2.0,
            height: 3.0,
        }]]);

        assert_eq!(kb, cmp);

        let parsed = ParseKeyboard::from(kb);

        assert_matches!(parsed, ParseKeyboard::Full(_))
    }

    #[test]
    fn parse_relative_key() {
        let k1 = "k".parse::<RelativeKey>();
        let k2 = "2.3k".parse::<RelativeKey>();
        let k3 = "5".parse::<RelativeKey>();
        let k4 = "".parse::<RelativeKey>();

        assert_eq!(
            k1,
            Ok(RelativeKey {
                width: 1.0,
                has_key: true
            })
        );
        assert_eq!(
            k2,
            Ok(RelativeKey {
                width: 2.3,
                has_key: true
            })
        );
        assert_eq!(
            k3,
            Ok(RelativeKey {
                width: 5.0,
                has_key: false
            })
        );

        match k4 {
            Ok(_) => panic!("Should be a `ParseFloatError`, actually: '{k4:?}'"),
            Err(e) => assert!(matches!(e.0.as_ref(), DE::ParseFloatError(_))),
        }
    }

    #[test]
    fn row_defined_keyboard() {
        let board_str = r#"
            [
                "k 3.2k   2 8k"
            ]
        "#;

        let kb = serde_json::from_str::<ParseKeyboard>(board_str)
            .expect("parsing of RelativeKeyboard failed");

        let cmp = ParseKeyboard::Relative(RelativeKeyboard(vec![vec![
            RelativeKey {
                width: 1.0,
                has_key: true,
            },
            RelativeKey {
                width: 3.2,
                has_key: true,
            },
            RelativeKey {
                width: 2.0,
                has_key: false,
            },
            RelativeKey {
                width: 8.0,
                has_key: true,
            },
        ]]));

        assert_eq!(kb, cmp);

        let keyboard = PhysicalKeyboard::try_from(cmp)
            .expect("couldn't convert to physical keyboard from parse keyboard: ");

        let parsed = ParseKeyboard::from(keyboard);

        println!("{parsed:?}");

        assert_matches!(parsed, ParseKeyboard::Relative(_))
    }

    #[test]
    fn named_parsed_keyboard() {
        let board_str = "\"ansi\"";

        let board = serde_json::from_str::<ParseKeyboard>(board_str)
            .expect("parsing of ParseKeyboard failed");

        assert_eq!(board, ParseKeyboard::Named(KeyboardType::Ansi));

        let kb = PhysicalKeyboard::try_from(board)
            .expect("error encountered while converting to physical keyboard: ")
            .resized(KeyboardType::Ansi.anchor(), vec![10, 11, 10].into());

        let cmp = &PhysicalKey {
            x: 11.25,
            y: 3.0,
            width: 1.0,
            height: 1.0,
        };

        match kb {
            Ok(kb) => match kb.last() {
                Some(r) => match r.last() {
                    Some(k) => assert_eq!(k, cmp),
                    None => panic!("Row has no keys"),
                },
                None => panic!("Keyboard is empty"),
            },
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn rows_parsed_keyboard() {
        let board_str = r#"[
            "1 k k k  2  k k k 1",
            "k k k k  2  k k k k",
            "  3   k k k k   3  "
        ]"#;

        let board: PhysicalKeyboard = serde_json::from_str::<ParseKeyboard>(board_str)
            .expect("parsing of ParseKeyboard failed")
            .try_into()
            .expect("error encountered while converting to physical keyboard: ");

        assert_eq!(board.inner()[0].len(), 6);
        assert_eq!(board.inner()[2].len(), 4);
        assert_eq!(board.inner()[0][3].x, 6.0);
    }
}
