use thiserror::Error;

use std::str::FromStr;

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
    RP
}

#[derive(Debug, Error)]
pub enum DefinitionError {
    #[error("Couldn't parse finger from '{0}'")]
    FingerParseError(String),
    #[error("Couldn't parse key from '{0}'")]
    KeyParseError(String)
}

impl ToString for Finger {
    fn to_string(&self) -> String {
        format!("{self:?}")
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
            _ => Err(DefinitionError::FingerParseError(s.to_string()))
        }
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

impl ToString for Key {
    fn to_string(&self) -> String {
        use Key::*;
        use SpecialKey::*;

        match self {
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
                Del => "del".into()
            },
            Layer { name } => name.clone(),
        }
    }
}

impl FromStr for Key {
    type Err = DefinitionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Key::*;
        use SpecialKey::*;
        match s.len() {
            0 => Err(DefinitionError::KeyParseError("<empty>".into())),
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
                _ => Ok(Layer { name: s.into() })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Finger::*;

    #[test]
    fn as_string() {
        assert_eq!(format!("{:?}", LP), "LP");
    }
}