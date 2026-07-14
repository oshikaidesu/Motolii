//! Effect/Keyframe/Definition等のdocument-local安定u64 ID(A8 / D2 / D1l)。
//!
//! LayerId/TrackIdと同じ「不変・非再利用」規律。EffectUse・EffectDefinition・Keyframeは
//! 1つの`next_stable_id`カウンタを共有し、型間の数値衝突を避ける。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum StableIdError {
    #[error("stable id sequence exhausted")]
    Exhausted,
    #[error("StableIdSeq.next ({next}) must be greater than observed max id ({max_id})")]
    InvalidNext { next: u64, max_id: u64 },
}

/// 非再利用の単調カウンタ。削除しても`next`は戻さない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StableIdSeq {
    next: u64,
}

impl StableIdSeq {
    pub const fn new() -> Self {
        Self { next: 0 }
    }

    /// 新規IDを発行する。呼び出しごとに`next`を進め、以後同じ値は出さない。
    pub fn allocate(&mut self) -> Result<u64, StableIdError> {
        let id = self.next;
        self.next = self.next.checked_add(1).ok_or(StableIdError::Exhausted)?;
        Ok(id)
    }

    pub const fn peek_next(self) -> u64 {
        self.next
    }

    /// ロード済みDocument内の実在IDと整合しているか(カウンタが実在最大値以下=破損)を検査する。
    pub fn validate_observed_max(self, max_observed: Option<u64>) -> Result<(), StableIdError> {
        if let Some(max_id) = max_observed {
            if self.next <= max_id {
                return Err(StableIdError::InvalidNext {
                    next: self.next,
                    max_id,
                });
            }
        }
        Ok(())
    }
}

impl Default for StableIdSeq {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! stable_id_newtype {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(u64);

        impl $name {
            pub const fn get(self) -> u64 {
                self.0
            }

            /// テスト/復元用。通常の採番は`DocumentWriter::allocate_*_id`経由。
            pub const fn from_raw(raw: u64) -> Self {
                Self(raw)
            }
        }
    };
}

stable_id_newtype!(
    EffectId,
    "EffectUseの恒久ID(A8 / D1l)。stack上のUse identity。旧EffectInstance.idからmigrationで引き継ぐ。"
);
stable_id_newtype!(
    EffectDefinitionId,
    "EffectDefinitionの恒久ID(D1l)。共有recipe identity。Useから参照される。"
);
stable_id_newtype!(
    KeyframeId,
    "DocKeyframeの恒久ID(A8)。時刻編集で不変、複製時は新規採番。"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_never_repeats() {
        let mut seq = StableIdSeq::new();
        let a = seq.allocate().unwrap();
        let b = seq.allocate().unwrap();
        assert_ne!(a, b);
        assert_eq!((a, b), (0, 1));
    }

    #[test]
    fn validate_observed_max_rejects_corrupt_counter() {
        let seq = StableIdSeq { next: 3 };
        assert!(seq.validate_observed_max(Some(2)).is_ok());
        assert_eq!(
            seq.validate_observed_max(Some(3)),
            Err(StableIdError::InvalidNext { next: 3, max_id: 3 })
        );
    }

    #[test]
    fn newtype_serde_roundtrip() {
        let id = EffectId::from_raw(7);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "7");
        let back: EffectId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }
}
