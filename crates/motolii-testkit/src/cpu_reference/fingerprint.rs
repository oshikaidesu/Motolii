//! 正準source fingerprint(`motolii-source-v1`、M2 spec)の独立参照実装。
//!
//! 実装側は`motolii-doc`の`SourceFingerprintV1`が持つが、受け入れテストは
//! ここの**別経路(sha2直叩き)**で期待文字列を組んで照合する。

use std::path::Path;

use sha2::{Digest, Sha256};

/// バイト列から正準content_hash文字列を組む。
pub fn expected_source_content_hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hash = String::from("motolii-source-v1:sha256:");
    for byte in digest {
        hash.push_str(&format!("{byte:02x}"));
    }
    hash
}

/// ファイルを読んで[`expected_source_content_hash`]を適用する便宜口。
pub fn expected_source_content_hash_of_file(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    expected_source_content_hash(&bytes)
}
