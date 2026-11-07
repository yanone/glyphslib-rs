use std::collections::BTreeMap;

use openstep_plist::{Dictionary, Plist};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{serde_as, OneOrMany};

use crate::{
    common::{
        current_format_version, Attributes, AttributesSerde, Color, CustomParameter, Feature,
        FeatureClass, FeaturePrefix, InstanceFactors, Kerning, KerningContext, NodeType,
        Orientation, SmartComponentSetting, Version,
    },
    serde::{
        bool_true, deserialize_export_type, int_to_bool, is_default, is_false, is_one_f32,
        is_scale_unit, is_true, one_f32, optional_int_to_bool, scale_unit, SerializeAsTuple,
    },
};

pub(crate) fn version_two() -> i32 {
    2
}

fn component_alignment_disabled() -> i8 {
    -1
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
struct ComponentSerde {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    alignment: Option<i8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    anchor: Option<String>,
    #[serde(default, rename = "anchorTo", skip_serializing_if = "Option::is_none")]
    anchor_to: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    angle: f32,
    #[serde(default, skip_serializing_if = "is_default")]
    attr: Dictionary,
    #[serde(
        default,
        skip_serializing_if = "is_default",
        deserialize_with = "int_to_bool"
    )]
    locked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "masterId")]
    master_id: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    orientation: Orientation,
    #[serde(rename = "piece", default, skip_serializing_if = "is_default")]
    smart_component_location: BTreeMap<String, f32>,
    #[serde(default, rename = "pos", skip_serializing_if = "is_default")]
    position: (f32, f32),
    #[serde(rename = "ref")]
    component_glyph: String,
    #[serde(default = "scale_unit", skip_serializing_if = "is_scale_unit")]
    scale: (f32, f32),
    #[serde(default, skip_serializing_if = "is_default")]
    slant: (f32, f32),
    #[serde(
        default = "bool_true",
        rename = "traverseAnchors",
        skip_serializing_if = "is_true",
        deserialize_with = "int_to_bool"
    )]
    traverse_anchors: bool,
    #[serde(default, rename = "userData", skip_serializing_if = "is_default")]
    user_data: Dictionary,
}

/// Glyphs file format version 3 document
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct Glyphs3 {
    /// The build number of Glyphs used to save the file. Example: `"3210"`.
    #[serde(
        rename = ".appVersion",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub app_version: String,
    /// The version of the file format. If unset, the file is considered to be version 1 as used by Glyphs 1 and Glyphs 2.
    #[serde(rename = ".formatVersion", default = "version_two")]
    pub format_version: i32,
    /// The strings of the Edit View tabs. Omitted when the `Write DisplayStrings` custom parameter is set to false.
    #[serde(rename = "DisplayStrings", skip_serializing_if = "is_default", default)]
    pub display_strings: Vec<String>,
    /// The designspace variation axes of the font.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub axes: Vec<Axis>,
    /// The OpenType layout classes of the font.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub classes: Vec<FeatureClass>,
    /// The custom parameters of the font.
    #[serde(
        default,
        rename = "customParameters",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub custom_parameters: Vec<CustomParameter>,
    /// The moment in time that is used as the creation date of exported font files including date, time, and timezone. Example: `"2024-07-17 03:14:15 +0000"`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub date: String,
    /// The font family name. Glyphs 4 keeps it only as the `dflt` value of
    /// the `familyNames` property; see [`Glyphs3::names_from_properties`].
    #[serde(
        rename = "familyName",
        default,
        skip_serializing_if = "name_lives_in_properties"
    )]
    pub family_name: String,
    /// The OpenType layout feature prefixes of the font.
    #[serde(
        default,
        rename = "featurePrefixes",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub feature_prefixes: Vec<FeaturePrefix>,
    /// The OpenType layout features of the font.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<Feature>,
    /// The masters of the font.
    #[serde(rename = "fontMaster", skip_serializing_if = "Vec::is_empty", default)]
    pub masters: Vec<Master>,
    /// The glyphs of the font. The order is used on export unless the `glyphOrder` custom parameter is set.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub glyphs: Vec<Glyph>,
    /// The instances of the font.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instances: Vec<Instance>,
    /// The left-to-right kerning of the font.
    #[serde(rename = "kerningLTR", default, skip_serializing_if = "is_default")]
    pub kerning: Kerning,
    /// The right-to-left kerning of the font.
    #[serde(rename = "kerningRTL", default, skip_serializing_if = "is_default")]
    pub kerning_rtl: Kerning,
    /// The vertical kerning of the font.
    #[serde(
        rename = "kerningVertical",
        default,
        skip_serializing_if = "is_default"
    )]
    pub kerning_vertical: Kerning,
    /// The context kerning of the font. Maps kerning contexts to master IDs and their kerning values.
    #[serde(rename = "kerningContext", default, skip_serializing_if = "is_default")]
    pub kerning_context: KerningContext,
    /// The metrics of the font.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metrics: Vec<Metric>,
    /// The note about the font.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
    /// The numbers of the font.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub numbers: Vec<Number>,
    /// The properties of the font.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<Property>,
    /// The settings of the font.
    #[serde(default, skip_serializing_if = "settings_not_written")]
    pub settings: Settings,
    /// The stems of the font.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stems: Vec<Stem>,
    /// The number of coordinate units on the em square.
    #[serde(rename = "unitsPerEm")]
    pub units_per_em: i32,
    /// Custom data associated with the font.
    #[serde(rename = "userData", default, skip_serializing_if = "is_default")]
    pub user_data: Dictionary,
    /// Version information.
    #[serde(flatten, default, skip_serializing_if = "is_default")]
    pub version: Version,
}

/// Number metric
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct Number {
    /// The name of the number.
    pub name: String,
}

/// Metric definition (`GSMetric`)
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct Metric {
    /// The filter of the metric limiting the scope of the metric to a subset of glyphs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    /// The name of the metric.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// The type of the metric.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "type")]
    pub metric_type: Option<MetricType>,
}

/// Metric type
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    /// Ascender metric
    #[serde(rename = "ascender")]
    Ascender,
    /// Cap height metric
    #[serde(rename = "cap height")]
    CapHeight,
    /// Slant height metric
    #[serde(rename = "slant height")]
    SlantHeight,
    /// X-height metric
    #[serde(rename = "x-height")]
    XHeight,
    /// Mid height metric
    #[serde(rename = "midHeight")]
    MidHeight,
    /// Top height metric
    #[serde(rename = "topHeight")]
    TopHeight,
    /// Body height metric
    #[serde(rename = "bodyHeight")]
    BodyHeight,
    /// Descender metric
    #[serde(rename = "descender")]
    Descender,
    /// Baseline metric
    #[serde(rename = "baseline")]
    Baseline,
    /// Italic angle metric
    #[serde(rename = "italic angle")]
    ItalicAngle,
    /// Italic slope metric
    #[serde(rename = "italic slope")]
    ItalicSlope,
}

