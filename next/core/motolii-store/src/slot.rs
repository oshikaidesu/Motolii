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
//!
//! ## 2026-08-23: 排他的三択 → base + 加算する modulator へ(裁定213)
//!
//! 利用者裁定「接続子は加算・ゲートはキーフレーム」がここを作り直した。
//! [`PropertySource`] は「`Track`/`Slot`/`Link` のどれか1つが勝つ」排他的な
//! 三択をやめ、**`base`(`Track`/`Slot` のどちらか、無くてもよい) + `modulators`
//! (加算で寄与する [`PropertyLink`] の列)**という構造化された形になった。
//! 値は `base の評価値 + Σ(modulators の寄与)`。
//!
//! **`Link` は排他枝から modulator 側へ移した**(判断・根拠は下記)。加算が
//! 「置き換え」を包含する——`base` を持たず modulator を1本だけ持てば、`None`
//! (何も足されない)+ その modulator の値 = その modulator の値そのものなので、
//! 旧 `PropertySource::Link` が表していた「この property を丸ごと別 property の
//! 値にする」という用法は [`PropertySource::link_only`] で完全に同じ値を返す。
//! 呼び手が1つ(`ui/motolii-inspector-pane::link`)しかいない今のうちに動かした
//! ——2人目の利用者が現れてから両方の形を生かす理由が無い(裁定13 と同じ判断)。
//!
//! **変調できる型の境界は発明しない** — `motolii_eval::Value` が自分で書いている
//! 「補間は Hold」の3型(`Bool`/`Enum`/`LayerId`)は加算も無意味なので変調不可
//! (単一 source が勝つ、[`motolii_eval::Value::add`] が常に `None` を返す)。
//! `F64`/`Vec2`/`Color`/`Path` は加算できる(`Color` はアルファ込みの4成分、
//! `Path` は `lerp` と同じ「頂点数と `closed` が一致する時だけ」)。
//!
//! **概念の穴と判断**(利用者からの追加裁定、5点、勝手に据え置かない):
//!
//! 1. **範囲外に出た時**: eval 層(ここ・`motolii-eval`)は **clamp しない**。
//!    `Value::lerp` の `Color` 実装が既に clamp していない(ベジェイージングの
//!    overshoot で範囲外の中間値が今でも作れる)ので、`add` だけ特別に clamp すると
//!    「lerp は素通し、add だけ丸める」という一貫しない挙動になる。**既存の
//!    domain-specific な消費側**(`StoreView::resolve` の layer opacity・
//!    `resolved_masks` の mask opacity、どちらも既に `.clamp(0.0, 1.0)` 済み)が
//!    F64 opacity は最終的に丸める——modulator 由来でも同じ経路を通るので**この
//!    2箇所は追加の変更なしで安全**。**`Value::Color` は今のところ resolve 側の
//!    clamp が無い**——ここ(store/eval)にドメイン知識(どの property が
//!    0..1 か)を埋めるのは裁定70 の思想(store は plugin/property の意味を知らない)
//!    に反するので、**厳しい側 = 拒否・報告** は export 境界(`motolii-export`、
//!    write-set 外)の仕事にする: 焼いた値が Lottie の妥当域を外れたら
//!    `effect_unsupported`/`check_audio_settings_unsupported` と同じ
//!    `UnsupportedForLottie` で**報告**し、黙って clamp/export しない
//!    (RETURN にこの follow-up を明記)
//! 2. **スケールが 0 を跨ぐ**: 意味のある表現(反転)なので潰さない。この
//!    crate(`world_affine`/`local_transform`/`LayerPlacement::from_transform`)は
//!    scale を**前向きに合成するだけ**で `.inverse()` を一度も呼ばない——
//!    `glam` が自己アサートで落ちるのは逆行列を取る時だけ(`AGENTS.md`)なので、
//!    ここでは 0 通過は無害。**危険なのは呼び手**(`ui/motolii-stage-pane` の
//!    gizmo/hit-test、`checked_inverse` 経由)側だが、そちらは write-set 外
//! 3. **override は表現できない**: 意図どおり——加算だけの機構では「base を
//!    無視してこの値にする」は書けない。**Ableton も変調は常にツマミ位置からの
//!    相対**なので、これは欠落ではなく仕様。base 自体を差し替えたければ
//!    `Intent::SetTrack`/`SetPropertySlot` で base 自体を編集する(または
//!    `link_only` のように base を持たない形にする)
//! 4. **和に上限を設けない**: Ableton の macro assignment のような modulator
//!    ごとの range は持たない——シンプルさを優先する判断。将来 2人目の利用者が
//!    range を要求したら、その時に `PropertyLink` へ足す(裁定13)
//! 5. **`time_offset` の負値は許可する**: `RationalTime`/`KeyframeTrack::eval`は
//!    既に負の時刻を普通に扱う(`eval` は `t <= keys[0].t` を「先頭キーフレーム
//!    より前」として先頭値を返すだけで符号を見ない)——source_t が comp 開始
//!    より前に出ても、既存の「最初のキーフレームより前は Hold」がそのまま
//!    適用される。新しい特別扱いは要らない(`tests/modulator.rs` で固定)

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

