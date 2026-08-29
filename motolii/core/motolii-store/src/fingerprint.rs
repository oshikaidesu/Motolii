//! 素材の指紋(sha256 + サイズ)。
//!
//! 旧 workspace `crates/motolii-doc/src/asset.rs:14-128` からの移植(2026-08-20 リセット)。
//! 再実装ではない。`Document` が「どのファイルを指しているか」を、パスではなく
//! 内容で言えるようにするための最小の意味。

use std::io::{self, Read};

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFingerprintV1 {
    digest: [u8; 32],
    size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceFingerprintDecode {
    V1(SourceFingerprintV1),
    MalformedV1Sha256,
    MissingSize,
    UnknownAlgorithm,
    LegacyOpaque,
}

#[derive(Debug, thiserror::Error)]
pub enum SourceFingerprintError {
    #[error("failed reading source for fingerprint: {source}")]
    Io {
        #[from]
        source: io::Error,
    },
    #[error("byte count overflowed u64 during fingerprinting")]
    ByteCountOverflow,
}

impl SourceFingerprintV1 {
    pub fn from_reader<R: Read>(mut reader: R) -> Result<Self, SourceFingerprintError> {
        let mut hasher = Sha256::new();
        let mut total = 0u64;
        let mut buffer = [0u8; 8192];

        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
            total = total
                .checked_add(count as u64)
                .ok_or(SourceFingerprintError::ByteCountOverflow)?;
        }

        let digest = hasher.finalize().into();

        Ok(Self {
            digest,
            size_bytes: total,
        })
    }

    pub fn decode_persisted(
        content_hash: &str,
        size_bytes: Option<u64>,
    ) -> SourceFingerprintDecode {
        const PREFIX: &str = "motolii-source-v1:";
        const SHA256_PREFIX: &str = "motolii-source-v1:sha256:";

        if !content_hash.starts_with(PREFIX) {
            return SourceFingerprintDecode::LegacyOpaque;
        }

        if !content_hash.starts_with(SHA256_PREFIX) {
            return SourceFingerprintDecode::UnknownAlgorithm;
        }

        let payload = &content_hash[SHA256_PREFIX.len()..];
        let Some(digest) = decode_lower_hex_64(payload) else {
            return SourceFingerprintDecode::MalformedV1Sha256;
        };

        match size_bytes {
            Some(size_bytes) => SourceFingerprintDecode::V1(Self { digest, size_bytes }),
            None => SourceFingerprintDecode::MissingSize,
        }
    }

    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    pub fn content_hash(&self) -> String {
        use std::fmt::Write as _;
        let mut hash = String::with_capacity(89);
        hash.push_str("motolii-source-v1:sha256:");
        for byte in &self.digest {
            write!(&mut hash, "{byte:02x}").expect("writing to String cannot fail");
        }
        hash
    }
}

fn decode_lower_hex_64(hex: &str) -> Option<[u8; 32]> {
    if hex.len() != 64 {
        return None;
    }

    let mut bytes = [0u8; 32];
    let bytes_in = hex.as_bytes();

    fn nibble(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            _ => None,
        }
    }

    for i in 0..32 {
        let hi = nibble(bytes_in[2 * i])?;
        let lo = nibble(bytes_in[2 * i + 1])?;
        bytes[i] = (hi << 4) | lo;
    }

    Some(bytes)
}
