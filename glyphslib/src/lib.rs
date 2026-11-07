//! # glyphslib-rs
//!
//! A Rust library for reading, writing, and manipulating Glyphs font source files (.glyphs and .glyphspackage).
//!
//! This crate provides full support for both Glyphs 2 and Glyphs 3 file formats, allowing you to:
//! - Load and parse Glyphs files
//! - Access and modify font data (masters, instances, glyphs, layers, etc.)
//! - Serialize modified data back to Glyphs format
//! - Convert between Glyphs 2 and Glyphs 3 formats
//!
//! ## File Format Support
//!
//! - **Glyphs 2**: The original file format using OpenStep Property Lists
//! - **Glyphs 3**: The modern file format with enhanced features and better structure
//!
//! The library automatically detects which format is being used and provides a unified interface
//! through the [`Font`] enum.
//!
//! ## Examples
//!
//! ### Loading a Glyphs file
//!
//! ```no_run
//! use glyphslib::Font;
//! use std::path::Path;
//!
//! // Load a .glyphs or .glyphspackage file
//! let font = Font::load(Path::new("MyFont.glyphs")).unwrap();
//!
//! // Check the format version
//! match &font {
//!     Font::Glyphs2(g2) => println!("Loaded Glyphs 2 file: {}", g2.family_name),
//!     Font::Glyphs3(g3) => println!("Loaded Glyphs 3 file: {}", g3.family_name),
//! }
//! ```
//!
//! ### Examining font data
//!
//! ```no_run
//! use glyphslib::{Font, GlyphsFile};
//! use std::path::Path;
//!
//! let font = Font::load(Path::new("MyFont.glyphs")).unwrap();
//!
//! // Access common properties regardless of version
//! let font = font.as_glyphs3().unwrap();
//! println!("Family: {}", font.family_name());
//! println!("Number of glyphs: {}", font.glyphs().len());
//!
//! // Iterate through glyphs
//! for glyph in font.glyphs() {
//!     println!("Glyph: {}", glyph.name());
//! }
//!
//! // Access masters
//! for master in font.masters() {
//!     println!("Master: {}", master.name());
//! }
//! ```
//!
//! ### Modifying font data
//!
//! ```no_run
//! use glyphslib::Font;
//! use std::path::Path;
//! use glyphslib::glyphs3::{Property, SingularPropertyKey};
//!
//! let mut font = Font::load(Path::new("MyFont.glyphs")).unwrap();
//!
//! // Modify properties based on version
//! match &mut font {
//!     Font::Glyphs2(g2) => {
//!         g2.designer = Some("Jane Doe".to_string());
//!         // Modify a glyph
//!         if let Some(glyph) = g2.glyphs.iter_mut().find(|g| g.name == "A") {
//!             glyph.export = false;
//!         }
//!     }
//!     Font::Glyphs3(g3) => {
//!         g3.properties.push(Property::singular(SingularPropertyKey::Designer, "Jane Doe".to_string()));
//!         // Modify a glyph
//!         if let Some(glyph) = g3.glyphs.iter_mut().find(|g| g.name == "A") {
//!             glyph.export = false;
//!         }
//!     }
//! }
//! ```
//!
//! ### Serializing back to Glyphs format
//!
//! ```no_run
//! use glyphslib::Font;
//! use std::path::Path;
//! use std::fs;
//!
//! let font = Font::load(Path::new("MyFont.glyphs")).unwrap();
//!
//! // Serialize to string
//! let output = match &font {
//!     Font::Glyphs2(g2) => openstep_plist::to_string(g2).unwrap(),
//!     Font::Glyphs3(g3) => openstep_plist::to_string(g3).unwrap(),
//! };
//!
//! // Write to file
//! fs::write("MyFont_modified.glyphs", output).unwrap();
//! ```
//!
//! ### Converting between formats
//!
//! ```no_run
//! use glyphslib::Font;
//! use std::path::Path;
//!
//! let font = Font::load(Path::new("MyFont.glyphs")).unwrap();
//!
//! // Convert Glyphs 2 to Glyphs 3
//! let glyphs3 = font.upgrade();
//! ```

#![deny(missing_docs)]
/// Common types and structures shared between Glyphs 2 and Glyphs 3 formats
pub mod common;
/// Glyphs 2 file format structures
pub mod glyphs2;
/// Glyphs 3 file format structures
pub mod glyphs3;
mod serde;
mod traits;
mod upgrade;
mod utils;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    ffi::OsStr,
    fs, path,
};

pub use traits::GlyphsFile;

use glyphs2::Glyphs2;
use glyphs3::Glyphs3;
pub use openstep_plist::Plist;
use openstep_plist::{de::Deserializer, Dictionary};

use crate::common::{Feature, FeatureClass, FeaturePrefix};
use utils::user_name_to_file_name;

