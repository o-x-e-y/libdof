#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub mod dofinitions;
pub mod interaction;
pub mod keyboard;
mod macros;
pub mod prelude;

use interaction::{KeyPos, Pos};
use keyboard::{ParseKeyboard, PhysicalKey, PhysicalKeyboard};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, DisplayFromStr};
use thiserror::Error;

use std::{collections::BTreeMap, num::ParseFloatError};

use dofinitions::*;

/// A struct to represent the dof keyboard layout spec. This struct is useful for interacting with dofs
/// and parsing to/from .dof using [`serde_json`](https://crates.io/crates/serde_json). For converting
/// other formats into dofs, consider taking a look at [`DofIntermediate`](crate::DofIntermediate).
///
/// # Example
///
/// Parsing into dof and getting the name of the layout:
///
/// ```
/// # use serde_json;
/// # use libdof::Dof;
/// # fn p() -> Result<(), Box<dyn std::error::Error>> {
/// let dof_str = include_str!("../example_dofs/minimal_valid.dof");
/// let dof = serde_json::from_str::<Dof>(dof_str)?;
/// let name = dof.name();
/// # Ok(()) }
/// # fn main() { p(); }
/// ```
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "DofIntermediate", into = "DofIntermediate")]
pub struct Dof {
    name: String,
    authors: Option<Vec<String>>,
    board: PhysicalKeyboard,
    parsed_board: ParseKeyboard,
    year: Option<u32>,
    description: Option<String>,
    languages: Vec<Language>,
    link: Option<String>,
    layers: BTreeMap<String, Layer>,
    anchor: Anchor,
    // alt_fingerings: Option<Vec<String>>,
    // combos: Option<HashMap<String, String>>,
    fingering: Fingering,
    fingering_name: Option<NamedFingering>,
    has_generated_shift: bool,
}

impl Dof {
    /// Get the name of the layout.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get an optional slice of authors of the layout.
    pub fn authors(&self) -> Option<&[String]> {
        self.authors.as_deref()
    }

    /// Get the [`KeyboardType`](crate::dofinitions::KeyboardType) of the layout.
    pub const fn board(&self) -> &PhysicalKeyboard {
        &self.board
    }

    /// Get the [`KeyboardType`](crate::dofinitions::KeyboardType) of the layout. `Custom::("")`
    /// if a custom physical keyboard was provided.
    pub fn board_type(&self) -> KeyboardType {
        match &self.parsed_board {
            ParseKeyboard::Named(n) => n.clone(),
            _ => KeyboardType::Custom("".into()),
        }
    }

    /// Get the optional publication year of the layout.
    pub const fn year(&self) -> Option<u32> {
        self.year
    }

    /// Get the optional description of the layout.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get the optional link of the layout.
    pub fn link(&self) -> Option<&str> {
        self.link.as_deref()
    }

    /// Get a slice of [Language](crate::Language) this layout was intended to be used for.
    pub fn languages(&self) -> &[Language] {
        &self.languages
    }

    /// Get a map containing the layer names and its corresponding layer on the layout.
    pub fn layers(&self) -> &BTreeMap<String, Layer> {
        &self.layers
    }

    /// Get the layout anchor, which specifies the coordinate of the top left corner of the layout compared to
    /// the physical keyboard it's on.
    pub const fn anchor(&self) -> Anchor {
        self.anchor
    }

    /// Get the shape of the fingering and layers of the dof
    pub fn shape(&self) -> Shape {
        self.fingering().shape()
    }

    /// Get the fingering of the keyboard, which specifies for each coordinate which finger is supposed to press
    /// what key.
    pub const fn fingering(&self) -> &Fingering {
        &self.fingering
    }

    /// If present, get a specified type of fingering that the layout uses.
    pub fn fingering_name(&self) -> Option<&NamedFingering> {
        self.fingering_name.as_ref()
    }

    /// Get the main layer of the layout. Contains a call to `expect()` but since creating a
    /// `Dof` without a main layer is impossible, it should never fail.
    pub fn main_layer(&self) -> &Layer {
        self.layers
            .get("main")
            .expect("Creating a Dof without a main layer should be impossible")
    }

