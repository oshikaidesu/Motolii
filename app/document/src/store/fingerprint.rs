
use std::io::{self, Read};

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFingerprintV1 {
    digest: [u8; 32],
    size_bytes: u64,
    algorithm: Algorithm,
}

/// What the digest covers. The edges form reads a megabyte from each end and
/// the size, so admitting a 4 GB clip costs the same as a 4 KB one; the full
/// form reads everything and is the stricter identity when it is wanted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Sha256Full,
    Sha256Edges,
}

/// How much of each end the edges form reads.
const EDGE_BYTES: u64 = 1 << 20;

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
            algorithm: Algorithm::Sha256Full,
        })
    }

    /// The quick identity: size, the first megabyte and the last, so a file
    /// is admitted as fast as it is opened. Files shorter than two megabytes
    /// are read whole, so the two forms agree on small files' bytes.
    pub fn from_edges(path: impl AsRef<std::path::Path>) -> Result<Self, SourceFingerprintError> {
        use std::io::{Seek, SeekFrom};
        let mut file = std::fs::File::open(path)?;
        let size = file.metadata()?.len();
        let mut hasher = Sha256::new();
        hasher.update(size.to_le_bytes());
        let mut head = Vec::new();
        (&mut file).take(EDGE_BYTES).read_to_end(&mut head)?;
        hasher.update(&head);
        if size > EDGE_BYTES * 2 {
            let mut tail = Vec::new();
            file.seek(SeekFrom::Start(size - EDGE_BYTES))?;
            file.read_to_end(&mut tail)?;
            hasher.update(&tail);
        } else if size > EDGE_BYTES {
            let mut rest = Vec::new();
            file.read_to_end(&mut rest)?;
            hasher.update(&rest);
        }
        Ok(Self {
            digest: hasher.finalize().into(),
            size_bytes: size,
            algorithm: Algorithm::Sha256Edges,
        })
    }

    pub fn decode_persisted(
        content_hash: &str,
        size_bytes: Option<u64>,
    ) -> SourceFingerprintDecode {
        const PREFIX: &str = "motolii-source-v1:";
        const SHA256_PREFIX: &str = "motolii-source-v1:sha256:";
        const EDGES_PREFIX: &str = "motolii-source-v1:sha256-edges:";

        if !content_hash.starts_with(PREFIX) {
            return SourceFingerprintDecode::LegacyOpaque;
        }

        let (payload, algorithm) = if let Some(rest) = content_hash.strip_prefix(EDGES_PREFIX) {
            (rest, Algorithm::Sha256Edges)
        } else if let Some(rest) = content_hash.strip_prefix(SHA256_PREFIX) {
            (rest, Algorithm::Sha256Full)
        } else {
            return SourceFingerprintDecode::UnknownAlgorithm;
        };

        let Some(digest) = decode_lower_hex_64(payload) else {
            return SourceFingerprintDecode::MalformedV1Sha256;
        };

        match size_bytes {
            Some(size_bytes) => SourceFingerprintDecode::V1(Self {
                digest,
                size_bytes,
                algorithm,
            }),
            None => SourceFingerprintDecode::MissingSize,
        }
    }

    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
    }

    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    pub fn content_hash(&self) -> String {
        use std::fmt::Write as _;
        let mut hash = String::with_capacity(96);
        hash.push_str(match self.algorithm {
            Algorithm::Sha256Full => "motolii-source-v1:sha256:",
            Algorithm::Sha256Edges => "motolii-source-v1:sha256-edges:",
        });
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

#[cfg(test)]
mod edges_tests {
    use super::*;

    fn temp(bytes: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "motolii-fp-{}-{}",
            std::process::id(),
            bytes.len()
        ));
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn edges_read_only_the_ends_of_a_big_file() {
        let mut big = vec![7u8; (EDGE_BYTES * 3) as usize];
        let a = SourceFingerprintV1::from_edges(temp(&big)).unwrap();
        big[(EDGE_BYTES + 100) as usize] = 9;
        let b = SourceFingerprintV1::from_edges(temp(&big)).unwrap();
        assert_eq!(a, b, "a change in the middle is not an edge");
        big[10] = 9;
        let c = SourceFingerprintV1::from_edges(temp(&big)).unwrap();
        assert_ne!(a, c, "a change in the head is");
        assert!(a.content_hash().starts_with("motolii-source-v1:sha256-edges:"));
    }

    #[test]
    fn the_edges_hash_round_trips_through_its_text() {
        let fp = SourceFingerprintV1::from_edges(temp(b"small file")).unwrap();
        let back = SourceFingerprintV1::decode_persisted(&fp.content_hash(), Some(fp.size_bytes()));
        assert_eq!(back, SourceFingerprintDecode::V1(fp));
    }
}
