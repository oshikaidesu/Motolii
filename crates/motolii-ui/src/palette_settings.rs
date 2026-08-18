use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

pub const PALETTE_SETTINGS_VERSION: u32 = 1;
pub const USER_PALETTE_FILE_NAME: &str = "palettes.json";
const MAX_FILE_BYTES: u64 = 1_048_576;
const MAX_ID_BYTES: usize = 128;
const MAX_NAME_BYTES: usize = 256;
const MAX_PALETTES: usize = 1_024;
const MAX_COLORS_PER_PALETTE: usize = 4_096;

macro_rules! stable_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn try_new(value: impl Into<String>) -> Result<Self, PaletteSettingsError> {
                let value = value.into();
                validate_text("id", &value, MAX_ID_BYTES)?;
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                Self::try_new(String::deserialize(deserializer)?).map_err(D::Error::custom)
            }
        }
    };
}

stable_id!(PaletteId);
stable_id!(PaletteColorId);

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Rgba {
    pub fn try_new(r: f32, g: f32, b: f32, a: f32) -> Result<Self, PaletteSettingsError> {
        let rgba = Self { r, g, b, a };
        rgba.validate()?;
        Ok(rgba)
    }

    fn validate(self) -> Result<(), PaletteSettingsError> {
        if [self.r, self.g, self.b, self.a]
            .into_iter()
            .all(|channel| channel.is_finite() && (0.0..=1.0).contains(&channel))
        {
            Ok(())
        } else {
            Err(PaletteSettingsError::InvalidColor)
        }
    }
}

impl<'de> Deserialize<'de> for Rgba {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Wire {
            r: f32,
            g: f32,
            b: f32,
            a: f32,
        }
        let value = Wire::deserialize(deserializer)?;
        Self::try_new(value.r, value.g, value.b, value.a).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaletteColor {
    pub id: PaletteColorId,
    pub color: Rgba,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Palette {
    pub id: PaletteId,
    pub name: String,
    pub colors: Vec<PaletteColor>,
}

impl Palette {
    pub fn try_new(id: PaletteId, name: impl Into<String>) -> Result<Self, PaletteSettingsError> {
        let name = name.into();
        validate_text("palette name", &name, MAX_NAME_BYTES)?;
        Ok(Self {
            id,
            name,
            colors: Vec::new(),
        })
    }

    pub fn add_color(&mut self, color: PaletteColor) -> Result<(), PaletteSettingsError> {
        if self.colors.len() >= MAX_COLORS_PER_PALETTE {
            return Err(PaletteSettingsError::ResourceLimit("colors per palette"));
        }
        if self.colors.iter().any(|existing| existing.id == color.id) {
            return Err(PaletteSettingsError::DuplicateId(color.id.0));
        }
        self.colors.push(color);
        Ok(())
    }

    pub fn reorder_color(&mut self, from: usize, to: usize) -> Result<(), PaletteSettingsError> {
        move_item(&mut self.colors, from, to)
    }

    pub fn delete_color(&mut self, id: &PaletteColorId) -> bool {
        remove_by(&mut self.colors, |color| &color.id == id)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaletteSettings {
    version: u32,
    palettes: Vec<Palette>,
}

impl Default for PaletteSettings {
    fn default() -> Self {
        Self {
            version: PALETTE_SETTINGS_VERSION,
            palettes: Vec::new(),
        }
    }
}

impl PaletteSettings {
    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn palettes(&self) -> &[Palette] {
        &self.palettes
    }

    pub fn add_palette(&mut self, palette: Palette) -> Result<(), PaletteSettingsError> {
        if self.palettes.len() >= MAX_PALETTES {
            return Err(PaletteSettingsError::ResourceLimit("palettes"));
        }
        if self
            .palettes
            .iter()
            .any(|existing| existing.id == palette.id)
        {
            return Err(PaletteSettingsError::DuplicateId(palette.id.0));
        }
        validate_palette(&palette)?;
        self.palettes.push(palette);
        Ok(())
    }

    pub fn rename_palette(
        &mut self,
        id: &PaletteId,
        name: impl Into<String>,
    ) -> Result<(), PaletteSettingsError> {
        let name = name.into();
        validate_text("palette name", &name, MAX_NAME_BYTES)?;
        let palette = self
            .palettes
            .iter_mut()
            .find(|palette| &palette.id == id)
            .ok_or(PaletteSettingsError::NotFound)?;
        palette.name = name;
        Ok(())
    }

    pub fn add_color(
        &mut self,
        palette_id: &PaletteId,
        color: PaletteColor,
    ) -> Result<(), PaletteSettingsError> {
        self.palettes
            .iter_mut()
            .find(|palette| &palette.id == palette_id)
            .ok_or(PaletteSettingsError::NotFound)?
            .add_color(color)
    }

    pub fn reorder_palette(&mut self, from: usize, to: usize) -> Result<(), PaletteSettingsError> {
        move_item(&mut self.palettes, from, to)
    }

    pub fn delete_palette(&mut self, id: &PaletteId) -> bool {
        remove_by(&mut self.palettes, |palette| &palette.id == id)
    }

    pub fn load(path: &Path) -> Result<Self, PaletteSettingsError> {
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::default())
            }
            Err(source) => return Err(PaletteSettingsError::Io(source)),
        };
        if metadata.len() > MAX_FILE_BYTES {
            return Err(PaletteSettingsError::ResourceLimit("file bytes"));
        }
        let bytes = fs::read(path)?;
        let settings: Self = serde_json::from_slice(&bytes)?;
        settings.validate()?;
        Ok(settings)
    }

    pub fn save(&self, path: &Path) -> Result<(), PaletteSettingsError> {
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self)?;
        if bytes.len() as u64 > MAX_FILE_BYTES {
            return Err(PaletteSettingsError::ResourceLimit("file bytes"));
        }
        write_atomically(path, &bytes)?;
        Ok(())
    }

