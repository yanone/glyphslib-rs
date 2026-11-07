use std::cell::Cell;
use std::collections::BTreeMap;
use std::fmt;

use crate::serde::{deserialize_commify, is_default, serialize_commify};
use openstep_plist::{Dictionary, Plist};
use serde::de::{self, SeqAccess, Visitor};
use serde::ser::SerializeTuple as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The OpenType layout classes of the font (`GSClass`)
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct FeatureClass {
    /// Whether the code of the class is generated automatically.
    #[serde(default, skip_serializing_if = "is_default")]
    pub automatic: bool,
    /// The code of the class.
    ///
    /// Note that this code may not just be a whitespace-separated list of glyph names but may also contain comments and other feature code constructs. Examples: "A B C", "noon-ar noon-ar.fina noon-ar.medi noon-ar.init # noon-ar glyphs".
    #[serde(default, skip_serializing_if = "code_lives_in_package_file")]
    pub code: String,
    /// Whether the class is disabled.
    #[serde(default, skip_serializing_if = "is_default")]
    pub disabled: bool,
    /// The `features/*.fea` file that holds the code in a Glyphs 4 package.
    /// Written only into a package's `fontinfo.plist`.
    #[serde(default, skip_serializing_if = "file_not_written")]
    pub file: Option<String>,
    /// The name of the class. The leading at sign (`@`) is not included. Examples: `"Uppercase"`, `"CombiningTopAccents"`.
    pub name: String,
    /// A string serving as a description or comment about the class.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Custom parameter (`GSCustomParameter`)
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct CustomParameter {
    /// Whether the custom parameter is disabled.
    #[serde(default, skip_serializing_if = "is_default")]
    pub disabled: bool,
    /// The name of the custom parameter.
    pub name: String,
    /// The value of the custom parameter.
    pub value: Plist,
}

/// Feature prefix (`GSFeaturePrefix`)
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct FeaturePrefix {
    /// Whether the code of the feature prefix is generated automatically.
    #[serde(default, skip_serializing_if = "is_default")]
    pub automatic: bool,
    /// The code of the feature prefix. Example: `"languagesystem DFLT dflt;"`.
    #[serde(default, skip_serializing_if = "code_lives_in_package_file")]
    pub code: String,
    /// Whether the feature prefix is disabled.
    #[serde(default, skip_serializing_if = "is_default")]
    pub disabled: bool,
    /// The `features/*.fea` file that holds the code in a Glyphs 4 package.
    /// Written only into a package's `fontinfo.plist`.
    #[serde(default, skip_serializing_if = "file_not_written")]
    pub file: Option<String>,
    /// The name of the feature prefix. Example: `"Languagesystems"`.
    #[serde(alias = "tag")] // Of course some random Glyphs version did this
    pub name: String,
    /// A string serving as a description or comment about the feature prefix.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Feature (`GSFeature`)
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct Feature {
    /// Whether the code of the feature is generated automatically.
    #[serde(default, skip_serializing_if = "is_default")]
    pub automatic: bool,
    /// The code of the feature. Example: `"sub a by a.alt;"`.
    #[serde(default, skip_serializing_if = "code_lives_in_package_file")]
    pub code: String,
    /// Whether the feature is disabled.
    #[serde(default, skip_serializing_if = "is_default")]
    pub disabled: bool,
    /// The `features/*.fea` file that holds the code in a Glyphs 4 package.
    /// Written only into a package's `fontinfo.plist`.
    #[serde(default, skip_serializing_if = "file_not_written")]
    pub file: Option<String>,
    /// The labels of the feature.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<StylisticSetLabel>,
    /// A string serving as a description or comment about the feature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// The four-letter tag of the feature. Example: `"calt"`.
    #[serde(alias = "name")]
    pub tag: String,
}

/// Stylistic set label (`GSInfoValue`)
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct StylisticSetLabel {
    /// The language tag of the string value. The tag is based on the OpenType Language System Tags but omits trailing whitespace. Examples: `"dflt"`, `"DEU"`.
    pub language: String,
    /// The localized string value.
    pub value: String,
}

// File format versions that change how [`Color`] is encoded (v3 uses `0...255`
// integers, v4 uses normalized `0...1` floats and adds palette references) as well
// as how [`Node`] is serialized.
thread_local! {
    static FORMAT_VERSION: Cell<Option<u8>> = const { Cell::new(None) };
}

/// Fallback used when [`current_format_version`] is called outside a scoped
/// load/save (shouldn't happen in practice). Matches `FormatVersion::default()`.
const DEFAULT_FORMAT_VERSION: u8 = 3;

/// Sets the format version for the current thread and restores the previous value
/// on drop, so the setting is cleared even if the load/save panics.
///
/// ```ignore
/// let _guard = with_format_version(4);
/// openstep_plist::ser::to_string(glyphs3)
/// ```
#[must_use = "the guard restores the previous version when dropped"]
pub fn with_format_version(version: u8) -> FormatVersionGuard {
    let previous = FORMAT_VERSION.with(|cell| cell.replace(Some(version)));
    FormatVersionGuard(previous)
}

/// Restores the previous format version when dropped. See [`with_format_version`].
pub struct FormatVersionGuard(Option<u8>);

impl Drop for FormatVersionGuard {
    fn drop(&mut self) {
        FORMAT_VERSION.with(|cell| cell.set(self.0));
    }
}

thread_local! {
    static WRITING_PACKAGE_FEATURE_FILES: Cell<bool> = const { Cell::new(false) };
}

/// While alive, feature, class and prefix code is left out of serialized
/// entries, because a Glyphs 4 package keeps it in `features/*.fea` files.
pub(crate) struct PackageFeatureFilesGuard(bool);

impl PackageFeatureFilesGuard {
    pub(crate) fn new() -> Self {
        Self(WRITING_PACKAGE_FEATURE_FILES.with(|cell| cell.replace(true)))
    }
}

impl Drop for PackageFeatureFilesGuard {
    fn drop(&mut self) {
        WRITING_PACKAGE_FEATURE_FILES.with(|cell| cell.set(self.0));
    }
}

#[allow(clippy::ptr_arg)] // serde passes the field as &String
fn code_lives_in_package_file(_code: &String) -> bool {
    WRITING_PACKAGE_FEATURE_FILES.with(Cell::get)
}

fn file_not_written(file: &Option<String>) -> bool {
    file.is_none() || !WRITING_PACKAGE_FEATURE_FILES.with(Cell::get)
}

/// Serializes `value` the way the current file format version writes it:
/// Glyphs 4 ends every element of a block array with a comma.
pub(crate) fn plist_string<T: Serialize>(
    value: &T,
) -> Result<String, openstep_plist::error::Error> {
    if current_format_version() >= 4 {
        openstep_plist::ser::to_string_with_trailing_commas(value)
    } else {
        openstep_plist::ser::to_string(value)
    }
}

pub(crate) fn current_format_version() -> u8 {
    FORMAT_VERSION
        .with(Cell::get)
        .unwrap_or(DEFAULT_FORMAT_VERSION)
}

/// Color representation
///
/// Can be:
/// - An RGB color with an alpha channel in the sRGB IEC61966-2.1 color space (4 components)
/// - A gray color with an alpha channel in a perceptual generic gray color space with γ = 2.2 (2 components)
/// - A CMYK color with an alpha channel, device-dependent color space (5 components)
/// - An integer index of the color label
/// - An index and alpha channel into the font's Color Palettes custom parameter (V4)
///
/// Tuples are stored normalized to `0...1`. They are scaled to `0...255`
/// integers when the active format version is below 4.
#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    /// The index of the color label.
    ColorInt(u8),
    /// Color tuple (RGB, Gray, or CMYK with alpha channel)
    ColorTuple(Vec<f64>),
    /// A color from a color palette
    ColorPaletteIndex {
        /// The index of the color in the palette.
        index: u16,
        /// The alpha channel of the color.
        alpha: f64,
    },
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Color::ColorInt(index) => serializer.serialize_u8(*index),
            Color::ColorPaletteIndex { index, alpha } => {
                let mut tuple = serializer.serialize_tuple(3)?;
                tuple.serialize_element("p")?;
                tuple.serialize_element(index)?;
                if current_format_version() >= 4 {
                    tuple.serialize_element(&round_color_component(*alpha))?;
                } else {
                    // Version 3 writes the alpha as an 8-bit factor.
                    tuple.serialize_element(&((alpha * 255.0).round() as u8))?;
                }
                tuple.end()
            }
            Color::ColorTuple(components) => {
                let mut tuple = serializer.serialize_tuple(components.len())?;
                if current_format_version() >= 4 {
                    for component in components {
                        tuple.serialize_element(&round_color_component(*component))?;
                    }
                } else {
                    for component in components {
                        let scaled = (component * 255.0).round() as u8;
                        tuple.serialize_element(&scaled)?;
                    }
                }
                tuple.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ColorVisitor)
    }
}

