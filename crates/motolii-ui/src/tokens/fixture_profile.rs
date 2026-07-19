use std::collections::BTreeMap;

use serde::Deserialize;

use super::error::ProfileError;
use super::schema::{DtcgColorValue, DtcgDimensionValue};
use super::TokenError;

/// U0e-1 機構証明用の synthetic parser profile。製品 DTCG 型閉集合ではない。
pub struct MechanismFixtureProfile;

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedToken {
    Color { r: u8, g: u8, b: u8, a: f32 },
    Dimension(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedMechanismTokens {
    pub tokens: BTreeMap<String, ResolvedToken>,
}

#[derive(Debug, Deserialize)]
struct MechanismFixtureRoot {
    #[serde(rename = "$schema")]
    schema: String,
    #[serde(rename = "$description", default)]
    description: Option<String>,
    #[serde(flatten)]
    nodes: BTreeMap<String, FixtureNode>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FixtureNode {
    Token(FixtureToken),
    Group(BTreeMap<String, FixtureNode>),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "$type", content = "$value")]
enum FixtureToken {
    #[serde(rename = "color")]
    Color(DtcgColorValue),
    #[serde(rename = "dimension")]
    Dimension(DtcgDimensionValue),
}

impl MechanismFixtureProfile {
    pub(crate) fn parse_str(raw: &str) -> Result<ResolvedMechanismTokens, TokenError> {
        let root: MechanismFixtureRoot = serde_json::from_str(raw)
            .map_err(|source| TokenError::Profile(ProfileError::Parse { source }))?;
        Self::resolve(root)
    }

    fn resolve(root: MechanismFixtureRoot) -> Result<ResolvedMechanismTokens, TokenError> {
        let _ = root.description;
        if !root.schema.contains("2025.10") {
            return Err(TokenError::Profile(ProfileError::Semantic(format!(
                "unexpected $schema (expected 2025.10 subset): {}",
                root.schema
            ))));
        }

        let mut tokens = BTreeMap::new();
        resolve_entries(root.nodes, String::new(), &mut tokens)?;
        if tokens.is_empty() {
            return Err(TokenError::Profile(ProfileError::Semantic(
                "no resolvable tokens".into(),
            )));
        }
        Ok(ResolvedMechanismTokens { tokens })
    }
}

fn resolve_entries(
    entries: BTreeMap<String, FixtureNode>,
    prefix: String,
    out: &mut BTreeMap<String, ResolvedToken>,
) -> Result<(), TokenError> {
    for (key, entry) in entries {
        let path = if prefix.is_empty() {
            key
        } else {
            format!("{prefix}.{key}")
        };
        match entry {
            FixtureNode::Token(token) => {
                out.insert(path.clone(), resolve_token(token, &path)?);
            }
            FixtureNode::Group(children) => {
                resolve_entries(children, path, out)?;
            }
        }
    }
    Ok(())
}

fn resolve_token(token: FixtureToken, path: &str) -> Result<ResolvedToken, TokenError> {
    match token {
        FixtureToken::Color(cv) => {
            if cv.color_space != "srgb" {
                return Err(TokenError::Profile(ProfileError::Semantic(format!(
                    "{path}: mechanism profile accepts srgb only, got {}",
                    cv.color_space
                ))));
            }
            let r = (cv.components[0].clamp(0.0, 1.0) * 255.0).round() as u8;
            let g = (cv.components[1].clamp(0.0, 1.0) * 255.0).round() as u8;
            let b = (cv.components[2].clamp(0.0, 1.0) * 255.0).round() as u8;
            Ok(ResolvedToken::Color {
                r,
                g,
                b,
                a: cv.alpha.clamp(0.0, 1.0),
            })
        }
        FixtureToken::Dimension(dv) => {
            if dv.unit != "px" {
                return Err(TokenError::Profile(ProfileError::Semantic(format!(
                    "{path}: mechanism profile accepts px only, got {}",
                    dv.unit
                ))));
            }
            Ok(ResolvedToken::Dimension(dv.value))
        }
    }
}
