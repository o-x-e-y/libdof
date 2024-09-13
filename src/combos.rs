//! A way to define combos for a keyboard layout.

use crate::{
    interaction::Pos, keyboard_conv, DofError, DofErrorInner as DErr, Key, Keyboard, Layer, Result,
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::{collections::BTreeMap, iter, str::FromStr};

/// Represents a combo by way of specifying a `Key`, and if there are multiple on the keyboard,
/// the nth index. If there are 2 `e` keys for example, you can specify `e-2`.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComboKey {
    key: Key,
    nth: usize,
}

impl ComboKey {
    fn new(s: &str) -> Self {
        let key = s.parse().unwrap();

        Self { key, nth: 0 }
    }

    fn new_nth(s: &str, nth: usize) -> Self {
        let key = s.parse().unwrap();

        Self { key, nth }
    }
}

impl FromStr for ComboKey {
    type Err = DofError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let ck = match s.len() {
            0 => return Err(DErr::EmptyComboKey.into()),
            1 | 2 => Self::new(s),
            _ => match s.chars().rev().position(|c| c == '-') {
                Some(p) => {
                    let (key, num) = s.split_at(s.len() - p - 1);
                    let num = &num[1..];

                    match num.parse::<usize>() {
                        Ok(nth) => Self::new_nth(key, nth.saturating_sub(1)),
                        Err(_) => Self::new(s),
                    }
                }
                None => Self::new(s),
            },
        };

        Ok(ck)
    }
}

impl std::fmt::Display for ComboKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.nth {
            0 => write!(f, "{}", self.key),
            nth => write!(f, "{}-{}", self.key, nth),
        }
    }
}

keyboard_conv!(ComboKey, ComboKeyStrAsRow);

/// Structure to store combos for a layout. Contains a map with layer names, where each layer
/// contains a map from a `Vec` of [`ComboKey`](crate::ComboKey)s to a single `Key`.
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParseCombos(
    #[serde_as(as = "BTreeMap<_, BTreeMap<ComboKeyStrAsRow, DisplayFromStr>>")]
    pub  BTreeMap<String, BTreeMap<Vec<ComboKey>, Key>>,
);

impl ParseCombos {
    /// Convert layers to a `Key` + row/column map.
    pub(crate) fn into_pos_layers(self, layers: &BTreeMap<String, Layer>) -> Result<Combos> {
        let layers = layers
            .iter()
            .map(|(name, layer)| {
                let layer = layer
                    .rows()
                    .enumerate()
                    .flat_map(|(i, row)| {
                        row.iter()
                            .enumerate()
                            .map(move |(j, key)| (Pos::new(i, j), key))
                    })
                    .collect::<Vec<_>>();
                (name.as_str(), layer)
            })
            .collect::<BTreeMap<_, _>>();

        self.0
            .into_iter()
            .flat_map(|(layer_name, combos)| {
                let layer = layers.get(layer_name.as_str()).map(|l| l.as_slice());
                iter::repeat((layer_name, layer)).zip(combos)
            })
            .map(|((layer_name, layer), (combo, output))| {
                let l = layer.ok_or_else(|| {
                    DErr::UnknownComboLayer(layer_name.clone(), combo_to_str(&combo))
                })?;

                combo
                    .iter()
                    .map(|ck| {
                        l.iter()
                            .filter_map(|(pos, key)| (**key == ck.key).then_some(*pos))
                            .nth(ck.nth)
                            .ok_or_else(|| {
                                DErr::InvalidKeyIndex(
                                    combo_to_str(&combo),
                                    ck.key.to_string(),
                                    ck.nth,
                                )
                                .into()
                            })
                    })
                    .collect::<Result<Vec<_>>>()
                    .map(|combo| (layer_name, (combo, output)))
            })
            .try_fold(
                BTreeMap::new(),
                |mut acc: BTreeMap<_, Vec<_>>, layer_combo| match layer_combo {
                    Ok((layer_name, combo)) => {
                        acc.entry(layer_name).or_default().push(combo);
                        Ok(acc)
                    }
                    Err(e) => Err(e),
                },
            )
            .map(Combos)
    }
}

/// Fully parsed `Dof` representation of combos on a layout. In here is a BTreeMap mapping layer
/// names by `String` to a vector of `(Vec<Pos>, Key)` which are all combos on a keyboard, mapped
/// by their row/column index.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Combos(pub BTreeMap<String, Vec<(Vec<Pos>, Key)>>);

