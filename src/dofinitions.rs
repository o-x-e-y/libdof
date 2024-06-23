//!Contains most types to represent elements of a keyboard layout with

use std::{convert::Infallible, fmt::Display, str::FromStr};

use crate::{Anchor, DofError, DofErrorInner, Fingering, Keyboard, Result};

/// Represents a finger. Implements `ToString` and `FromStr`, where each finger can either be represented
/// in string form as `LP`, `LR` (left pinky, left ring) or as a number where `LP`= 0, `LR`= 1 up to
/// `RP`= 9
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

impl Finger {
    /// Array containing all 10 fingers in order from `LP` to `RP`.
    pub const FINGERS: [Self; 10] = [
        Self::LP,
        Self::LR,
        Self::LM,
        Self::LI,
        Self::LT,
        Self::RT,
        Self::RI,
        Self::RM,
        Self::RR,
        Self::RP,
    ];
}

impl Display for Finger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl FromStr for Finger {
    type Err = DofError;

    fn from_str(s: &str) -> Result<Self> {
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
            _ => Err(DofErrorInner::FingerParseError(s.to_string()).into()),
        }
    }
}

/// Represents known fingerings with names. Currently these are `Traditional` and `Angle`. A `Custom` type
/// is also specified, though this isn't particularly useful in use with the rest of the library. `FromStr`
/// uses `standard` and `traditional` for `Traditional`, and `angle` for `Angle`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum NamedFingering {
    /// Traditional fingering. Default value.
    #[default]
    Traditional,
    /// Fingering for angle mod
    Angle,
    /// Any custom type of fingering. This is technically valid in a .dof, but not supported to be worked with.
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

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let res = match s.to_lowercase().as_str() {
            "standard" | "traditional" => Self::Traditional,
            "angle" => Self::Angle,
            name => Self::Custom(name.into()),
        };

        Ok(res)
    }
}

/// Covers a wide range of keys that don't necessarily output characters, but are still commonly found on a
/// keyboard. Shift is meant to function the same as a `Key::Layer { layer: "shift" }` key.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    Menu,
    Fn,
    Backspace,
    Del,
}

/// Covers all keys commonly found on a keyboard. Implements `ToString` and `FromStr`, where the latter has
/// some rules about how it works:
/// * if the length is 0, output `Key::Empty`,
/// * if the length is 1, output:
///     - `Key::Empty` when it's equal to `~`
///     - `Key::Transparent` when it's equal to `*`
///     - `Key::Special(SpecialKey::Space)` when it's equal to a space,
///     - `Key::Special(SpecialKey::Enter)` when it's equal to `\n`,
///     - `Key::Special(SpecialKey::Tab)` when it's equal to `\t`,
///     - `Key::Char` otherwise.
/// * if the length is more than 1, outputs
///     - `Key::Char('~')` and `Key::Char('*')` if they contain `\\~` and `\\*` respectively,
///     - `Key::Special` based on their names in the readme. You can also check the `FromStr`
///        implementation itself,
///     - `Key::Layer` if it leads with an `@`.
///     - `Key::Word` with its first character removed if it starts with `#`, `\\#` or`\\@`,
///     - `Key::Word` otherwise.
#[allow(missing_docs)]
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum Key {
    #[default]
    Empty,
    Transparent,
    Char(char),
    Word(String),
    Special(SpecialKey),
    Layer {
        name: String,
    },
}

impl Key {
    /// Turns lowercase characters into their qwerty shift output, and turns `Special`` keys `Transparent`.
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

    /// Check if the key is of type [`Key::Char`](crate::dofinitions::Key::Char) which outputs
    /// a single character.
    pub fn is_char(&self) -> bool {
        matches!(self, Key::Char(_))
    }

    /// Check if the key is of type [`Key::Word`](crate::dofinitions::Key::Word) which outputs a specific
    /// string.
    pub fn is_word(&self) -> bool {
        matches!(self, Key::Word(_))
    }

    /// Check if the key is of type [`Key::Empty`](crate::dofinitions::Key::Empty) which doesn't output
    /// anything.
    pub fn is_empty(&self) -> bool {
        matches!(self, Key::Empty)
    }

    /// Check if the key is of type [`Key::Transparent`](crate::dofinitions::Key::Char) which outputs
    /// whatever it is the main layer outputs in that position.
    pub fn is_transparent(&self) -> bool {
        matches!(self, Key::Transparent)
    }

    /// Check if the key is of type [`Key::Layer`](crate::dofinitions::Key::Layer) which holds the name.
    /// of a layer on the layout
    pub fn is_layer(&self) -> bool {
        matches!(self, Key::Layer { name: _ })
    }