/// property の値の出処(裁定213)。**値 = `base` の評価値 + `modulators` の寄与の和**。
///
/// `base` は今までどおり `Track`(この property 自身の `KeyframeTrack`)か
/// `Slot`(comp の [`Slot`] 表への参照)のどちらか、または**無し**(=このproperty
/// 自身は値を持たず、modulator の和だけが値になる——旧 `PropertySource::Link`
/// 相当、[`Self::link_only`] 参照)。`modulators` は加算で寄与する
/// [`PropertyLink`] の列(裁定213 の「接続子は加算」)。
///
/// **書き込みは常にこの明示形**(`{"base": ..., "modulators": [...]}`)。
/// **読み込みは3つの旧形式もそのまま受け付ける**(後方互換、`Deserialize` 実装
/// 参照): 裸の `KeyframeTrack` オブジェクト(旧 `Track`)・裸の文字列(旧
/// `Slot`)・`source_layer`/`source_property`/`time_offset`/`plugin_id`/`params`
/// の5フィールドオブジェクト(裁定206当時の旧 `Link`、`base` キーを持たないので
/// 新形式と構造的に衝突しない)。どの旧形式も [`PropertySource::track`]/
/// [`PropertySource::slot`]/[`PropertySource::link_only`] のいずれかへ写る。
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PropertySource {
    /// この property 自身の値の由来。`None` = このproperty自身は何も持たない
    /// (modulator の和だけが値になる)。
    pub base: Option<PropertyBase>,
    /// 加算で寄与する modulator の列(裁定213)。空なら `base` だけの値
    /// (既存の全 Document と完全互換の意味)。
    ///
    /// **常に書く**(空でも `skip_serializing_if` しない) — `base: Option<_>` は
    /// serde の既定で「キーが無ければ `None`」に落ちてしまい(`#[serde(default)]`
    /// を付けていなくても serde_derive がそう扱う)、`base` の有無だけでは旧3形式
    /// (裸 `Track`/裸 `Slot`/裁定206単独 `Link`、どれも `base`/`modulators` という
    /// キーを持たない)と新形式を区別できない。**`modulators` キーの有無**で区別する
    /// ([`Deserialize`] 実装の `Explicit` がこのキーを必須にしている)。
    pub modulators: Vec<PropertyLink>,
}

/// [`PropertySource::base`] の中身。**排他的な二択**(`properties/property sid`
/// の note どおり) — この2つは「この property 自身が持つ静止/アニメーション
/// 値」の由来であって、modulator(他 property からの加算寄与)とは性質が違う
/// ので混ぜない。
///
/// `#[serde(untagged)]`: `Track` の wire 形は `KeyframeTrack` の JSON
/// (`{"keys":[...]}`)、`Slot` は裸の文字列——構造的に衝突しない。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyBase {
    Track(motolii_eval::KeyframeTrack),
    Slot(SlotId),
}

