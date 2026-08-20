//! composition 水準の Slots 表 — テンプレートの差し替え口。
//!
//! `slot` 発注単位(4行)。`composition/animation/slots`(表そのもの)・
//! `helpers/slot p`(スロットが差し込む値)・`helpers/slottable-object sid`
//! (スロット参照の識別子)・`properties/property sid`(property 側の参照口)を
//! まとめて1つの機構で閉じる。
//!
//! **第二の差し替え機構を作らない**(地図の note)。text-1 の切片が
//! `TextDocument::slot_id` として先に建てた「slots と同じ口に乗せる参照識別子」を
//! そのまま実体化しているだけで、text 用に別のスロット表は持たない —
//! [`SlotId`] は `TextDocument::slot_id` と全く同じ型で、両方が同じ
//! [`Composition`](後述) 水準の表を指す。
//!
//! property 側も新しい component を増やさない。既存の `descriptor_track(property)`
//! (裁定92 の平坦 `PropertyId` → `TrackJson`)が持つ JSON の中身を
//! [`PropertySource::{Track,Slot}`](PropertySource) の2択へ広げるだけ
//! (`properties/property sid` の note「値/トラック か スロット参照 かの enum」)。
//! `#[serde(untagged)]` にしてあるので、`Track` 側の wire 形は今までの
//! `KeyframeTrack` の JSON とビット単位で同じ(オブジェクト `{"keys":[...]}`)。
//! `Slot` 側は `SlotId`(ニュータイプ `String`)が裸の JSON 文字列へ潰れるので、
//! 2つの形は構造的に衝突しない — 既存の保存済み track を1つも書き換えずに
//! この機構へ移行できる。

use serde::{Deserialize, Serialize};

use crate::StoreError;

/// スロットの識別子。**Lottie の `sid` は文字列**(`helpers/slottable-object sid` /
/// `composition/animation/slots` の辞書キー)であって、mask/effect の連番 id とは
/// 由来が違う — スロットは「利用者が名付けるテンプレート引数」なので人が読める名前が
/// 本体になる(Lottie 実物の "primary_color" のような id が典型)。
///
/// `TextDocument::slot_id` が最初に建てた `Option<String>` の口と**同じ型**にする
/// ためにニュータイプでラップしてあるだけで、シリアライズ形は裸の `String` と
/// ビット単位で同じ(newtype は透過的)。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SlotId(pub String);

impl std::fmt::Display for SlotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// `helpers/slot`。このスロットが差し込む値そのもの(`p` / Property Value)。
///
/// Lottie の `p` は静止値・keyframe のどちらも許す「Property」なので、Motolii では
/// 既存の `KeyframeTrack` をそのまま使う(裁定92 の平坦トラックと同じ形 — スロット専用の
/// 値表現を新しく作らない)。動かないスロットは1キーの Hold track で表す(他の静止
/// property と同じ規約、裁定20)。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Slot {
    pub id: SlotId,
    pub track: motolii_eval::KeyframeTrack,
}

/// property の値の出処。`properties/property sid` の note が指定した二択そのもの。
///
/// **`Track`**: 今までどおりこの property 自身の `KeyframeTrack`。
/// **`Slot`**: comp の [`Slot`] 表の該当行を指す参照 — 値はそこから引く(テンプレートの
/// 差し替え口。同じ `sid` を持つ複数の property が、comp の1箇所を直せば揃って変わる)。
///
/// `#[serde(untagged)]` の理由はモジュール doc 参照。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertySource {
    Track(motolii_eval::KeyframeTrack),
    Slot(SlotId),
}

/// 同じ id のスロットが2枚あると、`PropertySource::Slot` がどちらを指しているか
/// 決まらない(mask/effect と同型の検査、`Intent::SetSlots` 1本が唯一の書き口)。
pub(crate) fn validate_unique_ids(slots: &[Slot]) -> Result<(), StoreError> {
    for (i, slot) in slots.iter().enumerate() {
        if slots[..i].iter().any(|other| other.id == slot.id) {
            return Err(StoreError::Property(format!(
                "スロット id \"{}\" が2枚ある",
                slot.id
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_core::RationalTime;
    use motolii_eval::{Interp, Keyframe, Value};

    fn hold(value: Value) -> motolii_eval::KeyframeTrack {
        let mut track = motolii_eval::KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value,
            interp: Interp::Hold,
            spatial: None,
        });
        track
    }

    /// `Track` 側の wire 形が裸の `KeyframeTrack` と同じであることの固定
    /// (移行コストゼロの根拠 — 既存の保存済み track JSON がそのまま
    /// `PropertySource::Track` として読める)。
    #[test]
    fn property_source_track_serializes_identically_to_a_bare_keyframe_track() {
        let track = hold(Value::F64(1.0));
        let bare = serde_json::to_string(&track).unwrap();
        let wrapped = serde_json::to_string(&PropertySource::Track(track)).unwrap();
        assert_eq!(
            bare, wrapped,
            "untagged Track が KeyframeTrack と別の形になっている"
        );
    }

    /// `SlotId` の wire 形が裸の `String` と同じであることの固定
    /// (`TextDocument::slot_id` の `Option<String>` と同じ口に乗ることの根拠)。
    #[test]
    fn slot_id_serializes_identically_to_a_bare_string() {
        let id = SlotId("primary_color".to_owned());
        assert_eq!(serde_json::to_string(&id).unwrap(), "\"primary_color\"");
    }

    #[test]
    fn duplicate_slot_ids_are_rejected() {
        let slots = vec![
            Slot {
                id: SlotId("a".to_owned()),
                track: hold(Value::F64(1.0)),
            },
            Slot {
                id: SlotId("a".to_owned()),
                track: hold(Value::F64(2.0)),
            },
        ];
        assert!(validate_unique_ids(&slots).is_err());
    }

    #[test]
    fn distinct_slot_ids_are_accepted() {
        let slots = vec![
            Slot {
                id: SlotId("a".to_owned()),
                track: hold(Value::F64(1.0)),
            },
            Slot {
                id: SlotId("b".to_owned()),
                track: hold(Value::F64(2.0)),
            },
        ];
        assert!(validate_unique_ids(&slots).is_ok());
    }
}
