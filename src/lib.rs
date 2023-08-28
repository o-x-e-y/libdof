pub mod definitions;
pub mod intermediate;

use definitions::*;
use intermediate::*;

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Dof {
    name: String,
    authors: Option<Vec<String>>,
    board: KeyboardType,
    year: Option<u32>,
    notes: Option<String>,
    layers: BTreeMap<String, Layer>,
    anchor: Anchor,
    // alt_fingerings: Option<Vec<String>>,
    // combos: Option<HashMap<String, String>>,
    fingerings: Fingering,
}

impl TryFrom<DofIntermediate> for Dof {
    type Error = DofError;

    fn try_from(inter: DofIntermediate) -> Result<Self, Self::Error> {
        inter.validate()?;

        let explicit_fingering = inter.explicit_fingering(inter.main_layer()?)?;

        Ok(Self {
            name: inter.name,
            authors: inter.authors,
            board: inter.board,
            year: inter.year,
            notes: inter.notes,
            layers: inter.layers,
            anchor: inter.anchor,
            fingerings: explicit_fingering,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn no_main_layer() {
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

        let v = Dof::try_from(minimal_test);

        assert_eq!(
            v,
            Err(DofError::from(IntermediateDofErrorInner::NoMainLayer))
        );
    }

    #[test]
    fn minimal_succesful() {
        let minimal_json = json!({
            "name": "Qwerty",
            "board": "ansi",
            "layers": {
                "main": [
                    "q w e r t  y u i o p  ",
                    "a s d f g  h j k l ; '",
                    "z x c v b  n m , . /  ",
                ]
            },
            "fingering": "angle"
        });

        let inter_minimal = serde_json::from_value::<DofIntermediate>(minimal_json)
            .expect("couldn't parse explicit json");

        let d = Dof::try_from(inter_minimal);

        assert!(d.is_ok())
    }

    #[test]
    fn maximal_succesful() {
        let maximal_json = json!({
            "name": "Qwerty",
            "authors": ["Christopher Latham Sholes"],
            "board": "ansi",
            "year": 1878,
            "notes": "the OG. Without Qwerty, none of this would be necessary.",
            "anchor": [0, 0],
            "layers": {
                "main": [
                    "` 1 2 3 4 5  6 7 8 9 0 - = bsp",
                    "tb q w e r t  y u i o p [ ] \\",
                    "cps a s d f g  h j k l ; ' ret",
                    "shft z x c v b  n m , . / shft",
                    "~ ~ ~ ~      spc     altgr ~ ~"
                ],
                "shift": [
                    "\\~ ! @ # $ %  ^ & \\* ( ) _ + bsp",
                    "tab  Q W E R T  Y U   I O P { } |",
                    "caps  A S D F G  H J   K L : \" ent",
                    "*      Z X C V B  N M   < > ? shft",
                    "~ ~ ~ ~        spc     altgr ~ ~"
                ],
                "altgr": [
                    "` * * * * *  * * * * * * * bsp",
                    "tb * * * * *  * ü * ö * * * *",
                    "cps ä * * * *  * * * * * * ret",
                    "shft * * * * *  * * * * * shft",
                    "~ ~ ~ ~      spc     * ~ ~"
                ]
            },
            "fingering": [
                "0  0  1  2  3  3   6  6  7  8  9  9  9  9",
                "LP LP LR LM LI LI  RI RI RM RR RP RP RP RP",
                "LP LP LR LM LI LI  RI RI RM RR RP RP RP",
                "LP LR LM LI LI LI  RI RI RM RR RP RP",
                "LP  LP  LT  LT    LT    RT  RT  RP"
            ]
        });

        let inter_maximal = serde_json::from_value::<DofIntermediate>(maximal_json)
            .expect("couldn't parse explicit json");

        let d = Dof::try_from(inter_maximal);

        assert!(d.is_ok())
    }
}
