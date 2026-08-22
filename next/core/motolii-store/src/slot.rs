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
//!
//! ## 2026-08-22: 型付き link で3択へ(`link` 発注単位、裁定206)
//!
//! `docs/reviews/2026-08-22-persona-touchdesigner-round2.md` §1 の設計案をそのまま
//! 実装した——TouchDesigner の CHOP Export(片方向・名前束縛・宛先だけが参照を持つ
//! 非対称性)と Blender の Driver(閉じた関数集合による変換)を先例に、
//! [`PropertySource::Link`] を第三の枝として足す。**発明ではなく既に稼働している
//! 機構の横展開**——2択を3択にするだけで新しい仕組みを増やさない、という本ファイル
//! 冒頭の法則をそのまま踏襲する。
//!
//! **裁定206**(`next/DECISIONS.md`)がこの拡張の出典問題を解いた: Lottie の
//! property-to-property 参照機構(`x`/Expression)は不採用のままだが、判定基準は
//! 「その機構の結果を Lottie へ無損失で書けるか」の一点——link は評価すると普通の
//! 値へ解決される(焼けば `KeyframeTrack` と区別がつかない)ので、フレームの意味は
//! 100% Lottie で表現可能。裁定191(描画語彙の正本)が縛るのは「1フレームが何を
//! 意味するか」であって、link のような編集機構(キーフレーム補間 UI や一括編集と
//! 同じ層)はその外にある——effect(裁定70/72: `ty` 不採用・`v` 採用)と同じ「vism圏の
//! 入れ物」に属する。
//!
//! v1 のスコープは閉じている(§1.2): 単一ソース→単一ターゲット・片方向のみ。
//! LookAt(2入力から1つを作る)は範囲外——`next/GOALS.md` が親子(型付き
//! Follow/LookAt)を link とは別に名指ししており、伸びる先として位置だけ示す。

use serde::{Deserialize, Serialize};

use crate::{LayerId, PropertyId, StoreError};

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

/// property の値の出処。`properties/property sid` の note が指定した二択に、
/// 2026-08-22(裁定206)で `Link` を足した三択。
///
/// **`Track`**: 今までどおりこの property 自身の `KeyframeTrack`。
/// **`Slot`**: comp の [`Slot`] 表の該当行を指す参照 — 値はそこから引く(テンプレートの
/// 差し替え口。同じ `sid` を持つ複数の property が、comp の1箇所を直せば揃って変わる)。
/// **`Link`**: 別 layer の別 property の値を読んで決める片方向参照(モジュール doc
/// 「2026-08-22」節参照)。
///
/// `#[serde(untagged)]` の理由はモジュール doc 参照——`Link` は3つ目のオブジェクト
/// 形(`source_layer`/`source_property`/`time_offset`/`plugin_id`/`params` の5
/// フィールド)なので、`Track` の `{"keys":[...]}` とも `Slot` の裸文字列とも構造的に
/// 衝突しない。旧 Document(`Track`/`Slot` のみ)は無改造で読める。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertySource {
    Track(motolii_eval::KeyframeTrack),
    Slot(SlotId),
    Link(PropertyLink),
}

/// **型付き link** 本体(`docs/reviews/2026-08-22-persona-touchdesigner-round2.md`
/// §1.3 の設計案そのもの)。property の値を、**別 property の値を読んで**決める
/// 片方向の参照。v1 のスコープは単一ソース→単一ターゲット・片方向のみ(モジュール
/// doc 参照、LookAt のような多入力は範囲外)。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertyLink {
    pub source_layer: LayerId,
    pub source_property: PropertyId,
    /// 評価時刻をずらす量。**Host が構造として知る必要がある**ので、`plugin_id` の
    /// ような不透明文字列に隠さない(scrub・cache key・依存解決に効くため)。
    pub time_offset: motolii_core::RationalTime,
    /// 値変換の閉じた集合を指す id。**`EffectInstance.plugin_id` と全く同じ扱い**
    /// (裁定70: Document は具体的な変換の中身を型で持たない)。[`translate_link`]
    /// が解決する v1 の閉じた集合は `"motolii.link.identity"` /
    /// `"motolii.link.linear"` / `"motolii.link.remap"` の3つだけ。
    pub plugin_id: String,
    /// 変換の係数(named param、`ResolvedEffect::params` と同じ形)。
    ///
    /// **v1 は静止値**——effect の param(`PropertyId::effect_param` が作る別 track)
    /// のように独立してキーフレームは打てない。link 自体が既に「時刻→値」を運ぶ
    /// property の一種なので、係数を時間で動かしたければ「別の(時間オフセット無しの)
    /// link を挟む」という既存語彙の組み合わせで表せる、というのが今回のスコープの
    /// 絞り方(器を通すのが目的——初版の `plugin_id` は最小限でよい、という発注の
    /// 指示どおり)。
    pub params: Vec<(String, motolii_eval::Value)>,
}

