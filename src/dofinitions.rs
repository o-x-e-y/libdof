use thiserror::Error;

use std::{convert::Infallible, fmt::Display, str::FromStr};

use crate::{Fingering, Anchor};

#[derive(Debug, Error, PartialEq)]
pub enum DofinitionError {
    #[error("Couldn't parse Finger from '{0}'")]
    FingerParseError(String),
    #[error("an empty string can't be parsed into a Key")]
    EmptyKeyError,
    #[error("{0}")]
    Infallible(#[from] std::convert::Infallible),
    #[error("Can't combine keyboard type '{0}' with fingering '{1}'")]
    UnsupportedKeyboardFingeringCombo(KeyboardType, NamedFingering),
    #[error("The shape of '{0}' does not overlap with the provided keymap")]
    NonOverlappingShapesError(NamedFingering),
    #[error("The given fingering is unknown. Valid inputs are: angle, traditional")]
    UnknownNamedFingering,
}

/// This should cover all fingers... for now
/// implements `ToString` and `FromStr`. The latter also allows parsing from numbers,
/// where `LP`: 1, `LR`: 2 etc.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Finger {
    LP,
    LR,
    LM,
    LI,
    LT,
    RT,
    RI,
    RM,
    RR,
    RP,
}

impl Display for Finger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl FromStr for Finger {
    type Err = DofinitionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Finger::*;

        let s = s.trim_start().trim_end();
        match s {
            "LP" | "0" => Ok(LP),
            "LR" | "1" => Ok(LR),
            "LM" | "2" => Ok(LM),
            "LI" | "3" => Ok(LI),
            "LT" | "4" => Ok(LT),
            "RT" | "5" => Ok(RT),
            "RI" | "6" => Ok(RI),
            "RM" | "7" => Ok(RM),
            "RR" | "8" => Ok(RR),
            "RP" | "9" => Ok(RP),
            _ => Err(DofinitionError::FingerParseError(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamedFingering {
    Traditional,
    Angle,
    Custom(String),
}

impl Display for NamedFingering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Traditional => "traditional",
            Self::Angle => "angle",
            Self::Custom(name) => name.as_str(),
        };

        write!(f, "{s}")
    }
}

impl FromStr for NamedFingering {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = match s.to_lowercase().as_str() {
            "standard" | "traditional" => Self::Traditional,
            "angle" => Self::Angle,
            name => Self::Custom(name.into()),
        };

        Ok(res)
    }
}

/// Covers a wide range of keys that don't output characters, but are still commonly found on a keyboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecialKey {
    Esc,
    Repeat,
    Space,
    Tab,
    Enter,
    Shift,
    Caps,
    Ctrl,
    Alt,
    Meta,
    Fn,
    Backspace,
    Del,
}

/// Covers all keys commonly found on a keyboard. Implements `ToString` and `FromStr`, where the latter
/// only fails if the string passed to it was empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Empty,
    Transparent,
    Char(char),
    Word(String),
    Special(SpecialKey),
    Layer { name: String },
}

impl Key {
    pub fn shifted(&self) -> Self {
        use Key::*;

        match self {
            Char(c) => match c {
                '`' => Char('~'),
                '1' => Char('!'),
                '2' => Char('@'),
                '3' => Char('#'),
                '4' => Char('$'),
                '5' => Char('%'),
                '6' => Char('^'),
                '7' => Char('*'),
                '9' => Char('('),
                '0' => Char(')'),
                '[' => Char('{'),
                ']' => Char('}'),
                '<' => Char('>'),
                '\'' => Char('"'),
                ',' => Char('<'),
                '.' => Char('>'),
                ';' => Char(':'),
                '/' => Char('?'),
                '=' => Char('+'),
                '-' => Char('_'),
                '\\' => Char('|'),
                c => {
                    let mut upper = c.to_uppercase();
                    if upper.clone().count() == 1 {
                        Char(upper.next().unwrap())
                    } else {
                        Word(upper.to_string())
                    }
                }
            },
            Special(_) => Transparent,
            k => k.clone(),
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Key::*;
        use SpecialKey::*;

        let s = match self {
            Empty => "~".into(),
            Transparent => "*".into(),
            Char(c) => match c {
                n @ ('~' | '*') => format!("\\{n}"),
                n => String::from(*n),
            },
            Word(w) => w.clone(),
            Special(s) => match s {
                Esc => "esc".into(),
                Repeat => "rpt".into(),
                Space => "spc".into(),
                Tab => "tab".into(),
                Enter => "ret".into(),
                Shift => "sft".into(),
                Caps => "cps".into(),
                Ctrl => "ctl".into(),
                Alt => "alt".into(),
                Meta => "mt".into(),
                Fn => "fn".into(),
                Backspace => "bsp".into(),
                Del => "del".into(),
            },
            Layer { name } => name.clone(),
        };

        write!(f, "{s}")
    }
}

impl FromStr for Key {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.into())
    }
}