/// Font settings
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct Settings {
    /// The color space of the font's colors (v4). Example: `"apple-rgb"`.
    #[serde(
        rename = "colorSpace",
        default,
        skip_serializing_if = "not_written_before_version_4"
    )]
    pub color_space: Option<String>,
    /// Maps dependency class names to dependency display names.
    #[serde(rename = "dependencies", default, skip_serializing_if = "is_default")]
    pub dependencies: BTreeMap<String, String>,
    /// Whether automatic alignment of components is disabled.
    #[serde(
        rename = "disablesAutomaticAlignment",
        default,
        skip_serializing_if = "is_default",
        deserialize_with = "int_to_bool"
    )]
    pub disables_automatic_alignment: bool,
    /// Whether to use production names instead of nice names.
    #[serde(
        rename = "disablesNiceNames",
        default,
        skip_serializing_if = "is_default",
        deserialize_with = "int_to_bool"
    )]
    pub disables_nice_names: bool,
    /// The main grid length.
    #[serde(rename = "gridLength", skip_serializing_if = "Option::is_none")]
    pub grid_length: Option<i32>,
    /// The grid sub-division size.
    #[serde(rename = "gridSubDivision", skip_serializing_if = "Option::is_none")]
    pub grid_sub_division: Option<i32>,
    /// The standard keyboard increment.
    #[serde(rename = "keyboardIncrement", skip_serializing_if = "Option::is_none")]
    pub keyboard_increment: Option<f32>,
    /// The keyboard increment when holding the Shift key.
    #[serde(
        rename = "keyboardIncrementBig",
        skip_serializing_if = "Option::is_none"
    )]
    pub keyboard_increment_big: Option<f32>,
    /// The keyboard increment when holding both the Shift and Command key.
    #[serde(
        rename = "keyboardIncrementHuge",
        skip_serializing_if = "Option::is_none"
    )]
    pub keyboard_increment_huge: Option<f32>,
    /// Whether to keep alternates glyphs together in Font View.
    #[serde(
        rename = "keepAlternatesTogether",
        default,
        skip_serializing_if = "is_default",
        deserialize_with = "int_to_bool"
    )]
    pub keep_alternates_together: bool,
    /// Whether to preview the effect of the Remove Overlaps filter in Edit View.
    /// Glyphs writes it also when it is off (`0`); `None` when the file
    /// does not say.
    #[serde(
        rename = "previewRemoveOverlap",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "optional_int_to_bool"
    )]
    pub preview_remove_overlap: Option<bool>,
    /// Whether snapping is enabled in Edit View.
    #[serde(
        rename = "snapToObjects",
        default,
        skip_serializing_if = "is_default",
        deserialize_with = "int_to_bool"
    )]
    pub snap_to_objects: bool,
    /// The type of the font.
    #[serde(rename = "fontType", default, skip_serializing_if = "is_default")]
    pub font_type: String,
}

/// Axis definition (`GSAxis`)
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct Axis {
    /// The default location on the axis. Written for the axes of Smart
    /// Glyphs (v4), where it is the masters' value of the property.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<f64>,
    /// Whether the axis is considered to be hidden from the font user.
    #[serde(default, skip_serializing_if = "is_default")]
    pub hidden: bool,
    /// The user-facing name of the axis.
    pub name: String,
    /// The user-facing, localized name of the axis.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub names: Vec<LocalizedValue>,
    /// The OpenType tag of the axis. Must be unique within the font. The
    /// axes of Smart Glyphs (v4) have none.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub tag: String,
    /// Custom data associated with the axis.
    #[serde(rename = "userData", default, skip_serializing_if = "is_default")]
    pub user_data: Dictionary,
}

/// Font master (`GSFontMaster`)
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct Master {
    /// Whether the master is active.
    #[serde(default = "bool_true", skip_serializing_if = "is_true")]
    pub active: bool,
    /// The designspace location of the master.
    #[serde(rename = "axesValues", default, skip_serializing_if = "Vec::is_empty")]
    pub axes_values: Vec<f32>,
    /// The custom parameters of the master.
    #[serde(
        rename = "customParameters",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub custom_parameters: Vec<CustomParameter>,
    /// The global guides of the master.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guides: Vec<Guide>,
    /// The name of the icon that represents the master. Generally omitted when equal to `Regular`, or equal to the default icon name of the master.
    #[serde(rename = "iconName", default, skip_serializing_if = "String::is_empty")]
    pub icon_name: String,
    /// The unique identifier of the master.
    pub id: String,
    /// The metric values of the master.
    #[serde(
        rename = "metricValues",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub metric_values: Vec<MetricValue>,
    /// The name of the master.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// The number values of the master.
    #[serde(
        rename = "numberValues",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub number_values: Vec<f32>,
    /// The properties of the master.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<Property>,
    /// The stem values of the master.
    #[serde(rename = "stemValues", default, skip_serializing_if = "Vec::is_empty")]
    pub stem_values: Vec<f32>,
    /// Auto-save files only: Temporary data associated with the master.
    #[serde(rename = "tempData", default, skip_serializing_if = "is_default")]
    pub temp_data: Dictionary,
    /// Custom data associated with the master.
    #[serde(rename = "userData", default, skip_serializing_if = "is_default")]
    pub user_data: Dictionary,
    /// Whether the master is visible in the preview.
    #[serde(default, skip_serializing_if = "is_default")]
    pub visible: bool,
}

/// Metric value store (`GSMetricStore`)
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct MetricValue {
    /// The overshoot of the metric value.
    #[serde(default, skip_serializing_if = "is_default")]
    pub over: f32,
    /// The offset from the baseline of the metric value.
    #[serde(default, skip_serializing_if = "is_default")]
    pub pos: f32,
}

/// Glyph definition (`GSGlyph`)
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct Glyph {
    /// The kerning group of the bottom side of the glyph.
    #[serde(rename = "kernBottom", skip_serializing_if = "Option::is_none")]
    pub kern_bottom: Option<String>,

    /// The Smart Glyph variation axes of the glyph.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub axes: Vec<Axis>,
    /// The case of the glyph. If unset, then the case is based on a glyph data lookup based on the glyph name.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub case: String,
    /// The category of the glyph. If unset, then the category is based on a glyph data lookup based on the glyph name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// The color label of the glyph.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    /// The writing direction of the glyph. If unset, then the writing direction is based on a glyph data lookup based on the glyph name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    /// Whether the glyph is exported.
    #[serde(default = "bool_true", skip_serializing_if = "is_true")]
    pub export: bool,
    /// The name of the glyph.
    #[serde(rename = "glyphname")]
    pub name: String,
    /// The glyph group name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// The index of the glyph within its group.
    #[serde(rename = "groupIdx", default, skip_serializing_if = "is_default")]
    pub group_idx: i32,
    /// The kerning group of the left side of the glyph.
    #[serde(rename = "kernLeft", skip_serializing_if = "Option::is_none")]
    pub kern_left: Option<String>,
    /// The kerning group of the right side of the glyph.
    #[serde(rename = "kernRight", skip_serializing_if = "Option::is_none")]
    pub kern_right: Option<String>,
    /// The kerning group of the top side of the glyph.
    #[serde(rename = "kernTop", skip_serializing_if = "Option::is_none")]
    pub kern_top: Option<String>,
    /// The date and time of the last change of the glyph. Example: `"2023-02-25 14:46:49 +0000"`.
    #[serde(rename = "lastChange", skip_serializing_if = "Option::is_none")]
    pub last_change: Option<String>,
    /// The layers of the glyph.
    pub layers: Vec<Layer>,
    /// Whether the glyph is locked.
    #[serde(default, skip_serializing_if = "is_default")]
    pub locked: bool,
    /// The bottom metrics key of the glyph.
    #[serde(default, skip_serializing_if = "is_default", rename = "metricBottom")]
    pub metric_bottom: Option<String>,
    /// The left metrics key of the glyph.
    #[serde(default, skip_serializing_if = "is_default", rename = "metricLeft")]
    pub metric_left: Option<String>,
    /// The right metrics key of the glyph.
    #[serde(default, skip_serializing_if = "is_default", rename = "metricRight")]
    pub metric_right: Option<String>,
    /// The top metrics key of the glyph.
    #[serde(default, skip_serializing_if = "is_default", rename = "metricTop")]
    pub metric_top: Option<String>,
    /// The vertical width metrics key of the glyph.
    #[serde(
        default,
        skip_serializing_if = "is_default",
        rename = "metricVertWidth"
    )]
    pub metric_vert_width: Option<String>,
    /// The width metrics key of the glyph.
    #[serde(default, skip_serializing_if = "is_default", rename = "metricWidth")]
    pub metric_width: Option<String>,
    /// A string serving as a description or comment about the glyph.
    #[serde(default, skip_serializing_if = "is_default")]
    pub note: String,
    /// A list of the Smart Glyph properties and their top/bottom values.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        rename = "partsSettings"
    )]
    pub smart_component_settings: Vec<SmartComponentSetting>,
    /// The production name of the glyph. If unset, then the production name is based on a glyph data lookup based on the glyph name or the Unicode code point.
    #[serde(default, skip_serializing_if = "is_default")]
    pub production: Option<String>,
    /// The script of the glyph. If unset, then the script is based on a glyph data lookup based on the glyph name.
    #[serde(default, skip_serializing_if = "is_default")]
    pub script: Option<String>,
    /// The sort name of the glyph. If unset, then the sort name is based on a glyph data lookup based on the glyph name.
    #[serde(default, skip_serializing_if = "is_default", rename = "sortName")]
    pub sort_name: Option<String>,
    /// The sort name of the glyph used in the *Keep Alternates Next to Base Glyph* display mode. If unset, then the sort name is based on a glyph data lookup based on the glyph name.
    #[serde(default, skip_serializing_if = "is_default", rename = "sortNameKeep")]
    pub sort_name_keep: Option<String>,
    /// The subcategory of the glyph. If unset, then the subcategory is based on a glyph data lookup based on the glyph name.
    #[serde(default, skip_serializing_if = "is_default", rename = "subCategory")]
    pub subcategory: Option<String>,
    /// The tags of the glyph, sorted lexicographically.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// The Unicode code points of the glyph.
    #[serde_as(
        deserialize_as = "OneOrMany<_>",
        serialize_as = "SerializeAsTuple<u32>"
    )]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unicode: Vec<u32>,
    /// Custom data associated with the glyph.
    #[serde(rename = "userData", default, skip_serializing_if = "is_default")]
    pub user_data: Dictionary,
}