struct ColorVisitor;

impl<'de> Visitor<'de> for ColorVisitor {
    type Value = Color;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .write_str("a color label index, a list of color components, or a palette reference")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Color::ColorInt(value as u8))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Color::ColorInt(value as u8))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        match seq.next_element::<ColorElement>()? {
            // Version 4 palette reference: ("p", index, alpha).
            Some(ColorElement::Marker(marker)) => {
                if marker != "p" {
                    return Err(de::Error::unknown_variant(&marker, &["p"]));
                }
                let index = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let alpha: f64 = seq
                    .next_element()?
                    .unwrap_or(match current_format_version() {
                        4.. => 1.0,
                        _ => 255.0,
                    });
                // Version 3 writes the alpha as an 8-bit factor.
                let alpha = if current_format_version() >= 4 {
                    alpha
                } else {
                    alpha / 255.0
                };
                Ok(Color::ColorPaletteIndex { index, alpha })
            }
            // A list of color components.
            Some(ColorElement::Number(first)) => {
                let mut components = vec![first];
                while let Some(component) = seq.next_element()? {
                    components.push(component);
                }
                if current_format_version() < 4 {
                    // Version 3 components are `0...255`; normalize to `0...1`.
                    for component in &mut components {
                        *component /= 255.0;
                    }
                }
                Ok(Color::ColorTuple(components))
            }
            None => Ok(Color::ColorTuple(Vec::new())),
        }
    }
}