/// Adds the code of the feature prefixes, classes and features stored as
/// `features/*.fea` files in a Glyphs 4 package to the loaded font.
///
/// Entries `fontinfo.plist` lists without their code get it from the file
/// [`package_feature_files`] names for them, which keeps their order and
/// flags. A file no entry claims becomes a new prefix (`_` before the name),
/// class (`@`) or feature (its tag), in file name order.
fn load_package_features(entries: &HashMap<String, String>, font: &mut Glyphs3) {
    let mut files: Vec<(&str, &str)> = entries
        .iter()
        .filter_map(|(path, contents)| {
            let file_name = path.strip_prefix("features/")?;
            if file_name.contains('/') || !file_name.ends_with(".fea") {
                return None;
            }
            Some((file_name, contents.as_str()))
        })
        .collect();
    files.sort_by(|a, b| a.0.cmp(b.0));

    let mut claimed = HashSet::new();
    for (kind, index, path) in package_feature_files(font) {
        // Glyphs 4 names each entry's file; older packages leave it to the
        // naming rules.
        let Some(code) = entries.get(&path) else {
            continue;
        };
        claimed.insert(path.trim_start_matches("features/").to_string());
        match kind {
            PackageFeatureKind::Prefix => font.feature_prefixes[index].code = code.clone(),
            PackageFeatureKind::Class => font.classes[index].code = code.clone(),
            PackageFeatureKind::Feature => font.features[index].code = code.clone(),
        }
    }

    for (file_name, code) in files {
        if claimed.contains(file_name) {
            continue;
        }
        let stem = strip_feature_file_suffix(file_name.trim_end_matches(".fea"));
        if let Some(name) = stem.strip_prefix('_') {
            font.feature_prefixes.push(FeaturePrefix {
                automatic: false,
                code: code.to_string(),
                disabled: false,
                file: Some(file_name.to_string()),
                name: name.to_string(),
                notes: None,
            });
        } else if let Some(name) = stem.strip_prefix('@') {
            font.classes.push(FeatureClass {
                automatic: false,
                code: code.to_string(),
                disabled: false,
                file: Some(file_name.to_string()),
                name: name.to_string(),
                notes: None,
            });
        } else {
            font.features.push(Feature {
                automatic: false,
                code: code.to_string(),
                disabled: false,
                file: Some(file_name.to_string()),
                labels: Vec::new(),
                notes: None,
                tag: stem.to_string(),
            });
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum PackageFeatureKind {
    Prefix,
    Class,
    Feature,
}

/// The `features/*.fea` file of every feature prefix, class and feature of a
/// Glyphs 4 package, by its position in the font's lists: `_` before a
/// prefix's name, `@` before a class's, a feature's tag, and `.2`, `.3`, ...
/// after a stem an earlier file already has.
fn package_feature_files(font: &Glyphs3) -> Vec<(PackageFeatureKind, usize, String)> {
    let stems = font
        .feature_prefixes
        .iter()
        .enumerate()
        .map(|(i, p)| {
            (
                PackageFeatureKind::Prefix,
                i,
                format!("_{}", p.name),
                &p.file,
            )
        })
        .chain(font.classes.iter().enumerate().map(|(i, c)| {
            (
                PackageFeatureKind::Class,
                i,
                format!("@{}", c.name),
                &c.file,
            )
        }))
        .chain(
            font.features
                .iter()
                .enumerate()
                .map(|(i, f)| (PackageFeatureKind::Feature, i, f.tag.clone(), &f.file)),
        );
    let mut taken = HashSet::new();
    let mut files = vec![];
    for (kind, index, stem, file) in stems {
        // Keep the file an entry already names, as long as no earlier entry
        // has it.
        if let Some(file) = file.as_deref().filter(|f| {
            f.ends_with(".fea") && !f.contains(['/', '\\']) && !taken.contains(&f.to_lowercase())
        }) {
            taken.insert(file.to_lowercase());
            files.push((kind, index, format!("features/{file}")));
            continue;
        }
        // A path separator cannot be part of a file name.
        let stem = stem.replace(['/', '\\'], "_");
        let mut candidate = stem.clone();
        let mut counter = 1;
        // Compared case-insensitively, as on the file systems Glyphs runs on.
        while !taken.insert(format!("{candidate}.fea").to_lowercase()) {
            counter += 1;
            candidate = format!("{stem}.{counter}");
        }
        files.push((kind, index, format!("features/{candidate}.fea")));
    }
    files
}

/// Strips a trailing `.2`, `.3`, ... deduplication suffix from a feature file
/// stem (see the Glyphs package file naming rules).
fn strip_feature_file_suffix(stem: &str) -> &str {
    if let Some((base, suffix)) = stem.rsplit_once('.') {
        if !base.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
            return base;
        }
    }
    stem
}

fn format_version(plist: &Plist) -> u8 {
    match plist
        .as_dict()
        .and_then(|d| d.get(".formatVersion"))
        .and_then(|v| v.as_i64())
    {
        Some(version) => match version {
            3 => 3,
            4 => 4,
            _ => 3,
        },
        None => 2,
    }
}

/// A font loaded from a Glyphs file, either version 2 or 3
///
/// This enum allows working with both Glyphs file format versions through a unified interface.
/// Use the [`GlyphsFile`] trait to access common properties regardless of version.
#[derive(Debug, Clone, PartialEq)]
pub enum Font {
    /// A Glyphs 2 format font
    Glyphs2(Glyphs2),
    /// A Glyphs 3 format font
    Glyphs3(Glyphs3),
}
impl Font {
    /// Load a Glyphs file from disk
    ///
    /// Supports both `.glyphs` files and `.glyphspackage` directories.
    /// The file format version is automatically detected.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use glyphslib::Font;
    /// use std::path::Path;
    ///
    /// let font = Font::load(Path::new("MyFont.glyphs")).unwrap();
    /// ```
    pub fn load(glyphs_file: &path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        if glyphs_file.extension() == Some(OsStr::new("glyphspackage")) {
            return Font::load_package(glyphs_file);
        }
        let raw_content = fs::read_to_string(glyphs_file)?;
        Self::load_str(&raw_content)
    }

    /// Load a Glyphs package from in-memory file entries.
    ///
    /// The map keys must be paths relative to the package root, for example:
    /// - `fontinfo.plist`
    /// - `order.plist`
    /// - `UIState.plist` (optional)
    /// - `glyphs/<glyph-file-name>.glyph`
    /// - `kerning.plist` (optional)
    /// - `note.md` (optional)
    /// - `features/<feature-file-name>.fea` (optional)
    pub fn load_package_entries(
        entries: &HashMap<String, String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let normalized_entries: HashMap<String, String> = entries
            .iter()
            .map(|(path, contents)| {
                (
                    path.replace('\\', "/")
                        .trim_start_matches("./")
                        .trim_start_matches('/')
                        .to_string(),
                    contents.clone(),
                )
            })
            .collect();

        let raw_content = normalized_entries
            .get("fontinfo.plist")
            .ok_or("Missing fontinfo.plist in glyphspackage entries")?;

        let mut toplevel = Plist::parse(raw_content)?.expect_dict()?;

        if let Some(ui_state) = normalized_entries.get("UIState.plist") {
            let ui_state_plist = Plist::parse(ui_state)?;
            // UIState.plist contains a dictionary with a key "displayStrings".
            // However. the Glyphs3 non-package format has this key as "DisplayStrings" (with a capital 'D').
            // So we can't just merge dictionaries, we have to rewrite the key.
            toplevel.insert(
                "DisplayStrings".into(),
                ui_state_plist
                    .expect_dict()?
                    .get("displayStrings")
                    .cloned()
                    .unwrap_or(Plist::Array(vec![])),
            );
        }

        // In a package the kerning dictionaries live in their own file; merge
        // them into the top level dictionary.
        if let Some(kerning_content) = normalized_entries.get("kerning.plist") {
            for (key, value) in Plist::parse(kerning_content)?.expect_dict()? {
                toplevel.insert(key, value);
            }
        }

        // The font note is stored as Markdown in its own file.
        if let Some(note) = normalized_entries.get("note.md") {
            toplevel.insert("note".into(), Plist::String(note.clone()));
        }

        let glyph_order_plist = normalized_entries
            .get("order.plist")
            .ok_or("Missing order.plist in glyphspackage entries")?;
        let glyph_order = Plist::parse(glyph_order_plist).and_then(|p| p.expect_array())?;

        let mut glyphs = vec![];
        for glyph in glyph_order.iter() {
            if let Some(name) = glyph.as_str() {
                let glyph_path = format!("glyphs/{}.glyph", user_name_to_file_name(name));
                if let Some(glyph_content) = normalized_entries.get(&glyph_path) {
                    let glyph_plist = Plist::parse(glyph_content)?;
                    glyphs.push(glyph_plist);
                }
            }
        }

        toplevel.insert("glyphs".into(), Plist::Array(glyphs));
        let mut font = Self::from_plist(Plist::Dictionary(toplevel))?;
        if let Font::Glyphs3(glyphs3) = &mut font {
            load_package_features(&normalized_entries, glyphs3);
        }
        Ok(font)
    }

    /// Load a Glyphs file from a string
    ///
    /// Parses the OpenStep Property List format and deserializes into the appropriate
    /// Glyphs 2 or Glyphs 3 structure based on the format version marker.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use glyphslib::Font;
    ///
    /// let glyphs_data = std::fs::read_to_string("MyFont.glyphs").unwrap();
    /// let font = Font::load_str(&glyphs_data).unwrap();
    /// ```
    pub fn load_str(raw_content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let plist = Plist::parse(raw_content)?;
        Font::from_plist(plist)
    }

    fn from_plist(plist: Plist) -> Result<Self, Box<dyn std::error::Error>> {
        let deserializer = &mut Deserializer::from_plist(&plist);
        if format_version(&plist) >= 3 {
            // `Color` needs the file format version while (de)serializing.
            let _guard = common::with_format_version(format_version(&plist));
            let mut glyphs3: Glyphs3 = serde_path_to_error::deserialize(deserializer)?;
            glyphs3.names_from_properties();
            Ok(Font::Glyphs3(glyphs3))
        } else {
            let glyphs2: Glyphs2 = serde_path_to_error::deserialize(deserializer)?;
            Ok(Font::Glyphs2(glyphs2))
        }
    }

    /// Get a reference to the font as a Glyphs 3 structure, if it is one
    ///
    /// Returns `None` if the font is in Glyphs 2 format.
    pub fn as_glyphs3(&self) -> Option<&Glyphs3> {
        match self {
            Font::Glyphs3(glyphs3) => Some(glyphs3),
            _ => None,
        }
    }

    /// Get a reference to the font as a Glyphs 2 structure, if it is one
    ///
    /// Returns `None` if the font is in Glyphs 3 format.
    pub fn as_glyphs2(&self) -> Option<&Glyphs2> {
        match self {
            Font::Glyphs2(glyphs2) => Some(glyphs2),
            _ => None,
        }
    }

    /// Returns a reference to font, ignoring the difference between Glyphs2 and Glyphs3.
    ///
    /// Sometimes you don't need to know which version of Glyphs you are dealing with,
    /// you just want to access the common functionality. The [GlyphsFile] trait
    /// provides an interface to methods which are common to both Glyphs2 and Glyphs3.
    pub fn font(&self) -> &dyn GlyphsFile {
        match self {
            Font::Glyphs2(glyphs2) => glyphs2,
            Font::Glyphs3(glyphs3) => glyphs3,
        }
    }

    /// Returns a mutable reference to font, ignoring the difference between Glyphs2 and Glyphs3.
    pub fn font_mut(&mut self) -> &mut dyn GlyphsFile {
        match self {
            Font::Glyphs2(glyphs2) => glyphs2,
            Font::Glyphs3(glyphs3) => glyphs3,
        }
    }

    /// Returns a Glyphs3 version of the font.
    pub fn upgrade(&self) -> Self {
        match self {
            Font::Glyphs2(glyphs2) => Font::Glyphs3(Into::into(glyphs2.clone())),
            Font::Glyphs3(_) => self.clone(),
        }
    }

    /// Turns a Glyphs2 font into a Glyphs3 font in place.
    pub fn upgrade_in_place(&mut self) {
        *self = self.upgrade();
    }

    /// Serializes the font to a a Plist in string format.
    pub fn to_string(&self) -> Result<String, openstep_plist::error::Error> {
        match self {
            Font::Glyphs2(glyphs2) => openstep_plist::ser::to_string(glyphs2),
            // Use the format-version field you added to `Glyphs3` here, e.g.
            // `glyphs3.format_version.into()`.
            Font::Glyphs3(glyphs3) => {
                let _guard = common::with_format_version(glyphs3.format_version as u8);
                if glyphs3.format_version >= 4 {
                    let mut glyphs4 = glyphs3.clone();
                    glyphs4.names_into_properties();
                    common::plist_string(&glyphs4)
                } else {
                    common::plist_string(glyphs3)
                }
            }
        }
    }

    /// Saves the font to a file.
    pub fn save(&self, path: &path::Path) -> Result<(), Box<dyn std::error::Error>> {
        if path.extension() == Some(OsStr::new("glyphspackage")) {
            return self.save_package(path);
        }

        fs::write(path, self.to_string()?)?;
        Ok(())
    }

    fn load_package(glyphs_file: &path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut entries = HashMap::new();

        entries.insert(
            "fontinfo.plist".to_string(),
            fs::read_to_string(glyphs_file.join("fontinfo.plist"))?,
        );

        if let Ok(ui_state) = fs::read_to_string(glyphs_file.join("UIState.plist")) {
            entries.insert("UIState.plist".to_string(), ui_state);
        }

        let glyph_order_plist = fs::read_to_string(glyphs_file.join("order.plist"))?;
        entries.insert("order.plist".to_string(), glyph_order_plist.clone());

        let glyph_order = Plist::parse(&glyph_order_plist).and_then(|p| p.expect_array())?;
        for glyph in glyph_order.iter() {
            if let Some(name) = glyph.as_str() {
                let glyph_file = glyphs_file
                    .join("glyphs")
                    .join(format!("{}.glyph", user_name_to_file_name(name)));
                if let Ok(glyph_content) = fs::read_to_string(glyph_file) {
                    entries.insert(
                        format!("glyphs/{}.glyph", user_name_to_file_name(name)),
                        glyph_content,
                    );
                }
            }
        }

        if let Ok(kerning) = fs::read_to_string(glyphs_file.join("kerning.plist")) {
            entries.insert("kerning.plist".to_string(), kerning);
        }
        if let Ok(note) = fs::read_to_string(glyphs_file.join("note.md")) {
            entries.insert("note.md".to_string(), note);
        }
        if let Ok(feature_dir) = fs::read_dir(glyphs_file.join("features")) {
            for feature in feature_dir.flatten() {
                if let Some(file_name) = feature.file_name().to_str() {
                    if file_name.ends_with(".fea") {
                        if let Ok(contents) = fs::read_to_string(feature.path()) {
                            entries.insert(format!("features/{file_name}"), contents);
                        }
                    }
                }
            }
        }

        Self::load_package_entries(&entries)
    }

    fn save_package(&self, glyphs_file: &path::Path) -> Result<(), Box<dyn std::error::Error>> {
        for (relative_path, contents) in self.to_package_entries()? {
            let file = glyphs_file.join(relative_path);
            if let Some(parent) = file.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(file, contents)?;
        }
        Ok(())
    }

    /// Serializes the font as `.glyphspackage` file entries, keyed by path
    /// relative to the package root: `fontinfo.plist`, `order.plist`,
    /// `UIState.plist` (only with display strings) and
    /// `glyphs/<glyph-file-name>.glyph` per glyph.
    ///
    /// This is the in-memory counterpart of [`Font::load_package_entries`]
    /// and writes nothing to disk. Only Glyphs 3 packages can be written;
    /// Glyphs 2 has no package format, and Glyphs 4 packages, which keep
    /// kerning, features and the note in separate files, are not supported yet.
    pub fn to_package_entries(
        &self,
    ) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
        let glyphs3 = self.package_glyphs3()?;
        let _guard = common::with_format_version(glyphs3.format_version as u8);
        let mut entries = BTreeMap::new();
        for glyph in &glyphs3.glyphs {
            entries.insert(
                package_glyph_path(&glyph.name),
                format!(
                    "{}\n",
                    common::plist_string(&package_glyph(glyph))?.trim_end()
                ),
            );
        }
        let names: Vec<&str> = glyphs3.glyphs.iter().map(|g| g.name.as_str()).collect();
        entries.insert(
            "order.plist".to_string(),
            // Unlike the other files, Glyphs ends order.plist without a newline.
            string_array_one_per_line(&names, glyphs3.format_version >= 4)?,
        );
        if !glyphs3.display_strings.is_empty() {
            let mut dict = Dictionary::new();
            // A UIState.plist spells this key with a lowercase 'd'.
            dict.insert(
                "displayStrings".into(),
                Plist::Array(
                    glyphs3
                        .display_strings
                        .iter()
                        .map(|s| Plist::String(s.to_string()))
                        .collect(),
                ),
            );
            entries.insert(
                "UIState.plist".to_string(),
                format!(
                    "{}\n",
                    common::plist_string(&Plist::Dictionary(dict))?.trim_end()
                ),
            );
        }
        let mut toplevel = glyphs3.clone();
        toplevel.glyphs.clear();
        toplevel.display_strings.clear();
        let fontinfo = if glyphs3.format_version >= 4 {
            toplevel.names_into_properties();
            Self::split_glyphs4_package(&mut toplevel, &mut entries)?;
            let _code_in_files = common::PackageFeatureFilesGuard::new();
            common::plist_string(&toplevel)?
        } else {
            common::plist_string(&toplevel)?
        };
        entries.insert(
            "fontinfo.plist".to_string(),
            format!("{}\n", fontinfo.trim_end()),
        );
        Ok(entries)
    }

    /// Serializes one glyph as its `.glyphspackage` file, returning the path
    /// relative to the package root and the file contents, exactly as
    /// [`Font::to_package_entries`] writes it. `None` if there is no glyph of
    /// that name.
    pub fn package_glyph_entry(
        &self,
        glyph_name: &str,
    ) -> Result<Option<(String, String)>, Box<dyn std::error::Error>> {
        let glyphs3 = self.package_glyphs3()?;
        let Some(glyph) = glyphs3.glyphs.iter().find(|g| g.name == glyph_name) else {
            return Ok(None);
        };
        let _guard = common::with_format_version(glyphs3.format_version as u8);
        Ok(Some((
            package_glyph_path(&glyph.name),
            format!(
                "{}\n",
                common::plist_string(&package_glyph(glyph))?.trim_end()
            ),
        )))
    }

    /// Moves what a Glyphs 4 package keeps in files of its own out of the
    /// font: the kerning into `kerning.plist`, the note into `note.md` and the
    /// code of feature prefixes, classes and features into `features/*.fea`.
    fn split_glyphs4_package(
        toplevel: &mut Glyphs3,
        entries: &mut BTreeMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[derive(::serde::Serialize)]
        struct PackageKerning<'a> {
            #[serde(rename = "kerningContext", skip_serializing_if = "BTreeMap::is_empty")]
            context: &'a common::KerningContext,
            #[serde(rename = "kerningLTR", skip_serializing_if = "BTreeMap::is_empty")]
            ltr: &'a common::Kerning,
            #[serde(rename = "kerningRTL", skip_serializing_if = "BTreeMap::is_empty")]
            rtl: &'a common::Kerning,
            #[serde(rename = "kerningVertical", skip_serializing_if = "BTreeMap::is_empty")]
            vertical: &'a common::Kerning,
        }
        if !(toplevel.kerning.is_empty()
            && toplevel.kerning_rtl.is_empty()
            && toplevel.kerning_vertical.is_empty()
            && toplevel.kerning_context.is_empty())
        {
            let kerning = PackageKerning {
                context: &toplevel.kerning_context,
                ltr: &toplevel.kerning,
                rtl: &toplevel.kerning_rtl,
                vertical: &toplevel.kerning_vertical,
            };
            entries.insert(
                "kerning.plist".to_string(),
                format!("{}\n", common::plist_string(&kerning)?.trim_end()),
            );
        }
        toplevel.kerning.clear();
        toplevel.kerning_rtl.clear();
        toplevel.kerning_vertical.clear();
        toplevel.kerning_context.clear();
        if !toplevel.note.is_empty() {
            entries.insert("note.md".to_string(), std::mem::take(&mut toplevel.note));
        }
        for (kind, index, path) in package_feature_files(toplevel) {
            let (code, file) = match kind {
                PackageFeatureKind::Prefix => {
                    let p = &mut toplevel.feature_prefixes[index];
                    (&p.code, &mut p.file)
                }
                PackageFeatureKind::Class => {
                    let c = &mut toplevel.classes[index];
                    (&c.code, &mut c.file)
                }
                PackageFeatureKind::Feature => {
                    let f = &mut toplevel.features[index];
                    (&f.code, &mut f.file)
                }
            };
            *file = Some(path.trim_start_matches("features/").to_string());
            entries.insert(path, code.clone());
        }
        Ok(())
    }

    fn package_glyphs3(&self) -> Result<&Glyphs3, Box<dyn std::error::Error>> {
        match self {
            Font::Glyphs3(glyphs3) => Ok(glyphs3),
            Font::Glyphs2(_) => Err("Glyphs 2 files cannot be saved as a glyphspackage".into()),
        }
    }
}

