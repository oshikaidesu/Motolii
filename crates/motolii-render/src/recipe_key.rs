//! Host-private canonical RecipeKeyV1 / ArtifactDigest codec (M4-P02-CODEC).

use std::fmt::Write as _;

use sha2::{Digest, Sha256};
use thiserror::Error;

/// Domain separation prefix for the v1 recipe key encoding.
const RECIPE_KEY_DOMAIN_V1: &[u8] = b"motolii-recipe-key-v1\0";

const TAG_RECIPE_FORMAT_VERSION: u8 = 1;
const TAG_NODE_ID: u8 = 2;
const TAG_NODE_VERSION: u8 = 3;
const TAG_PARAMS: u8 = 4;
const TAG_INPUT_DIGESTS: u8 = 5;
const TAG_TIME: u8 = 6;
const TAG_QUALITY: u8 = 7;
const TAG_PLATFORM_SALT: u8 = 8;

/// SHA-256 of publish対象の actual bytes と、そのバイト長。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArtifactDigest {
    pub sha256: [u8; 32],
    pub size: u64,
}

impl ArtifactDigest {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Self {
            sha256: hasher.finalize().into(),
            size: bytes.len() as u64,
        }
    }

    pub fn canonical_string(&self) -> String {
        let mut out = String::with_capacity(27 + 64 + 1 + 20);
        out.push_str("motolii-artifact-v1:sha256:");
        for byte in &self.sha256 {
            write!(&mut out, "{byte:02x}").expect("writing to String cannot fail");
        }
        write!(&mut out, ":{}", self.size).expect("writing to String cannot fail");
        out
    }
}

/// Host-private recipe key: SHA-256 of the canonical tagged encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecipeKeyV1(pub [u8; 32]);

/// 正本順の RecipeKeyV1 入力。CacheEpoch / JobId / AssetId / path / mtime / 表示名は持たない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeKeyV1Input {
    pub recipe_format_version: u32,
    pub node_id: String,
    pub node_version: u32,
    pub params: Vec<(String, Vec<u8>)>,
    pub input_digests: Vec<ArtifactDigest>,
    pub time: (i64, i64),
    pub quality: u8,
    pub platform_salt: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RecipeKeyError {
    #[error("duplicate param id in RecipeKeyV1 params")]
    DuplicateParamId,
    #[error("invalid RecipeKeyV1 time denominator (must be > 0)")]
    InvalidTime,
}

impl RecipeKeyV1 {
    pub fn encode(input: &RecipeKeyV1Input) -> Result<Self, RecipeKeyError> {
        let (num, den) = input.time;
        if den <= 0 {
            return Err(RecipeKeyError::InvalidTime);
        }

        let params_payload = encode_params_payload(&input.params)?;
        let input_digests_payload = encode_input_digests_payload(&input.input_digests);
        let time_payload = {
            let mut buf = [0u8; 16];
            buf[..8].copy_from_slice(&num.to_le_bytes());
            buf[8..].copy_from_slice(&den.to_le_bytes());
            buf
        };

        let mut encoded = Vec::new();
        encoded.extend_from_slice(RECIPE_KEY_DOMAIN_V1);
        append_field(
            &mut encoded,
            TAG_RECIPE_FORMAT_VERSION,
            &input.recipe_format_version.to_le_bytes(),
        );
        append_field(&mut encoded, TAG_NODE_ID, input.node_id.as_bytes());
        append_field(
            &mut encoded,
            TAG_NODE_VERSION,
            &input.node_version.to_le_bytes(),
        );
        append_field(&mut encoded, TAG_PARAMS, &params_payload);
        append_field(&mut encoded, TAG_INPUT_DIGESTS, &input_digests_payload);
        append_field(&mut encoded, TAG_TIME, &time_payload);
        append_field(&mut encoded, TAG_QUALITY, &[input.quality]);
        append_field(&mut encoded, TAG_PLATFORM_SALT, &input.platform_salt);

        let mut hasher = Sha256::new();
        hasher.update(&encoded);
        Ok(Self(hasher.finalize().into()))
    }

    pub fn canonical_string(&self) -> String {
        let mut out = String::with_capacity(25 + 64);
        out.push_str("motolii-recipe-v1:sha256:");
        for byte in &self.0 {
            write!(&mut out, "{byte:02x}").expect("writing to String cannot fail");
        }
        out
    }
}

fn append_field(buf: &mut Vec<u8>, tag: u8, payload: &[u8]) {
    buf.push(tag);
    buf.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    buf.extend_from_slice(payload);
}

fn encode_params_payload(params: &[(String, Vec<u8>)]) -> Result<Vec<u8>, RecipeKeyError> {
    let mut sorted: Vec<(&str, &[u8])> = params
        .iter()
        .map(|(id, value)| (id.as_str(), value.as_slice()))
        .collect();
    sorted.sort_by(|a, b| a.0.cmp(b.0));
    for window in sorted.windows(2) {
        if window[0].0 == window[1].0 {
            return Err(RecipeKeyError::DuplicateParamId);
        }
    }

    let mut out = Vec::new();
    for (id, value) in sorted {
        let id_bytes = id.as_bytes();
        out.extend_from_slice(&(id_bytes.len() as u64).to_le_bytes());
        out.extend_from_slice(id_bytes);
        out.extend_from_slice(&(value.len() as u64).to_le_bytes());
        out.extend_from_slice(value);
    }
    Ok(out)
}

fn encode_input_digests_payload(digests: &[ArtifactDigest]) -> Vec<u8> {
    let mut out = Vec::with_capacity(digests.len() * 40);
    for digest in digests {
        out.extend_from_slice(&digest.sha256);
        out.extend_from_slice(&digest.size.to_le_bytes());
    }
    out
}