/// A `0…1` color component as Glyphs 4 writes it, to three decimals.
pub(crate) fn round_color_component(component: f64) -> f64 {
    (component * 1000.0).round() / 1000.0
}

/// First element of a `Color` array: either the `"p"` palette marker or a
/// numeric color component.
#[derive(Deserialize)]
#[serde(untagged)]
enum ColorElement {
    Marker(String),
    Number(f64),
}

/// The user data and other attributes of an anchor or guide.
///
/// Glyphs 3 writes the user data under `userData` and the other attributes
/// under `attr`; Glyphs 4 writes the user data inside `attr`, as
/// `attr = { userData = {...}; }`. See [`AttributesSerde`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Attributes {
    /// The user data.
    pub dict: Dictionary,
    /// The other attributes.
    pub other: Dictionary,
}

/// How [`Attributes`] are written in the current file format version: the
/// `attr` and `userData` keys of the object.
pub(crate) struct AttributesSerde {
    pub attr: Dictionary,
    pub user_data: Dictionary,
}

impl From<&Attributes> for AttributesSerde {
    fn from(attributes: &Attributes) -> Self {
        let mut attr = attributes.other.clone();
        if current_format_version() >= 4 {
            if !attributes.dict.is_empty() {
                attr.insert(
                    "userData".into(),
                    Plist::Dictionary(attributes.dict.clone()),
                );
            }
            AttributesSerde {
                attr,
                user_data: Dictionary::new(),
            }
        } else {
            AttributesSerde {
                attr,
                user_data: attributes.dict.clone(),
            }
        }
    }
}

impl From<AttributesSerde> for Attributes {
    fn from(written: AttributesSerde) -> Self {
        let mut other = written.attr;
        let mut dict = match other.remove("userData") {
            Some(Plist::Dictionary(user_data)) => user_data,
            Some(value) => {
                other.insert("userData".into(), value);
                Dictionary::new()
            }
            None => Dictionary::new(),
        };
        dict.extend(written.user_data);
        Attributes { dict, other }
    }
}

impl Attributes {
    /// Returns the underlying dictionary.
    pub fn as_dict(&self) -> &Dictionary {
        &self.dict
    }

    /// Returns the underlying dictionary mutably.
    pub fn as_dict_mut(&mut self) -> &mut Dictionary {
        &mut self.dict
    }
}

impl From<Dictionary> for Attributes {
    fn from(dict: Dictionary) -> Self {
        Attributes {
            dict,
            other: Dictionary::new(),
        }
    }
}

impl std::ops::Deref for Attributes {
    type Target = Dictionary;

    fn deref(&self) -> &Self::Target {
        &self.dict
    }
}

impl std::ops::DerefMut for Attributes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.dict
    }
}

/// Kerning definition mapping master IDs to kerning definitions, which map glyph names or class names to kerning partners.
pub type Kerning = BTreeMap<String, BTreeMap<String, BTreeMap<String, f32>>>;

/// Context kerning definition mapping kerning contexts to master IDs and their kerning values.
pub type KerningContext = BTreeMap<String, BTreeMap<String, f32>>;

/// Guide alignment (`GSElementOrientation`)
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Copy)]
pub enum Orientation {
    /// Left alignment
    #[default]
    #[serde(rename = "left")]
    Left,
    /// Center alignment
    #[serde(rename = "center")]
    Center,
    /// Right alignment
    #[serde(rename = "right")]
    Right,
}