/// Layer definition (`GSLayer`)
///
/// We manually serialize this because background layers serialize differently,
/// and I don't want to have a separate BackgroundLayer struct.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Layer {
    /// Whether the layer is active.
    #[serde(default = "bool_true")]
    pub active: bool,
    /// The anchors of the layer.
    #[serde(default)]
    pub anchors: Vec<Anchor>,
    /// The annotations of the layer.
    #[serde(default)]
    pub annotations: Vec<Dictionary>,
    /// The unique identifier of the associated master. Omitted when equal to the layer ID.
    #[serde(rename = "associatedMasterId", default)]
    pub associated_master_id: Option<String>,
    /// The attributes of the layer.
    #[serde(default)]
    pub attr: Dictionary,
    /// The background layer.
    #[serde(default)]
    pub background: Option<Box<Layer>>,
    /// The background image of the layer.
    #[serde(rename = "backgroundImage", default)]
    pub background_image: Option<BackgroundImage>,
    /// The color label of the layer.
    #[serde(default)]
    pub color: Option<Color>,
    /// The guides of the layer.
    #[serde(default)]
    pub guides: Vec<Guide>,
    /// The hints of the layer.
    #[serde(default)]
    pub hints: Vec<Dictionary>, // This thing's an absolute minefield
    /// The unique ID of the layer. Matches the master ID when the layer is a master layer.
    #[serde(rename = "layerId", default)]
    // Not required for background layers
    pub layer_id: String,
    /// The bottom metrics key of the layer.
    #[serde(rename = "metricBottom", default)]
    pub metric_bottom: Option<String>,
    /// The left metrics key of the layer.
    #[serde(rename = "metricLeft", default)]
    pub metric_left: Option<String>,
    /// The right metrics key of the layer.
    #[serde(rename = "metricRight", default)]
    pub metric_right: Option<String>,
    /// The top metrics key of the layer.
    #[serde(rename = "metricTop", default)]
    pub metric_top: Option<String>,
    /// The vertical origin metrics key of the layer.
    #[serde(rename = "metricVertOrigin", default)]
    pub metric_vert_origin: Option<String>,
    /// The vertical width metrics key of the layer.
    #[serde(
        rename = "metricVertWidth",
        default,
        skip_serializing_if = "is_default"
    )]
    pub metric_vert_width: Option<String>,
    /// The width metrics key of the layer.
    #[serde(rename = "metricWidth", default)]
    pub metric_width: Option<String>,
    /// The name of the layer. Master layers and other special layers display a name in the Glyphs UI that is derived from the layers role. These derived names are not written to the file.
    #[serde(default)]
    pub name: Option<String>,
    /// The Smart Glyph setting of the layer. The keys are the property names. The values are either `1` if the layer corresponds to the bottom value of the property or `2` if the layer corresponds to the top value of the property.
    #[serde(rename = "partSelection", default)]
    pub part_selection: BTreeMap<String, u8>,
    /// The shapes of the layer.
    #[serde(default)]
    pub shapes: Vec<Shape>,
    /// Custom data associated with the layer.
    #[serde(rename = "userData", default)]
    pub user_data: Dictionary,
    /// The vertical origin of the layer.
    #[serde(rename = "vertOrigin", default)]
    pub vert_origin: Option<f32>,
    /// The vertical width of the layer.
    #[serde(rename = "vertWidth", default)]
    pub vert_width: Option<f32>,
    /// Whether the layer is shown in the layer panel (its eye symbol). Off
    /// unless the file says `visible = 1`.
    #[serde(default)]
    pub visible: bool,
    /// The width of the layer.
    #[serde(default)]
    pub width: f32,
}

/// Anchor definition (`GSAnchor`)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(from = "AnchorSerde", into = "AnchorSerde")]
pub struct Anchor {
    /// The user data and other attributes of the anchor.
    pub attr: Attributes,
    /// Whether the anchor is locked.
    #[serde(default, skip_serializing_if = "is_default")]
    pub locked: bool,
    /// The name of the anchor.
    pub name: String,
    /// The orientation of the anchor.
    #[serde(default, skip_serializing_if = "is_default")]
    pub orientation: Orientation,
    #[serde(default, skip_serializing_if = "is_default")]
    /// The position of the anchor.
    pub pos: (f32, f32),
}

/// An [`Anchor`] as written: `attr` and `userData` where the file format
/// version puts them.
#[derive(Serialize, Deserialize)]
struct AnchorSerde {
    #[serde(default, skip_serializing_if = "is_default")]
    attr: Dictionary,
    #[serde(default, skip_serializing_if = "is_default")]
    locked: bool,
    name: String,
    #[serde(default, skip_serializing_if = "is_default")]
    orientation: Orientation,
    #[serde(default, skip_serializing_if = "is_default")]
    pos: (f32, f32),
    #[serde(default, rename = "userData", skip_serializing_if = "is_default")]
    user_data: Dictionary,
}

impl From<AnchorSerde> for Anchor {
    fn from(a: AnchorSerde) -> Self {
        Anchor {
            attr: AttributesSerde {
                attr: a.attr,
                user_data: a.user_data,
            }
            .into(),
            locked: a.locked,
            name: a.name,
            orientation: a.orientation,
            pos: a.pos,
        }
    }
}

impl From<Anchor> for AnchorSerde {
    fn from(a: Anchor) -> Self {
        let written = AttributesSerde::from(&a.attr);
        AnchorSerde {
            attr: written.attr,
            locked: a.locked,
            name: a.name,
            orientation: a.orientation,
            pos: a.pos,
            user_data: written.user_data,
        }
    }
}

/// Background image (`GSImage`)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BackgroundImage {
    /// The alpha value of the image.
    #[serde(default = "one_f32", skip_serializing_if = "is_one_f32")]
    pub alpha: f32,
    /// The rotation angle of the image in degrees clockwise.
    #[serde(default, skip_serializing_if = "is_default")]
    pub angle: f32,
    /// The attributes of the image
    #[serde(default, skip_serializing_if = "is_default")]
    pub attr: Dictionary,
    /// The cropped frame of the image, specified as the crop origin X/Y and size width/height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crop: Option<(f32, f32, f32, f32)>,
    /// The file path of the image file relative to the document file.
    #[serde(rename = "imagePath")]
    pub image_path: String,
    /// The URL bookmark data of the image file path.
    #[serde(default, rename = "imageURL", skip_serializing_if = "is_default")]
    pub image_url: String,
    /// Whether the image is locked.
    #[serde(default, skip_serializing_if = "is_default")]
    pub locked: bool,
    /// The position of the image.
    #[serde(default, skip_serializing_if = "is_default")]
    pub pos: (f32, f32),
    /// The scale factor of the image.
    #[serde(default = "scale_unit", skip_serializing_if = "is_scale_unit")]
    pub scale: (f32, f32),
    /// The slant of the image.
    #[serde(default, skip_serializing_if = "is_default")]
    pub slant: (f32, f32),
}