impl<T> From<T> for Key
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        use Key::*;
        use SpecialKey::*;

        let s = value.as_ref();

        match s.chars().count() {
            0 => Empty,
            1 => match s {
                "~" => Empty,
                "*" => Transparent,
                " " => Special(Space),
                "\n" => Special(Enter),
                "\t" => Special(Tab),
                _ => Char(s.chars().next().unwrap()),
            },
            _ => match s.to_lowercase().as_str() {
                "\\~" => Char('~'),
                "\\*" => Char('*'),
                "esc" => Special(Esc),
                "repeat" | "rpt" => Special(Repeat),
                "space" | "spc" => Special(Space),
                "tab" | "tb" => Special(Tab),
                "enter" | "return" | "ret" | "ent" | "rt" => Special(Enter),
                "shift" | "shft" | "sft" | "st" => Special(Shift),
                "caps" | "cps" | "cp" => Special(Caps),
                "ctrl" | "ctl" | "ct" => Special(Ctrl),
                "alt" | "lalt" | "ralt" | "lt" => Special(Alt),
                "meta" | "mta" | "met" | "mt" | "super" | "sup" | "sp" => Special(Meta),
                "fn" => Special(Fn),
                "backspace" | "bksp" | "bcsp" | "bsp" => Special(Backspace),
                "del" => Special(Del),
                _ if s.starts_with("@") => Layer {
                    name: s.chars().skip(1).collect(),
                },
                _ if s.starts_with("#") || s.starts_with("\\#") || s.starts_with("\\@") => {
                    Word(s.chars().skip(1).collect())
                }
                _ => Word(s.into()),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shape(Vec<usize>);

impl From<Vec<usize>> for Shape {
    fn from(value: Vec<usize>) -> Self {
        Shape(value)
    }
}

impl Shape {
    pub fn inner(&self) -> &Vec<usize> {
        &self.0
    }

    pub fn into_inner(self) -> Vec<usize> {
        self.0
    }

    pub fn row_count(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyboardType {
    Ansi,
    Iso,
    Ortho,
    Colstag,
    Custom(String),
}

impl KeyboardType {
    pub fn shape(&self) -> Shape {
        self.fingering(&NamedFingering::Traditional)
            .unwrap()
            .shape()
    }

    pub fn fingering(
        &self,
        named_fingering: &NamedFingering,
    ) -> Result<Fingering, DofinitionError> {
        use Finger::*;
        use KeyboardType::*;
        use NamedFingering::*;

        let fingering = match (self, &named_fingering) {
            (Ansi, Traditional) => vec![
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP, RP],
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP, RP],
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP],
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP],
                vec![LP, LP, LT, LT, LT, RT, RT, RP],
            ]
            .into(),
            (Ansi, Angle) => vec![
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP, RP],
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP, RP],
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP],
                vec![LP, LR, LM, LI, LI, LI, RI, RI, RM, RR, RP, RP],
                vec![LP, LP, LT, LT, LT, RT, RT, RP],
            ]
            .into(),
            (Iso, Traditional) => vec![
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP, RP],
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP, RP],
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP],
                vec![LP, LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP],
                vec![LP, LP, LT, LT, LT, RT, RT, RP],
            ]
            .into(),
            (Iso, Angle) => vec![
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP, RP],
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP, RP],
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP],
                vec![LP, LP, LR, LM, LI, LI, LI, RI, RI, RM, RR, RP, RP],
                vec![LP, LP, LT, LT, LT, RT, RT, RP],
            ]
            .into(),
            (Ortho, Traditional) => vec![
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP],
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP],
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP],
                vec![LT, LT, LT, RT, RT, RT],
            ]
            .into(),
            (Colstag, Traditional) => vec![
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP],
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP],
                vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP],
                vec![LT, LT, LT, RT, RT, RT],
            ]
            .into(),
            (board, &f) => {
                return Err(DofinitionError::UnsupportedKeyboardFingeringCombo(
                    board.clone(),
                    f.clone(),
                ))
            }
        };

        Ok(fingering)
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    pub fn anchor(&self) -> Anchor {
        match self {
            KeyboardType::Ansi => Anchor::new(1, 1),
            KeyboardType::Iso => Anchor::new(1, 1),
            KeyboardType::Ortho => Anchor::new(0, 0),
            KeyboardType::Colstag => Anchor::new(0, 0),
            KeyboardType::Custom(_) => Anchor::new(0, 0),
        }
    }
}

impl Display for KeyboardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use KeyboardType::*;

        let s = match self {
            Ansi => "ansi",
            Iso => "iso",
            Ortho => "ortho",
            Colstag => "colstag",
            Custom(name) => name.as_str(),
        };

        write!(f, "{s}")
    }
}

impl FromStr for KeyboardType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use KeyboardType::*;

        match s.to_lowercase().as_str() {
            "ansi" => Ok(Ansi),
            "iso" => Ok(Iso),
            "ortho" => Ok(Ortho),
            "colstag" => Ok(Colstag),
            name => Ok(Custom(name.into())),
        }
    }
}
