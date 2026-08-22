//! 投影(`SelectionProjection`)── Document の写しではなく、1度描くための
//! 使い捨て値(`timeline_pane::rows` と同じ形、裁定5)。
//!
//! **持つ**: [`SelectionProjection`]とその構成要素([`ComponentSlot`]/
//! [`RowValue`]/[`TransformRowProjection`]/[`AttrsProjection`]/
//! [`MaskRowProjection`]/[`EffectRowProjection`]/[`TextSectionProjection`]/
//! [`KeyCellProjection`]/[`LayerCandidate`])・組み立て本体([`project`])。
//! **読むだけ** ── Document には一切書かない。
//!
//! **持たない**: 投影の中身をどう描くか(各 section の view 関数)・投影を
//! 元にした書き口(`commit_inspector_field` 等、`crate::transform` の仕事)。
//!
//! ## 2026-08-22 発注「レイヤーを指す」文法(裁定177「1意図=1つの家」の適用)
//! `LayerAttrs::matte`/型付き `PropertyLink` はどちらも「別レイヤーを指す」
//! ことが唯一の共通の形——「指す」入力欄1つ(pick_list)を2箇所で使い回す。
//! **候補の絞り込み(自分自身を選べない・循環しない)はここ([`project`])で
//! 済ませる** — `&StoreView` を持つのはここだけで、view 側(`matte.rs`/
//! `link.rs`)は「拒否される選択肢が最初から無い」投影を渡されるだけになる。
//! matte の循環は書き込み時に store が拒む柵が無い(`LayerAttrs::matte` の doc
//! 参照 — `parent`/`PropertyLink` と違い store 側に `validate_no_*_cycle` が
//! 無い)ので、[`matte_would_cycle`] はここでしか行われない絞り込み。link の
//! 循環は store 側に既に書き込み時拒否がある
//! (`next/core/motolii-store/src/document.rs::validate_no_link_cycle`、
//! private fn)——[`link_would_cycle`] はその**同じ絞り込みロジックの UI 側
//! 複製**(`StoreView::property_source` という公開 API だけを使って書き直した、
//! store 側 fn を呼べないので複製する以外の道が無い)。

use std::collections::HashSet;

use motolii_core::{Fps, RationalTime};
use motolii_shell_state::Session;
use motolii_store::{
    property, EffectId, LayerId, LayerSource, MaskId, MaskMode, Matte, PropertyId, PropertySource,
    StoreError, StoreView, TextDocumentStyle, TextJustify, Value,
};

use crate::attrs::speed_percent;
use crate::effects::{plugin_display_name, plugin_params};
use crate::link::{LinkRowProjection, LinkSourceCandidate, LinkTarget};
use crate::text::{default_text_document, default_text_style, text_document_content};
use crate::transform::{
    field_decimals, has_real_keys, key_cell_state, key_row_property_id, KeyCellState, KeyRow,
    TransformField,
};

// ---------------------------------------------------------------------------
// 投影 — Document の写しではなく、1度描くための使い捨て値(`timeline_pane::rows` と同じ形)
// ---------------------------------------------------------------------------

/// 1成分(X/Y/Z、または scalar 1個)の投影。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComponentSlot {
    pub axis: &'static str,
    /// store の意味モデルにこの軸があるか。無ければ表示は `—`(mock の
    /// `emptyComponent` と同じ — Rotation の X/Y、Scale/Anchor の Z は Motolii の
    /// 2.5D モデル(裁定113)に無いので `false`)。
    pub present: bool,
    /// 表示単位での値(Opacity だけ % — store は 0..1)。`present=false` なら無意味。
    pub value: f64,
    /// 編集可能か。**present な成分は常に `true`**(Q0: キー数で触れなくなる
    /// 状態を作らない — 2026-08-22 発注で旧規則「keys.len()<=1 のみ編集可」を
    /// 撤去した。キー持ち track の編集は playhead へのキー upsert =
    /// [`edited_value_track`])。`present=false` の穴だけ `false`。
    pub editable: bool,
    /// この track が実キーを持つか([`has_real_keys`])。表示は accent —
    /// 「編集すると記録される」ことの視覚合図(AE のキー付き値と同型)。
    pub keyed: bool,
    /// この成分が編集される時に動く field。`present=false` なら `None`。
    pub field: Option<TransformField>,
}