/// Guide type (`GSGuideType`)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default, Copy)]
pub enum GuideType {
    /// Line guide
    #[default]
    Line,
    /// Circle guide
    Circle,
    /// Rectangle guide
    Rect,
}

/// Guide definition (`GSGuide`)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(from = "GuideSerde", into = "GuideSerde")]
pub struct Guide {
    /// The angle at which the guide is drawn in degrees clockwise.
    #[serde(default, skip_serializing_if = "is_default")]
    pub angle: f32,
    /// The user data and other attributes of the guide.
    pub attr: Attributes,
    /// The filter of the guide. The syntax is the description of [NSPredicate]
    #[serde(default, skip_serializing_if = "is_default")]
    pub filter: String,
    /// The grid of the guide.
    #[serde(default, skip_serializing_if = "is_default")]
    pub grid: f32,
    /// The length of a line-type guide.
    #[serde(default, skip_serializing_if = "is_default")]
    pub length: f32,
    /// Whether the angle of the guide is locked.
    #[serde(default, skip_serializing_if = "is_default", rename = "lockAngle")]
    pub lock_angle: bool,
    /// Whether the guide is locked.
    #[serde(default, skip_serializing_if = "is_default")]
    pub locked: bool,
    /// The name of the guide.
    #[serde(default, skip_serializing_if = "is_default")]
    pub name: String,
    /// The orientation of the guide.
    #[serde(default, skip_serializing_if = "is_default")]
    pub orientation: Orientation,
    /// The position of the guide.
    #[serde(default, skip_serializing_if = "is_default")]
    pub pos: (f32, f32),
    /// Whether the measurement of the guide is shown
    #[serde(
        default,
        skip_serializing_if = "is_default",
        rename = "showMeasurement"
    )]
    pub show_measurement: bool,
    /// The size of the guide.
    #[serde(default = "scale_unit", skip_serializing_if = "is_scale_unit")]
    pub size: (f32, f32),
    /// Whether the guide has a slope.
    #[serde(default, skip_serializing_if = "is_default")]
    pub slope: bool,
    /// The type of the guide
    #[serde(default, skip_serializing_if = "is_default", rename = "type")]
    pub guide_type: GuideType,
}

/// A [`Guide`] as written: `attr` and `userData` where the file format
/// version puts them.
#[derive(Serialize, Deserialize)]
struct GuideSerde {
    #[serde(default, skip_serializing_if = "is_default")]
    angle: f32,
    #[serde(default, skip_serializing_if = "is_default")]
    attr: Dictionary,
    #[serde(default, skip_serializing_if = "is_default")]
    filter: String,
    #[serde(default, skip_serializing_if = "is_default")]
    grid: f32,
    #[serde(default, skip_serializing_if = "is_default")]
    length: f32,
    #[serde(default, skip_serializing_if = "is_default", rename = "lockAngle")]
    lock_angle: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    locked: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    name: String,
    #[serde(default, skip_serializing_if = "is_default")]
    orientation: Orientation,
    #[serde(default, skip_serializing_if = "is_default")]
    pos: (f32, f32),
    #[serde(
        default,
        skip_serializing_if = "is_default",
        rename = "showMeasurement"
    )]
    show_measurement: bool,
    #[serde(default = "scale_unit", skip_serializing_if = "is_scale_unit")]
    size: (f32, f32),
    #[serde(default, skip_serializing_if = "is_default")]
    slope: bool,
    #[serde(default, skip_serializing_if = "is_default", rename = "type")]
    guide_type: GuideType,
    #[serde(default, rename = "userData", skip_serializing_if = "is_default")]
    user_data: Dictionary,
}

impl From<GuideSerde> for Guide {
    fn from(g: GuideSerde) -> Self {
        Guide {
            attr: AttributesSerde {
                attr: g.attr,
                user_data: g.user_data,
            }
            .into(),
            angle: g.angle,
            filter: g.filter,
            grid: g.grid,
            length: g.length,
            lock_angle: g.lock_angle,
            locked: g.locked,
            name: g.name,
            orientation: g.orientation,
            pos: g.pos,
            show_measurement: g.show_measurement,
            size: g.size,
            slope: g.slope,
            guide_type: g.guide_type,
        }
    }
}

impl From<Guide> for GuideSerde {
    fn from(g: Guide) -> Self {
        let written = AttributesSerde::from(&g.attr);
        GuideSerde {
            attr: written.attr,
            user_data: written.user_data,
            angle: g.angle,
            filter: g.filter,
            grid: g.grid,
            length: g.length,
            lock_angle: g.lock_angle,
            locked: g.locked,
            name: g.name,
            orientation: g.orientation,
            pos: g.pos,
            show_measurement: g.show_measurement,
            size: g.size,
            slope: g.slope,
            guide_type: g.guide_type,
        }
    }
}

/// Shape - either a path or a component
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Shape {
    /// Component reference
    Component(Component),
    /// Outline path
    Path(Path),
    /// An image
    Image(BackgroundImage),
    /// A shape group (Glyphs 4 only)
    ShapeGroup(ShapeGroup),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// A group of shapes within a glyph (Glyphs 4 only).
pub struct ShapeGroup {
    /// The attributes of the shape group.
    #[serde(default, skip_serializing_if = "is_default")]
    pub attr: Dictionary,
    /// The group ID
    #[serde(rename = "groupId")]
    pub group_id: String,
}

/// Path definition (`GSPath`)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Path {
    /// The attributes of the path.
    #[serde(default, skip_serializing_if = "is_default")]
    pub attr: Dictionary,
    /// Whether the path is closed.
    #[serde(default, deserialize_with = "int_to_bool")]
    pub closed: bool,
    /// The nodes of the path.
    pub nodes: Vec<Node>,
}

/// Path node (`GSNode`)
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// The x-coordinate of the node.
    pub x: f32,
    /// The y-coordinate of the node.
    pub y: f32,
    /// The type of the node.
    pub node_type: NodeType,
    /// Is this node a tangent connection? (v4)
    pub tangent: bool,
    /// Is this node locked? (v4)
    pub locked: bool,
    /// The orientation of the node (v4)
    pub orientation: Orientation,
    /// Custom data associated with the node.
    pub user_data: Option<Dictionary>,
    /// v4 Higher Order Interpolation attributes.
    pub hoi: Option<Dictionary>,
}

/// Component reference (`GSComponent`)
#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    /// Controls the automatic alignment of the component. `-1`: disabled (no alignment), `0`: default (alignment is based on context), `1`: force alignment (align regardless of context), `3`: horizontal alignment (align horizontally, but allow for manual vertical placement).
    pub alignment: i8,
    /// The name of the attachment anchor. Set to specify a specific anchor when there are multiple candidates.
    pub anchor: Option<String>,
    /// Undocumented anchor targeting.
    pub anchor_to: Option<String>,
    /// The rotation angle of the component in degrees clockwise.
    pub angle: f32,
    /// The attributes of the component.
    pub attr: Dictionary,
    /// Whether the component is locked.
    pub locked: bool,
    /// The ID of the master from which the component is derived.
    pub master_id: Option<String>,
    /// The orientation of the component.
    pub orientation: Orientation,
    /// The Smart Component settings of the component, mapping property names to values.
    pub smart_component_location: BTreeMap<String, f32>,
    /// The position (translation transform) of the component.
    pub position: (f32, f32),
    /// The name of the referenced glyph.
    pub component_glyph: String,
    /// The scale transform of the component.
    pub scale: (f32, f32),
    /// The slant transform of the component.
    pub slant: (f32, f32),
    /// Custom data associated with the component.
    pub user_data: Dictionary,

    /// Is there an explicit alignment?
    pub alignment_explicit: bool,
    /// Whether the component should traverse anchors.
    pub traverse_anchors: bool,
}