impl Combos {
    pub(crate) fn into_parse_combos(self, layers: &BTreeMap<String, Layer>) -> Option<ParseCombos> {
        if self.0.is_empty() {
            return None;
        }

        let parse_combos = self
            .0
            .into_iter()
            .map(|(name, combos)| {
                let layer = &layers.get(&name).unwrap().0;

                let layer_combos = combos
                    .into_iter()
                    .map(move |(combo, key)| {
                        let combo = combo
                            .into_iter()
                            .map(|pos| {
                                let key = layer[pos.row()][pos.col()].clone();
                                let nth = layer[..(pos.row() + 1)]
                                    .iter()
                                    .flat_map(move |row| &row[..(pos.col() + 1)])
                                    .filter(|k| k == &&key)
                                    .count();
                                let nth = match nth {
                                    0 | 1 => 0,
                                    n => n,
                                };
                                ComboKey::new_nth(&key.to_string(), nth)
                            })
                            .collect::<Vec<_>>();
                        (combo, key)
                    })
                    .collect();
                (name, layer_combos)
            })
            .collect();

        Some(ParseCombos(parse_combos))
    }
}

fn combo_to_str(combos: &[ComboKey]) -> String {
    if combos.is_empty() {
        String::new()
    } else {
        combos
            .iter()
            .take(combos.len() - 1)
            .map(|c| format!("{c} "))
            .chain([combos.last().unwrap().to_string()])
            .collect::<String>()
    }
}

#[cfg(test)]
pub(crate) fn ck(key: Key, nth: usize) -> ComboKey {
    ComboKey { key, nth }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Key::*, SpecialKey::*};

    #[test]
    fn parse_combos() {
        let json = r#"
            {
                "main": {
                    "a b": "x"
                },
                "edge-cases": {
                    "-1 1-": "6",
                    "--1": "d",
                    "---": "X",
                    "🦀-12": "rpt",
                    "a-1 b-2 c-3 ~-4 rpt-5": "*"
                }
            }
        "#;

        let parse =
            serde_json::from_str::<ParseCombos>(json).expect("couldn't parse combos json: ");

        let reference = ParseCombos(BTreeMap::from([
            (
                "main".to_string(),
                BTreeMap::from([(vec![ck(Char('a'), 0), ck(Char('b'), 0)], Char('x'))]),
            ),
            (
                "edge-cases".to_string(),
                BTreeMap::from([
                    (
                        vec![ck(Word("-1".into()), 0), ck(Word("1-".into()), 0)],
                        Char('6'),
                    ),
                    (vec![ck(Char('-'), 0)], Char('d')),
                    (vec![ck(Word("---".into()), 0)], Char('X')),
                    (vec![ck(Char('🦀'), 11)], Special(Repeat)),
                    (
                        vec![
                            ck(Char('a'), 0),
                            ck(Char('b'), 1),
                            ck(Char('c'), 2),
                            ck(Empty, 3),
                            ck(Special(Repeat), 4),
                        ],
                        Transparent,
                    ),
                ]),
            ),
        ]));

        assert_eq!(parse, reference);
    }

    #[test]
    fn to_combos_simple() {
        let json = r#"
            {
                "main": {
                    "a b": "x",
                    "e-2 b e": "rpt"
                }
            }
        "#;

        let parse =
            serde_json::from_str::<ParseCombos>(json).expect("couldn't parse combos json: ");

        let layers = BTreeMap::from_iter([(
            "main".to_owned(),
            vec![vec![Char('a'), Char('e'), Char('b'), Char('c'), Char('e')]].into(),
        )]);

        let combos = parse.into_pos_layers(&layers);

        assert_eq!(
            combos,
            Ok(Combos(BTreeMap::from_iter([(
                "main".to_owned(),
                vec![
                    (vec![Pos::new(0, 0), Pos::new(0, 2)], Char('x')),
                    (
                        vec![Pos::new(0, 4), Pos::new(0, 2), Pos::new(0, 1)],
                        Special(Repeat)
                    )
                ]
            )])))
        );

        let parse_combos = combos.unwrap().into_parse_combos(&layers);

        let s = serde_json::to_string(&parse_combos).unwrap();

        println!("{s}")
    }
}