    /// Get the shift layer of the layout. Contains a call to `expect()` but since creating a
    /// `Dof` without a main layer is impossible, it should never fail.
    pub fn shift_layer(&self) -> &Layer {
        self.layers
            .get("shift")
            .expect("Creating a Dof without a shift layer should be impossible")
    }

    /// Get a specific layer on the keyboard, if it exists.
    pub fn layer(&self, name: &str) -> Option<&Layer> {
        self.layers.get(name)
    }

    /// Get a vector of keys with metadata for each key attached. This can be useful if you want
    /// to filter or any other way look at a specific set of keys on the keyboard.
    pub fn keys(&self) -> Vec<DescriptiveKey> {
        self.layers()
            .iter()
            .flat_map(|(name, layer)| {
                layer
                    .rows()
                    .enumerate()
                    .zip(self.fingering.rows())
                    .zip(self.board.rows())
                    .flat_map(move |(((row, key_row), finger_row), phys_row)| {
                        key_row
                            .iter()
                            .enumerate()
                            .zip(finger_row)
                            .zip(phys_row)
                            .map(move |(((col, key), &finger), phys)| {
                                DescriptiveKey::new(key, name, row, col, finger, phys)
                            })
                    })
            })
            .collect()
    }
}

impl TryFrom<DofIntermediate> for Dof {
    type Error = DofError;