impl PropertySource {
    /// 普通の track を base に持つ形(modulator 無し)。
    pub fn track(track: motolii_eval::KeyframeTrack) -> Self {
        Self {
            base: Some(PropertyBase::Track(track)),
            modulators: Vec::new(),
        }
    }

    /// スロット参照を base に持つ形(modulator 無し)。
    pub fn slot(id: SlotId) -> Self {
        Self {
            base: Some(PropertyBase::Slot(id)),
            modulators: Vec::new(),
        }
    }

    /// 旧 `PropertySource::Link`(裁定206)に相当する形 — base を持たず
    /// modulator 1本だけ。加算の特殊形として全く同じ値を返す(`None` に相当する
    /// 「何も足さない」+ この modulator の値 = この modulator の値そのもの)。
    pub fn link_only(link: PropertyLink) -> Self {
        Self {
            base: None,
            modulators: vec![link],
        }
    }

    /// この source が「旧 Link 相当」(base 無し・modulator 1本)なら、その
    /// modulator を返す。呼び手(`ui/motolii-inspector-pane` の LINK section)が
    /// 「これは実質 link だ」と判定したい時のための最小の互換アクセサ
    /// (裁定213 で型が変わった呼び手への手当て)。
    pub fn as_link_only(&self) -> Option<&PropertyLink> {
        match (&self.base, self.modulators.as_slice()) {
            (None, [link]) => Some(link),
            _ => None,
        }
    }
}