impl Default for Component {
    fn default() -> Self {
        Component {
            alignment: 0,
            anchor: None,
            anchor_to: None,
            angle: 0.0,
            attr: Dictionary::default(),
            locked: false,
            master_id: None,
            orientation: Orientation::default(),
            smart_component_location: BTreeMap::new(),
            position: (0.0, 0.0),
            component_glyph: String::new(),
            scale: (0.0, 0.0),
            slant: (0.0, 0.0),
            user_data: Dictionary::default(),
            alignment_explicit: false,
            traverse_anchors: true,
        }
    }
}

impl Serialize for Component {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let alignment =
            if self.alignment != component_alignment_disabled() || self.alignment_explicit {
                Some(self.alignment)
            } else {
                None
            };

        // Version 4 writes the component's user data inside `attr`, as
        // `attr = { userData = {...}; }`; earlier versions in a `userData` key
        // of its own.
        let (attr, user_data) = if current_format_version() >= 4 {
            let mut attr = self.attr.clone();
            if !self.user_data.is_empty() {
                attr.insert("userData".into(), Plist::Dictionary(self.user_data.clone()));
            }
            (attr, Dictionary::default())
        } else {
            (self.attr.clone(), self.user_data.clone())
        };

        ComponentSerde {
            alignment,
            anchor: self.anchor.clone(),
            anchor_to: self.anchor_to.clone(),
            angle: self.angle,
            attr,
            locked: self.locked,
            master_id: self.master_id.clone(),
            orientation: self.orientation,
            smart_component_location: self.smart_component_location.clone(),
            position: self.position,
            component_glyph: self.component_glyph.clone(),
            scale: self.scale,
            slant: self.slant,
            traverse_anchors: self.traverse_anchors,
            user_data,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Component {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut component = ComponentSerde::deserialize(deserializer)?;
        // Version 4 user data, inside `attr`.
        if let Some(Plist::Dictionary(user_data)) = component.attr.remove("userData") {
            component.user_data.extend(user_data);
        }
        Ok(Component {
            alignment: component
                .alignment
                .unwrap_or_else(component_alignment_disabled),
            anchor: component.anchor,
            anchor_to: component.anchor_to,
            angle: component.angle,
            attr: component.attr,
            locked: component.locked,
            master_id: component.master_id,
            orientation: component.orientation,
            smart_component_location: component.smart_component_location,
            position: component.position,
            component_glyph: component.component_glyph,
            scale: component.scale,
            slant: component.slant,
            user_data: component.user_data,
            alignment_explicit: component.alignment.is_some(),
            traverse_anchors: component.traverse_anchors,
        })
    }
}

/// Instance definition (`GSInstance`)
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct Instance {
    /// The internal axis locations of the instance. These values are also used for the external axis locations, if no external axis locations are specified separately.
    #[serde(default, rename = "axesValues", skip_serializing_if = "is_default")]
    pub axes_values: Vec<f32>,
    /// The custom parameters of the instance.
    #[serde(
        default,
        rename = "customParameters",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub custom_parameters: Vec<CustomParameter>,
    /// Whether the instance is exported.
    #[serde(default = "bool_true", skip_serializing_if = "is_true")]
    pub exports: bool,
    /// The unique ID of the instance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The interpolation factors where the keys are the master IDs.
    #[serde(
        default,
        rename = "instanceInterpolations",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub instance_interpolations: BTreeMap<String, InstanceFactors>,
    /// Whether the instance is bold.
    #[serde(default, rename = "isBold", skip_serializing_if = "is_false")]
    pub is_bold: bool,
    /// Whether the instance is italic.
    #[serde(default, rename = "isItalic", skip_serializing_if = "is_false")]
    pub is_italic: bool,
    /// The name of the style-linked instance.
    #[serde(default, rename = "linkStyle", skip_serializing_if = "Option::is_none")]
    pub link_style: Option<String>,
    /// Whether the `instanceInterpolations` values are used. Otherwise, the values are calculated from the axis values.
    #[serde(
        default,
        rename = "manualInterpolation",
        skip_serializing_if = "is_false"
    )]
    pub manual_interpolation: bool,
    /// The style name of the instance. Examples: `"Regular"`, `"Bold"`, `"Italic"`, `"Bold Italic"`.
    /// Glyphs 4 keeps it only as the `dflt` value of the `styleNames` property.
    #[serde(default, skip_serializing_if = "name_lives_in_properties")]
    pub name: String,
    /// The properties of the instance.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<Property>,
    /// The type of the instance.
    #[serde(
        rename = "type",
        default,
        skip_serializing_if = "is_default",
        deserialize_with = "deserialize_export_type"
    )]
    pub export_type: ExportType,
    /// Custom data associated with the instance.
    #[serde(default, rename = "userData", skip_serializing_if = "is_default")]
    pub user_data: Dictionary,
    /// The weight class of the instance.
    #[serde(
        default,
        rename = "weightClass",
        skip_serializing_if = "Option::is_none"
    )]
    pub weight_class: Option<i32>,
    /// The width class of the instance.
    #[serde(
        default,
        rename = "widthClass",
        skip_serializing_if = "Option::is_none"
    )]
    pub width_class: Option<i32>,
}

/// Instance export type
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default, Copy)]
pub enum ExportType {
    /// Static instance
    #[serde(rename = "static")]
    Static,
    /// Variable instance
    #[serde(rename = "variable")]
    Variable,
    /// Icon instance
    #[serde(rename = "icon")]
    Icon,
    /// Single
    #[default]
    #[serde(rename = "single")]
    Single,
    /// Particles
    #[serde(rename = "particles")]
    Particles,
}

/// Font property (`GSInfoProperty`)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Property {
    /// Singular (non-localized) property
    SingularProperty {
        /// Property key
        key: SingularPropertyKey,
        /// Property value
        value: String,
    },
    /// Localized property with multiple language variants
    LocalizedProperty {
        /// Property key
        key: LocalizedPropertyKey,
        /// Localized values
        values: Vec<LocalizedValue>,
    },
    /// Unrecognized property format
    Junk(Plist),
}

impl Property {
    /// The `key` the property is written with, such as `familyNames`.
    pub fn key_name(&self) -> Option<String> {
        property_key(self)
    }

    /// Create a singular property
    pub fn singular(key: SingularPropertyKey, value: String) -> Self {
        Property::SingularProperty { key, value }
    }
    /// Create a localized property with a default language value
    pub fn localized_with_default(key: LocalizedPropertyKey, value: String) -> Self {
        Property::LocalizedProperty {
            key,
            values: vec![LocalizedValue {
                language: "dflt".to_string(),
                value,
            }],
        }
    }
}

/// Localized property key
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum LocalizedPropertyKey {
    /// Family names
    #[serde(rename = "familyNames")]
    FamilyNames,
    /// Copyright notices
    #[serde(rename = "copyrights")]
    Copyrights,
    /// Designer names
    #[serde(rename = "designers")]
    Designers,
    /// Manufacturer names
    #[serde(rename = "manufacturers")]
    Manufacturers,
    /// License texts
    #[serde(rename = "licenses")]
    Licenses,
    /// Trademark notices
    #[serde(rename = "trademarks")]
    Trademarks,
    /// Font descriptions
    #[serde(rename = "descriptions")]
    Descriptions,
    /// Sample texts
    #[serde(rename = "sampleTexts")]
    SampleTexts,
    /// Compatible full names
    #[serde(rename = "compatibleFullNames")]
    CompatibleFullNames,
    /// Style names
    #[serde(rename = "styleNames")]
    StyleNames,
    /// PostScript full names
    #[serde(rename = "postscriptFullNames")]
    PostscriptFullNames,
}

