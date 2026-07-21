use serde::de::{DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

pub const GENERATOR_ID: &str = "motolii-ui-token-gen";
pub const GENERATOR_VERSION: u32 = 1;
pub const SCHEMA_URI: &str = "https://www.designtokens.org/schemas/2025.10/format.json";
const MAX_SOURCE_BYTES: usize = 1024 * 1024;
const MAX_DEPTH: usize = 32;
const MAX_TOKENS: usize = 4096;
const MAX_SEGMENT_BYTES: usize = 128;
const MAX_PATH_BYTES: usize = 512;
const OUTPUTS: [&str; 2] = ["tokens.rs", "manifest.json"];

#[derive(Debug, Clone)]
pub struct ThemeSource {
    pub id: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedBundle {
    pub tokens_rs: Vec<u8>,
    pub manifest_json: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("theme source list must not be empty")]
    NoThemes,
    #[error("source for theme `{theme}` exceeds 1 MiB")]
    SourceTooLarge { theme: String },
    #[error("invalid JSON for theme `{theme}`: {source}")]
    InvalidJson {
        theme: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("duplicate JSON key at `{path}`")]
    DuplicateKey { path: String },
    #[error("JSON nesting exceeds {MAX_DEPTH} at `{path}`")]
    NestingLimit { path: String },
    #[error("invalid schema in theme `{theme}`")]
    InvalidSchema { theme: String },
    #[error("invalid name `{name}` at `{path}`")]
    InvalidName { path: String, name: String },
    #[error("invalid structure at `{path}`: {reason}")]
    InvalidStructure { path: String, reason: String },
    #[error("unsupported feature `{feature}` at `{path}`")]
    UnsupportedFeature { path: String, feature: String },
    #[error("unknown property `{property}` at `{path}`")]
    UnknownProperty { path: String, property: String },
    #[error("missing type at token `{path}`")]
    MissingType { path: String },
    #[error("invalid value for `{kind}` at `{path}`: {reason}")]
    InvalidValue {
        path: String,
        kind: String,
        reason: String,
    },
    #[error("token count exceeds {MAX_TOKENS}")]
    TokenLimit,
    #[error("token path exceeds {MAX_PATH_BYTES} bytes: `{path}`")]
    PathLimit { path: String },
    #[error("theme ID `{id}` is duplicated")]
    DuplicateThemeId { id: String },
    #[error("themes do not have identical token paths and types: {differences:?}")]
    ThemeMismatch { differences: Vec<String> },
    #[error("Rust field collision `{field}` for paths {paths:?}")]
    FieldCollision { field: String, paths: Vec<String> },
    #[error("Rust field `{field}` from `{path}` is a keyword")]
    FieldKeyword { path: String, field: String },
    #[error("theme variant collision `{variant}` for IDs {ids:?}")]
    VariantCollision { variant: String, ids: Vec<String> },
    #[error("theme variant `{variant}` from `{id}` is a keyword")]
    VariantKeyword { id: String, variant: String },
    #[error("output directory contains unexpected entry `{path}`")]
    UnexpectedOutputEntry { path: PathBuf },
    #[error("output `{path}` is missing or is not a regular file")]
    MissingOutput { path: PathBuf },
    #[error("generated output drift at `{path}`")]
    Drift { path: PathBuf },
    #[error("I/O failed at `{path}`: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("manifest serialization failed: {0}")]
    Manifest(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
enum JsonValue {
    Null,
    Bool,
    Number(serde_json::Number),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

struct ValueSeed {
    path: String,
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for ValueSeed {
    type Value = JsonValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if self.depth > MAX_DEPTH {
            return Err(D::Error::custom(format!("__NEST__{}", self.path)));
        }
        deserializer.deserialize_any(ValueVisitor {
            path: self.path,
            depth: self.depth,
        })
    }
}

struct ValueVisitor {
    path: String,
    depth: usize,
}

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = JsonValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E> {
        Ok(JsonValue::Bool)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(JsonValue::Number(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(JsonValue::Number(value.into()))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        serde_json::Number::from_f64(value)
            .map(JsonValue::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(JsonValue::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(JsonValue::String(value))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(JsonValue::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(JsonValue::Null)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(ValueSeed {
            path: format!("{}[{}]", self.path, values.len()),
            depth: self.depth + 1,
        })? {
            values.push(value);
        }
        Ok(JsonValue::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = BTreeMap::new();
        while let Some(key) = map.next_key::<String>()? {
            let child_path = if self.path == "$" {
                format!("$.{key}")
            } else {
                format!("{}.{key}", self.path)
            };
            if values.contains_key(&key) {
                return Err(A::Error::custom(format!("__DUP__{child_path}")));
            }
            let value = map.next_value_seed(ValueSeed {
                path: child_path,
                depth: self.depth + 1,
            })?;
            values.insert(key, value);
        }
        Ok(JsonValue::Object(values))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
enum TokenType {
    Color,
    Dimension,
    Duration,
    CubicBezier,
}

impl TokenType {
    fn parse(value: &str, path: &str) -> Result<Self, Error> {
        match value {
            "color" => Ok(Self::Color),
            "dimension" => Ok(Self::Dimension),
            "duration" => Ok(Self::Duration),
            "cubicBezier" => Ok(Self::CubicBezier),
            other => Err(Error::UnsupportedFeature {
                path: path.to_owned(),
                feature: format!("type:{other}"),
            }),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Color => "color",
            Self::Dimension => "dimension",
            Self::Duration => "duration",
            Self::CubicBezier => "cubicBezier",
        }
    }

    fn rust_type(self) -> &'static str {
        match self {
            Self::Color => "egui::Color32",
            Self::Dimension | Self::Duration => "f32",
            Self::CubicBezier => "[f32; 4]",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TokenValue {
    Color([u8; 4]),
    Scalar(f32),
    Bezier([f32; 4]),
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenType,
    value: TokenValue,
}

#[derive(Debug)]
struct Theme {
    id: String,
    variant: String,
    tokens: BTreeMap<String, Token>,
}

#[derive(Serialize)]
struct Manifest<'a> {
    schema: &'static str,
    generator: Generator,
    input_sha256: &'a str,
    themes: Vec<&'a str>,
    tokens: Vec<ManifestToken<'a>>,
    outputs: [&'static str; 2],
}

#[derive(Serialize)]
struct Generator {
    id: &'static str,
    version: u32,
}

#[derive(Serialize)]
struct ManifestToken<'a> {
    path: &'a str,
    #[serde(rename = "type")]
    kind: &'static str,
}

pub fn generate_bundle(mut sources: Vec<ThemeSource>) -> Result<GeneratedBundle, Error> {
    if sources.is_empty() {
        return Err(Error::NoThemes);
    }
    sources.sort_by(|left, right| left.id.as_bytes().cmp(right.id.as_bytes()));
    let mut seen_ids = BTreeSet::new();
    for source in &sources {
        validate_name(&source.id, "$theme")?;
        if !seen_ids.insert(source.id.clone()) {
            return Err(Error::DuplicateThemeId {
                id: source.id.clone(),
            });
        }
    }

    let input_sha256 = bundle_hash(&sources);
    let mut themes = Vec::with_capacity(sources.len());
    for source in &sources {
        if source.bytes.len() > MAX_SOURCE_BYTES {
            return Err(Error::SourceTooLarge {
                theme: source.id.clone(),
            });
        }
        themes.push(parse_theme(source)?);
    }
    validate_theme_parity(&themes)?;
    validate_identifiers(&themes)?;

    let tokens_rs = render_rust(&themes, &input_sha256).into_bytes();
    let first = &themes[0];
    let manifest = Manifest {
        schema: SCHEMA_URI,
        generator: Generator {
            id: GENERATOR_ID,
            version: GENERATOR_VERSION,
        },
        input_sha256: &input_sha256,
        themes: themes.iter().map(|theme| theme.id.as_str()).collect(),
        tokens: first
            .tokens
            .iter()
            .map(|(path, token)| ManifestToken {
                path,
                kind: token.kind.as_str(),
            })
            .collect(),
        outputs: OUTPUTS,
    };
    let mut manifest_json = serde_json::to_string_pretty(&manifest)?.into_bytes();
    manifest_json.push(b'\n');
    Ok(GeneratedBundle {
        tokens_rs,
        manifest_json,
    })
}

pub fn generate_to_dir(sources: Vec<ThemeSource>, out_dir: &Path) -> Result<(), Error> {
    let bundle = generate_bundle(sources)?;
    inspect_output_dir(out_dir, false)?;
    fs::create_dir_all(out_dir).map_err(|source| Error::Io {
        path: out_dir.to_owned(),
        source,
    })?;
    write_output(out_dir.join("tokens.rs"), &bundle.tokens_rs)?;
    write_output(out_dir.join("manifest.json"), &bundle.manifest_json)?;
    Ok(())
}

pub fn check_dir(sources: Vec<ThemeSource>, out_dir: &Path) -> Result<(), Error> {
    let bundle = generate_bundle(sources)?;
    inspect_output_dir(out_dir, true)?;
    check_output(out_dir.join("tokens.rs"), &bundle.tokens_rs)?;
    check_output(out_dir.join("manifest.json"), &bundle.manifest_json)?;
    Ok(())
}

fn parse_theme(source: &ThemeSource) -> Result<Theme, Error> {
    let mut deserializer = serde_json::Deserializer::from_slice(&source.bytes);
    let root = match (ValueSeed {
        path: "$".to_owned(),
        depth: 0,
    })
    .deserialize(&mut deserializer)
    {
        Ok(value) => value,
        Err(error) => {
            let message = error.to_string();
            if let Some(path) = marker_path(&message, "__DUP__") {
                return Err(Error::DuplicateKey { path });
            }
            if let Some(path) = marker_path(&message, "__NEST__") {
                return Err(Error::NestingLimit { path });
            }
            return Err(Error::InvalidJson {
                theme: source.id.clone(),
                source: error,
            });
        }
    };
    deserializer
        .end()
        .map_err(|source_error| Error::InvalidJson {
            theme: source.id.clone(),
            source: source_error,
        })?;
    let object = as_object(&root, "$")?;
    match object.get("$schema") {
        Some(JsonValue::String(schema)) if schema == SCHEMA_URI => {}
        _ => {
            return Err(Error::InvalidSchema {
                theme: source.id.clone(),
            });
        }
    }
    reject_feature_keys(object, "$")?;
    let mut tokens = BTreeMap::new();
    walk_group(object, "$", None, &mut Vec::new(), &mut tokens, true)?;
    if tokens.is_empty() {
        return Err(Error::InvalidStructure {
            path: "$".to_owned(),
            reason: "theme contains no tokens".to_owned(),
        });
    }
    let variant = theme_variant(&source.id);
    Ok(Theme {
        id: source.id.clone(),
        variant,
        tokens,
    })
}

fn marker_path(message: &str, marker: &str) -> Option<String> {
    let start = message.find(marker)? + marker.len();
    let remainder = &message[start..];
    let end = remainder.find(" at line ").unwrap_or(remainder.len());
    Some(remainder[..end].to_owned())
}

fn walk_group(
    object: &BTreeMap<String, JsonValue>,
    json_path: &str,
    inherited: Option<TokenType>,
    segments: &mut Vec<String>,
    tokens: &mut BTreeMap<String, Token>,
    root: bool,
) -> Result<(), Error> {
    reject_feature_keys(object, json_path)?;
    validate_metadata(object, json_path, root)?;
    let group_type = parse_optional_type(object.get("$type"), json_path)?.or(inherited);
    for (key, value) in object {
        if key.starts_with('$') {
            continue;
        }
        validate_name(key, json_path)?;
        let child = as_object(value, &format!("{json_path}.{key}"))?;
        let child_path = format!("{json_path}.{key}");
        segments.push(key.clone());
        let logical_path = segments.join(".");
        if logical_path.len() > MAX_PATH_BYTES {
            return Err(Error::PathLimit { path: logical_path });
        }
        if child.contains_key("$value") {
            if child.keys().any(|entry| !entry.starts_with('$')) {
                return Err(Error::InvalidStructure {
                    path: child_path,
                    reason: "token cannot contain child tokens or groups".to_owned(),
                });
            }
            validate_metadata(child, &child_path, false)?;
            reject_feature_keys(child, &child_path)?;
            let kind = parse_optional_type(child.get("$type"), &child_path)?
                .or(group_type)
                .ok_or_else(|| Error::MissingType {
                    path: logical_path.clone(),
                })?;
            let raw = child.get("$value").expect("checked above");
            let value = parse_token_value(raw, kind, &logical_path)?;
            tokens.insert(logical_path, Token { kind, value });
            if tokens.len() > MAX_TOKENS {
                return Err(Error::TokenLimit);
            }
        } else {
            walk_group(child, &child_path, group_type, segments, tokens, false)?;
        }
        segments.pop();
    }
    Ok(())
}

fn validate_metadata(
    object: &BTreeMap<String, JsonValue>,
    path: &str,
    root: bool,
) -> Result<(), Error> {
    for (key, value) in object {
        match key.as_str() {
            "$schema" if root => {
                if !matches!(value, JsonValue::String(_)) {
                    return Err(Error::InvalidStructure {
                        path: path.to_owned(),
                        reason: "$schema must be a string".to_owned(),
                    });
                }
            }
            "$schema" => {
                return Err(Error::UnknownProperty {
                    path: path.to_owned(),
                    property: key.clone(),
                });
            }
            "$type" | "$value" => {}
            "$description" if matches!(value, JsonValue::String(_)) => {}
            "$deprecated" if matches!(value, JsonValue::Bool | JsonValue::String(_)) => {}
            "$extensions" if matches!(value, JsonValue::Object(_)) => {}
            property if property.starts_with('$') => {
                return Err(Error::UnknownProperty {
                    path: path.to_owned(),
                    property: property.to_owned(),
                });
            }
            _ => {}
        }
    }
    Ok(())
}

fn reject_feature_keys(object: &BTreeMap<String, JsonValue>, path: &str) -> Result<(), Error> {
    for feature in ["$root", "$extends"] {
        if object.contains_key(feature) {
            return Err(Error::UnsupportedFeature {
                path: path.to_owned(),
                feature: feature.to_owned(),
            });
        }
    }
    Ok(())
}

fn parse_optional_type(value: Option<&JsonValue>, path: &str) -> Result<Option<TokenType>, Error> {
    match value {
        None => Ok(None),
        Some(JsonValue::String(kind)) => TokenType::parse(kind, path).map(Some),
        Some(_) => Err(Error::InvalidStructure {
            path: path.to_owned(),
            reason: "$type must be a string".to_owned(),
        }),
    }
}

fn parse_token_value(value: &JsonValue, kind: TokenType, path: &str) -> Result<TokenValue, Error> {
    if matches!(value, JsonValue::String(text) if text.starts_with('{') && text.ends_with('}')) {
        return Err(Error::UnsupportedFeature {
            path: path.to_owned(),
            feature: "alias".to_owned(),
        });
    }
    match kind {
        TokenType::Color => parse_color(value, path),
        TokenType::Dimension => parse_unit_scalar(value, path, kind, "px", 1.0),
        TokenType::Duration => parse_duration(value, path),
        TokenType::CubicBezier => parse_bezier(value, path),
    }
}

fn parse_color(value: &JsonValue, path: &str) -> Result<TokenValue, Error> {
    let object = exact_object(value, path, &["colorSpace", "components"], &["alpha"])?;
    if !matches!(object.get("colorSpace"), Some(JsonValue::String(space)) if space == "srgb") {
        return invalid_value(path, "color", "colorSpace must be `srgb`");
    }
    let components = match object.get("components") {
        Some(JsonValue::Array(values)) if values.len() == 3 => values,
        _ => return invalid_value(path, "color", "components must be an array of length 3"),
    };
    let mut rgba = [0_u8; 4];
    for (index, raw) in components.iter().enumerate() {
        rgba[index] = quantize_color(number(raw, path, "color component")?, path)?;
    }
    rgba[3] = match object.get("alpha") {
        Some(raw) => quantize_color(number(raw, path, "alpha")?, path)?,
        None => 255,
    };
    Ok(TokenValue::Color(rgba))
}

fn quantize_color(value: f64, path: &str) -> Result<u8, Error> {
    if !(0.0..=1.0).contains(&value) {
        return invalid_value(path, "color", "component must be within 0..=1");
    }
    Ok((value * 255.0).round() as u8)
}

fn parse_unit_scalar(
    value: &JsonValue,
    path: &str,
    kind: TokenType,
    unit: &str,
    multiplier: f64,
) -> Result<TokenValue, Error> {
    let object = exact_object(value, path, &["value", "unit"], &[])?;
    if !matches!(object.get("unit"), Some(JsonValue::String(actual)) if actual == unit) {
        return invalid_value(path, kind.as_str(), "unit is unsupported");
    }
    let scalar = number(
        object.get("value").expect("required key checked"),
        path,
        kind.as_str(),
    )?;
    scalar_to_value(scalar * multiplier, path, kind)
}

fn parse_duration(value: &JsonValue, path: &str) -> Result<TokenValue, Error> {
    let object = exact_object(value, path, &["value", "unit"], &[])?;
    let multiplier = match object.get("unit") {
        Some(JsonValue::String(unit)) if unit == "ms" => 1.0,
        Some(JsonValue::String(unit)) if unit == "s" => 1000.0,
        _ => return invalid_value(path, "duration", "unit must be `ms` or `s`"),
    };
    let scalar = number(
        object.get("value").expect("required key checked"),
        path,
        "duration",
    )?;
    scalar_to_value(scalar * multiplier, path, TokenType::Duration)
}

fn scalar_to_value(value: f64, path: &str, kind: TokenType) -> Result<TokenValue, Error> {
    if !value.is_finite() || value < 0.0 {
        return invalid_value(path, kind.as_str(), "value must be finite and nonnegative");
    }
    let narrowed = value as f32;
    if !narrowed.is_finite() {
        return invalid_value(path, kind.as_str(), "value is not finite as f32");
    }
    Ok(TokenValue::Scalar(narrowed))
}

fn parse_bezier(value: &JsonValue, path: &str) -> Result<TokenValue, Error> {
    let values = match value {
        JsonValue::Array(values) if values.len() == 4 => values,
        _ => return invalid_value(path, "cubicBezier", "value must be an array of length 4"),
    };
    let mut result = [0.0_f32; 4];
    for (index, raw) in values.iter().enumerate() {
        let value = number(raw, path, "cubicBezier component")?;
        if matches!(index, 0 | 2) && !(0.0..=1.0).contains(&value) {
            return invalid_value(path, "cubicBezier", "x coordinates must be within 0..=1");
        }
        let narrowed = value as f32;
        if !narrowed.is_finite() {
            return invalid_value(path, "cubicBezier", "component is not finite as f32");
        }
        result[index] = narrowed;
    }
    Ok(TokenValue::Bezier(result))
}

fn exact_object<'a>(
    value: &'a JsonValue,
    path: &str,
    required: &[&str],
    optional: &[&str],
) -> Result<&'a BTreeMap<String, JsonValue>, Error> {
    let object = as_object(value, path)?;
    for key in required {
        if !object.contains_key(*key) {
            return invalid_value(path, "object", &format!("missing key `{key}`"));
        }
    }
    let allowed: BTreeSet<&str> = required.iter().chain(optional.iter()).copied().collect();
    if let Some(key) = object.keys().find(|key| !allowed.contains(key.as_str())) {
        return invalid_value(path, "object", &format!("unexpected key `{key}`"));
    }
    Ok(object)
}

fn number(value: &JsonValue, path: &str, kind: &str) -> Result<f64, Error> {
    let JsonValue::Number(number) = value else {
        return invalid_value(path, kind, "expected JSON number");
    };
    let value = number.as_f64().ok_or_else(|| Error::InvalidValue {
        path: path.to_owned(),
        kind: kind.to_owned(),
        reason: "number is not representable as f64".to_owned(),
    })?;
    if !value.is_finite() {
        return invalid_value(path, kind, "number must be finite");
    }
    Ok(value)
}

fn invalid_value<T>(path: &str, kind: &str, reason: &str) -> Result<T, Error> {
    Err(Error::InvalidValue {
        path: path.to_owned(),
        kind: kind.to_owned(),
        reason: reason.to_owned(),
    })
}

fn as_object<'a>(
    value: &'a JsonValue,
    path: &str,
) -> Result<&'a BTreeMap<String, JsonValue>, Error> {
    match value {
        JsonValue::Object(object) => Ok(object),
        _ => Err(Error::InvalidStructure {
            path: path.to_owned(),
            reason: "expected object".to_owned(),
        }),
    }
}

fn validate_name(name: &str, path: &str) -> Result<(), Error> {
    let valid = !name.is_empty()
        && name.len() <= MAX_SEGMENT_BYTES
        && name.is_ascii()
        && name.as_bytes()[0].is_ascii_lowercase()
        && name.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        })
        && !name.ends_with(['-', '_'])
        && !name.contains("--")
        && !name.contains("__")
        && !name.contains("-_")
        && !name.contains("_-");
    if valid {
        Ok(())
    } else {
        Err(Error::InvalidName {
            path: path.to_owned(),
            name: name.to_owned(),
        })
    }
}

fn validate_theme_parity(themes: &[Theme]) -> Result<(), Error> {
    let reference = &themes[0];
    let mut differences = Vec::new();
    for theme in &themes[1..] {
        let paths: BTreeSet<_> = reference.tokens.keys().chain(theme.tokens.keys()).collect();
        for path in paths {
            match (reference.tokens.get(path), theme.tokens.get(path)) {
                (Some(left), Some(right)) if left.kind != right.kind => differences.push(format!(
                    "{}:{path}:type:{}!={}",
                    theme.id,
                    left.kind.as_str(),
                    right.kind.as_str()
                )),
                (Some(_), None) => differences.push(format!("{}:{path}:missing", theme.id)),
                (None, Some(_)) => differences.push(format!("{}:{path}:extra", theme.id)),
                _ => {}
            }
        }
    }
    if differences.is_empty() {
        Ok(())
    } else {
        Err(Error::ThemeMismatch { differences })
    }
}

fn validate_identifiers(themes: &[Theme]) -> Result<(), Error> {
    let mut variants: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for theme in themes {
        if rust_keywords().contains(theme.variant.as_str()) {
            return Err(Error::VariantKeyword {
                id: theme.id.clone(),
                variant: theme.variant.clone(),
            });
        }
        variants.entry(&theme.variant).or_default().push(&theme.id);
    }
    if let Some((variant, ids)) = variants.iter().find(|(_, ids)| ids.len() > 1) {
        return Err(Error::VariantCollision {
            variant: (*variant).to_owned(),
            ids: ids.iter().map(|id| (*id).to_owned()).collect(),
        });
    }

    let mut fields: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for path in themes[0].tokens.keys() {
        let field = token_field(path);
        if rust_keywords().contains(field.as_str()) {
            return Err(Error::FieldKeyword {
                path: path.clone(),
                field,
            });
        }
        fields.entry(field).or_default().push(path.clone());
    }
    if let Some((field, paths)) = fields.iter().find(|(_, paths)| paths.len() > 1) {
        return Err(Error::FieldCollision {
            field: field.clone(),
            paths: paths.clone(),
        });
    }
    Ok(())
}

fn rust_keywords() -> BTreeSet<&'static str> {
    [
        "Self", "abstract", "as", "async", "await", "become", "box", "break", "const", "continue",
        "crate", "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if",
        "impl", "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv",
        "pub", "ref", "return", "self", "static", "struct", "super", "trait", "true", "try",
        "type", "typeof", "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
    ]
    .into_iter()
    .collect()
}

fn token_field(path: &str) -> String {
    path.split('.')
        .map(|segment| segment.replace('-', "_"))
        .collect::<Vec<_>>()
        .join("__")
}

fn theme_variant(id: &str) -> String {
    id.split(['-', '_'])
        .map(|word| {
            let mut bytes = word.as_bytes().to_vec();
            bytes[0] = bytes[0].to_ascii_uppercase();
            String::from_utf8(bytes).expect("validated ASCII")
        })
        .collect()
}

fn bundle_hash(sources: &[ThemeSource]) -> String {
    let mut digest = Sha256::new();
    for source in sources {
        digest.update((source.id.len() as u64).to_be_bytes());
        digest.update(source.id.as_bytes());
        digest.update((source.bytes.len() as u64).to_be_bytes());
        digest.update(&source.bytes);
    }
    format!("{:x}", digest.finalize())
}

fn render_rust(themes: &[Theme], hash: &str) -> String {
    let mut output = format!(
        "// @generated by {GENERATOR_ID} v{GENERATOR_VERSION}; DO NOT EDIT.\n// input-sha256: {hash}\n\n"
    );
    output.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum GeneratedThemeId {\n");
    for theme in themes {
        output.push_str(&format!("    {},\n", theme.variant));
    }
    output.push_str("}\n\n#[derive(Debug, Clone, Copy, PartialEq)]\npub struct GeneratedTheme {\n");
    for (path, token) in &themes[0].tokens {
        output.push_str(&format!(
            "    pub {}: {},\n",
            token_field(path),
            token.kind.rust_type()
        ));
    }
    output.push_str(
        "}\n\npub fn generated_theme(id: GeneratedThemeId) -> GeneratedTheme {\n    match id {\n",
    );
    for theme in themes {
        output.push_str(&format!(
            "        GeneratedThemeId::{} => GeneratedTheme {{\n",
            theme.variant
        ));
        for (path, token) in &theme.tokens {
            output.push_str(&format!(
                "            {}: {},\n",
                token_field(path),
                render_value(&token.value)
            ));
        }
        output.push_str("        },\n");
    }
    output.push_str("    }\n}\n");
    output
}

fn render_value(value: &TokenValue) -> String {
    match value {
        TokenValue::Color([r, g, b, a]) => {
            format!("egui::Color32::from_rgba_unmultiplied({r}, {g}, {b}, {a})")
        }
        TokenValue::Scalar(value) => format!("f32::from_bits(0x{:08x})", value.to_bits()),
        TokenValue::Bezier(values) => format!(
            "[{}, {}, {}, {}]",
            render_float(values[0]),
            render_float(values[1]),
            render_float(values[2]),
            render_float(values[3])
        ),
    }
}

fn render_float(value: f32) -> String {
    format!("f32::from_bits(0x{:08x})", value.to_bits())
}

fn inspect_output_dir(out_dir: &Path, require_all: bool) -> Result<(), Error> {
    if !out_dir.exists() {
        if require_all {
            return Err(Error::MissingOutput {
                path: out_dir.to_owned(),
            });
        }
        return Ok(());
    }
    let entries = fs::read_dir(out_dir).map_err(|source| Error::Io {
        path: out_dir.to_owned(),
        source,
    })?;
    let mut found = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|source| Error::Io {
            path: out_dir.to_owned(),
            source,
        })?;
        let name = entry.file_name();
        let name_text = name.to_string_lossy();
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        if !OUTPUTS.contains(&name_text.as_ref()) || !file_type.is_file() {
            return Err(Error::UnexpectedOutputEntry { path });
        }
        found.insert(name_text.into_owned());
    }
    if require_all {
        for output in OUTPUTS {
            if !found.contains(output) {
                return Err(Error::MissingOutput {
                    path: out_dir.join(output),
                });
            }
        }
    }
    Ok(())
}

fn write_output(path: PathBuf, bytes: &[u8]) -> Result<(), Error> {
    fs::write(&path, bytes).map_err(|source| Error::Io { path, source })
}

fn check_output(path: PathBuf, expected: &[u8]) -> Result<(), Error> {
    let observed = fs::read(&path).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            Error::MissingOutput { path: path.clone() }
        } else {
            Error::Io {
                path: path.clone(),
                source,
            }
        }
    })?;
    if observed == expected {
        Ok(())
    } else {
        Err(Error::Drift { path })
    }
}

#[cfg(test)]
mod tests;