pub(crate) fn absent_component(axis: &'static str) -> ComponentSlot {
    ComponentSlot {
        axis,
        present: false,
        value: 0.0,
        editable: false,
        keyed: false,
        field: None,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RowValue {
    Vector([ComponentSlot; 3]),
    Scalar(ComponentSlot),
}

/// Key セルの投影(K1)。`row` は click 時に `Message::KeyPressed` が運ぶ宛先、
/// `state` は3状態 oracle([`key_cell_state`])の結果。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyCellProjection {
    pub row: KeyRow,
    pub state: KeyCellState,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransformRowProjection {
    pub label: &'static str,
    pub value: RowValue,
    pub decimals: usize,
    /// Key 列(K1)。5行全部が持つ — 「触れそうで触れない」空予約は残さない(Q0)。
    pub key: KeyCellProjection,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AttrsProjection {
    pub name: String,
    pub hidden: bool,
    /// **クリックで巡回するボタンの現在値**(BL2)。`Message::CycleBlendMode` を
    /// 押すたび engine が対応する次の mode へ進む(現状 Normal→Add→Normal、
    /// [`SUPPORTED_BLEND_MODES`] 参照)。`BlendMode` の `Debug` 表示をそのまま使う
    /// (`Normal`/`Add`/…)。
    pub blend_mode: String,
    /// `LayerTiming.speed` の表示 %(100=等速、SP1 第一波)。`meta` が読めない
    /// (起こらないはず)場合は等速(100.0)。[`speed_percent`] がこの写像の実体。
    pub speed_percent: f64,
    /// ラベル色の palette index(B03、`LayerAttrs.label_color` の写し)。
    /// `None` = 未割当 — チップは timeline スウォッチと同じフォールバック
    /// (`way_timeline`)で塗る(`lane_bar::swatch_color` と同じ源・同じ既定)。
    pub label_color: Option<u8>,
    /// `LayerAttrs.matte`(2026-08-22 発注「レイヤーを指す」文法)。engine は
    /// 本日 `MatteMode`/`Matte` の消費を開始済み(`motolii_engine::Engine::
    /// apply_matte`)——ここが初の書き口。`None` = マットにされていない。
    pub matte: Option<Matte>,
    /// [`Self::matte`] の元候補(pick_list の選択肢)。**自分自身を除外**し、
    /// **matte 連鎖の循環も除外**(`matte_would_cycle` — store 側に書き込み時
    /// 拒否が無いので、ここが唯一の絞り込み)。
    pub matte_candidates: Vec<LayerCandidate>,
}

/// effect 1本ぶんの投影(EFFECTS section、B38 第3切片・裁定184 型別 section
/// 第2号)。静止する部分(名前・enabled)は store の [`EffectInstance`] の写し、
/// 動く部分(param)は既存の値セル行文法([`TransformRowProjection`])を
/// そのまま再利用する — [`MaskRowProjection`] と同じ分担。
#[derive(Clone, Debug, PartialEq)]
pub struct EffectRowProjection {
    pub id: EffectId,
    /// 表示名([`plugin_display_name`] — 既知 plugin は人間可読名、未知は
    /// plugin_id そのまま)。
    pub name: String,
    /// `effects/effect/en`。false = bypass 中(消えてはいない)。
    pub enabled: bool,
    /// param 値行([`plugin_params`] のカタログ順)。未知 plugin は空 —
    /// store は catalog を知らないので param 行を捏造しない(M13)。
    pub params: Vec<TransformRowProjection>,
}

/// mask 1枚ぶんの投影(MASK section、B02 第1切片・裁定184)。静止する部分
/// (mode/inverted)は store の [`Mask`] の写し、動く部分(opacity)は既存の
/// 値セル行文法([`TransformRowProjection`])をそのまま再利用する — view は
/// opacity 行を [`transform_row`] と同じ関数で描く(新しい編集文法を発明しない)。
#[derive(Clone, Debug, PartialEq)]
pub struct MaskRowProjection {
    pub id: MaskId,
    pub mode: MaskMode,
    pub inverted: bool,
    /// `mask.{id}.opacity` track の値行(label="Opacity"・decimals=0・% 表示 —
    /// layer Opacity 行と同じ形)。Key 列は [`KeyRow::MaskOpacity`]。
    pub opacity: TransformRowProjection,
}

/// 「別レイヤーを指す」pick_list の1候補(2026-08-22 発注)。**id で区別でき、
/// 人が読んで分かる**表示にする発注どおり — `label` は id を必ず含む
/// (`layer_label` 参照。名前が同じレイヤーが複数在っても id で見分けられる)。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayerCandidate {
    pub id: LayerId,
    pub label: String,
}

/// [`LayerCandidate::label`] の組み立て本体 — 名前があれば `"name (#id)"`、
/// 無ければ [`crate::ident_band`] の placeholder と同じ `"layer {id}"`。
pub(crate) fn layer_label(store: &StoreView<'_>, id: LayerId) -> String {
    let name = store
        .attrs(id)
        .ok()
        .flatten()
        .map(|attrs| attrs.name)
        .unwrap_or_default();
    if name.is_empty() {
        format!("layer {}", id.0)
    } else {
        format!("{name} (#{})", id.0)
    }
}

/// `candidate` を matte 元として選ぶと、matte 連鎖を辿って `target` 自身へ
/// 戻ってくるか。[`crate::document::validate_no_parent_cycle`]
/// (`next/core/motolii-store`、private fn)と同型の複製 —
/// **matte には store 側の書き込み時循環拒否が無い**(`LayerAttrs::matte` の
/// doc 参照)ので、この絞り込みは UI 側でしか行われない。`seen` は防御的な保険
/// (壊れた Document が既に循環していても無限に回らない)。
fn matte_would_cycle(store: &StoreView<'_>, target: LayerId, candidate: LayerId) -> bool {
    let mut current = Some(candidate);
    let mut seen = HashSet::new();
    while let Some(layer) = current {
        if layer == target {
            return true;
        }
        if !seen.insert(layer) {
            break;
        }
        current = store
            .attrs(layer)
            .ok()
            .flatten()
            .and_then(|attrs| attrs.matte)
            .map(|matte| matte.layer);
    }
    false
}

/// `(candidate_layer, candidate_property)` を `target` の link 元として選ぶと、
/// link 参照鎖を辿って `target` 自身へ戻ってくるか。
/// `next/core/motolii-store/src/document.rs::validate_no_link_cycle`
/// (private fn)と**同じ絞り込みロジックの UI 側複製** — store 側 fn を直接
/// 呼べない(private)ので、公開 API([`StoreView::property_source`])だけで
/// 書き直した。store は書き込み時にこの循環を`Err`で拒むので、ここでの絞り込み
/// は「拒否される選択肢を UI に最初から出さない」ための先回りに過ぎない
/// (store 側の柵が最終防衛線のまま)。
fn link_would_cycle(
    store: &StoreView<'_>,
    target: (LayerId, &PropertyId),
    candidate: (LayerId, PropertyId),
) -> bool {
    let mut current = Some(candidate);
    let mut seen = HashSet::new();
    while let Some((layer, property)) = current {
        if layer == target.0 && &property == target.1 {
            return true;
        }
        if !seen.insert((layer, property.clone())) {
            break;
        }
        current = store
            .property_source(layer, &property)
            .ok()
            .flatten()
            .and_then(|source| match source {
                PropertySource::Link(link) => Some((link.source_layer, link.source_property)),
                _ => None,
            });
    }
    false
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectionProjection {
    pub layer: LayerId,
    /// ident 帯の種別ラベル(mock の `.ident s` = 「clip · 0 shared FX」相当)。
    /// **`LayerSource` の実データから引く** — mock の「shared FX」は store に
    /// 対応する概念が無い(effect に "shared" フラグは無い)ので、種別だけ載せ、
    /// FX 件数は捏造しない(M13: 無い意味を有るふりで出さない)。
    pub kind: &'static str,
    pub transform: Vec<TransformRowProjection>,
    pub attrs: AttrsProjection,
    /// MASK section の行(store の並びどおり)。**空なら section 自体を出さない**
    /// (Q0: mask の無い layer に効かない chrome を並べない — `empty_state` と
    /// 同じ判断)。裁定184「型別 section で拡張」の第1号。
    pub masks: Vec<MaskRowProjection>,
    /// EFFECTS section の行(store の並び = 適用順どおり)。空なら section 自体を
    /// 出さない(masks と同じ Q0 判断)。裁定184 型別 section の第2号。
    pub effects: Vec<EffectRowProjection>,
    /// TEXT section の投影(B46 第1切片、裁定184)。`LayerSource::Text` の
    /// layer でのみ `Some`(MASK section と同じ「型が合わない chrome を出さ
    /// ない」判断、Q0)。`text_document` が未着手でも [`default_text_document`]
    /// で表示専用の既定値を作る(store には書かない、`default_vec2` と同じ形)。
    pub text: Option<TextSectionProjection>,
    /// AUDIO section の投影(B42、裁定184 型別 section 第4号)。
    /// `LayerSource::Media` の layer でのみ `Some`(TEXT section と同じ
    /// 「型が合わない chrome を出さない」判断、Q0)。
    pub audio: Option<AudioSectionProjection>,
    /// LINK section の行(2026-08-22 発注「レイヤーを指す」文法 第3号)。
    /// 標準 property 5種([`LinkTarget::ALL`])を固定で持つ(mask/effect と違い
    /// 「無ければ出さない」ではない — link は任意の layer の任意の標準
    /// property に張れるので、選択層の種別を問わず常に現れる)。
    pub links: Vec<LinkRowProjection>,
}

/// AUDIO section の投影(B42、裁定184 型別 section 第4号)。4行とも
/// per-id ではない layer 単位の標準 property(`property::LEVEL`/`PAN`/
/// `FADE_IN`/`FADE_OUT`)で、既存の値セル行文法([`TransformRowProjection`])を
/// そのまま再利用する — MASK/EFFECTS の「動く部分は既存文法を再利用」と
/// 同じ分担だが、per-id の一覧ではなく layer につき常に高々1組固定なので、
/// `Vec` ではなく named field の束(`AttrsProjection` と同じ形)で持つ。
///
/// **gate は `LayerSource::Media`**: 実際にその素材が音声を持つかは
/// decode してみないと分からない(engine `layer_mix_source` も同じ理由で
/// 「音声を持たない素材は mix 対象から除外」を実行時に判定する、
/// `motolii-audio::program` doc 参照)。Inspector はここまで踏み込まず、
/// 「音声を持ち得る素材種」で section の有無を決める(TEXT section が
/// `LayerSource::Text` で決めるのと同じ粒度の判断)。
#[derive(Clone, Debug, PartialEq)]
pub struct AudioSectionProjection {
    /// `property::LEVEL`(clip の音量、gain)。store は倍率(1.0=等倍)、
    /// **表示は %**(Speed 欄と同じ「比をパーセントで見せる」慣習を踏襲 —
    /// dB 換算はしない、store 自身が dB を持たないので変換を発明しない
    /// という発注書「store の既存意味に従う」の帰結)。上限クランプは
    /// しない(ブースト = 100%超を許す)。
    pub level: TransformRowProjection,
    /// `property::PAN`。store 自体が -1.0(全振り左)〜1.0(全振り右)の
    /// 人間可読単位(W3C `StereoPannerNode` と同じ約束、`property::PAN` doc
    /// 参照)なので、**表示も変換なしでそのまま**。L/R ラベルへの変換は
    /// 新しい表示規約になり、既存の「数値のみ」の値セル文法と食い違う
    /// ため見送った(先例: 他の値セル行はどれも生の数値表示。RETURN の
    /// 見送り台帳参照)。
    pub pan: TransformRowProjection,
    /// `property::FADE_IN`(秒)。0.0 = 無効。
    pub fade_in: TransformRowProjection,
    /// `property::FADE_OUT`(秒)。0.0 = 無効。[`fade_in`](field)と対称。
    pub fade_out: TransformRowProjection,
}

/// TEXT section の投影(裁定98: `styles[0]` = document 既定行のみを対象 —
/// 範囲スタイル表・アニメーターは次切片)。Key 列は無い — 対象フィールドは
/// どれも `KeyframeTrack` に乗らない静止値(裁定92)なので、Position/Scale 等
/// の3状態 oracle は適用対象外。
#[derive(Clone, Debug, PartialEq)]
pub struct TextSectionProjection {
    /// `text-document t`(Text、本文)。**Hold 評価済みの現在値**
    /// (`text_document_content` — [`RationalTime::ZERO`] 固定、`text.rs`
    /// `TextField::Content` doc「複数行の扱い」参照)。2026-08-22 発注「歌詞が
    /// 入れられる道を通す」で追加(致命的欠落: TEXT section に本文の入力欄が
    /// 無かった)。
    pub content: String,
    /// `text-document f`(Font Family)。
    pub font_family: String,
    /// `text-document s`(Font Size)。
    pub size: f32,
    /// `text-document lh`(Line Height)。`None` = Auto。
    pub line_height: Option<f32>,
    /// `text-document tr`(Tracking)。
    pub tracking: f32,
    /// `text-document j`(Justify)。
    pub justify: TextJustify,
    /// スタイル表の既定行そのもの(裁定98、`styles[0]`)。2026-08-22 発注で
    /// 色エディタ([`crate::color::color_row`])へ渡すために追加 —
    /// `fill`/`stroke_color` は個別フィールドへ分解せず、`color_row` が
    /// 期待する `&TextDocumentStyle` の形のまま持つ(2箇所で同じ値を別の形に
    /// 二重管理しない)。上の `font_family`/`size`/`line_height`/`tracking` は
    /// 既存 UI 呼び出し口(`text_field_row` 等)の互換のため残す — 同じ値の
    /// 冗長な保持だが、意味は完全に一致する(この `style` が正本、上4フィールドは
    /// そこからの複写)。
    pub style: TextDocumentStyle,
}

/// [`SelectionProjection::kind`] の出典。`LayerSource` の variant 名をそのまま
/// 小文字化した語彙(発明ではなく store の型そのものを読める言葉にしただけ)。
pub(crate) fn source_kind_label(source: &LayerSource) -> &'static str {
    match source {
        LayerSource::Solid { .. } => "solid",
        LayerSource::Media { .. } => "media",
        LayerSource::Null => "null",
        LayerSource::Shape => "shape",
        LayerSource::Text => "text",
        // 裁定173。H3(親選択 UI/Inspector の parent 表示)は非目標のまま —
        // ここは非網羅マッチをコンパイラに教えてもらって埋めた最小限の一言のみ。
        LayerSource::Group => "group",
    }
}

fn scalar_component(
    store: &StoreView<'_>,
    layer: LayerId,
    name: &str,
    axis: &'static str,
    field: TransformField,
    t: RationalTime,
    default: f64,
) -> Result<ComponentSlot, StoreError> {
    let property = PropertyId::new(name)?;
    let track = store.track(layer, &property)?;
    let keyed = has_real_keys(track.as_ref());
    let value = match store.value_at(layer, &property, t)? {
        Some(Value::F64(v)) => v,
        _ => default,
    };
    Ok(ComponentSlot {
        axis,
        present: true,
        value,
        editable: true,
        keyed,
        field: Some(field),
    })
}

fn vec2_components(
    store: &StoreView<'_>,
    layer: LayerId,
    name: &str,
    field_x: TransformField,
    field_y: TransformField,
    t: RationalTime,
    default: [f64; 2],
) -> Result<[ComponentSlot; 2], StoreError> {
    let property = PropertyId::new(name)?;
    let track = store.track(layer, &property)?;
    let keyed = has_real_keys(track.as_ref());
    let [x, y] = match store.value_at(layer, &property, t)? {
        Some(Value::Vec2(v)) => [v[0], v[1]],
        _ => default,
    };
    Ok([
        ComponentSlot {
            axis: "X",
            present: true,
            value: x,
            editable: true,
            keyed,
            field: Some(field_x),
        },
        ComponentSlot {
            axis: "Y",
            present: true,
            value: y,
            editable: true,
            keyed,
            field: Some(field_y),
        },
    ])
}

/// `store`/`session` から選択層の Inspector 投影を組み立てる。**読むだけ**。
/// 選択なし・選択層が削除済み(present でない)・comp が無い、のいずれも `Ok(None)`
/// (M13: 壊れているのではなく「まだ映す物が無い」)。
///
/// **`&Session` を直接取る**(裁定160 切片7 以降、crate doc 参照)——`Session`
/// は `motolii-shell-state` leaf crate に住む(`motolii-timeline-pane` と同じ
/// 依存)ので、root(`motolii-shell`)を経由せずにここへ持ち込める。切片7以前は
/// `Session` が `motolii-shell` root に残っていたため、循環を避けて
/// `selection`/`playhead` の2引数へ分解する回避策を取っていたが、切片7の
/// leaf crate 化でこの回避策は不要になった。
pub fn project(
    store: &StoreView<'_>,
    session: &Session,
) -> Result<Option<SelectionProjection>, StoreError> {
    let Some(layer) = session.selection else {
        return Ok(None);
    };
    if !store.has_layer(layer) {
        return Ok(None);
    }
    let Some(composition) = store.composition()? else {
        return Ok(None);
    };
    let t = RationalTime::try_from_frame(session.playhead, composition.fps)
        .unwrap_or(RationalTime::ZERO);

    // Key 列(K1)の3状態を行ごとに読む(表示と click 判定の共通 oracle =
    // `key_cell_state`)。playhead は timeline と同じ frame 粒度で照合する。
    let key_cell = |row: KeyRow| -> Result<KeyCellProjection, StoreError> {
        let property = key_row_property_id(row)?;
        let track = store.track(layer, &property)?;
        Ok(KeyCellProjection {
            row,
            state: key_cell_state(track.as_ref(), session.playhead, composition.fps),
        })
    };

    let position_xy = vec2_components(
        store,
        layer,
        property::POSITION,
        TransformField::PositionX,
        TransformField::PositionY,
        t,
        [0.0, 0.0],
    )?;
    let position_z = scalar_component(
        store,
        layer,
        property::POSITION_Z,
        "Z",
        TransformField::PositionZ,
        t,
        0.0,
    )?;
    let position_row = TransformRowProjection {
        label: "Position",
        value: RowValue::Vector([position_xy[0], position_xy[1], position_z]),
        decimals: 3,
        key: key_cell(KeyRow::Position)?,
    };

    let scale_xy = vec2_components(
        store,
        layer,
        property::SCALE,
        TransformField::ScaleX,
        TransformField::ScaleY,
        t,
        [1.0, 1.0],
    )?;
    let scale_row = TransformRowProjection {
        label: "Scale",
        value: RowValue::Vector([scale_xy[0], scale_xy[1], absent_component("Z")]),
        decimals: 3,
        key: key_cell(KeyRow::Scale)?,
    };

    let rotation_z = scalar_component(
        store,
        layer,
        property::ROTATION,
        "Z",
        TransformField::Rotation,
        t,
        0.0,
    )?;
    let rotation_row = TransformRowProjection {
        label: "Rotation",
        value: RowValue::Vector([absent_component("X"), absent_component("Y"), rotation_z]),
        decimals: 1,
        key: key_cell(KeyRow::Rotation)?,
    };

    let mut opacity = scalar_component(
        store,
        layer,
        property::OPACITY,
        "Opacity",
        TransformField::Opacity,
        t,
        1.0,
    )?;
    opacity.value *= 100.0; // store は 0..1、表示は %。
    let opacity_row = TransformRowProjection {
        label: "Opacity",
        value: RowValue::Scalar(opacity),
        decimals: 0,
        key: key_cell(KeyRow::Opacity)?,
    };

    let anchor_xy = vec2_components(
        store,
        layer,
        property::ANCHOR,
        TransformField::AnchorX,
        TransformField::AnchorY,
        t,
        [0.0, 0.0],
    )?;
    let anchor_row = TransformRowProjection {
        label: "Anchor",
        value: RowValue::Vector([anchor_xy[0], anchor_xy[1], absent_component("Z")]),
        decimals: 3,
        key: key_cell(KeyRow::Anchor)?,
    };

    let attrs = store.attrs(layer)?.unwrap_or_default();
    // `kind`/`speed_percent` は同じ `meta()` から読む(2回叩かない)。
    let meta = store.meta(layer)?;
    // MATTE 元の候補(2026-08-22 発注「レイヤーを指す」文法): 自分自身を除外し
    // (発注書「マット元は自分自身を選べてはいけない」)、matte 連鎖の循環も
    // 除外する([`matte_would_cycle`] — store 側に書き込み時拒否が無いので
    // ここが唯一の絞り込み)。
    let matte_candidates: Vec<LayerCandidate> = store
        .layers()
        .into_iter()
        .filter(|&candidate| candidate != layer && !matte_would_cycle(store, layer, candidate))
        .map(|candidate| LayerCandidate {
            id: candidate,
            label: layer_label(store, candidate),
        })
        .collect();
    let attrs_projection = AttrsProjection {
        name: attrs.name,
        hidden: attrs.hidden,
        blend_mode: format!("{:?}", attrs.blend_mode),
        speed_percent: meta
            .as_ref()
            .map(|meta| speed_percent(meta.timing.speed.num(), meta.timing.speed.den()))
            .unwrap_or(100.0),
        label_color: attrs.label_color,
        matte: attrs.matte,
        matte_candidates,
    };

    let kind = meta
        .as_ref()
        .map(|meta| source_kind_label(&meta.source))
        .unwrap_or("layer");

    // MASK section(B02 第1切片): store の並びどおり。opacity 行は layer
    // Opacity 行と同じ組み方(track を読み、無ければ既定 1.0 → 表示 %)。
    let mut mask_rows = Vec::new();
    for mask in store.masks(layer)? {
        let property = PropertyId::mask_opacity(mask.id);
        let track = store.track(layer, &property)?;
        let keyed = has_real_keys(track.as_ref());
        let value = match store.value_at(layer, &property, t)? {
            Some(Value::F64(v)) => v,
            _ => 1.0, // `motolii-store::mask` の既定(比 1.0 = 全掩)。
        };
        let state = key_cell_state(track.as_ref(), session.playhead, composition.fps);
        mask_rows.push(MaskRowProjection {
            id: mask.id,
            mode: mask.mode,
            inverted: mask.inverted,
            opacity: TransformRowProjection {
                label: "Opacity",
                value: RowValue::Scalar(ComponentSlot {
                    axis: "Opacity",
                    present: true,
                    value: value * 100.0, // store は 0..1、表示は %(layer Opacity と同じ)。
                    editable: true,
                    keyed,
                    field: Some(TransformField::MaskOpacity(mask.id)),
                }),
                decimals: 0,
                key: KeyCellProjection {
                    row: KeyRow::MaskOpacity(mask.id),
                    state,
                },
            },
        });
    }

    // EFFECTS section(B38 第3切片): store の並び = 適用順どおり。param 行は
    // 既知 plugin のカタログ([`plugin_params`])分だけ — track を読み、無ければ
    // engine 既定の写し([`GlowParam::default_value`])。表示 = store 単位
    // (opacity 系と違い % 換算しない)。
    let mut effect_rows = Vec::new();
    for effect in store.effects(layer)? {
        let mut param_rows = Vec::new();
        for param in plugin_params(&effect.plugin_id) {
            let property = PropertyId::effect_param(effect.id, param.name())?;
            let track = store.track(layer, &property)?;
            let keyed = has_real_keys(track.as_ref());
            let value = match store.value_at(layer, &property, t)? {
                Some(Value::F64(v)) => v,
                _ => param.default_value(),
            };
            let state = key_cell_state(track.as_ref(), session.playhead, composition.fps);
            param_rows.push(TransformRowProjection {
                label: param.label(),
                value: RowValue::Scalar(ComponentSlot {
                    axis: param.label(),
                    present: true,
                    value,
                    editable: true,
                    keyed,
                    field: Some(TransformField::EffectParam(effect.id, *param)),
                }),
                decimals: field_decimals(TransformField::EffectParam(effect.id, *param)),
                key: KeyCellProjection {
                    row: KeyRow::EffectParam(effect.id, *param),
                    state,
                },
            });
        }
        effect_rows.push(EffectRowProjection {
            id: effect.id,
            name: plugin_display_name(&effect.plugin_id).to_owned(),
            enabled: effect.enabled,
            params: param_rows,
        });
    }

    // TEXT section(B46 第1切片、裁定184): `LayerSource::Text` の layer での
    // み現れる(MASK section の「空なら出さない」と同じ Q0 判断 — ここは
    // 「型が合わない」判断)。`text_document` 未着手でも [`default_text_document`]
    // で表示専用の既定値を作る(store には書かない、Position 等の
    // `default_vec2` と同じ形)。
    let text = match meta.as_ref().map(|meta| &meta.source) {
        Some(LayerSource::Text) => {
            let document = store.text_document(layer)?.unwrap_or_else(default_text_document);
            let style = document
                .styles
                .first()
                .cloned()
                .unwrap_or_else(default_text_style);
            Some(TextSectionProjection {
                content: text_document_content(&document),
                font_family: style.font.family.clone(),
                size: style.size,
                line_height: style.line_height,
                tracking: style.tracking,
                justify: document.justify,
                style,
            })
        }
        _ => None,
    };

    // AUDIO section(B42、裁定184 型別 section 第4号): `LayerSource::Media`
    // の layer でのみ現れる(TEXT section と同じ「型が合わない」判断・
    // AudioSectionProjection doc の gate 説明参照)。4行とも layer 単位の
    // 標準 property なので、mask/effect のような id 起点のループは無い —
    // Opacity 行(scalar_component + 手動 % 換算)と同じ組み方をそのまま
    // 4回繰り返すだけ。
    let audio = match meta.as_ref().map(|meta| &meta.source) {
        Some(LayerSource::Media { .. }) => {
            let mut level = scalar_component(
                store,
                layer,
                property::LEVEL,
                "Level",
                TransformField::Level,
                t,
                1.0,
            )?;
            level.value *= 100.0; // store は倍率、表示は %(layer Opacity と同じ換算)。
            let pan = scalar_component(
                store,
                layer,
                property::PAN,
                "Pan",
                TransformField::Pan,
                t,
                0.0,
            )?;
            let fade_in = scalar_component(
                store,
                layer,
                property::FADE_IN,
                "Fade In",
                TransformField::FadeIn,
                t,
                0.0,
            )?;
            let fade_out = scalar_component(
                store,
                layer,
                property::FADE_OUT,
                "Fade Out",
                TransformField::FadeOut,
                t,
                0.0,
            )?;
            Some(AudioSectionProjection {
                level: TransformRowProjection {
                    label: "Level",
                    value: RowValue::Scalar(level),
                    decimals: field_decimals(TransformField::Level),
                    key: key_cell(KeyRow::Level)?,
                },
                pan: TransformRowProjection {
                    label: "Pan",
                    value: RowValue::Scalar(pan),
                    decimals: field_decimals(TransformField::Pan),
                    key: key_cell(KeyRow::Pan)?,
                },
                fade_in: TransformRowProjection {
                    label: "Fade In",
                    value: RowValue::Scalar(fade_in),
                    decimals: field_decimals(TransformField::FadeIn),
                    key: key_cell(KeyRow::FadeIn)?,
                },
                fade_out: TransformRowProjection {
                    label: "Fade Out",
                    value: RowValue::Scalar(fade_out),
                    decimals: field_decimals(TransformField::FadeOut),
                    key: key_cell(KeyRow::FadeOut)?,
                },
            })
        }
        _ => None,
    };

    // LINK section(2026-08-22 発注「レイヤーを指す」文法 第3号)。標準
    // property 5種([`LinkTarget::ALL`])を対象に固定する(発注書「plugin_id は
    // 最小限で構わない——器が通ることが目的」の範囲、mask/effect param のような
    // id 付き property は対象外)。候補は自分自身を除外し
    // ([`link_would_cycle`] が同一 layer・同一 property を最初のホップで
    // 検出するので、自己参照は循環判定に自然に含まれる)、循環する組も除外する
    // — store 側の書き込み時拒否(`validate_no_link_cycle`)と**同じ判定**を
    // 先回りするだけで、最終防衛線は store のまま。
    let mut link_rows = Vec::new();
    for target in LinkTarget::ALL {
        let target_property = target.property_id();
        let current = match store.property_source(layer, &target_property)? {
            Some(PropertySource::Link(link)) => {
                LinkTarget::from_property_name(link.source_property.name()).map(|property| {
                    LinkSourceCandidate {
                        layer: LayerCandidate {
                            id: link.source_layer,
                            label: layer_label(store, link.source_layer),
                        },
                        property,
                    }
                })
            }
            _ => None,
        };
        let mut candidates = Vec::new();
        for candidate_layer in store.layers() {
            if candidate_layer == layer {
                continue; // 自分自身は選べない(発注書「参照先も同様」)。
            }
            for candidate_target in LinkTarget::ALL {
                let candidate_property = candidate_target.property_id();
                if link_would_cycle(
                    store,
                    (layer, &target_property),
                    (candidate_layer, candidate_property),
                ) {
                    continue;
                }
                candidates.push(LinkSourceCandidate {
                    layer: LayerCandidate {
                        id: candidate_layer,
                        label: layer_label(store, candidate_layer),
                    },
                    property: candidate_target,
                });
            }
        }
        link_rows.push(LinkRowProjection {
            target,
            current,
            candidates,
        });
    }

    Ok(Some(SelectionProjection {
        layer,
        kind,
        transform: vec![
            position_row,
            scale_row,
            rotation_row,
            opacity_row,
            anchor_row,
        ],
        attrs: attrs_projection,
        masks: mask_rows,
        effects: effect_rows,
        text,
        audio,
        links: link_rows,
    }))
}