/// Singular (non-localized) property key
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum SingularPropertyKey {
    /// Designer name
    #[serde(rename = "designer")]
    Designer,
    /// Manufacturer name
    #[serde(rename = "manufacturer")]
    Manufacturer,
    /// Designer URL
    #[serde(rename = "designerURL")]
    DesignerUrl,
    /// Manufacturer URL
    #[serde(rename = "manufacturerURL")]
    ManufacturerUrl,
    /// License URL
    #[serde(rename = "licenseURL")]
    LicenseUrl,
    /// PostScript full name
    #[serde(rename = "postscriptFullName")]
    PostscriptFullName,
    /// PostScript font name
    #[serde(rename = "postscriptFontName")]
    PostscriptFontName,
    /// WWS family name
    #[serde(rename = "WWSFamilyName")]
    WwsFamilyName,
    /// Version string
    #[serde(rename = "versionString")]
    VersionString,
    /// Vendor ID
    #[serde(rename = "vendorID")]
    VendorID,
    /// Unique ID
    #[serde(rename = "uniqueID")]
    UniqueID,
}

/// Localized string value (`GSInfoValue`)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LocalizedValue {
    /// The language tag of the string value. The tag is based on the OpenType Language System Tags but omits trailing whitespace. Examples: `"dflt"`, `"DEU"`.
    pub language: String,
    /// The localized string value.
    pub value: String,
}

/// Stem definition
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Stem {
    /// Whether the stem is a horizontal stem.
    #[serde(default, skip_serializing_if = "is_default")]
    pub horizontal: bool,
    /// The name of the stem.
    pub name: String,
}

/// Whether there is nothing to write of the settings in the current version.
fn settings_not_written(settings: &Settings) -> bool {
    let written = Settings {
        color_space: settings
            .color_space
            .clone()
            .filter(|_| current_format_version() >= 4),
        ..settings.clone()
    };
    written == Settings::default()
}

/// Glyphs 4 leaves the color space out when it saves a version 3 file.
fn not_written_before_version_4(value: &Option<String>) -> bool {
    value.is_none() || current_format_version() < 4
}

/// The stroke position of a path in the meaning of file format `to`, from
/// its value in file format `from`. Version 3 runs from outside (0) through
/// centered (0.5) to inside (1), version 4 from inside (-1) through
/// centered (0) to outside (1).
fn convert_stroke_position(value: &Plist, from: i32, to: i32) -> Option<Plist> {
    let number = match value {
        Plist::Integer(i) => *i as f64,
        Plist::Float(f) => *f,
        Plist::String(s) => s.parse().ok()?,
        _ => return None,
    };
    let converted = match (from >= 4, to >= 4) {
        (false, true) => 1.0 - 2.0 * number,
        (true, false) => (1.0 - number) / 2.0,
        _ => return None,
    };
    Some(if converted.fract() == 0.0 {
        Plist::Integer(converted as i64)
    } else {
        Plist::Float(converted)
    })
}

/// Line caps 2 and 3 swap places between the versions: Glyphs 4 writes a
/// version 3 cap 2 as 3 and a cap 3 as 2, and back (verified with Glyphs
/// 4.1.1; the specs give both numbers the same meaning in both versions).
fn convert_line_caps_in(attr: &mut Dictionary) {
    for key in ["lineCapStart", "lineCapEnd"] {
        let swapped = match attr.get(key).and_then(plist_f64) {
            Some(v) if v == 2.0 => 3,
            Some(v) if v == 3.0 => 2,
            _ => continue,
        };
        attr.insert(key.into(), Plist::Integer(swapped));
    }
}

fn convert_stroke_position_in(attr: &mut Dictionary, from: i32, to: i32) {
    if let Some(value) = attr.get("strokePos") {
        if let Some(converted) = convert_stroke_position(value, from, to) {
            attr.insert("strokePos".into(), converted);
        }
    }
    if (from >= 4) != (to >= 4) {
        convert_line_caps_in(attr);
    }
}

fn convert_layer_stroke_positions(layer: &mut Layer, from: i32, to: i32) {
    convert_stroke_position_in(&mut layer.attr, from, to);
    for shape in &mut layer.shapes {
        match shape {
            Shape::Path(path) => convert_stroke_position_in(&mut path.attr, from, to),
            Shape::ShapeGroup(group) => convert_stroke_position_in(&mut group.attr, from, to),
            _ => {}
        }
    }
    if let Some(background) = layer.background.as_deref_mut() {
        convert_layer_stroke_positions(background, from, to);
    }
}

/// A color as written in file format `from`, as `to` writes it: version 3
/// uses `0…255` integers (also for a palette reference's alpha), version 4
/// `0…1` numbers with `decimals` decimals: Glyphs 4 writes three for the
/// colors of shapes and five in the `Color Palettes` custom parameter. A
/// color label index stays.
fn convert_color(color: &Plist, from: i32, to: i32) -> Option<Plist> {
    convert_color_with(color, from, to, 3)
}

fn convert_color_with(color: &Plist, from: i32, to: i32, decimals: i32) -> Option<Plist> {
    let up = from < 4 && to >= 4;
    let number = |p: &Plist| match p {
        Plist::Integer(i) => Some(*i as f64),
        Plist::Float(f) => Some(*f),
        Plist::String(s) => s.parse().ok(),
        _ => None,
    };
    let convert = |value: f64| {
        if up {
            let scale = 10f64.powi(decimals);
            let v = (value / 255.0 * scale).round() / scale;
            if v.fract() == 0.0 {
                Plist::Integer(v as i64)
            } else {
                Plist::Float(v)
            }
        } else {
            Plist::Integer((value * 255.0).round() as i64)
        }
    };
    let Plist::Array(items) = color else {
        return None;
    };
    if items.first().and_then(Plist::as_str) == Some("p") {
        let mut out = items.clone();
        if let Some(alpha) = items.get(2).and_then(number) {
            out[2] = convert(alpha);
        }
        return Some(Plist::Array(out));
    }
    let components: Option<Vec<f64>> = items.iter().map(number).collect();
    Some(Plist::Array(components?.into_iter().map(convert).collect()))
}

/// A gradient as `to` writes it: its color stops' colors, and a radial
/// gradient's type, `circle` in version 3 and `radial` in version 4.
fn convert_gradient(gradient: &mut Plist, from: i32, to: i32) {
    let Plist::Dictionary(gradient) = gradient else {
        return;
    };
    if let Some(Plist::Array(stops)) = gradient.get_mut("colors") {
        for stop in stops {
            if let Plist::Array(stop) = stop {
                if let Some(color) = stop.first().and_then(|c| convert_color(c, from, to)) {
                    stop[0] = color;
                }
            }
        }
    }
    let radial = match gradient.get("type").and_then(Plist::as_str) {
        Some("circle") if to >= 4 => Some("radial"),
        Some("radial") if to < 4 => Some("circle"),
        _ => None,
    };
    if let Some(radial) = radial {
        gradient.insert("type".into(), Plist::String(radial.into()));
    }
}

/// The colors and gradients of a shape's attributes, as `to` writes them.
fn convert_shape_colors(attr: &mut Dictionary, from: i32, to: i32) {
    for key in ["fillColor", "strokeColor"] {
        if let Some(color) = attr.get(key).and_then(|c| convert_color(c, from, to)) {
            attr.insert(key.into(), color);
        }
    }
    for key in ["gradient", "strokeGradient"] {
        if let Some(gradient) = attr.get_mut(key) {
            convert_gradient(gradient, from, to);
        }
    }
}

/// Hint targets Glyphs 4 leaves to the hint's type: a ghost hint's (`down`
/// for a top ghost, `up` for a bottom one), which Glyphs 3 writes, and a cap
/// component's.
fn convert_layer_hints(layer: &mut Layer, to: i32) {
    for hint in &mut layer.hints {
        // A cap component's target is the node after its origin; Glyphs 4
        // does not write it, nor does it write it back for Glyphs 3.
        if to >= 4 && hint.get("type").and_then(Plist::as_str) == Some("Cap") {
            hint.remove("target");
            continue;
        }
        let target = match hint.get("type").and_then(Plist::as_str) {
            Some("TopGhost") => "down",
            Some("BottomGhost") => "up",
            _ => continue,
        };
        if to >= 4 {
            if matches!(
                hint.get("target").and_then(Plist::as_str),
                Some("up" | "down")
            ) {
                hint.remove("target");
            }
        } else if !hint.contains_key("target") {
            hint.insert("target".into(), Plist::String(target.into()));
        }
    }
    if let Some(background) = layer.background.as_deref_mut() {
        convert_layer_hints(background, to);
    }
}

