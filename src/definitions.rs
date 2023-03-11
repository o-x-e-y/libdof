use thiserror::Error;

use std::{fmt::Display, str::FromStr};

#[derive(Debug, Error)]
pub enum DefinitionError {
    #[error("Couldn't parse Finger from '{0}'")]
    FingerParseError(String),
    #[error("an empty string can't be parsed into a Key")]
    EmptyKeyError,
    #[error("{0}")]
    Infallible(#[from] std::convert::Infallible),
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
    type Err = DefinitionError;

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
            _ => Err(DefinitionError::FingerParseError(s.to_string())),
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
        use NamedFingering::*;

        let s = match self {
            Traditional => "traditional",
            Angle => "angle",
            Custom(name) => name.as_str(),
        };

        write!(f, "{s}")
    }
}

impl FromStr for NamedFingering {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use NamedFingering::*;

        let res = match s.to_lowercase().as_str() {
            "standard" | "traditional" => Traditional,
            "angle" => Angle,
            name => Custom(name.into()),
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
    Special(SpecialKey),
    Layer { name: String },
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Key::*;
        use SpecialKey::*;

        let s = match self {
            Empty => "~".into(),
            Transparent => "*".into(),
            Char(c) => String::from(*c),
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
    type Err = DefinitionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Key::*;
        use SpecialKey::*;
        match s.len() {
            0 => Err(DefinitionError::EmptyKeyError),
            1 => match s {
                "~" => Ok(Empty),
                "*" => Ok(Transparent),
                " " => Ok(Special(Space)),
                "\n" => Ok(Special(Enter)),
                "\t" => Ok(Special(Tab)),
                _ => Ok(Char(s.chars().next().unwrap())),
            },
            _ => match s.to_lowercase().as_str() {
                "\\~" => Ok(Char('~')),
                "\\*" => Ok(Char('*')),
                "esc" => Ok(Special(Esc)),
                "repeat" | "rpt" => Ok(Special(Repeat)),
                "space" | "spc" => Ok(Special(Space)),
                "tab" | "tb" => Ok(Special(Tab)),
                "enter" | "return" | "ret" | "ent" | "rt" => Ok(Special(Enter)),
                "shift" | "shft" | "sft" | "st" => Ok(Special(Shift)),
                "caps" | "cps" | "cp" => Ok(Special(Caps)),
                "ctrl" | "ctl" | "ct" => Ok(Special(Ctrl)),
                "alt" | "lalt" | "ralt" | "lt" => Ok(Special(Alt)),
                "meta" | "mta" | "met" | "mt" | "super" | "sup" | "sp" => Ok(Special(Meta)),
                "fn" => Ok(Special(Fn)),
                "backspace" | "bksp" | "bcsp" | "bsp" => Ok(Special(Backspace)),
                "del" => Ok(Special(Del)),
                _ => Ok(Layer { name: s.into() }),
            },
        }
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