/// 後方互換の核(裁定213): 旧3形式(裸 `Track`/裸 `Slot`/裁定206単独 `Link`)と
/// 新形式(`{"base":...,"modulators":[...]}`)を1つの `PropertySource` へ写す。
///
/// **手書きの `Visitor`**(`#[serde(untagged)]` は使わない)。最初の実装は
/// `#[serde(untagged)]` な `Wire` enum で4形式を順に試す形だったが、untagged は
/// 「まず入力全体を汎用 `Content` へバッファし、そこから各 variant へ変換を
/// 試す」という実装のため、大きな `KeyframeTrack`(数百 keyframe)を持つ
/// property で二重コストになり、`tests/document.rs::edit_storm_with_the_real_track_type`
/// (R0-1 の性能予算固定)を **1000µs → 7023µs** まで悪化させた(2026-08-23
/// 実測・`cargo test` で発覚)。`KeyframeTrack` 自身は元々
/// `#[serde(try_from = "KeyframeTrackDe")]` という**バッファ無しの直接変換**
/// だった(`track()` コストの97%が serde_json 解析という裁定140の計測は、この
/// 直接変換が前提)ので、`PropertySource` 側で untagged を挟むとその前提が崩れる。
///
/// ここでは **最初の1キーだけを見て**(ストリーミング、`Content` を作らない)
/// 4形式のどれかへ即分岐する——`"keys"` なら `Track`(値をそのまま
/// `Vec<Keyframe>` として直接読み、[`motolii_eval::KeyframeTrack::try_from_keys`]
/// で検証込みで組む=`KeyframeTrackDe` と同じ経路)、`"base"`/`"modulators"` なら
/// 新形式、それ以外(`source_layer` 等)なら旧 `Link`(5フィールド、小さいので
/// 汎用の逐次読みで十分——性能問題が起きるのは大きい `Track` の側だけ)。
impl<'de> Deserialize<'de> for PropertySource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Keys,
            Base,
            Modulators,
            SourceLayer,
            SourceProperty,
            TimeOffset,
            PluginId,
            Params,
        }

        struct PropertySourceVisitor;

        impl<'de> serde::de::Visitor<'de> for PropertySourceVisitor {
            type Value = PropertySource;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(
                    "a PropertySource: bare KeyframeTrack object, bare Slot string, \
                     legacy Link object, or {\"base\":...,\"modulators\":[...]}",
                )
            }

            // 旧 `PropertySource::Slot`(裸文字列)。
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PropertySource::slot(SlotId(v.to_owned())))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PropertySource::slot(SlotId(v)))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                use serde::de::Error as _;

                let Some(first_key) = map.next_key::<Field>()? else {
                    return Err(A::Error::custom(
                        "PropertySource: 空オブジェクトはどの形式にも合わない",
                    ));
                };

                match first_key {
                    // **旧 `Track`**: 唯一のフィールドを直接 `Vec<Keyframe>` として
                    // 読む(`Content` を経由しない、`KeyframeTrackDe` と同じ形の
                    // 直接変換)。
                    Field::Keys => {
                        let keys: Vec<motolii_eval::Keyframe> = map.next_value()?;
                        // 想定外の追加キーが来ても寛容に読み捨てる(旧形式の
                        // 寛容さをそのまま踏襲、deny_unknown_fields はしない)。
                        while map
                            .next_entry::<serde::de::IgnoredAny, serde::de::IgnoredAny>()?
                            .is_some()
                        {}
                        let track = motolii_eval::KeyframeTrack::try_from_keys(keys)
                            .map_err(A::Error::custom)?;
                        Ok(PropertySource::track(track))
                    }
                    // **新形式**(`base`/`modulators`、どちらが先に来ても受ける)。
                    // `base` は未指定でも `None`(=このproperty自身は値を持たない)
                    // として妥当なので、専用の追跡フラグは要らない。
                    Field::Base | Field::Modulators => {
                        let mut base: Option<PropertyBase> = None;
                        let mut modulators: Option<Vec<PropertyLink>> = None;
                        let mut key = Some(first_key);
                        loop {
                            let field = match key.take() {
                                Some(f) => f,
                                None => match map.next_key::<Field>()? {
                                    Some(f) => f,
                                    None => break,
                                },
                            };
                            match field {
                                Field::Base => base = map.next_value()?,
                                Field::Modulators => modulators = Some(map.next_value()?),
                                _ => {
                                    let _: serde::de::IgnoredAny = map.next_value()?;
                                }
                            }
                        }
                        let modulators = modulators
                            .ok_or_else(|| A::Error::missing_field("modulators"))?;
                        Ok(PropertySource { base, modulators })
                    }
                    // **旧 `Link`**(裁定206、5フィールド)。小さいので汎用の
                    // 逐次読みで十分(性能問題は大きい `Track` の側だけ)。
                    _ => {
                        let mut source_layer = None;
                        let mut source_property = None;
                        let mut time_offset = None;
                        let mut plugin_id = None;
                        let mut params = None;
                        let mut key = Some(first_key);
                        loop {
                            let field = match key.take() {
                                Some(f) => f,
                                None => match map.next_key::<Field>()? {
                                    Some(f) => f,
                                    None => break,
                                },
                            };
                            match field {
                                Field::SourceLayer => source_layer = Some(map.next_value()?),
                                Field::SourceProperty => {
                                    source_property = Some(map.next_value()?)
                                }
                                Field::TimeOffset => time_offset = Some(map.next_value()?),
                                Field::PluginId => plugin_id = Some(map.next_value()?),
                                Field::Params => params = Some(map.next_value()?),
                                Field::Keys | Field::Base | Field::Modulators => {
                                    let _: serde::de::IgnoredAny = map.next_value()?;
                                }
                            }
                        }
                        let link = PropertyLink {
                            source_layer: source_layer
                                .ok_or_else(|| A::Error::missing_field("source_layer"))?,
                            source_property: source_property
                                .ok_or_else(|| A::Error::missing_field("source_property"))?,
                            time_offset: time_offset
                                .ok_or_else(|| A::Error::missing_field("time_offset"))?,
                            plugin_id: plugin_id
                                .ok_or_else(|| A::Error::missing_field("plugin_id"))?,
                            params: params.ok_or_else(|| A::Error::missing_field("params"))?,
                        };
                        Ok(PropertySource::link_only(link))
                    }
                }
            }
        }

        deserializer.deserialize_any(PropertySourceVisitor)
    }
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

    /// **裁定213 で書き込み形が変わった**: `PropertySource` は今は常に明示的な
    /// `{"base":...,"modulators":[...]}` 形で書く(bit単位で裸 `KeyframeTrack` と
    /// 同じではなくなった) — 読み込み側の後方互換は別途固定する
    /// (`a_bare_keyframe_track_json_still_deserializes_as_a_track_base` 参照)。
    /// ここでは新形式が自分自身と往復することだけを固定する。
    #[test]
    fn property_source_track_round_trips_through_the_explicit_wire_shape() {
        let source = PropertySource::track(hold(Value::F64(1.0)));
        let json = serde_json::to_string(&source).unwrap();
        let back: PropertySource = serde_json::from_str(&json).unwrap();
        assert_eq!(back, source, "新形式の往復が一致しない: {json}");
    }

    /// **後方互換の核**: 裁定213 より前に書かれた裸の `KeyframeTrack` JSON
    /// (`PropertySource::Track` の旧 wire 形)が、無改造でそのまま
    /// `base: Some(Track(..))` として読める。
    #[test]
    fn a_bare_keyframe_track_json_still_deserializes_as_a_track_base() {
        let track = hold(Value::F64(1.0));
        let bare_json = serde_json::to_string(&track).unwrap();
        let source: PropertySource = serde_json::from_str(&bare_json).unwrap();
        assert_eq!(source, PropertySource::track(track));
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

    /// 新形式(明示的な `{"base":...,"modulators":[...]}`)の4パターン
    /// (`track`/`slot`/`link_only`/base+modulator の和)が、それぞれ自分自身と
    /// 往復すること——`base` キーが常に付くので、パターン同士が構造的に
    /// 衝突しないことの直接証拠(裁定213)。
    #[test]
    fn property_source_round_trips_for_every_shape() {
        let track = PropertySource::track(hold(Value::F64(1.0)));
        let slot = PropertySource::slot(SlotId("s".to_owned()));
        let link_only = PropertySource::link_only(link(
            PropertyId::new("opacity").unwrap(),
            "motolii.link.identity",
            Vec::new(),
        ));
        let summed = PropertySource {
            base: Some(PropertyBase::Track(hold(Value::F64(2.0)))),
            modulators: vec![link(
                PropertyId::new("rotation").unwrap(),
                "motolii.link.identity",
                Vec::new(),
            )],
        };

        for source in [track, slot, link_only, summed] {
            let json = serde_json::to_string(&source).unwrap();
            let back: PropertySource = serde_json::from_str(&json).unwrap();
            assert_eq!(back, source, "新形式の往復が一致しない: {json}");
        }
    }

    /// 裁定206 当時(裁定213 の前日)に書かれた「単独 Link」の裸5フィールド
    /// オブジェクト——`base` キーを持たない旧形式——が、`base: None` ・
    /// modulator 1本として読める(後方互換)。
    #[test]
    fn a_legacy_bare_link_object_still_deserializes_as_link_only() {
        let l = link(
            PropertyId::new("rotation").unwrap(),
            "motolii.link.linear",
            vec![("scale".to_owned(), Value::F64(2.0))],
        );
        let legacy_json = serde_json::to_string(&l).unwrap();
        let source: PropertySource = serde_json::from_str(&legacy_json).unwrap();
        assert_eq!(source, PropertySource::link_only(l));
    }

    /// 裸の `SlotId` 文字列(裁定213 より前の `PropertySource::Slot`)も同様に読める。
    #[test]
    fn a_bare_slot_id_string_still_deserializes_as_a_slot_base() {
        let id = SlotId("brand".to_owned());
        let bare_json = serde_json::to_string(&id).unwrap();
        let source: PropertySource = serde_json::from_str(&bare_json).unwrap();
        assert_eq!(source, PropertySource::slot(id));
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