/// Writes an array of strings one element per line, as Glyphs does for the
/// package's `order.plist`; the serializer would put simple values inline.
/// Glyphs 4 follows the last element with a comma too.
fn string_array_one_per_line(
    items: &[&str],
    trailing_comma: bool,
) -> Result<String, openstep_plist::error::Error> {
    let mut out = String::from("(\n");
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        out.push_str(&openstep_plist::ser::to_string(&Plist::String(
            item.to_string(),
        ))?);
    }
    if trailing_comma && !items.is_empty() {
        out.push(',');
    }
    out.push_str("\n)");
    Ok(out)
}

/// A glyph as Glyphs writes it into a package: without the time of its last
/// change, which would make every save touch the file.
fn package_glyph(glyph: &glyphs3::Glyph) -> glyphs3::Glyph {
    glyphs3::Glyph {
        last_change: None,
        ..glyph.clone()
    }
}

/// The path of a glyph's file inside a `.glyphspackage`, relative to the
/// package root, e.g. `glyphs/A_.glyph` for `A`.
pub fn package_glyph_path(glyph_name: &str) -> String {
    format!("glyphs/{}.glyph", user_name_to_file_name(glyph_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use glyphs3::Shape;
    use openstep_plist::de::Deserializer;
    // use pretty_assertions::assert_eq;
    use rstest::rstest;
    #[rstest]
    fn test_load_upgrade(#[files("resources/*glyphs")] path: PathBuf) {
        let font = Font::load(&path).unwrap();
        if font.as_glyphs2().is_some() {
            let newfont = font.upgrade();
            let outdir = path::Path::new("resources/upgraded/");
            std::fs::create_dir_all(outdir).unwrap();
            newfont
                .save(&outdir.join(path.file_name().unwrap()))
                .unwrap();
        }
    }

    #[test]
    fn test_component() {
        let file = "resources/RadioCanadaDisplay.glyphs";
        let font = Font::load(path::Path::new(file)).unwrap();
        let glyphs3 = font.as_glyphs3().unwrap();
        if let Shape::Component(component) = glyphs3
            .glyphs
            .iter()
            .find(|g| g.name == "eacute")
            .unwrap()
            .layers
            .first()
            .unwrap()
            .shapes
            .get(1)
            .unwrap()
        {
            assert_eq!(component.component_glyph, "acutecomb");
            assert_eq!(component.position, (152.0, 0.0));
        }
    }

    #[test]
    fn test_component_missing_alignment_defaults_to_manual() {
        let plist = Plist::parse(
            r#"{
                pos = (172,0);
                ref = dotaccentcomb;
            }"#,
        )
        .unwrap();
        let deserializer = &mut Deserializer::from_plist(&plist);
        let component: glyphs3::Component = serde_path_to_error::deserialize(deserializer).unwrap();

        assert_eq!(component.alignment, -1);
    }

    #[test]
    fn test_component_manual_alignment_is_omitted_on_serialize() {
        let component = glyphs3::Component {
            component_glyph: "dotaccentcomb".to_string(),
            position: (172.0, 0.0),
            alignment: -1,
            ..Default::default()
        };

        let serialized = openstep_plist::ser::to_string(&component).unwrap();

        assert!(!serialized.contains("alignment"));
        assert!(serialized.contains("ref = dotaccentcomb;"));
        assert!(serialized.contains("pos = (172,0);"));
    }

    #[test]
    fn test_component_explicit_manual_alignment_roundtrips() {
        let plist = Plist::parse(
            r#"{
                alignment = -1;
                pos = (172,0);
                ref = dotaccentcomb;
            }"#,
        )
        .unwrap();
        let deserializer = &mut Deserializer::from_plist(&plist);
        let component: glyphs3::Component = serde_path_to_error::deserialize(deserializer).unwrap();

        assert_eq!(component.alignment, -1);

        let serialized = openstep_plist::ser::to_string(&component).unwrap();

        assert!(serialized.contains("alignment = -1;"));
        assert!(serialized.contains("ref = dotaccentcomb;"));
    }

    #[test]
    fn test_anchor_user_data_moves_into_attr_for_v4() {
        // Glyphs 3 writes the user data under `userData`, after the other keys.
        let plist = Plist::parse(r#"{ name = top; userData = { foo = bar; }; }"#).unwrap();
        let anchor: glyphs3::Anchor = {
            let _guard = crate::common::with_format_version(3);
            let d = &mut Deserializer::from_plist(&plist);
            serde_path_to_error::deserialize(d).unwrap()
        };
        assert_eq!(anchor.attr.get("foo").and_then(|v| v.as_str()), Some("bar"));

        // Glyphs 4 writes it inside `attr`, as Glyphs 4.1 does.
        let plist4 =
            Plist::parse(r#"{ attr = { userData = { foo = bar; }; }; name = top; }"#).unwrap();
        let anchor4: glyphs3::Anchor = {
            let _guard = crate::common::with_format_version(4);
            let d = &mut Deserializer::from_plist(&plist4);
            serde_path_to_error::deserialize(d).unwrap()
        };
        assert_eq!(anchor4, anchor);

        let v3 = {
            let _guard = crate::common::with_format_version(3);
            openstep_plist::ser::to_string(&anchor).unwrap()
        };
        assert_eq!(v3, "{\nname = top;\nuserData = {\nfoo = bar;\n};\n}");
        let v4 = {
            let _guard = crate::common::with_format_version(4);
            openstep_plist::ser::to_string(&anchor).unwrap()
        };
        assert_eq!(
            v4,
            "{\nattr = {\nuserData = {\nfoo = bar;\n};\n};\nname = top;\n}"
        );
    }

    #[test]
    fn test_component_user_data_moves_into_attr_for_v4() {
        let plist =
            Plist::parse(r#"{ ref = A; attr = { fill = 1; }; userData = { foo = bar; }; }"#)
                .unwrap();
        let component: glyphs3::Component = {
            let _guard = crate::common::with_format_version(3);
            let d = &mut Deserializer::from_plist(&plist);
            serde_path_to_error::deserialize(d).unwrap()
        };

        // v3 keeps the attribute dict and the user data separate.
        let v3 = {
            let _guard = crate::common::with_format_version(3);
            openstep_plist::ser::to_string(&component).unwrap()
        };
        assert!(v3.contains("attr = {\nfill = 1;\n};"), "got {v3}");
        assert!(v3.contains("userData = {\nfoo = bar;\n};"), "got {v3}");

        // v4 writes the user data inside `attr`, and reads it back from there.
        let v4 = {
            let _guard = crate::common::with_format_version(4);
            openstep_plist::ser::to_string(&component).unwrap()
        };
        assert!(
            v4.contains("attr = {\nfill = 1;\nuserData = {\nfoo = bar;\n};\n};"),
            "got {v4}"
        );
        let back: glyphs3::Component = {
            let _guard = crate::common::with_format_version(4);
            let plist = Plist::parse(&v4).unwrap();
            let d = &mut Deserializer::from_plist(&plist);
            serde_path_to_error::deserialize(d).unwrap()
        };
        assert_eq!(back, component);
    }

    #[test]
    fn test_v4_node_attr_is_split_into_user_data_and_hoi() {
        // In v4 the fourth tuple element is a `nodeAttr` object holding `hoi`
        // and `userData`; both should surface as separate fields.
        let plist = Plist::parse(
            r#"(100, 200, "c", { hoi = { wght = { ip = (10, 20); }; }; userData = { foo = bar; }; })"#,
        )
        .unwrap();
        let _guard = crate::common::with_format_version(4);
        let d = &mut Deserializer::from_plist(&plist);
        let node: glyphs3::Node = serde_path_to_error::deserialize(d).unwrap();

        assert_eq!(
            node.user_data
                .as_ref()
                .and_then(|d| d.get("foo"))
                .and_then(|p| p.as_str()),
            Some("bar")
        );
        assert!(node.hoi.as_ref().unwrap().contains_key("wght"));

        // v4 recombines both parts back into a single `nodeAttr` object.
        let serialized = {
            let _guard = crate::common::with_format_version(4);
            openstep_plist::ser::to_string(&node).unwrap()
        };
        assert!(serialized.contains("hoi"));
        assert!(serialized.contains("userData"));

        // v3 writes the user data on its own and drops `hoi`.
        let serialized3 = {
            let _guard = crate::common::with_format_version(3);
            openstep_plist::ser::to_string(&node).unwrap()
        };
        assert!(serialized3.contains("foo = bar;"));
        assert!(!serialized3.contains("hoi"));
    }

    #[test]
    fn test_v3_node_attr_is_user_data() {
        // In v3 the fourth element is a bare `userData` dictionary.
        let plist = Plist::parse(r#"(100, 200, "c", { foo = bar; })"#).unwrap();
        let _guard = crate::common::with_format_version(3);
        let d = &mut Deserializer::from_plist(&plist);
        let node: glyphs3::Node = serde_path_to_error::deserialize(d).unwrap();

        assert_eq!(
            node.user_data
                .as_ref()
                .and_then(|d| d.get("foo"))
                .and_then(|p| p.as_str()),
            Some("bar")
        );
        assert!(node.hoi.is_none());
    }

    #[test]
    fn test_node_type_and_flags_roundtrip() {
        use crate::common::{NodeType, Orientation};

        let plist = Plist::parse(r#"(1, 2, "cstCX", {})"#).unwrap();
        let _guard = crate::common::with_format_version(4);
        let d = &mut Deserializer::from_plist(&plist);
        let node: glyphs3::Node = serde_path_to_error::deserialize(d).unwrap();

        assert_eq!(node.node_type, NodeType::CurveSmooth);
        assert!(node.tangent, "`t` should set tangent");
        assert_eq!(
            node.orientation,
            Orientation::Center,
            "`C` should set center"
        );
        assert!(node.locked, "`X` should set locked");

        let node = glyphs3::Node {
            x: 1.0,
            y: 2.0,
            node_type: NodeType::CurveSmooth,
            tangent: true,
            locked: true,
            orientation: Orientation::Right,
            user_data: None,
            hoi: None,
        };
        let serialized = {
            let _guard = crate::common::with_format_version(4);
            openstep_plist::ser::to_string(&node).unwrap()
        };
        assert!(serialized.contains("cstRX"), "got {serialized}");
    }

    #[test]
    fn test_load_package_entries_reads_extra_files() {
        let mut entries = HashMap::new();
        entries.insert(
            "fontinfo.plist".to_string(),
            r#"{ .formatVersion = 4; familyName = "Test"; unitsPerEm = 1000; }"#.to_string(),
        );
        entries.insert("order.plist".to_string(), "()".to_string());
        entries.insert(
            "kerning.plist".to_string(),
            r#"{ kerningLTR = { m1 = { A = { B = -10; }; }; }; kerningContext = { "A|B" = { m1 = 5; }; }; }"#
                .to_string(),
        );
        entries.insert("note.md".to_string(), "# Hello\n\nSome note.".to_string());
        entries.insert(
            "features/_Languagesystems.fea".to_string(),
            "languagesystem DFLT dflt;".to_string(),
        );
        entries.insert("features/@Uppercase.fea".to_string(), "A B C".to_string());
        entries.insert(
            "features/calt.fea".to_string(),
            "sub a by a.alt;".to_string(),
        );
        entries.insert(
            "features/calt.2.fea".to_string(),
            "sub b by b.alt;".to_string(),
        );

        let font = Font::load_package_entries(&entries).unwrap();
        let g3 = font.as_glyphs3().unwrap();

        // note.md
        assert_eq!(g3.note, "# Hello\n\nSome note.");

        // kerning.plist is merged into the top level
        assert_eq!(
            g3.kerning
                .get("m1")
                .and_then(|m| m.get("A"))
                .and_then(|p| p.get("B"))
                .copied(),
            Some(-10.0)
        );
        assert_eq!(
            g3.kerning_context
                .get("A|B")
                .and_then(|m| m.get("m1"))
                .copied(),
            Some(5.0)
        );

        // features/*.fea files become prefixes, classes and features
        assert_eq!(g3.feature_prefixes.len(), 1);
        assert_eq!(g3.feature_prefixes[0].name, "Languagesystems");
        assert_eq!(g3.feature_prefixes[0].code, "languagesystem DFLT dflt;");

        assert_eq!(g3.classes.len(), 1);
        assert_eq!(g3.classes[0].name, "Uppercase");
        assert_eq!(g3.classes[0].code, "A B C");

        // the `.2` dedup suffix is stripped, so both files map to the `calt` tag
        assert_eq!(g3.features.len(), 2);
        assert!(g3.features.iter().all(|f| f.tag == "calt"));
        let codes: Vec<&str> = g3.features.iter().map(|f| f.code.as_str()).collect();
        assert!(codes.contains(&"sub a by a.alt;"));
        assert!(codes.contains(&"sub b by b.alt;"));
    }

    #[test]
    fn test_load_package_directory_reads_extra_files() {
        let package = std::env::temp_dir().join(format!(
            "glyphslib-package-test-{}.glyphspackage",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&package);
        std::fs::create_dir_all(package.join("glyphs")).unwrap();
        std::fs::create_dir_all(package.join("features")).unwrap();
        std::fs::write(
            package.join("fontinfo.plist"),
            r#"{ .formatVersion = 4; familyName = "Test"; unitsPerEm = 1000; }"#,
        )
        .unwrap();
        std::fs::write(package.join("order.plist"), "()").unwrap();
        std::fs::write(
            package.join("kerning.plist"),
            r#"{ kerningLTR = { m1 = { A = { B = -10; }; }; }; }"#,
        )
        .unwrap();
        std::fs::write(package.join("note.md"), "hello").unwrap();
        std::fs::write(package.join("features").join("calt.fea"), "sub a by a.alt;").unwrap();

        let font = Font::load(&package).unwrap();
        let g3 = font.as_glyphs3().unwrap();
        assert_eq!(g3.note, "hello");
        assert_eq!(g3.features.len(), 1);
        assert_eq!(g3.features[0].tag, "calt");
        assert!(g3.kerning.contains_key("m1"));

        let _ = std::fs::remove_dir_all(&package);
    }

    #[test]
    fn test_v2_instance_missing_interpolation_defaults_to_100() {
        // Glyphs 2 omits interpolationWeight/Width when the instance is at the default (100), exactly
        // like a master omits weightValue/widthValue. They MUST default to the same value (100), or the
        // instance desyncs from its master (instance at design 0, master at 100) and source
        // instantiation fails with "No master was present at location Design {wght: 0.0}".
        let plist = Plist::parse(
            r#"{
                name = Thin;
            }"#,
        )
        .unwrap();
        let deserializer = &mut Deserializer::from_plist(&plist);
        let inst: glyphs2::Instance = serde_path_to_error::deserialize(deserializer).unwrap();
        assert_eq!(
            inst.weight_value, 100.0,
            "omitted interpolationWeight must default to 100, not 0"
        );
        assert_eq!(
            inst.width_value, 100.0,
            "omitted interpolationWidth must default to 100, not 0"
        );

        // an explicit value is still honoured
        let plist2 = Plist::parse(r#"{ name = Light; interpolationWeight = 300; }"#).unwrap();
        let d2 = &mut Deserializer::from_plist(&plist2);
        let inst2: glyphs2::Instance = serde_path_to_error::deserialize(d2).unwrap();
        assert_eq!(inst2.weight_value, 300.0);
    }

    #[test]
    fn test_v2_unquoted_all_digit_hex_unicode() {
        // Glyphs 2 writes `unicode` as hex; an all-digit value is unquoted, so the
        // plist reads it as an integer. Its digits are still hex.
        for (text, expected) in [("2013", 0x2013), ("0041", 0x41), ("00C1", 0xC1)] {
            let plist = Plist::parse(&format!(
                "{{ glyphname = g; layers = (); unicode = {text}; }}"
            ))
            .unwrap();
            let deserializer = &mut Deserializer::from_plist(&plist);
            let glyph: glyphs2::Glyph = serde_path_to_error::deserialize(deserializer).unwrap();
            assert_eq!(glyph.unicode, vec![expected], "unicode = {text};");
        }
    }

    #[test]
    fn test_v2_background_image_missing_transform_defaults_to_identity() {
        // Glyphs.app omits the `transform` key when a background image is at its native position;
        // glyphsLib defaults it to the identity matrix. Such files must load, not error with
        // "missing field `transform`".
        let plist = Plist::parse(
            r#"{
                imagePath = "../numerals/2.jpg";
            }"#,
        )
        .unwrap();
        let deserializer = &mut Deserializer::from_plist(&plist);
        let bg: glyphs2::BackgroundImage = serde_path_to_error::deserialize(deserializer).unwrap();

        assert_eq!(bg.image_path, "../numerals/2.jpg");
        assert_eq!(bg.transform, glyphs2::Transform::default());
        // identity, NOT the all-zeros matrix derive(Default) used to give
        assert_eq!(
            bg.transform,
            glyphs2::Transform {
                m11: 1.0,
                m12: 0.0,
                m21: 0.0,
                m22: 1.0,
                t_x: 0.0,
                t_y: 0.0
            }
        );
    }

    #[test]
    fn test_v2_background_image_explicit_transform_roundtrips() {
        let plist = Plist::parse(
            r#"{
                imagePath = "../numerals/1.jpg";
                transform = "{10, 0, 0, 10, -241.369, -238.194}";
            }"#,
        )
        .unwrap();
        let deserializer = &mut Deserializer::from_plist(&plist);
        let bg: glyphs2::BackgroundImage = serde_path_to_error::deserialize(deserializer).unwrap();
        assert_eq!(bg.transform.m11, 10.0);
        assert_eq!(bg.transform.t_x, -241.369);

        // an identity transform is omitted on serialize (matches Glyphs.app)
        let identity = glyphs2::BackgroundImage {
            image_path: "x.png".to_string(),
            ..Default::default()
        };
        let serialized = openstep_plist::ser::to_string(&identity).unwrap();
        assert!(!serialized.contains("transform"));
    }

    use path::PathBuf;

    #[rstest]
    fn test_roundtrip(
        #[files("resources/*glyphs")]
        #[exclude("resources/RadioCanadaDisplay.glyphs")]
        #[exclude("resources/RoadRage.glyphs")]
        path: PathBuf,
    ) {
        let raw_content = fs::read_to_string(path).unwrap();
        let plist = Plist::parse(&raw_content).unwrap();
        let font = Font::load_str(&raw_content).unwrap();
        let serialised = font.to_string().unwrap();
        let new_plist = Plist::parse(&serialised).unwrap();
        pretty_assertions::assert_eq!(plist, new_plist);
    }
}

#[cfg(test)]
mod package_entry_tests {
    use super::*;

    fn read_package(root: &path::Path) -> HashMap<String, String> {
        let mut entries = HashMap::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    let relative = path.strip_prefix(root).unwrap().to_string_lossy();
                    entries.insert(
                        relative.replace('\\', "/"),
                        fs::read_to_string(&path).unwrap(),
                    );
                }
            }
        }
        entries
    }

    #[test]
    fn package_entries_reproduce_a_glyphs_written_package() {
        let root = path::Path::new("resources/packages/YanoneKaffeesatz.glyphspackage");
        let original = read_package(root);
        let font = Font::load_package_entries(&original).unwrap();
        let written = font.to_package_entries().unwrap();

        let mut written_paths: Vec<&String> = written.keys().collect();
        let mut original_paths: Vec<&String> = original.keys().collect();
        written_paths.sort();
        original_paths.sort();
        assert_eq!(written_paths, original_paths);
        // Glyph files and the glyph order come out byte for byte as Glyphs
        // wrote them. fontinfo.plist only differs in how the master value
        // arrays are wrapped: the serializer inlines arrays of numbers, Glyphs
        // puts these on separate lines.
        for (path, contents) in &written {
            if path == "fontinfo.plist" {
                let original_lines: std::collections::HashSet<&str> =
                    original[path].lines().collect();
                let extra: Vec<&str> = contents
                    .lines()
                    .filter(|l| !original_lines.contains(l))
                    .collect();
                assert!(
                    extra
                        .iter()
                        .all(|l| l.starts_with("axesValues = (") || l.starts_with("stemValues = (")),
                    "unexpected fontinfo.plist differences: {extra:?}"
                );
            } else {
                assert_eq!(contents, &original[path], "{path} differs");
            }
        }
        assert_eq!(
            Font::load_package_entries(&written.into_iter().collect()).unwrap(),
            font
        );
    }

    #[test]
    fn package_glyph_entry_matches_the_full_package() {
        let root = path::Path::new("resources/packages/YanoneKaffeesatz.glyphspackage");
        let font = Font::load_package_entries(&read_package(root)).unwrap();
        let written = font.to_package_entries().unwrap();
        let (path, contents) = font.package_glyph_entry("A").unwrap().unwrap();
        assert_eq!(path, "glyphs/A_.glyph");
        assert_eq!(&contents, &written[&path]);
        assert!(font.package_glyph_entry("no-such-glyph").unwrap().is_none());
    }

    #[test]
    fn layer_visibility_round_trips() {
        let root = path::Path::new("resources/packages/YanoneKaffeesatz.glyphspackage");
        let original = read_package(root);
        let glyph_file = &original["glyphs/E_f-cy.loclB_G_R_.glyph"];
        assert!(glyph_file.contains("visible = 1;"));
        let font = Font::load_package_entries(&original).unwrap();
        let glyph = &font.as_glyphs3().unwrap().glyphs;
        let ef = glyph.iter().find(|g| g.name == "Ef-cy.loclBGR").unwrap();
        assert!(ef.layers.iter().any(|l| l.visible));
        assert!(ef.layers.iter().any(|l| !l.visible));
    }

    /// The example package as a Glyphs 4 font, with the things a Glyphs 4
    /// package keeps in files of their own.
    fn glyphs4_font() -> Font {
        let root = path::Path::new("resources/packages/YanoneKaffeesatz.glyphspackage");
        let Font::Glyphs3(mut glyphs3) = Font::load_package_entries(&read_package(root)).unwrap()
        else {
            panic!("not a Glyphs 3 font");
        };
        glyphs3.format_version = 4;
        glyphs3.note = "A note\nover two lines".to_string();
        let mut second = glyphs3.features[0].clone();
        second.code = "# second\n".to_string();
        second.disabled = true;
        second.notes = Some("kept".to_string());
        glyphs3.features.push(second);
        Font::Glyphs3(glyphs3)
    }

    #[test]
    fn glyphs4_package_keeps_kerning_note_and_features_in_their_own_files() {
        let font = glyphs4_font();
        let glyphs3 = font.as_glyphs3().unwrap();
        let written = font.to_package_entries().unwrap();

        assert!(written["kerning.plist"].contains("kerningLTR"));
        assert_eq!(written["note.md"], glyphs3.note);
        let fontinfo = &written["fontinfo.plist"];
        assert!(!fontinfo.contains("kerningLTR"));
        assert!(!fontinfo.contains("A note"));
        assert!(!fontinfo.contains("code = "));

        let tag = &glyphs3.features[0].tag;
        assert_eq!(
            written[&format!("features/{tag}.fea")],
            glyphs3.features[0].code
        );
        assert_eq!(written[&format!("features/{tag}.2.fea")], "# second\n");
        let class = &glyphs3.classes[0];
        assert_eq!(
            written[&format!("features/@{}.fea", class.name)],
            class.code
        );

        // Reading it back gives the same font: order, flags and code. The
        // entries now name their files, and the family name is a property.
        let reloaded = Font::load_package_entries(&written.clone().into_iter().collect()).unwrap();
        let again = reloaded.as_glyphs3().unwrap();
        assert_eq!(again.family_name, glyphs3.family_name);
        let without_files = |features: &[Feature]| {
            features
                .iter()
                .cloned()
                .map(|f| Feature { file: None, ..f })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            without_files(&again.features),
            without_files(&glyphs3.features)
        );
        assert_eq!(
            again.features.last().unwrap().file.as_deref(),
            Some(&*format!("{tag}.2.fea"))
        );
        assert_eq!(reloaded.to_package_entries().unwrap(), written);
        // A glyph file is the same as in a Glyphs 3 package.
        let (path, contents) = font.package_glyph_entry("A").unwrap().unwrap();
        assert_eq!(written[&path], contents);
    }

    /// Lines a written file may have that Glyphs writes differently, in
    /// Glyphs 3 as in Glyphs 4: numeric dictionary keys in quotes, and
    /// arrays Glyphs puts one element per line written inline.
    fn known_formatting_gap(line: &str) -> bool {
        let quoted_number_key = line.starts_with('"')
            && line[1..]
                .split('"')
                .next()
                .is_some_and(|k| !k.is_empty() && k.chars().all(|c| c.is_ascii_digit()));
        let inline_array = ["axesValues = (", "numberValues = (", "value = ("]
            .iter()
            .any(|p| line.starts_with(p))
            && line.ends_with(");");
        quoted_number_key || inline_array
    }

    fn assert_same_but_known_gaps(path: &str, written: &str, original: &str) {
        let original_lines: HashSet<&str> = original.lines().collect();
        let extra: Vec<&str> = written
            .lines()
            .filter(|l| !original_lines.contains(l) && !known_formatting_gap(l))
            .collect();
        assert!(extra.is_empty(), "{path}: unexpected lines {extra:?}");
    }

    /// A file saved by Glyphs 4 (Fustat, cut down to a few glyphs).
    #[test]
    fn glyphs4_file_from_glyphs_round_trips() {
        let original = fs::read_to_string("resources/FustatG4-subset.glyphs").unwrap();
        let font = Font::load_str(&original).unwrap();
        let glyphs3 = font.as_glyphs3().unwrap();
        assert_eq!(glyphs3.format_version, 4);
        // Glyphs 4 keeps these names only in properties.
        assert_eq!(glyphs3.family_name, "Fustat");
        assert_eq!(glyphs3.instances[0].name, "ExtraLight");
        assert_eq!(glyphs3.settings.color_space.as_deref(), Some("apple-rgb"));

        let written = font.to_string().unwrap();
        assert!(!written.contains("\nfamilyName = "));
        assert!(written.contains("key = familyNames;"));
        // Block arrays end with a comma after the last element.
        assert!(written.contains(",\n);"));
        assert_same_but_known_gaps("FustatG4-subset.glyphs", &written, &original);
        assert_eq!(Font::load_str(&written).unwrap(), font);
    }

    #[test]
    fn glyphs4_package_from_glyphs_round_trips() {
        let root = path::Path::new("resources/packages/FustatG4.glyphspackage");
        let original = read_package(root);
        let font = Font::load_package_entries(&original).unwrap();
        let glyphs3 = font.as_glyphs3().unwrap();
        assert_eq!(glyphs3.family_name, "Fustat");
        let ccmp: Vec<&str> = glyphs3
            .features
            .iter()
            .filter(|f| f.tag == "ccmp")
            .map(|f| f.code.as_str())
            .collect();
        assert_eq!(ccmp.len(), 2);
        assert_eq!(ccmp[1], original["features/ccmp.2.fea"]);

        let written = font.to_package_entries().unwrap();
        let paths = |m: &BTreeMap<String, String>| m.keys().cloned().collect::<Vec<_>>();
        let mut original_paths: Vec<String> = original.keys().cloned().collect();
        original_paths.sort();
        assert_eq!(paths(&written), original_paths);
        for (path, contents) in &written {
            if path == "fontinfo.plist" || path == "kerning.plist" {
                assert_same_but_known_gaps(path, contents, &original[path]);
            } else {
                assert_eq!(contents, &original[path], "{path} differs");
            }
        }
        let reloaded = Font::load_package_entries(&written.into_iter().collect()).unwrap();
        assert_eq!(reloaded, font);
    }

    /// The flat file and the package Glyphs 4 saved of the same font read
    /// the same, apart from the files the package names for features.
    #[test]
    fn glyphs4_file_and_package_read_alike() {
        let flat = Font::load(path::Path::new("resources/FustatG4-subset.glyphs")).unwrap();
        let package =
            Font::load(path::Path::new("resources/packages/FustatG4.glyphspackage")).unwrap();
        let Font::Glyphs3(mut package) = package else {
            panic!("not a Glyphs 3 font");
        };
        for f in &mut package.features {
            f.file = None;
        }
        for c in &mut package.classes {
            c.file = None;
        }
        for p in &mut package.feature_prefixes {
            p.file = None;
        }
        assert_eq!(&package, flat.as_glyphs3().unwrap());
    }
}