    fn try_from(mut inter: DofIntermediate) -> std::result::Result<Self, Self::Error> {
        let main_layer = inter.main_layer()?;

        inter.validate_layer_keys(main_layer)?;
        inter.validate_layer_shapes(main_layer)?;

        let explicit_fingering = inter.explicit_fingering(main_layer)?;
        let implicit_fingering = match inter.fingering.clone().unwrap_or_default() {
            ParsedFingering::Implicit(f) => Some(f),
            _ => None,
        };

        let has_generated_shift = if !inter.layers.contains_key("shift") {
            inter.layers.insert(
                "shift".into(),
                DofIntermediate::generate_shift_layer(main_layer),
            );
            true
        } else {
            false
        };

        let anchor = match inter.anchor {
            None => inter.board.anchor(),
            Some(a) => a,
        };

        let board = PhysicalKeyboard::try_from(inter.board.clone())?
            .resized(anchor, explicit_fingering.shape())?
            .into_iter()
            .map(|v| {
                v.into_iter()
                    .map(PhysicalKey::normalized)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
            .into();

        let languages = match inter.languages {
            Some(l) => l,
            None => vec![Language::default()],
        };

        Ok(Self {
            name: inter.name,
            authors: inter.authors,
            board,
            parsed_board: inter.board,
            year: inter.year,
            description: inter.description,
            languages,
            link: inter.link,
            layers: inter.layers,
            anchor,
            fingering: explicit_fingering,
            fingering_name: implicit_fingering,
            has_generated_shift,
        })
    }
}

impl From<Dof> for DofIntermediate {
    fn from(mut dof: Dof) -> DofIntermediate {
        if dof.has_generated_shift {
            dof.layers.remove("shift");
        }

        let fingering = dof
            .fingering_name
            .map(ParsedFingering::Implicit)
            .unwrap_or(ParsedFingering::Explicit(dof.fingering));

        let fingering = if fingering == ParsedFingering::default() {
            None
        } else {
            Some(fingering)
        };

        let languages = match dof.languages.as_slice() {
            [lang] if lang == &Language::default() => None,
            _ => Some(dof.languages.clone()),
        };

        let anchor = match &dof.parsed_board {
            ParseKeyboard::Named(n) => match n.anchor() {
                a if a == dof.anchor => None,
                a => Some(a),
            },
            _ => None,
        };

        DofIntermediate {
            name: dof.name,
            authors: dof.authors,
            board: dof.parsed_board,
            year: dof.year,
            description: dof.description,
            languages,
            link: dof.link,
            layers: dof.layers,
            anchor,
            fingering,
        }
    }
}

#[derive(Debug, Error, PartialEq)]
enum DofErrorInner {
    #[error("This layout is missing a main layer")]
    NoMainLayer,
    #[error("Found these layer keys '{0:?}' however these layers do not actually exist")]
    LayersNotFound(Vec<String>),
    #[error("The shape of these layers: '{0:?}' are not the same as the main layer")]
    IncompatibleLayerShapes(Vec<String>),
    #[error("The layer shapes do not match the fingering shape")]
    IncompatibleFingeringShape,
    #[error("The provided layout + anchor don't fit within the given fingering")]
    LayoutDoesntFit,
    #[error("The anchor provided is bigger than the layout it is used for")]
    AnchorBiggerThanLayout,

    #[error("Couldn't parse Finger from '{0}'")]
    FingerParseError(String),
    #[error("Can't combine keyboard type '{0}' with fingering '{1}'")]
    UnsupportedKeyboardFingeringCombo(KeyboardType, NamedFingering),
    #[error("Default fingering only exists for known keyboards: ansi, iso, ortho and colstag")]
    FingeringForCustomKeyboard,

    #[error("Couldn't parse physical key from '{0}' because a float couldn't be parsed")]
    KeyParseError(String),
    #[error("Couldn't parse physical key because the string is empty")]
    EmptyPhysKey,
    #[error("Expected 2, 3 or 4 values in physical key definition, found {0} for '{1}'")]
    ValueAmountError(usize, String),
    #[error("Keyboard type '{0}' does not match a default physical keyboard.")]
    UnknownKeyboardType(KeyboardType),

    #[error("the provided layer name '{0}' is invalid")]
    LayerDoesntExist(String),
    #[error("the given position ({0}, {1}) is not available on the keyboard")]
    InvalidPosition(u8, u8),

    #[error("{0}")]
    Infallible(#[from] std::convert::Infallible),
    #[error("{0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("{0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("{0}")]
    Custom(String),
}

use DofErrorInner as DErr;

type Result<T> = std::result::Result<T, DofError>;

/// The main error struct of the library. Internally it uses a Box containing [`DofErrorInner`](crate::DofErrorInner)
/// to save space.
#[derive(Debug, Error, PartialEq)]
#[error("{0}")]
pub struct DofError(#[source] Box<DofErrorInner>);

impl DofError {
    /// Allows users of the crate to create their own error messages if needed.
    pub fn custom(msg: &str) -> Self {
        DofError(Box::new(DErr::Custom(msg.into())))
    }
}

impl From<DofErrorInner> for DofError {
    fn from(value: DofErrorInner) -> Self {
        Self(Box::new(value))
    }
}

impl From<ParseFloatError> for DofError {
    fn from(value: ParseFloatError) -> Self {
        DErr::ParseFloatError(value).into()
    }
}

/// Used to represent the language(s) a layout is optimized for, containing the name of a language
/// as well as a weight, the latter being useful for layouts that are made for a combination of
/// languages with some amount of % split.
///
/// The Default implementation of Language is English with weight 100.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Language {
    /// Language
    pub language: String,
    /// Weight of the language, meaning how important it is compared to possible other languages in
    /// the `Dof`.
    pub weight: usize,
}

impl Default for Language {
    fn default() -> Self {
        Language {
            language: "English".into(),
            weight: 100,
        }
    }
}

impl Language {
    /// Create a new language
    pub fn new(language: &str, weight: usize) -> Self {
        let language = language.into();
        Self { language, weight }
    }

    /// Presets the weight to be 100, small shorthand for when you only need one language though in theory
    /// you could use two of these languages to represent a `(100 + 100) / 2 = 50%` split.
    pub fn only(language: &str) -> Self {
        Self {
            language: language.into(),
            weight: 100,
        }
    }
}

/// Overarching trait for any type that contains a `Vec<Vec<K>>` represinting one aspect of
/// a keyboard(layout). In libdof these are `Layer` and `Fingering`, but this could also be
/// implemented for a heatmap type or a physical keyboard for example.
pub trait Keyboard {
    /// A type representing a key.
    type K: Clone;

    /// Get an iterator over each row of the keyboard.
    fn rows(&self) -> impl Iterator<Item = &Vec<Self::K>> {
        self.inner().iter()
    }

    /// Get an iterator over the individual keys of the keyboard.
    fn keys(&self) -> impl Iterator<Item = &Self::K> {
        self.rows().flatten()
    }

    /// Get the shape of the keyboard.
    fn shape(&self) -> Shape {
        self.rows().map(|r| r.len()).collect::<Vec<_>>().into()
    }

    /// Get the amount of rows of the keyboard.
    fn row_count(&self) -> usize {
        self.rows().count()
    }

    /// Get a reference to the inner rows of the keyboard.
    fn inner(&self) -> &[Vec<Self::K>];

    /// Convert into underlying vectors of the keyboard.
    fn into_inner(self) -> Vec<Vec<Self::K>>;

    /// For each row of the keyboard, checks whether or not it's smaller or equal to the given
    /// shape's row.
    fn fits_in(&self, shape: &Shape) -> bool {
        self.shape().fits_in(shape)
    }

    /// Given a specific keyboard, an [`Anchor`](crate::Anchor) and the [`Shape`](crate::Shape),
    /// resize to the given shape. Returns an error if the shape is bigger than the provided keyboard.
    fn resized(&self, Anchor(x, y): Anchor, desired_shape: Shape) -> Result<Vec<Vec<Self::K>>> {
        let (offset_x, offset_y) = (x as usize, y as usize);

        let anchor_resized = self
            .inner()
            .get(offset_y..)
            .ok_or(DErr::AnchorBiggerThanLayout)?
            .iter()
            .map(|r| r.get(offset_x..).ok_or(DErr::AnchorBiggerThanLayout.into()))
            .collect::<Result<Vec<_>>>()?;

        anchor_resized
            .into_iter()
            .zip(desired_shape.into_inner())
            .map(|(row, shape_size)| {
                row.get(..shape_size)
                    .ok_or(DErr::LayoutDoesntFit.into())
                    .map(|v| v.to_vec())
            })
            .collect::<Result<Vec<_>>>()
            .map(Into::into)
    }
}

/// Struct that represents the fingering of each layout. It is an abstraction over `Vec<Vec<Finger>>`.
#[serde_as]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fingering(#[serde_as(as = "Vec<FingeringStrAsRow>")] Vec<Vec<Finger>>);

impl Keyboard for Fingering {
    type K = Finger;

    fn inner(&self) -> &[Vec<Self::K>] {
        &self.0
    }

    fn into_inner(self) -> Vec<Vec<Self::K>> {
        self.0
    }
}

impl From<Vec<Vec<Finger>>> for Fingering {
    fn from(f: Vec<Vec<Finger>>) -> Self {
        Self(f)
    }
}

keyboard_conv!(Finger, FingeringStrAsRow);

/// Abstraction over the way an actual .dof file is allowed to represent the fingering of a layout, being either
/// explicit through providing a list of fingerings for each key, or implicit, by providing a name.
#[serde_as]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParsedFingering {
    /// Covers the case where fingering is specified explicitly for each key
    Explicit(Fingering),
    /// Covers the case where fingering is specified implicitly, by providing a name like `traditional`,
    /// `standard` or `angle`
    Implicit(#[serde_as(as = "DisplayFromStr")] NamedFingering),
}

impl Default for ParsedFingering {
    fn default() -> Self {
        Self::Implicit(Default::default())
    }
}

/// An abstraction of `Vec<Vec<Key>>` to represent a layer on a layout.
#[serde_as]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layer(#[serde_as(as = "Vec<LayerStrAsRow>")] Vec<Vec<Key>>);

impl Keyboard for Layer {
    type K = Key;

    fn inner(&self) -> &[Vec<Self::K>] {
        &self.0
    }

    fn into_inner(self) -> Vec<Vec<Self::K>> {
        self.0
    }
}

impl From<Vec<Vec<Key>>> for Layer {
    fn from(f: Vec<Vec<Key>>) -> Self {
        Self(f)
    }
}

keyboard_conv!(Key, LayerStrAsRow);

/// An anchor represents where the top left key on a `Dof` is compared to where it would be on a physical
/// keyboard. For example, if you were to provide a 3x10 raster of letters but would like this applied to an
/// ANSI keyboard, the `Anchor` would be (1, 1), as the top left corner of the `Dof` (being where qwerty `q`
/// is) would need to be shifted one left and one up to be in the top left corner of the physical keyboard.
/// Therefore, the default value of an anchor is dependent on the physical keyboard it is applied to.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Anchor(u8, u8);

impl Anchor {
    /// Create a new anchor
    pub const fn new(x: u8, y: u8) -> Self {
        Anchor(x, y)
    }

    /// Return the x coordinate as usize
    pub const fn x(&self) -> usize {
        self.0 as usize
    }

    /// Return the y coordinate as usize
    pub const fn y(&self) -> usize {
        self.1 as usize
    }
}

/// A Key with metadata attached. These are produced by calling [`Dof::keys()`](crate::Dof::keys()).
#[derive(Clone, Debug, PartialEq)]
pub struct DescriptiveKey<'a> {
    output: &'a Key,
    layer: &'a str,
    pos: Pos,
    finger: Finger,
    phys: &'a PhysicalKey,
}

impl<'a> DescriptiveKey<'a> {
    /// Create a new DescriptiveKey.
    fn new(
        output: &'a Key,
        layer: &'a str,
        row: usize,
        col: usize,
        finger: Finger,
        physical_pos: &'a PhysicalKey,
    ) -> Self {
        let pos = Pos::new(row, col);
        Self {
            output,
            layer,
            pos,
            finger,
            phys: physical_pos,
        }
    }

    /// Get the [`KeyPos`](crate::interaction::KeyPos) of a certain key, containing the layer name as well
    /// its row and column coordinates.
    pub fn keypos(&self) -> KeyPos {
        KeyPos::new(self.layer, self.pos)
    }

    /// Get the key's row and column.
    pub const fn pos(&self) -> Pos {
        self.pos
    }

    /// Get the key's row.
    pub const fn row(&self) -> usize {
        self.pos.row()
    }

    /// Get the key's column.
    pub const fn col(&self) -> usize {
        self.pos.col()
    }

    /// Get the finger the key is supposed to be pressed with.
    pub const fn finger(&self) -> Finger {
        self.finger
    }

    /// Get the key's output.
    pub const fn output(&self) -> &Key {
        self.output
    }

    /// Get the key's physical location
    pub const fn physical_pos(&self) -> &PhysicalKey {
        self.phys
    }

    /// Get the name of the layer of the key.
    pub fn layer_name(&self) -> &str {
        self.layer
    }

    /// Check if the key is on a certain finger.
    pub const fn is_on_finger(&self, finger: Finger) -> bool {
        (self.finger as u8) == (finger as u8)
    }

    /// Check if the key is on any of the provided fingers.
    pub fn is_on_fingers(&self, fingers: &[Finger]) -> bool {
        fingers.iter().any(|f| self.finger == *f)
    }

    /// Check if the key is on left hand, including left thumb.
    pub const fn is_on_left_hand(&self) -> bool {
        self.finger.is_on_left_hand()
    }

    /// Check if the key is on left hand, including left thumb.
    pub const fn is_on_right_hand(&self) -> bool {
        self.finger.is_on_right_hand()
    }

    /// Check if the key is on a specific layer.
    pub fn is_on_layer(&self, layer: &str) -> bool {
        self.layer == layer
    }

    /// Check if the key is of type [`Key::Char`](crate::dofinitions::Key::Char) which outputs
    /// a single character.
    pub const fn is_char_key(&self) -> bool {
        self.output.is_char()
    }

    /// Check if the key is of type [`Key::Word`](crate::dofinitions::Key::Word) which outputs a specific
    /// string.
    pub const fn is_word_key(&self) -> bool {
        self.output.is_word()
    }

    /// Check if the key is of type [`Key::Empty`](crate::dofinitions::Key::Empty) which doesn't output
    /// anything.
    pub const fn is_empty_key(&self) -> bool {
        self.output.is_empty()
    }

    /// Check if the key is of type [`Key::Transparent`](crate::dofinitions::Key::Char) which outputs
    /// whatever it is the main layer outputs in that position.
    pub const fn is_transparent_key(&self) -> bool {
        self.output.is_transparent()
    }

    /// Check if the key is of type [`Key::Layer`](crate::dofinitions::Key::Layer) which holds the name.
    /// of a layer on the layout
    pub const fn is_layer_key(&self) -> bool {
        self.output.is_layer()
    }

    /// Get the output if the key is of type [`Key::Char`](crate::dofinitions::Key::Char).
    pub const fn char_output(&self) -> Option<char> {
        self.output.char_output()
    }

    /// Get the output if the key is of type [`Key::Word`](crate::dofinitions::Key::Word).
    pub fn word_output(&self) -> Option<&str> {
        self.output.word_output()
    }

    /// Get the layer name if the key is of type [`Key::Layer`](crate::dofinitions::Key::Layer).
    pub fn layer_output(&self) -> Option<&str> {
        self.output.layer_output()
    }
}

/// Main struct to use for parsing, and a more or less literal interpretation of what a .dof file can contain.
/// As its fields are public, this can also be useful for implementing `TryFrom<Type> for Dof` because at the
/// end of that function you can call `intermediate.try_into()` to handle all validation for you.
#[allow(missing_docs)]
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DofIntermediate {
    pub name: String,
    pub authors: Option<Vec<String>>,
    // #[serde_as(as = "DisplayFromStr")]
    // pub board: KeyboardType,
    pub board: ParseKeyboard,
    pub year: Option<u32>,
    pub description: Option<String>,
    pub languages: Option<Vec<Language>>,
    pub link: Option<String>,
    pub layers: BTreeMap<String, Layer>,
    pub anchor: Option<Anchor>,
    // pub alt_fingerings: Option<Vec<String>>,
    // pub combos: Option<HashMap<String, String>>,
    pub fingering: Option<ParsedFingering>,
}

impl DofIntermediate {
    /// Get the main layer if it exists. If it doesn't return a `NoMainLayer` error.
    pub fn main_layer(&self) -> Result<&Layer> {
        self.layers.get("main").ok_or(DErr::NoMainLayer.into())
    }

    /// If not provided, will generate a default shift layer with some sane defaults. This is useful
    /// if your shift layer isn't doing anything special. The defaults are:
    /// * Letters are uppercased, unless their uppercase version spans multiple characters,
    /// * Symbols and numbers are given their qwerty uppercase. This means that `7` becomes `&`, `'`
    /// becomes `"`, `[` becomes `{`, etc,
    /// * Special keys become Transparent.
    ///
    /// **Words are unaffected!** This means that if you would like Word keys to output something different,
    /// you must specify a custom shift layer.
    pub fn generate_shift_layer(main: &Layer) -> Layer {
        main.0
            .iter()
            .map(|row| row.iter().map(|k| k.shifted()).collect::<Vec<_>>())
            .collect::<Vec<_>>()
            .into()
    }

    /// Validation check to see if the layers the [`Key::Layer`](crate::dofinitions::Key::Layer)
    /// keys point to layers that actually exist.
    pub fn validate_layer_keys(&self, main: &Layer) -> Result<()> {
        let layers_dont_exist = main
            .keys()
            .filter_map(|k| match k {
                Key::Layer { name: n } if !self.layers.contains_key(n) => Some(n.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        if layers_dont_exist.is_empty() {
            Ok(())
        } else {
            Err(DErr::LayersNotFound(layers_dont_exist).into())
        }
    }

    /// Validation check to see if all layers are the same shape as the main layer.
    pub fn validate_layer_shapes(&self, main: &Layer) -> Result<()> {
        let main_shape = main.shape();

        let incompatible_shapes = self
            .layers
            .iter()
            .map(|(name, l)| (name, l.shape()))
            .filter(|(_, shape)| shape != &main_shape)
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();

        if incompatible_shapes.is_empty() {
            Ok(())
        } else {
            Err(DErr::IncompatibleLayerShapes(incompatible_shapes).into())
        }
    }

    /// Validation check to see if the provided fingering has the same shape as the main layer.
    /// If left implicit (by leaving just a name of a fingering, like `traditional` or `angle`)
    /// will try to generate a fingering with the same shape as the main layer.
    pub fn explicit_fingering(&self, main: &Layer) -> Result<Fingering> {
        use ParsedFingering::*;

        let d = Default::default();
        let fingering = match &self.fingering {
            Some(f) => f,
            None => &d,
        };

        match fingering {
            Explicit(f) if f.shape() == main.shape() => Ok(f.clone()),
            Explicit(_) => Err(DErr::IncompatibleFingeringShape.into()),
            Implicit(named) => {
                let fingering = self.board.fingering(named)?;

                let anchor = match self.anchor {
                    Some(a) => a,
                    None => self.board.anchor(),
                };

                fingering.resized(anchor, main.shape()).map(Into::into)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use keyboard::{RelativeKey, RelativeKeyboard};

    use super::*;

    #[test]
    fn no_main_layer() {
        let minimal_test = DofIntermediate {
            name: "Qwerty".into(),
            authors: None,
            board: ParseKeyboard::Named(KeyboardType::Ansi),
            year: None,
            description: None,
            languages: Default::default(),
            link: None,
            anchor: None,
            layers: BTreeMap::new(),
            fingering: Some(ParsedFingering::Implicit(NamedFingering::Angle)),
        };

        let v = Dof::try_from(minimal_test);

        assert_eq!(v, Err(DofError::from(DErr::NoMainLayer)));
    }

    #[test]
    fn parse_minimal() {
        let minimal_json = include_str!("../example_dofs/minimal_parsable.dof");

        let minimal_test = DofIntermediate {
            name: "Qwerty".into(),
            authors: None,
            board: ParseKeyboard::Named(KeyboardType::Ansi),
            year: None,
            description: None,
            languages: None,
            link: None,
            anchor: None,
            layers: BTreeMap::new(),
            fingering: None,
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
            board: PhysicalKeyboard::try_from(ParseKeyboard::Named(KeyboardType::Ansi))
                .unwrap()
                .resized(KeyboardType::Ansi.anchor(), vec![10, 11, 10].into())
                .unwrap()
                .into(),
            parsed_board: ParseKeyboard::Named(KeyboardType::Ansi),
            year: None,
            description: None,
            languages: vec![Default::default()],
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
        };

        assert_eq!(d, d_manual);

        let reconvert_json =
            serde_json::to_string_pretty(&d).expect("Couldn't reconvert to json value");

        println!("{reconvert_json}")
    }

    #[test]
    fn parse_aptmak() {
        use Finger::*;
        use Key::*;

        let aptmak_json = include_str!("../example_dofs/aptmak.dof");

        let d = serde_json::from_str::<Dof>(aptmak_json).expect("Couldn't serialize as Dof");

        let d_manual = Dof {
            name: "Aptmak".into(),
            authors: None,
            board: PhysicalKeyboard::try_from(ParseKeyboard::Named(KeyboardType::Colstag))
                .unwrap()
                .resized(KeyboardType::Colstag.anchor(), vec![10, 10, 10, 6].into())
                .unwrap()
                .into(),
            parsed_board: ParseKeyboard::Named(KeyboardType::Colstag),
            year: None,
            description: None,
            languages: vec![Default::default()],
            link: None,
            anchor: KeyboardType::Colstag.anchor(),
            layers: BTreeMap::from_iter([
                (
                    "main".into(),
                    vec![
                        vec![
                            Char('v'),
                            Char('w'),
                            Char('f'),
                            Char('p'),
                            Char('b'),
                            Char('j'),
                            Char('l'),
                            Char('u'),
                            Char('y'),
                            Char('\''),
                        ],
                        vec![
                            Char('r'),
                            Char('s'),
                            Char('t'),
                            Char('h'),
                            Char('k'),
                            Char('x'),
                            Char('n'),
                            Char('a'),
                            Char('i'),
                            Char('o'),
                        ],
                        vec![
                            Char(';'),
                            Char('c'),
                            Char('g'),
                            Char('d'),
                            Char('q'),
                            Char('z'),
                            Char('m'),
                            Char(','),
                            Char('.'),
                            Char('/'),
                        ],
                        vec![
                            Empty,
                            Special(SpecialKey::Space),
                            Empty,
                            Empty,
                            Char('e'),
                            Empty,
                        ],
                    ]
                    .into(),
                ),
                (
                    "shift".into(),
                    vec![
                        vec![
                            Char('V'),
                            Char('W'),
                            Char('F'),
                            Char('P'),
                            Char('B'),
                            Char('J'),
                            Char('L'),
                            Char('U'),
                            Char('Y'),
                            Char('"'),
                        ],
                        vec![
                            Char('R'),
                            Char('S'),
                            Char('T'),
                            Char('H'),
                            Char('K'),
                            Char('X'),
                            Char('N'),
                            Char('A'),
                            Char('I'),
                            Char('O'),
                        ],
                        vec![
                            Char(':'),
                            Char('C'),
                            Char('G'),
                            Char('D'),
                            Char('Q'),
                            Char('Z'),
                            Char('M'),
                            Char('<'),
                            Char('>'),
                            Char('?'),
                        ],
                        vec![Empty, Transparent, Empty, Empty, Char('E'), Empty],
                    ]
                    .into(),
                ),
            ]),
            fingering: {
                vec![
                    vec![LP, LR, LM, LI, LI, RI, RI, RM, RR, RP],
                    vec![LP, LR, LM, LI, LI, RI, RI, RM, RR, RP],
                    vec![LP, LR, LM, LI, LI, RI, RI, RM, RR, RP],
                    vec![LT, LT, LT, RT, RT, RT],
                ]
                .into()
            },
            fingering_name: Some(NamedFingering::Traditional),
            has_generated_shift: true,
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
            board: ParseKeyboard::Named(KeyboardType::Ansi),
            year: None,
            description: None,
            languages: None,
            link: None,
            anchor: None,
            layers: BTreeMap::new(),
            fingering: Some(ParsedFingering::Implicit(NamedFingering::Angle)),
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

    fn rk(width: f64) -> RelativeKey {
        RelativeKey {
            width,
            has_key: true,
        }
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
            year: Some(1878),
            description: Some("the OG. Without Qwerty, none of this would be necessary.".into()),
            languages: None,
            link: Some("https://en.wikipedia.org/wiki/QWERTY".into()),
            anchor: Some(Anchor::new(0, 0)),
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
                Some(ParsedFingering::Explicit(Fingering::from(vec![
                    vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP, RP],
                    vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP, RP],
                    vec![LP, LP, LR, LM, LI, LI, RI, RI, RM, RR, RP, RP, RP],
                    vec![LP, LR, LM, LI, LI, LI, RI, RI, RM, RR, RP, RP],
                    vec![LP, LP, LT, LT, LT, RT, RT, RP],
                ])))
            },
            board: ParseKeyboard::Relative(RelativeKeyboard::from(vec![
                vec![
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(2.0),
                ],
                vec![
                    rk(1.5),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.5),
                ],
                vec![
                    rk(1.75),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(2.25),
                ],
                vec![
                    rk(2.25),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(1.0),
                    rk(2.75),
                ],
                vec![
                    rk(1.25),
                    rk(1.25),
                    rk(1.25),
                    rk(6.25),
                    rk(1.25),
                    rk(1.25),
                    rk(1.25),
                    rk(1.25),
                ],
            ])),
        };

        let dof_maximal = serde_json::from_str::<DofIntermediate>(maximal_json)
            .expect("couldn't parse explicit json");

        assert_eq!(dof_maximal, maximal_test);
    }

    #[test]
    fn lang_fn() {
        let languages = &[Language::new("English", 100)];

        let languages = match languages {
            [lang] if lang == &Language::default() => None,
            _ => Some(languages),
        };

        println!("{:?}", languages)
    }
}
