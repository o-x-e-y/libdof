use thiserror::Error;

use std::str::FromStr;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecialKey {
    Esc,
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
    PrtScreen,
    Ins,
    Home,
    End,
    PageUp,
    PageDown,
    NumLock,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Empty,
    MainLayerEq,
    Char(char),
    Special(SpecialKey),
    Layer { name: String },
}

impl ToString for Key {
    fn to_string(&self) -> String {
        format!("{self:?}")
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
                "*" => Ok(MainLayerEq),
                " " => Ok(Special(Space)),
                "\n" => Ok(Special(Enter)),
                "\t" => Ok(Special(Tab)),
                _ => Ok(Char(s.chars().next().unwrap())),
            },
            _ => match s.to_lowercase().as_str() {
                "\\~" => Ok(Char('~')),
                "\\*" => Ok(Char('*')),
                "esc" => Ok(Special(Esc)),
                "space" | "spc" => Ok(Special(Space)),
                "tab" | "tb" => Ok(Special(Tab)),
                "enter" | "return" | "ret" | "ent" | "rt" => Ok(Special(Enter)),
                "shift" | "shft" | "sft" | "st" => Ok(Special(Shift)),
                "caps" | "cps" | "cp" => Ok(Special(Caps)),
                "ctrl" | "ctl" | "ct" => Ok(Special(Ctrl)),
                "alt" | "lalt" | "ralt" | "lt" => Ok(Special(Alt)),
                "meta" | "mta" | "met" | "mt" => Ok(Special(Meta)),
                "fn" => Ok(Special(Fn)),
                "backspace" | "bksp" | "bcsp" | "bsp" => Ok(Special(Backspace)),
                "del" => Ok(Special(Del)),
                "prtscreen" | "prt" => Ok(Special(PrtScreen)),
                "ins" => Ok(Special(Ins)),
                "home" => Ok(Special(Home)),
                "end" => Ok(Special(End)),
                "pageup" | "pgu" | "pu"=> Ok(Special(PageUp)),
                "pagedown" | "pgd" | "pd" => Ok(Special(PageDown)),
                "numlock" | "nlk" | "nml" => Ok(Special(NumLock)),
                "f1" => Ok(Special(F1)),
                "f2" => Ok(Special(F2)),
                "f3" => Ok(Special(F3)),
                "f4" => Ok(Special(F4)),
                "f5" => Ok(Special(F5)),
                "f6" => Ok(Special(F6)),
                "f7" => Ok(Special(F7)),
                "f8" => Ok(Special(F8)),
                "f9" => Ok(Special(F9)),
                "f10" => Ok(Special(F10)),
                "f11" => Ok(Special(F11)),
                "f12" => Ok(Special(F12)),
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