/// `plugin_id` が指す変換を適用する。**`translate_glow_params`
/// (`next/engine/motolii-engine/src/lib.rs:1281-1297`)と同型の閉じた match** ——
/// 名前つき param を `find` で読み、型不一致・不明な `plugin_id` は黙って `None`
/// を返す(EXACT TARGET #2 の規約と同じ精神: 近似しない。effect 側が「pass を1本も
/// 積まない」のに対し、link 側は「値を1つも返さない」——その property は
/// `value_at` から見て「値が無い」に潰れる、裁定20 の「キーを打っていない property」
/// と同じ扱い)。
///
/// v1 の閉じた集合(§1.2): `identity`(そのまま渡す、型を問わない)/ `linear`
/// (`scale`・`offset` の1次式、F64・Vec2)/ `remap`(区間→区間、`clamp` 任意、F64
/// のみ)。
pub(crate) fn translate_link(
    plugin_id: &str,
    params: &[(String, motolii_eval::Value)],
    value: motolii_eval::Value,
) -> Option<motolii_eval::Value> {
    use motolii_eval::Value;

    let find_f64 = |name: &str, default: f64| -> Option<f64> {
        match params.iter().find(|(param_name, _)| param_name == name) {
            Some((_, Value::F64(v))) => Some(*v),
            Some(_other_type) => None,
            None => Some(default),
        }
    };
    let find_bool = |name: &str, default: bool| -> Option<bool> {
        match params.iter().find(|(param_name, _)| param_name == name) {
            Some((_, Value::Bool(v))) => Some(*v),
            Some(_other_type) => None,
            None => Some(default),
        }
    };

    match plugin_id {
        "motolii.link.identity" => Some(value),
        "motolii.link.linear" => {
            let scale = find_f64("scale", 1.0)?;
            let offset = find_f64("offset", 0.0)?;
            match value {
                Value::F64(v) => Some(Value::F64(v * scale + offset)),
                Value::Vec2(v) => {
                    Some(Value::Vec2(std::array::from_fn(|i| v[i] * scale + offset)))
                }
                _ => None,
            }
        }
        "motolii.link.remap" => {
            let in_min = find_f64("in_min", 0.0)?;
            let in_max = find_f64("in_max", 1.0)?;
            let out_min = find_f64("out_min", 0.0)?;
            let out_max = find_f64("out_max", 1.0)?;
            let clamp = find_bool("clamp", false)?;
            let Value::F64(v) = value else {
                return None;
            };
            if in_max == in_min {
                return None; // 区間の長さ0は写像が定義できない。
            }
            let mut u = (v - in_min) / (in_max - in_min);
            if clamp {
                u = u.clamp(0.0, 1.0);
            }
            Some(Value::F64(out_min + u * (out_max - out_min)))
        }
        _ => None,
    }
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

    // -----------------------------------------------------------------
    // link(裁定206) — translate_link の閉じた match
    // -----------------------------------------------------------------

    fn link(source_property: PropertyId, plugin_id: &str, params: Vec<(String, Value)>) -> PropertyLink {
        PropertyLink {
            source_layer: LayerId(1),
            source_property,
            time_offset: RationalTime::ZERO,
            plugin_id: plugin_id.to_owned(),
            params,
        }
    }

    #[test]
    fn translate_link_identity_passes_value_through_unchanged() {
        assert_eq!(
            translate_link("motolii.link.identity", &[], Value::F64(3.5)),
            Some(Value::F64(3.5))
        );
        // 型を問わない — identity は「そのまま渡す」だけ。
        assert_eq!(
            translate_link("motolii.link.identity", &[], Value::Vec2([1.0, 2.0])),
            Some(Value::Vec2([1.0, 2.0]))
        );
    }

    #[test]
    fn translate_link_linear_applies_scale_and_offset() {
        let params = vec![
            ("scale".to_owned(), Value::F64(2.0)),
            ("offset".to_owned(), Value::F64(10.0)),
        ];
        assert_eq!(
            translate_link("motolii.link.linear", &params, Value::F64(5.0)),
            Some(Value::F64(20.0)),
            "5*2+10 = 20 のはず"
        );
    }

    /// param を1つも渡さないと `scale=1.0`/`offset=0.0`(= identity と同じ結果)
    /// になる——`translate_glow_params` の「track の無い param は既定値」と同じ規約。
    #[test]
    fn translate_link_linear_defaults_to_identity_when_params_are_absent() {
        assert_eq!(
            translate_link("motolii.link.linear", &[], Value::F64(7.0)),
            Some(Value::F64(7.0))
        );
    }

    #[test]
    fn translate_link_linear_applies_uniformly_to_vec2_components() {
        let params = vec![
            ("scale".to_owned(), Value::F64(0.5)),
            ("offset".to_owned(), Value::F64(1.0)),
        ];
        assert_eq!(
            translate_link("motolii.link.linear", &params, Value::Vec2([10.0, 20.0])),
            Some(Value::Vec2([6.0, 11.0]))
        );
    }

    #[test]
    fn translate_link_remap_maps_between_ranges() {
        let params = vec![
            ("in_min".to_owned(), Value::F64(0.0)),
            ("in_max".to_owned(), Value::F64(10.0)),
            ("out_min".to_owned(), Value::F64(0.0)),
            ("out_max".to_owned(), Value::F64(100.0)),
        ];
        assert_eq!(
            translate_link("motolii.link.remap", &params, Value::F64(5.0)),
            Some(Value::F64(50.0))
        );
    }

    /// `clamp=true` の時だけ範囲外の入力が端で止まる——既定(`clamp` 省略)は
    /// 範囲を超えて外挿する(AE の remap 系エフェクトと同じ既定)。
    #[test]
    fn translate_link_remap_clamps_only_when_requested() {
        let base = vec![
            ("in_min".to_owned(), Value::F64(0.0)),
            ("in_max".to_owned(), Value::F64(10.0)),
            ("out_min".to_owned(), Value::F64(0.0)),
            ("out_max".to_owned(), Value::F64(100.0)),
        ];
        assert_eq!(
            translate_link("motolii.link.remap", &base, Value::F64(20.0)),
            Some(Value::F64(200.0)),
            "clamp を渡さなければ外挿するはず"
        );

        let mut clamped = base;
        clamped.push(("clamp".to_owned(), Value::Bool(true)));
        assert_eq!(
            translate_link("motolii.link.remap", &clamped, Value::F64(20.0)),
            Some(Value::F64(100.0)),
            "clamp=true なら上限で止まるはず"
        );
    }

    /// **型不一致は近似しない**(EXACT TARGET #2 と同じ規約) — `remap` は F64 専用
    /// なので、Vec2 が来たら黙って `None`(この property は「値が無い」に潰れる)。
    #[test]
    fn translate_link_rejects_type_mismatch_instead_of_approximating() {
        let params = vec![("in_max".to_owned(), Value::F64(10.0))];
        assert_eq!(
            translate_link("motolii.link.remap", &params, Value::Vec2([1.0, 2.0])),
            None
        );
        // param 自体の型が違う場合も同様(F64 のはずが Bool)。
        let bad_params = vec![("scale".to_owned(), Value::Bool(true))];
        assert_eq!(
            translate_link("motolii.link.linear", &bad_params, Value::F64(1.0)),
            None
        );
    }

    /// 未知の `plugin_id` はパニックせず無音で `None`
    /// (`translate_effect_passes` の「無音で skip」と同じ fail-closed)。
    #[test]
    fn translate_link_returns_none_for_unknown_plugin_id() {
        assert_eq!(
            translate_link("motolii.link.does_not_exist", &[], Value::F64(1.0)),
            None
        );
    }

    // -----------------------------------------------------------------
    // link — 保存形式(移行コストゼロ)
    // -----------------------------------------------------------------

    /// `Link` は3つ目のオブジェクト形なので、`Track`(裸の `{"keys":[...]}`)とも
    /// `Slot`(裸の文字列)とも構造的に衝突しない——`#[serde(untagged)]` の3択で
    /// 元の型へ戻る。旧 Document(Track/Slot のみ)がこの変更で1つも壊れないことの
    /// 直接証拠。
    #[test]
    fn property_source_link_round_trips_without_colliding_with_track_or_slot() {
        let track = PropertySource::Track(hold(Value::F64(1.0)));
        let slot = PropertySource::Slot(SlotId("s".to_owned()));
        let property_link = PropertySource::Link(link(
            PropertyId::new("opacity").unwrap(),
            "motolii.link.identity",
            Vec::new(),
        ));

        for source in [track, slot, property_link] {
            let json = serde_json::to_string(&source).unwrap();
            let back: PropertySource = serde_json::from_str(&json).unwrap();
            assert_eq!(back, source, "untagged 3択の往復が一致しない: {json}");
        }
    }

    /// `PropertyId` は裸の文字列として符号化される(`SlotId` と同じ透過ニュータイプの
    /// 流儀) — `PropertyLink::source_property` がこの形で保存へ乗ることの固定。
    #[test]
    fn property_link_serializes_source_property_as_a_bare_string() {
        let l = link(
            PropertyId::new("rotation").unwrap(),
            "motolii.link.linear",
            vec![("scale".to_owned(), Value::F64(2.0))],
        );
        let json = serde_json::to_value(&l).unwrap();
        assert_eq!(
            json["source_property"], "rotation",
            "source_property が裸の文字列で符号化されていない: {json}"
        );

        let back: PropertyLink = serde_json::from_value(json).unwrap();
        assert_eq!(back, l);
    }

    /// 予約語(layer 自身の component 名)を `source_property` に持つ壊れた JSON は
    /// 復元時に拒む——`PropertyId::new` の柵がそのまま効くことの固定。
    #[test]
    fn a_reserved_name_in_source_property_fails_to_deserialize() {
        let json = r#"{
            "source_layer": 1,
            "source_property": "masks",
            "time_offset": {"num": 0, "den": 1},
            "plugin_id": "motolii.link.identity",
            "params": []
        }"#;
        assert!(
            serde_json::from_str::<PropertyLink>(json).is_err(),
            "予約語 `masks` を持つ PropertyLink が読めてしまっている"
        );
    }
}