/// Node type for path nodes
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Copy)]
pub enum NodeType {
    /// Line node
    #[serde(rename = "l")]
    Line,
    /// Curve node
    #[serde(rename = "c")]
    Curve,
    /// QCurve node
    #[serde(rename = "q")]
    QCurve,
    /// Off-curve node
    #[serde(rename = "o")]
    OffCurve,
    /// Line smooth node
    #[serde(rename = "ls")]
    LineSmooth,
    /// Curve smooth node
    #[serde(rename = "cs")]
    CurveSmooth,
    /// QCurve smooth node
    #[serde(rename = "qs")]
    QCurveSmooth,
    /// Quartic curve
    #[serde(rename = "u")]
    Quartic,
    /// Quartic smooth curve
    #[serde(rename = "us")]
    QuarticSmooth,
    /// Hobby curve
    #[serde(rename = "h")]
    Hobby,
    /// Hobby smooth curve
    #[serde(rename = "hs")]
    HobbySmooth,
    /// Raph New Spiral curve
    #[serde(rename = "r")]
    RaphNewSpiral,
    /// Raph New Spiral smooth curve
    #[serde(rename = "rs")]
    RaphNewSpiralSmooth,
}

impl NodeType {
    /// The token used to represent this node type in the Glyphs file format,
    /// including the smooth (`s`) suffix where applicable.
    pub fn as_str(self) -> &'static str {
        match self {
            NodeType::Line => "l",
            NodeType::Curve => "c",
            NodeType::QCurve => "q",
            NodeType::OffCurve => "o",
            NodeType::LineSmooth => "ls",
            NodeType::CurveSmooth => "cs",
            NodeType::QCurveSmooth => "qs",
            NodeType::Quartic => "u",
            NodeType::QuarticSmooth => "us",
            NodeType::Hobby => "h",
            NodeType::HobbySmooth => "hs",
            NodeType::RaphNewSpiral => "r",
            NodeType::RaphNewSpiralSmooth => "rs",
        }
    }

    /// Splits a node configuration token into its node type and the trailing
    /// flag characters (tangent `t`, orientation `R`/`C`, locking `X`).
    ///
    /// Returns `None` if the token does not start with a known node type.
    pub fn split_token(token: &str) -> Option<(Self, &str)> {
        // The two-character (smooth) tokens must be tried before their
        // one-character counterparts, so that e.g. `cs` is not read as `c`
        // followed by an unknown flag.
        const TOKENS: &[(&str, NodeType)] = &[
            ("ls", NodeType::LineSmooth),
            ("cs", NodeType::CurveSmooth),
            ("qs", NodeType::QCurveSmooth),
            ("us", NodeType::QuarticSmooth),
            ("hs", NodeType::HobbySmooth),
            ("rs", NodeType::RaphNewSpiralSmooth),
            ("l", NodeType::Line),
            ("c", NodeType::Curve),
            ("q", NodeType::QCurve),
            ("o", NodeType::OffCurve),
            ("u", NodeType::Quartic),
            ("h", NodeType::Hobby),
            ("r", NodeType::RaphNewSpiral),
        ];
        TOKENS.iter().find_map(|(token_str, node_type)| {
            token.strip_prefix(token_str).map(|rest| (*node_type, rest))
        })
    }
}

/// Version information
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct Version {
    /// The major version number of the font.
    #[serde(default, rename = "versionMajor")]
    pub major: i32,
    /// The minor version number of the font.
    #[serde(default, rename = "versionMinor")]
    pub minor: i32,
}

/// Instance interpolation factors
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct InstanceFactors(
    #[serde(
        deserialize_with = "deserialize_commify",
        serialize_with = "serialize_commify",
        default
    )]
    pub Vec<f32>,
);

/// Smart component property setting (`GSPartProperty`)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SmartComponentSetting {
    /// The lower end of the value range of the property.
    #[serde(default, rename = "bottomValue")]
    pub bottom_value: i32,
    /// The upper end of the value range of the property.
    #[serde(default, rename = "topValue")]
    pub top_value: i32,
    /// The name of the property.
    pub name: String,
    /// The name of the bottom value (version 3 only; unused).
    #[serde(
        default,
        rename = "bottomName",
        skip_serializing_if = "not_written_after_version_3"
    )]
    pub bottom_name: Option<String>,
    /// The name of the top value (version 3 only; unused).
    #[serde(
        default,
        rename = "topName",
        skip_serializing_if = "not_written_after_version_3"
    )]
    pub top_name: Option<String>,
}

/// Version 4 dropped the unused names of smart component values.
fn not_written_after_version_3(name: &Option<String>) -> bool {
    name.is_none() || current_format_version() >= 4
}