fn convert_layer_colors(layer: &mut Layer, from: i32, to: i32) {
    for shape in &mut layer.shapes {
        match shape {
            Shape::Path(path) => convert_shape_colors(&mut path.attr, from, to),
            Shape::ShapeGroup(group) => convert_shape_colors(&mut group.attr, from, to),
            Shape::Image(image) => convert_shape_colors(&mut image.attr, from, to),
            Shape::Component(_) => {}
        }
    }
    if let Some(background) = layer.background.as_deref_mut() {
        convert_layer_colors(background, from, to);
    }
}

/// The `Color Palettes` custom parameter as `to` writes it: a list of
/// palettes, each a list of colors, in version 3; `{palettes = ({colors =
/// (...);});}` in version 4, whose palette names and background styles
/// version 3 cannot hold.
fn convert_color_palettes(value: &Plist, from: i32, to: i32) -> Option<Plist> {
    let colors = |list: &[Plist]| -> Vec<Plist> {
        list.iter()
            .map(|c| convert_color_with(c, from, to, 5).unwrap_or_else(|| c.clone()))
            .collect()
    };
    if from < 4 && to >= 4 {
        let palettes = value.as_array()?;
        let palettes: Vec<Plist> = palettes
            .iter()
            .map(|palette| {
                let mut out = Dictionary::new();
                out.insert(
                    "colors".into(),
                    Plist::Array(colors(palette.as_array().unwrap_or(&[]))),
                );
                Plist::Dictionary(out)
            })
            .collect();
        let mut out = Dictionary::new();
        out.insert("palettes".into(), Plist::Array(palettes));
        Some(Plist::Dictionary(out))
    } else if from >= 4 && to < 4 {
        let palettes = value.as_dict()?.get("palettes")?.as_array()?;
        Some(Plist::Array(
            palettes
                .iter()
                .map(|palette| {
                    let list = palette
                        .as_dict()
                        .and_then(|p| p.get("colors"))
                        .and_then(Plist::as_array)
                        .unwrap_or(&[]);
                    Plist::Array(colors(list))
                })
                .collect(),
        ))
    } else {
        None
    }
}

fn plist_number(value: f64) -> Plist {
    if value.fract() == 0.0 {
        Plist::Integer(value as i64)
    } else {
        Plist::Float(value)
    }
}

fn plist_f64(value: &Plist) -> Option<f64> {
    match value {
        Plist::Integer(i) => Some(*i as f64),
        Plist::Float(f) => Some(*f),
        Plist::String(s) => s.parse().ok(),
        _ => None,
    }
}

/// A Smart Glyph's properties as Glyphs 4 writes them. Glyphs 3 lists them
/// with their bottom and top values (`partsSettings`) and marks each layer
/// as a property's bottom (1) or top (2) (`partSelection`), or gives its
/// value in `coordinates`. Glyphs 4 lists them as glyph axes whose default
/// is the masters' value, and gives every other layer's values in
/// `coordinates`, after a `-` for each font axis and with `-` for a
/// property the layer does not set.
fn smart_glyph_to_v4(glyph: &mut Glyph, master_ids: &[String], font_axes: usize) {
    // A glyph that has both is kept as it is.
    if glyph.smart_component_settings.is_empty() || !glyph.axes.is_empty() {
        return;
    }
    let settings = std::mem::take(&mut glyph.smart_component_settings);
    let value = |setting: &crate::common::SmartComponentSetting, selection: u8| {
        if selection == 2 {
            setting.top_value as f64
        } else {
            setting.bottom_value as f64
        }
    };
    let is_master = |layer: &Layer| {
        layer.associated_master_id.is_none() && master_ids.contains(&layer.layer_id)
    };
    let master_selection = glyph
        .layers
        .iter()
        .find(|layer| is_master(layer))
        .map(|layer| layer.part_selection.clone())
        .unwrap_or_default();
    glyph.axes = settings
        .iter()
        .map(|setting| {
            // Without a selection, the masters are at the bottom value.
            let default = master_selection
                .get(&setting.name)
                .map(|&selection| value(setting, selection))
                .unwrap_or(setting.bottom_value as f64);
            Axis {
                default: (default != 0.0).then_some(default),
                name: setting.name.clone(),
                ..Default::default()
            }
        })
        .collect();
    for layer in &mut glyph.layers {
        let selection = std::mem::take(&mut layer.part_selection);
        if is_master(layer) {
            continue;
        }
        let explicit: Vec<Plist> = match layer.attr.get("coordinates") {
            Some(Plist::Array(values)) => values.clone(),
            _ => Vec::new(),
        };
        if selection.is_empty() && explicit.is_empty() {
            continue;
        }
        let mut coordinates = vec![Plist::String("-".into()); font_axes];
        for (index, setting) in settings.iter().enumerate() {
            coordinates.push(match selection.get(&setting.name) {
                Some(&selection) => plist_number(value(setting, selection)),
                None => explicit
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| Plist::String("-".into())),
            });
        }
        layer
            .attr
            .insert("coordinates".into(), Plist::Array(coordinates));
    }
}

/// A Smart Glyph's axes as Glyphs 3 writes them: the reverse of
/// [`smart_glyph_to_v4`]. A property's bottom and top values are the lowest
/// and highest of the masters' value and the layers' values.
fn smart_glyph_to_v3(glyph: &mut Glyph, master_ids: &[String], font_axes: usize) {
    if glyph.axes.is_empty() || !glyph.smart_component_settings.is_empty() {
        return;
    }
    let axes = std::mem::take(&mut glyph.axes);
    let is_master = |layer: &Layer| {
        layer.associated_master_id.is_none() && master_ids.contains(&layer.layer_id)
    };
    let part_values = |layer: &Layer| -> Vec<Option<f64>> {
        let values: Vec<Plist> = match layer.attr.get("coordinates") {
            Some(Plist::Array(values)) => values.clone(),
            _ => Vec::new(),
        };
        (0..axes.len())
            .map(|i| values.get(font_axes + i).and_then(plist_f64))
            .collect()
    };
    let ranges: Vec<(f64, f64)> = axes
        .iter()
        .enumerate()
        .map(|(i, axis)| {
            let default = axis.default.unwrap_or(0.0);
            glyph
                .layers
                .iter()
                .filter(|layer| !is_master(layer))
                .filter_map(|layer| part_values(layer)[i])
                .fold((default, default), |(lo, hi), v| (lo.min(v), hi.max(v)))
        })
        .collect();
    glyph.smart_component_settings = axes
        .iter()
        .zip(&ranges)
        .map(
            |(axis, &(bottom, top))| crate::common::SmartComponentSetting {
                bottom_value: bottom as i32,
                top_value: top as i32,
                name: axis.name.clone(),
                bottom_name: None,
                top_name: None,
            },
        )
        .collect();
    let selection_of = |value: f64, (bottom, top): (f64, f64)| {
        if bottom == top {
            None
        } else if value == bottom {
            Some(1u8)
        } else if value == top {
            Some(2u8)
        } else {
            None
        }
    };
    for layer in &mut glyph.layers {
        if is_master(layer) {
            for (axis, &range) in axes.iter().zip(&ranges) {
                if let Some(selection) = selection_of(axis.default.unwrap_or(0.0), range) {
                    layer.part_selection.insert(axis.name.clone(), selection);
                }
            }
            continue;
        }
        let values = part_values(layer);
        if values.iter().all(Option::is_none) {
            continue;
        }
        let mut intermediate = false;
        for ((axis, &range), value) in axes.iter().zip(&ranges).zip(&values) {
            if let Some(value) = value {
                match selection_of(*value, range) {
                    Some(selection) => {
                        layer.part_selection.insert(axis.name.clone(), selection);
                    }
                    None => intermediate = true,
                }
            }
        }
        if intermediate {
            let coordinates = axes
                .iter()
                .zip(&values)
                .map(|(axis, value)| plist_number(value.unwrap_or(axis.default.unwrap_or(0.0))))
                .collect();
            layer.part_selection.clear();
            layer
                .attr
                .insert("coordinates".into(), Plist::Array(coordinates));
        } else {
            layer.attr.remove("coordinates");
        }
    }
}