    fn validate(&self) -> Result<(), PaletteSettingsError> {
        if self.version != PALETTE_SETTINGS_VERSION {
            return Err(PaletteSettingsError::UnsupportedVersion(self.version));
        }
        if self.palettes.len() > MAX_PALETTES {
            return Err(PaletteSettingsError::ResourceLimit("palettes"));
        }
        let mut ids = std::collections::HashSet::new();
        for palette in &self.palettes {
            if !ids.insert(palette.id.as_str()) {
                return Err(PaletteSettingsError::DuplicateId(palette.id.0.clone()));
            }
            validate_palette(palette)?;
        }
        Ok(())
    }
}

pub fn default_user_palette_settings_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library/Application Support/Motolii")
            .join(USER_PALETTE_FILE_NAME),
    )
}

#[derive(Debug, thiserror::Error)]
pub enum PaletteSettingsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid palette settings JSON: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("unsupported palette settings version {0}")]
    UnsupportedVersion(u32),
    #[error("{0} must be non-empty and at most its byte limit")]
    InvalidText(&'static str),
    #[error("RGBA channels must be finite values from 0 through 1")]
    InvalidColor,
    #[error("resource limit exceeded: {0}")]
    ResourceLimit(&'static str),
    #[error("duplicate stable id: {0}")]
    DuplicateId(String),
    #[error("item not found")]
    NotFound,
    #[error("index is out of bounds")]
    IndexOutOfBounds,
}

fn validate_text(
    label: &'static str,
    value: &str,
    max_bytes: usize,
) -> Result<(), PaletteSettingsError> {
    if value.trim().is_empty() || value.len() > max_bytes {
        Err(PaletteSettingsError::InvalidText(label))
    } else {
        Ok(())
    }
}

fn validate_palette(palette: &Palette) -> Result<(), PaletteSettingsError> {
    validate_text("palette name", &palette.name, MAX_NAME_BYTES)?;
    if palette.colors.len() > MAX_COLORS_PER_PALETTE {
        return Err(PaletteSettingsError::ResourceLimit("colors per palette"));
    }
    let mut ids = std::collections::HashSet::new();
    for color in &palette.colors {
        color.color.validate()?;
        if !ids.insert(color.id.as_str()) {
            return Err(PaletteSettingsError::DuplicateId(color.id.0.clone()));
        }
    }
    Ok(())
}

fn move_item<T>(items: &mut Vec<T>, from: usize, to: usize) -> Result<(), PaletteSettingsError> {
    if from >= items.len() || to >= items.len() {
        return Err(PaletteSettingsError::IndexOutOfBounds);
    }
    if from != to {
        let item = items.remove(from);
        items.insert(to, item);
    }
    Ok(())
}

fn remove_by<T>(items: &mut Vec<T>, predicate: impl Fn(&T) -> bool) -> bool {
    let Some(index) = items.iter().position(predicate) else {
        return false;
    };
    items.remove(index);
    true
}

/// user 設定層の**唯一の書き方**: 同じディレクトリの temp へ書いて fsync し、
/// rename で差し替えてディレクトリも fsync する。途中で落ちても、既にある
/// ファイルが半端な内容へ化けない。
///
/// ここに居るのは palette が最初にこれを要ったからで、置き場も同じ
/// (`~/Library/Application Support/Motolii/`)である。あとから増えた設定
/// (`last_project`)は**新しい保存方式を作らず**この1本を通る。
pub(crate) fn write_atomically(path: &Path, bytes: &[u8]) -> Result<(), std::io::Error> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let (temp_path, mut temp) = create_temp(parent, path.file_name().and_then(|v| v.to_str()))?;
    let result = (|| {
        temp.write_all(bytes)?;
        temp.sync_all()?;
        drop(temp);
        fs::rename(&temp_path, path)?;
        sync_directory(parent)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

fn create_temp(parent: &Path, name: Option<&str>) -> Result<(PathBuf, fs::File), std::io::Error> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    for attempt in 0..100_u32 {
        let path = parent.join(format!(
            ".{}.{}.{}.tmp",
            name.unwrap_or("palettes"),
            std::process::id(),
            nonce + u128::from(attempt)
        ));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "could not allocate palette settings temp file",
    ))
}

fn sync_directory(path: &Path) -> Result<(), std::io::Error> {
    match fs::File::open(path).and_then(|directory| directory.sync_all()) {
        Ok(()) => Ok(()),
        Err(_error) if cfg!(windows) => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "motolii-palette-{name}-{}-{}.json",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn palette(id: &str, name: &str, colors: &[(&str, f32)]) -> Palette {
        let mut palette = Palette::try_new(PaletteId::try_new(id).unwrap(), name).unwrap();
        for (id, red) in colors {
            palette
                .add_color(PaletteColor {
                    id: PaletteColorId::try_new(*id).unwrap(),
                    color: Rgba::try_new(*red, 0.2, 0.3, 1.0).unwrap(),
                })
                .unwrap();
        }
        palette
    }

    #[test]
    fn roundtrip_preserves_order_and_stable_ids() {
        let path = temp_path("roundtrip");
        let mut settings = PaletteSettings::default();
        settings
            .add_palette(palette("warm", "Warm", &[("red", 1.0), ("orange", 0.9)]))
            .unwrap();
        settings
            .add_palette(palette("cool", "Cool", &[("blue", 0.1)]))
            .unwrap();
        settings.reorder_palette(1, 0).unwrap();
        settings.save(&path).unwrap();

        let loaded = PaletteSettings::load(&path).unwrap();
        assert_eq!(loaded, settings);
        assert_eq!(loaded.palettes()[0].id.as_str(), "cool");
        assert_eq!(loaded.palettes()[1].colors[0].id.as_str(), "red");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn missing_is_empty_but_corrupt_and_unsupported_are_errors() {
        let path = temp_path("errors");
        assert_eq!(
            PaletteSettings::load(&path).unwrap(),
            PaletteSettings::default()
        );
        fs::write(&path, b"not json").unwrap();
        assert!(matches!(
            PaletteSettings::load(&path),
            Err(PaletteSettingsError::Decode(_))
        ));
        fs::write(&path, br#"{"version":2,"palettes":[]}"#).unwrap();
        assert!(matches!(
            PaletteSettings::load(&path),
            Err(PaletteSettingsError::UnsupportedVersion(2))
        ));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn invalid_values_are_rejected_without_replacing_existing_file() {
        assert!(PaletteId::try_new(" ").is_err());
        assert!(Rgba::try_new(f32::NAN, 0.0, 0.0, 1.0).is_err());
        assert!(serde_json::from_str::<PaletteSettings>(
            r#"{"version":1,"palettes":[{"id":"p","name":"P","colors":[{"id":"c","color":{"r":2.0,"g":0.0,"b":0.0,"a":1.0}}]}]}"#
        )
        .is_err());

        let path = temp_path("preserve");
        fs::write(&path, b"existing").unwrap();
        let invalid = PaletteSettings {
            version: 2,
            palettes: Vec::new(),
        };
        assert!(invalid.save(&path).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"existing");
        let _ = fs::remove_file(path);
    }
}