    /// Get the output if the key is of type [`Key::Char`](crate::dofinitions::Key::Char).
    pub fn char_output(&self) -> Option<char> {
        match self {
            Key::Char(c) => Some(*c),
            _ => None,
        }
    }

    /// Get the output if the key is of type [`Key::Word`](crate::dofinitions::Key::Word).
    pub fn word_output(&self) -> Option<&str> {
        match &self {
            Key::Word(s) => Some(s),
            _ => None,
        }
    }

    /// Get the layer name if the key is of type [`Key::Layer`](crate::dofinitions::Key::Layer).
    pub fn layer_output(&self) -> Option<&str> {
        match &self {
            Key::Layer { name } => Some(name),
            _ => None,
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
                Menu => "mn".into(),
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

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
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
                "menu" | "mnu" | "mn" => Special(Menu),
                "fn" => Special(Fn),
                "backspace" | "bksp" | "bcsp" | "bsp" => Special(Backspace),
                "del" => Special(Del),
                _ if s.starts_with('@') => Layer {
                    name: s.chars().skip(1).collect(),
                },
                _ if s.starts_with('#') || s.starts_with("\\#") || s.starts_with("\\@") => {
                    Word(s.chars().skip(1).collect())
                }
                _ => Word(s.into()),
            },
        }
    }
}

/// Abstraction of `Vec<usize>` where each index represents a row on a layout with a specific amount of keys.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Shape(Vec<usize>);

impl From<Vec<usize>> for Shape {
    fn from(value: Vec<usize>) -> Self {
        Shape(value)
    }
}

impl<const N: usize> From<[usize; N]> for Shape {
    fn from(value: [usize; N]) -> Self {
        Shape(value.into())
    }
}

impl Shape {
    /// Get a slice of all rows in the shape.
    pub fn inner(&self) -> &[usize] {
        &self.0
    }

    /// Consume self to get the Vector of rows in the shape.
    pub fn into_inner(self) -> Vec<usize> {
        self.0
    }

    /// Return the amount of rows in the shape.
    pub fn row_count(&self) -> usize {
        self.0.len()
    }

    /// If all rows of this shape <= the rows of the destination, it fits into the other
    pub fn fits_in(&self, cmp: &Self) -> bool {
        if self.row_count() > cmp.row_count() {
            false
        } else {
            self.inner().iter().zip(cmp.inner()).all(|(r, c)| r <= c)
        }
    }
}

/// Some default form factors. Options are Ansi, Iso, Ortho (being 3x10 + 3 thumb keys per thumb), Colstag
/// (being 3x10 + 3 thumb keys per thumb) and a custom option if any anything but the prior options is provided.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyboardType {
    Ansi,
    Iso,
    Ortho,
    Colstag,
    Custom(String),
}

impl KeyboardType {
    /// Get the shape of a certain keyboard type.
    pub fn shape(&self) -> Shape {
        self.fingering(&NamedFingering::Traditional)
            .unwrap()
            .shape()
    }

    /// Given a known fingering from `NamedFingering`, provide a `Fingering` object with all keys on a board
    /// like that specified. Will Return an error if any combination is provided that isn't valid, like
    /// `KeyboardType::Ortho` and `NamedFingering::Angle`.
    pub fn fingering(&self, named_fingering: &NamedFingering) -> Result<Fingering> {
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
                vec![LP, LR, LM, LI, LI, RI, RI, RM, RR, RP],
                vec![LP, LR, LM, LI, LI, RI, RI, RM, RR, RP],
                vec![LP, LR, LM, LI, LI, RI, RI, RM, RR, RP],
                vec![LT, LT, LT, RT, RT, RT],
            ]
            .into(),
            (Colstag, Traditional) => vec![
                vec![LP, LR, LM, LI, LI, RI, RI, RM, RR, RP],
                vec![LP, LR, LM, LI, LI, RI, RI, RM, RR, RP],
                vec![LP, LR, LM, LI, LI, RI, RI, RM, RR, RP],
                vec![LT, LT, LT, RT, RT, RT],
            ]
            .into(),
            (board, &f) => {
                return Err(DofErrorInner::UnsupportedKeyboardFingeringCombo(
                    board.clone(),
                    f.clone(),
                )
                .into())
            }
        };

        Ok(fingering)
    }

    /// Checks if the keyboard is Custom.
    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    /// Get the default anchor for each keyboard type. This is (1, 1) for `Ansi` and `Iso` boards (as the
    /// vast majority of keyboard layouts doesn't remap the number row or special keys on the left) and
    /// (0, 0) for the rest.
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

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
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