impl Glyphs3 {
    /// Writes the font's Smart Glyph properties the way file format version
    /// `to` does: as `partsSettings` and `partSelection` (version 3), or as
    /// glyph axes and layer coordinates (version 4). Either way may be read.
    pub fn convert_smart_glyphs(&mut self, to: i32) {
        let master_ids: Vec<String> = self.masters.iter().map(|m| m.id.clone()).collect();
        let font_axes = self.axes.len();
        for glyph in &mut self.glyphs {
            if to >= 4 {
                smart_glyph_to_v4(glyph, &master_ids, font_axes);
            } else {
                smart_glyph_to_v3(glyph, &master_ids, font_axes);
            }
        }
    }

    /// Converts what the font holds as written in file format version
    /// `from` into what it means in version `to`, where the two write the
    /// same thing differently and glyphslib keeps it as read: the stroke
    /// positions and line caps of paths, which they number differently, the
    /// colors and gradients of shapes, ghost hints, Smart Glyph properties
    /// and the `Color Palettes` custom parameter.
    pub fn convert_format_version(&mut self, from: i32, to: i32) {
        if (from >= 4) == (to >= 4) {
            return;
        }
        let master_ids: Vec<String> = self.masters.iter().map(|m| m.id.clone()).collect();
        let font_axes = self.axes.len();
        for glyph in &mut self.glyphs {
            for layer in &mut glyph.layers {
                convert_layer_stroke_positions(layer, from, to);
                convert_layer_colors(layer, from, to);
                convert_layer_hints(layer, to);
            }
            if to >= 4 {
                smart_glyph_to_v4(glyph, &master_ids, font_axes);
            } else {
                smart_glyph_to_v3(glyph, &master_ids, font_axes);
            }
        }
        // The font, a master or an instance can carry the palettes.
        let parameters = self
            .custom_parameters
            .iter_mut()
            .chain(
                self.masters
                    .iter_mut()
                    .flat_map(|m| m.custom_parameters.iter_mut()),
            )
            .chain(
                self.instances
                    .iter_mut()
                    .flat_map(|i| i.custom_parameters.iter_mut()),
            );
        for parameter in parameters {
            if parameter.name == "Color Palettes" {
                if let Some(value) = convert_color_palettes(&parameter.value, from, to) {
                    parameter.value = value;
                }
            }
        }
    }
}

/// Glyphs 4 writes the family name and an instance's style name only in
/// their `properties`.
#[allow(clippy::ptr_arg)] // serde passes the field as &String
fn name_lives_in_properties(_name: &String) -> bool {
    current_format_version() >= 4
}

impl Glyphs3 {
    /// Fills the family name and the instances' style names from the
    /// `dflt` values of their `familyNames` and `styleNames` properties,
    /// where Glyphs 4 keeps them.
    pub fn names_from_properties(&mut self) {
        if self.family_name.is_empty() {
            if let Some(name) = default_value(&self.properties, LocalizedPropertyKey::FamilyNames) {
                self.family_name = name;
            }
        }
        for instance in &mut self.instances {
            if instance.name.is_empty() {
                if let Some(name) =
                    default_value(&instance.properties, LocalizedPropertyKey::StyleNames)
                {
                    instance.name = name;
                }
            }
        }
    }

    /// Puts the family name and the instances' style names into the `dflt`
    /// values of their `familyNames` and `styleNames` properties, where
    /// Glyphs 4 expects them, and sorts all properties by key, ignoring case,
    /// as Glyphs 4 writes them.
    pub fn names_into_properties(&mut self) {
        let sort = |properties: &mut Vec<Property>| {
            properties.sort_by_key(|p| p.key_name().unwrap_or_default().to_lowercase());
        };
        sort(&mut self.properties);
        for master in &mut self.masters {
            sort(&mut master.properties);
        }
        for instance in &mut self.instances {
            sort(&mut instance.properties);
        }
        set_default_value(
            &mut self.properties,
            LocalizedPropertyKey::FamilyNames,
            &self.family_name,
        );
        for instance in &mut self.instances {
            set_default_value(
                &mut instance.properties,
                LocalizedPropertyKey::StyleNames,
                &instance.name,
            );
        }
    }
}

fn default_value(properties: &[Property], wanted: LocalizedPropertyKey) -> Option<String> {
    properties.iter().find_map(|property| match property {
        Property::LocalizedProperty { key, values } if *key == wanted => values
            .iter()
            .find(|v| v.language == "dflt")
            .or(values.first())
            .map(|v| v.value.clone()),
        _ => None,
    })
}

fn set_default_value(properties: &mut Vec<Property>, wanted: LocalizedPropertyKey, name: &str) {
    if name.is_empty() {
        return;
    }
    for property in properties.iter_mut() {
        if let Property::LocalizedProperty { key, values } = property {
            if *key == wanted {
                match values.iter_mut().find(|v| v.language == "dflt") {
                    Some(value) => value.value = name.to_string(),
                    None => values.insert(
                        0,
                        LocalizedValue {
                            language: "dflt".to_string(),
                            value: name.to_string(),
                        },
                    ),
                }
                return;
            }
        }
    }
    // Glyphs keeps properties sorted by key, ignoring case.
    let wanted_key = property_key(&Property::localized_with_default(wanted, String::new()))
        .unwrap_or_default()
        .to_lowercase();
    let at = properties
        .iter()
        .position(|p| property_key(p).is_some_and(|k| k.to_lowercase() > wanted_key))
        .unwrap_or(properties.len());
    properties.insert(
        at,
        Property::localized_with_default(wanted, name.to_string()),
    );
}

/// The `key` a property is written with, such as `familyNames`.
fn property_key(property: &Property) -> Option<String> {
    let key = match property {
        Property::SingularProperty { key, .. } => openstep_plist::ser::to_string(key),
        Property::LocalizedProperty { key, .. } => openstep_plist::ser::to_string(key),
        Property::Junk(plist) => {
            return plist
                .as_dict()
                .and_then(|d| d.get("key"))
                .and_then(|k| k.as_str())
                .map(str::to_string)
        }
    };
    key.ok().map(|k| k.trim_matches('"').to_string())
}

#[cfg(test)]
mod tests {
    use openstep_plist::de::Deserializer;

    use super::*;

    #[test]
    fn test_guide() {
        let data = r#"
        {
            angle = 45.0;
            lockAngle = 1;
            locked = 0;
            orientation = left;
            pos = (100.0, 200.0);
            showMeasurement = 1;
            size = (300.0, 400.0);
            type = "Line";
            userData = {
                customKey = "customValue";
            };
        }
        "#;
        let plist = Plist::parse(data).expect("Failed to parse plist");
        let deserializer = &mut Deserializer::from_plist(&plist);
        let guide: Guide =
            serde_path_to_error::deserialize(deserializer).expect("Failed to deserialize Guide");

        assert_eq!(guide.angle, 45.0);
        assert!(guide.lock_angle);
        assert!(!guide.locked);
        assert_eq!(guide.orientation, Orientation::Left);
        assert_eq!(guide.pos, (100.0, 200.0));
        assert!(guide.show_measurement);
        assert_eq!(guide.size, (300.0, 400.0));
        assert_eq!(guide.guide_type, GuideType::Line);
        assert_eq!(
            guide.attr.get("customKey").and_then(|v| v.as_str()),
            Some("customValue")
        );
    }
}
