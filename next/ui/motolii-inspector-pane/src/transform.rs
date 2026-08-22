//! TRANSFORM/APPEARANCE ── 値セル1本を動かす「型別 editor」の全機構。
//!
//! **持つ**: [`TransformField`](値セル片が動かす対象の識別。MASK/EFFECTS
//! section も `MaskOpacity`/`EffectParam` 拡張でここへ乗る)・[`KeyRow`]/
//! [`KeyCellState`]と3状態 oracle([`key_cell_state`])・click の意味
//! ([`toggled_key_track`])・値編集の意味([`edited_value_track`]、AE 作法の
//! playhead upsert)・[`FieldDraft`]/[`FieldDragState`] という2種の transient
//! 状態・それらを読み書きする自由関数群(`commit_inspector_field`/
//! `commit_inspector_name`/`start_field_drag`/`continue_field_drag`/
//! `finish_field_drag`/`cancel_field_interaction` ── 裁定160 切片8の crate doc
//! が名指しした「書ける物を持たない、が書き口は持つ」束はまとめてここに置く)・
//! drag-to-scrub の感度表([`dragged_value`]/[`drag_origin`])・表示整形
//! ([`format_number`]/[`display_number`])・TRANSFORM/APPEARANCE 行そのものの
//! view([`transform_row`] ── MASK/EFFECTS section もこれを再利用する)。
//!
//! **持たない**: MASK/EFFECTS 固有の一覧編集(`mask.rs`/`effects.rs`)・
//! TEXT/ATTRS 固有の書き口(`text.rs`/`attrs.rs`)・値セルの意匠そのもの
//! ([`crate::chrome`] ── `value_cell`/`HoverValueBox`)・投影の組み立て
//! ([`crate::projection`])。`GlowParam`(EFFECTS のパラメータカタログ)は
//! [`TransformField::EffectParam`]/[`KeyRow::EffectParam`] の型として要るので
//! `crate::effects` から読むだけ(定義はしない)。

use motolii_core::{Fps, RationalTime};
use motolii_settings_pane::chrome::parse_number;
use motolii_store::{
    property, Document, EffectId, Intent, Interp, Keyframe, KeyframeTrack, LayerAttrsPatch,
    LayerId, MaskId, PropertyId, StoreError, Value,
};
use motolii_tokens_rs::{Colors, Dimensions};

use iced::widget::{row as row_widget, text};
use iced::{Element, Length};

use crate::effects::GlowParam;
use crate::projection::{RowValue, SelectionProjection, TransformRowProjection};
use crate::chrome::{bordered_row, blank_value_cell, key_glyph, sibling_gap_px, value_cell};
use crate::Message;

// ---------------------------------------------------------------------------
// 型別 editor の対象 field
// ---------------------------------------------------------------------------

/// Transform 行が動かす field の識別。**`LayerId` を持たない** — 対象は常に
/// `Session::selection`(commit 時に読む)。選択が edit の合間に変わる稀なケースは
/// 「そのまま捨てる」で安全側に倒す(`motolii_shell::Shell::commit_inspector_field` 参照)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TransformField {
    PositionX,
    PositionY,
    PositionZ,
    ScaleX,
    ScaleY,
    Rotation,
    Opacity,
    AnchorX,
    AnchorY,
    /// この mask の不透明度(MASK section、B02 第1切片)。**新しい編集文法を
    /// 発明しない** — 既存の値セル文法(`FieldDraft`/drag-to-scrub/
    /// `commit_inspector_field` → `Intent::SetTrack`)へそのまま乗るために
    /// `TransformField` を拡張する形を採る。track 名は id から決まる
    /// (`PropertyId::mask_opacity`)ので、mask の並べ替え・削除で別の mask へ
    /// 付き直さない(store の同一性設計そのまま)。
    MaskOpacity(MaskId),
    /// この effect の param(EFFECTS section、B38 第3切片)。
    /// [`TransformField::MaskOpacity`] と同型の拡張 — 既存の値セル文法
    /// (`FieldDraft`/drag-to-scrub/`commit_inspector_field` → `Intent::SetTrack`)
    /// へそのまま乗る。track 名は id + param 名から決まる
    /// (`PropertyId::effect_param` — `effect.{id}.param.{name}`)ので、stack の
    /// 並べ替え・削除で別の effect へ付き直さない。param は [`GlowParam`]
    /// (既知 plugin のカタログ)に閉じる — store は plugin の param カタログを
    /// 知らない(裁定70、`ResolvedEffect::params` doc)ので、既定値を埋める
    /// 仕事はこの「plugin 定義を知っている層」の側([`GlowParam::default_value`])。
    EffectParam(EffectId, GlowParam),

    // ---- AUDIO section(B42、裁定184 型別 section 第4号) ----
    /// `property::LEVEL`(clip の音量、gain)。**mask/effect と違い per-id
    /// ではない** — layer につき高々1本の標準 property なので、
    /// Position/Rotation/Opacity と同じ「静的な named field」の形で足す
    /// (`TransformField::MaskOpacity`/`EffectParam` のような id 付き
    /// variant にしない)。
    Level,
    /// `property::PAN`。W3C `StereoPannerNode` と同じ -1.0(全振り左)〜
    /// 1.0(全振り右)、既定0.0(中央)。
    Pan,
    /// `property::FADE_IN`(秒)。0.0 = 無効。
    FadeIn,
    /// `property::FADE_OUT`(秒)。0.0 = 無効。
    FadeOut,
}

/// この field の store 上の property。標準 property は予約語でも空でもなく、
/// mask opacity は `PropertyId::mask_opacity`(構築が失敗し得ない形)、effect
/// param も名前が [`GlowParam::name`](静的・非予約語)に閉じるので実質失敗
/// し得ない — `motolii_shell::Shell` はこの `Result` を「コードの誤り」
/// として扱ってよい。
pub fn property_id(field: TransformField) -> Result<PropertyId, StoreError> {
    match field {
        TransformField::PositionX | TransformField::PositionY => PropertyId::new(property::POSITION),
        TransformField::PositionZ => PropertyId::new(property::POSITION_Z),
        TransformField::ScaleX | TransformField::ScaleY => PropertyId::new(property::SCALE),
        TransformField::Rotation => PropertyId::new(property::ROTATION),
        TransformField::Opacity => PropertyId::new(property::OPACITY),
        TransformField::AnchorX | TransformField::AnchorY => PropertyId::new(property::ANCHOR),
        TransformField::MaskOpacity(id) => Ok(PropertyId::mask_opacity(id)),
        TransformField::EffectParam(id, param) => PropertyId::effect_param(id, param.name()),
        TransformField::Level => PropertyId::new(property::LEVEL),
        TransformField::Pan => PropertyId::new(property::PAN),
        TransformField::FadeIn => PropertyId::new(property::FADE_IN),
        TransformField::FadeOut => PropertyId::new(property::FADE_OUT),
    }
}

/// 入力欄の下書き。**Document ではない** — commit(Enter)まで store に触らない。
#[derive(Clone, Debug, PartialEq)]
pub struct FieldDraft {
    pub field: TransformField,
    pub text: String,
}

/// `field` を編集した結果の新しい `Value`。Vec2 系(Position/Scale/Anchor の X/Y)は
/// **現在値の他成分を保つ** — X だけ書き換えて Y を 0 に潰す事故を防ぐ
/// (`current_vec2` は commit 側が `value_at` で読んだ今の値、無ければ [`default_vec2`])。
pub fn next_value(field: TransformField, input: f64, current_vec2: [f64; 2]) -> Value {
    match field {
        TransformField::PositionX | TransformField::ScaleX | TransformField::AnchorX => {
            Value::Vec2([input, current_vec2[1]])
        }
        TransformField::PositionY | TransformField::ScaleY | TransformField::AnchorY => {
            Value::Vec2([current_vec2[0], input])
        }
        TransformField::PositionZ | TransformField::Rotation => Value::F64(input),
        // effect param は表示 = store 単位(換算なし)。clamp もしない — 値域は
        // plugin(engine 側 shader)の意味で、editor が知ったかぶりしない
        // (engine `translate_glow_params` も clamp しない)。
        TransformField::EffectParam(_, _) => Value::F64(input),
        // 表示は % だが store は 0..1 の比(`property::OPACITY` の既定と同じ単位)。
        // mask opacity も同じ単位(比、`motolii_store::mask` doc「不透明度は比」)。
        TransformField::Opacity | TransformField::MaskOpacity(_) => {
            Value::F64((input / 100.0).clamp(0.0, 1.0))
        }
        // AUDIO(B42、裁定184 型別 section 第4号)。
        // Level: 表示は %(Speed 欄と同じ慣習)、store は倍率(1.0=等倍)。
        // Opacity と違い上限クランプはしない(ブースト = 100%超を許す、
        // gain に上限を課す意味論は store 側に無い) — 下限だけ 0(mute)。
        TransformField::Level => Value::F64((input / 100.0).max(0.0)),
        // Pan: store 自体が -1.0..1.0 の人間可読単位(doc 参照)なので変換なし。
        // engine `apply_pan_stereo` も clamp する(mix.rs doc)が、書き込み時点
        // でも安全側にクランプしておく(Opacity と同じ「commit 側で範囲を守る」判断)。
        TransformField::Pan => Value::F64(input.clamp(-1.0, 1.0)),
        // Fade In/Out: 秒、負の尺は意味を持たない。
        TransformField::FadeIn | TransformField::FadeOut => Value::F64(input.max(0.0)),
    }
}

/// track が無い(まだキーを打っていない)Vec2 property の既定値。Scale だけ等倍(1.0)、
/// 他は 0(`view.rs::resolve` の既定と同じ、裁定20 の応用)。
pub fn default_vec2(field: TransformField) -> [f64; 2] {
    match field {
        TransformField::ScaleX | TransformField::ScaleY => [1.0, 1.0],
        _ => [0.0, 0.0],
    }
}

/// 静的値を書く唯一の形。**1キー `Hold`**(発注書が名指しした流儀)。時刻は
/// `RationalTime::ZERO` — 1キーだけの track は `KeyframeTrack::eval` がどの時刻でも
/// 同じ値を返す(`t <= keys[0].t` と `t >= keys[last].t` が同じキーに落ちる、
/// `motolii-eval` の実装どおり)ので、時刻自体に意味は無い。
pub fn single_hold_track(value: Value) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: RationalTime::ZERO,
        value,
        interp: Interp::Hold,
        spatial: None,
    });
    track
}

// ---------------------------------------------------------------------------
// K1: Key 列 — 行→property の対応・3状態 oracle・click の意味(全部純関数)
// ---------------------------------------------------------------------------

/// Key セルが動かす行の識別。**行 = property 1本**(X/Y/Z 軸は現行モデルどおり
/// 1 track に畳まれている — 軸別キーは対象外)。Position 行の Key は
/// `property::POSITION`(Vec2)だけを対象にし、`POSITION_Z`(別 property)は
/// 含めない — 1 click = 1 `SetTrack` = 1 undo を保つため(RETURN に仕様として
/// 注記、逸脱ではない)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyRow {
    Position,
    Scale,
    Rotation,
    Opacity,
    Anchor,
    /// mask の不透明度行(MASK section、B02 第1切片)。既存 Key 列文法
    /// (3状態 oracle・`toggled_key_track`)へそのまま乗る —
    /// [`TransformField::MaskOpacity`] と同じ拡張の形。
    MaskOpacity(MaskId),
    /// effect param 行(EFFECTS section、B38 第3切片)。同上 —
    /// [`TransformField::EffectParam`] と同じ拡張の形。
    EffectParam(EffectId, GlowParam),
    /// effect の on/off 行(EFFECTS section、裁定213/214 で `EffectInstance::
    /// enabled` という静止 `bool` から `effect.{id}.enabled` の普通の track へ
    /// 移った——**Inspector に映る物は全て時間軸で評価できる**という裁定214の
    /// 帰結で、この行にも他の行と全く同じ3状態 Key oracle がそのまま乗る
    /// (`crate::effects::toggle_inspector_effect_bypass` が値そのものの
    /// 反転を書き、こちらは playhead へのキー打点/除去を書く——別の書き口
    /// だが同じ property を狙う点は他の行と同型)。
    EffectEnabled(EffectId),
    /// AUDIO section の Level 行(B42、裁定184 型別 section 第4号)。
    /// [`TransformField::Level`] と同じ「per-id ではない静的 field」の形。
    Level,
    /// AUDIO section の Pan 行。
    Pan,
    /// AUDIO section の Fade In 行。
    FadeIn,
    /// AUDIO section の Fade Out 行。
    FadeOut,
}

impl KeyRow {
    /// 標準 property 行の名前(旧 `property_name` — mask 行は id から動的に
    /// 決まるので、この対応表には載らない。[`key_row_property_id`] が正本)。
    fn static_property_name(self) -> Option<&'static str> {
        match self {
            Self::Position => Some(property::POSITION),
            Self::Scale => Some(property::SCALE),
            Self::Rotation => Some(property::ROTATION),
            Self::Opacity => Some(property::OPACITY),
            Self::Anchor => Some(property::ANCHOR),
            Self::Level => Some(property::LEVEL),
            Self::Pan => Some(property::PAN),
            Self::FadeIn => Some(property::FADE_IN),
            Self::FadeOut => Some(property::FADE_OUT),
            Self::MaskOpacity(_) | Self::EffectParam(_, _) | Self::EffectEnabled(_) => None,
        }
    }
}

/// この行の store 上の property。標準 property・mask opacity・effect param の
/// いずれも構築が失敗し得ない([`property_id`] と同じ理由 — 呼び手は `Result`
/// を「コードの誤り」として扱ってよい)。
pub fn key_row_property_id(row: KeyRow) -> Result<PropertyId, StoreError> {
    match row {
        KeyRow::MaskOpacity(mask) => Ok(PropertyId::mask_opacity(mask)),
        KeyRow::EffectParam(effect, param) => PropertyId::effect_param(effect, param.name()),
        KeyRow::EffectEnabled(effect) => Ok(PropertyId::effect_enabled(effect)),
        _ => PropertyId::new(
            row.static_property_name()
                .expect("mask/effect 以外の行は静的な property 名を持つ"),
        ),
    }
}

/// track がまだ無い行の既定値(`project` の各行の default と同じ値 —
/// Scale だけ等倍、Opacity(layer/mask とも)は store 単位の 0..1 で 1.0、
/// 他は 0)。静的値も無い行を初キー化する時の値の正本。
pub fn key_row_default_value(row: KeyRow) -> Value {
    match row {
        KeyRow::Position | KeyRow::Anchor => Value::Vec2([0.0, 0.0]),
        KeyRow::Scale => Value::Vec2([1.0, 1.0]),
        KeyRow::Rotation => Value::F64(0.0),
        KeyRow::Opacity | KeyRow::MaskOpacity(_) => Value::F64(1.0),
        // effect param の既定は plugin カタログ(= engine 既定の写し)から。
        KeyRow::EffectParam(_, param) => Value::F64(param.default_value()),
        // キーを打っていない = 既定で有効(`PropertyId::effect_enabled` doc)。
        KeyRow::EffectEnabled(_) => Value::Bool(true),
        // AUDIO(B42): Level は等倍・Pan は中央・Fade は無効、いずれも
        // `property::LEVEL`/`PAN`/`FADE_IN`/`FADE_OUT` の store 既定と同じ
        // (`motolii-store::property` doc 参照)。
        KeyRow::Level => Value::F64(1.0),
        KeyRow::Pan | KeyRow::FadeIn | KeyRow::FadeOut => Value::F64(0.0),
    }
}

/// Key セルの3状態(AE 文法 — 意図優先・裁定174)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCellState {
    /// track 無し(または正準静的表現 = [`single_hold_track`] の形)。空菱形 ◇。
    Static,
    /// track 有り・playhead のフレームにキー有り。実菱形 ◆(accent・強い面)。
    AtKey,
    /// track 有り・playhead のフレームにキー無し。実菱形 ◆(accent・弱い面 —
    /// mock `.keyButton.animated` の転写)。
    Between,
}

/// この track が「静的値の正準表現」([`single_hold_track`] が書く 1キー
/// `Hold` @`ZERO`)か。**キーが打たれているとは見なさない** — Inspector の
/// 静的値編集が書いた track を Key セルが「キー有り」と誤読すると、値を
/// 打っただけの行が playhead=0 で実菱形になってしまう。既知の限界: 利用者が
/// 本当に frame 0 に `Hold` キーを1個だけ置いた track も同じ形なので静的と
/// 読まれる(現行 UI にその作成経路は無い — Key click の先頭 insert は
/// `Linear`。RETURN に注記)。
fn is_canonical_static_track(track: &KeyframeTrack) -> bool {
    track.keys().len() == 1
        && matches!(track.keys()[0].interp, Interp::Hold)
        && track.keys()[0].t == RationalTime::ZERO
}

/// playhead のフレームに載っているキーの添字。照合は timeline と同じ
/// `try_to_frame_round`(frame 粒度 — `timeline_pane::write` の
/// `commit_key_frames`/`delete_selected_keys` と同じ規約、RationalTime の
/// 厳密一致にしない)。
fn key_index_at_frame(track: &KeyframeTrack, frame: i64, fps: Fps) -> Option<usize> {
    track
        .keys()
        .iter()
        .position(|key| key.t.try_to_frame_round(fps) == Ok(frame))
}

/// **3状態 oracle**(表示と click 判定の共通正本 — view と
/// [`toggled_key_track`] の両方がここを通るので、見た目と操作が食い違わない)。
pub fn key_cell_state(
    track: Option<&KeyframeTrack>,
    playhead_frame: i64,
    fps: Fps,
) -> KeyCellState {
    let Some(track) = track else {
        return KeyCellState::Static;
    };
    if track.keys().is_empty() || is_canonical_static_track(track) {
        return KeyCellState::Static;
    }
    if key_index_at_frame(track, playhead_frame, fps).is_some() {
        KeyCellState::AtKey
    } else {
        KeyCellState::Between
    }
}

/// **Key click の意味**: 今の track から、click 後に `Intent::SetTrack` で
/// 書くべき新しい track を作る(純関数 — Document には触れない)。
///
/// - **Static** → 現在の静的値([`single_hold_track`] が有ればその値、無ければ
///   呼び手が渡す `current_value`)で playhead 時刻にキー1個。track 先頭
///   insert の interp 既定は `Linear`(発注書)。
/// - **AtKey** → そのキーを除去。最後の1個なら track ごと静的化
///   ([`single_hold_track`] に消したキーの値を移す — AE のストップウォッチ
///   解除と等価、値は失わない。undo は `SetTrack` 経由なので効く)。
/// - **Between** → その時刻の `KeyframeTrack::eval` 値でキー追加。interp は
///   直前のキーの流儀を継ぎ、最初のキーより前への insert は `Linear`。
///   `spatial` は `None`(空間タンジェントの分割は対象外 — RETURN に注記)。
pub fn toggled_key_track(
    track: Option<&KeyframeTrack>,
    playhead_frame: i64,
    fps: Fps,
    current_value: Value,
) -> Result<KeyframeTrack, String> {
    let t = RationalTime::try_from_frame(playhead_frame, fps)
        .map_err(|error| format!("playhead を時刻へ写せない: {error}"))?;
    match key_cell_state(track, playhead_frame, fps) {
        KeyCellState::Static => {
            let value = track
                .filter(|tr| !tr.keys().is_empty())
                .map(|tr| tr.keys()[0].value.clone())
                .unwrap_or(current_value);
            let mut new_track = KeyframeTrack::new();
            new_track.insert(Keyframe {
                t,
                value,
                interp: Interp::Linear,
                spatial: None,
            });
            Ok(new_track)
        }
        KeyCellState::AtKey => {
            // state が AtKey なら track と添字は必ず有る(oracle と同じ判定)。
            let track = track.expect("AtKey なら track が有る");
            let index = key_index_at_frame(track, playhead_frame, fps)
                .expect("AtKey なら playhead 上のキーが有る");
            let removed_value = track.keys()[index].value.clone();
            let mut new_track = KeyframeTrack::new();
            for (i, key) in track.keys().iter().enumerate() {
                if i != index {
                    new_track.insert(key.clone());
                }
            }
            if new_track.keys().is_empty() {
                Ok(single_hold_track(removed_value))
            } else {
                Ok(new_track)
            }
        }
        KeyCellState::Between => {
            let track = track.expect("Between なら track が有る");
            let value = track.eval(t);
            let interp = track
                .keys()
                .iter()
                .rev()
                .find(|key| key.t < t)
                .map(|key| key.interp)
                .unwrap_or(Interp::Linear);
            let mut new_track = track.clone();
            new_track.insert(Keyframe {
                t,
                value,
                interp,
                spatial: None,
            });
            Ok(new_track)
        }
    }
}

/// この track が「実キーを持つ」か(空・track 無し・正準静的表現
/// [`is_canonical_static_track`] はどれも実キー無し)。値編集の意味
/// ([`edited_value_track`])と投影の `keyed`(accent 表示)の共通判定。
pub(crate) fn has_real_keys(track: Option<&KeyframeTrack>) -> bool {
    track.is_some_and(|tr| !tr.keys().is_empty() && !is_canonical_static_track(tr))
}

/// **値編集の意味**(AE 作法、2026-08-22 発注 — 利用者実窓指摘「キーが1つ
/// しか打てない」の根治): 値セルの Enter 確定・数値ドラッグ確定が
/// `Intent::SetTrack` で書くべき新しい track を作る(純関数)。
///
/// - **キー無し**(track 無し・空・正準静的表現)→ 従来どおり静的値の
///   書き換え([`single_hold_track`] — キーは生えない。キー化は Key 列 click
///   が明示的に行う)。
/// - **キー持ち**(実キー >= 1)→ **playhead 位置へのキー upsert**:
///   playhead のフレームにキーが有ればそのキーの値だけ更新(時刻・interp・
///   spatial は保つ)、無ければ新キー挿入。interp は Between 挿入
///   ([`toggled_key_track`])と同規則 — 直前キーの流儀を継ぎ、先頭は
///   `Linear`。`spatial` は `None`。**track を静的に戻さない** — これが
///   「値を変えるとキーが増える」の AE 文法。
pub fn edited_value_track(
    track: Option<&KeyframeTrack>,
    playhead_frame: i64,
    fps: Fps,
    value: Value,
) -> Result<KeyframeTrack, String> {
    if !has_real_keys(track) {
        return Ok(single_hold_track(value));
    }
    let track = track.expect("実キーが有るなら track も有る");
    let t = RationalTime::try_from_frame(playhead_frame, fps)
        .map_err(|error| format!("playhead を時刻へ写せない: {error}"))?;
    let mut new_track = track.clone();
    if let Some(index) = key_index_at_frame(track, playhead_frame, fps) {
        // 既存キーの値更新(個数不変)。時刻は**既存キーのもの**を保つ —
        // 照合は frame 丸め(`key_index_at_frame`)なので、丸め前の厳密時刻で
        // insert し直すと同フレームに2個生える事故になる。
        let existing = &track.keys()[index];
        new_track.insert(Keyframe {
            t: existing.t,
            value,
            interp: existing.interp,
            spatial: existing.spatial.clone(),
        });
    } else {
        let interp = track
            .keys()
            .iter()
            .rev()
            .find(|key| key.t < t)
            .map(|key| key.interp)
            .unwrap_or(Interp::Linear);
        new_track.insert(Keyframe {
            t,
            value,
            interp,
            spatial: None,
        });
    }
    Ok(new_track)
}

pub fn format_number(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

/// 値セルの**表示**文字数の上限(裁定169、**I-tokens(2026-08-22)で再較正**)。
/// 旧値6は `inspector_value_width=38px` 時代のアンカー(「1.000」5字は収まる・
/// 「960.000」7字は先頭末尾が clip される、いずれも 2026-08-21 深夜の実窓実測、
/// φ 検収)—`inspector_value_width` が I-tokens で 38→64px へ束ごと再転写された
/// ため、このアンカーも引き直す必要がある(発注書の指示どおり)。
///
/// **再較正の方法**: 実窓実測(φ 期のように実際のビルドを目で見る)がこの
/// レーンでは行えないため、`value_cell_legibility.rs` と同じ手口 —
/// `iced_test` で実フォント(0.15 fork の "Fira Sans" スタック、実窓と同一
/// レンダラ)を使い `text(content).size(dims.body_text)` の自然幅(`Target::Text`
/// の layout bounds、`iced_widget::text::layout` が `Length::Shrink` を
/// clamp する前の値)を直接測る決定論的な代替測定を採った(px からの近似換算は
/// 等幅仮定が崩れて信用できないという旧アンカーの理由はそのまま — ここでは
/// 「文字数から px への換算」ではなく「候補文字列ごとの実測 px」を使うので、
/// その理由に反しない)。新しいアンカー2点(数字のみ・小数点あり文字列の
/// 自然幅、実測 2026-08-22):
/// - 「1234567.000」(11字)の自然幅 58.861px は箱幅64pxの**内**(収まる)。
/// - 「12345678.000」(12字)の自然幅 64.922px は箱幅64pxの**外**(clip される)。
///
/// 旧アンカー(箱幅38px)も同じ相対マージンで境界に立っていた(6字
/// 「960.00」32.7px=箱の86%・7字「960.000」38.83px=箱の102%)— 新アンカー
/// (11字58.9px=箱の92%・12字64.9px=箱の101%)は同型の境界なので、この
/// 再較正は旧アンカーの選び方をそのまま踏襲した外挿(6→11は単純な比例
/// 64/38≈1.68倍では6×1.68≈10.1になるところ、実測の境界に合わせて11とした)。
/// **実窓での目視確認はこの発注の範囲外**(見送り — 利用者チェックリストへ
/// 追加候補として RETURN に記録)。
pub const MAX_VALUE_CELL_CHARS: usize = 11;

/// セルに収まる精度へ落とした**表示専用**の整形(裁定169)。field 既定の
/// `decimals` から始め、[`MAX_VALUE_CELL_CHARS`] を超える間 1 桁ずつ落とす
/// (最小 0 — 整数部だけでも超える値はそのまま出す: clip(true) が防波堤)。
/// **編集 draft は全精度のまま**([`value_cell`] の editing 分岐は
/// [`format_number`] を直接呼ぶ) — 表示は丸め、編集は真値、の分担。
/// モック出典: inspector-library の値サンプルは「24.0」「80.0」「2.50」と
/// 幅広値ほど短精度(AE の Position 1桁小数と同型)。
pub fn display_number(value: f64, decimals: usize) -> String {
    let mut d = decimals;
    loop {
        let s = format_number(value, d);
        if s.chars().count() <= MAX_VALUE_CELL_CHARS || d == 0 {
            return s;
        }
        d -= 1;
    }
}

/// [`project`] が組む行の decimals は field ごとに固定(Position/Scale/Anchor=3、
/// Rotation=1、Opacity=0)。click→type 編集の下書き初期値を作るのに要る
/// (`TransformRowProjection::decimals` を持つ行を毎回作り直さずに済む)。
pub fn field_decimals(field: TransformField) -> usize {
    match field {
        TransformField::Rotation => 1,
        TransformField::Opacity | TransformField::MaskOpacity(_) => 0,
        // Glow 既定(1.0/0.75/1.0)の桁がそのまま読める最小の桁。
        TransformField::EffectParam(_, _) => 2,
        // Level は Speed 欄と同じ1桁(% 表示)。Pan/Fade は store の生の刻み
        // (-1.00..1.00・秒)がそのまま読める2桁。
        TransformField::Level => 1,
        TransformField::Pan | TransformField::FadeIn | TransformField::FadeOut => 2,
        _ => 3,
    }
}


// ---------------------------------------------------------------------------
// drag-to-scrub — 1px の cursor 移動を「表示単位」の量へ写す(editor registry)
// ---------------------------------------------------------------------------

/// `field` の drag 感度(1px あたりの表示単位の変化量)。**tokens ではなくここに
/// 置く** — 寸法・色ではなく値の型に紐づく振る舞いなので([`motolii_tokens_rs`] は
/// 裁定117により寸法・色専用、Document 由来でも値の意味でもない — 型別 editor の
/// 一部としてここへ置く)。
///
/// | field                | 1px    | 理由 |
/// |-----------------------|--------|------|
/// | Position X/Y/Z        | 1.0    | 画面座標と1:1(After Effects 系の慣習) |
/// | Anchor X/Y             | 1.0    | Position と同じ空間感覚(単位が同じ) |
/// | Scale X/Y              | 0.01   | 既定1.0からの微調整域 — 1px=1.0では数pxで0や負へ振り切れる |
/// | Rotation                | 0.5(度)| 720px の drag で1周(360度)分 — 粗すぎず細かすぎない |
/// | Opacity                  | 1(%)   | 0〜100の全域が100pxの drag で動く |
/// | Level(B42)              | 1(%)   | Opacity と同じ感度 |
/// | Pan(B42)                | 0.01   | Scale と同じ微調整域(-1..1 の全域が100pxで動く) |
/// | Fade In/Out(B42)        | 0.02(秒)| 100pxで2秒動く(目安値、実窓較正はこの発注の範囲外) |
fn drag_step_per_pixel(field: TransformField) -> f64 {
    match field {
        TransformField::PositionX
        | TransformField::PositionY
        | TransformField::PositionZ
        | TransformField::AnchorX
        | TransformField::AnchorY => 1.0,
        TransformField::ScaleX | TransformField::ScaleY => 0.01,
        TransformField::Rotation => 0.5,
        // effect param は Scale と同じ微調整域(既定 0.75〜1.0 前後の値 —
        // 1px=1.0 では数 px で意味域を振り切る、Scale と同じ理由)。
        TransformField::EffectParam(_, _) => 0.01,
        // mask opacity は layer Opacity と同じ感度(0〜100% が 100px で動く)。
        TransformField::Opacity | TransformField::MaskOpacity(_) => 1.0,
        // Level は Opacity と同じ感度(% 表示、100px で100%動く)。
        TransformField::Level => 1.0,
        // Pan は Scale と同じ微調整域(-1..1 の全域が100pxで動く)。
        TransformField::Pan => 0.01,
        // Fade は秒。100pxで2秒動く(0〜数秒が典型域という想定、目安値 —
        // 実窓較正はこの発注の範囲外、RETURN 参照)。
        TransformField::FadeIn | TransformField::FadeOut => 0.02,
    }
}

/// Shift 押下中の微調整係数(発注書「Shift+drag = 1/10 微調整」)。
pub const DRAG_SHIFT_FACTOR: f64 = 0.1;

/// press 開始点からの x 差分(px)を「表示単位」の新しい値へ写す。`fine` は
/// Shift 押下中かどうか([`DRAG_SHIFT_FACTOR`] を掛ける)。純粋関数 — 呼び手
/// (`continue_field_drag`)が結果を `next_value` へ渡して store の
/// `Value` へ変換する。
pub fn dragged_value(field: TransformField, start_value: f64, delta_px: f32, fine: bool) -> f64 {
    let step = drag_step_per_pixel(field);
    let factor = if fine { DRAG_SHIFT_FACTOR } else { 1.0 };
    start_value + f64::from(delta_px) * step * factor
}

/// drag(または click→type 編集)を始める前に読む、`field` の現在値。
/// **投影から読むだけ** — `project` が計算した表示単位の値をそのまま使う
/// (Opacity の % 換算などを2箇所に書かない)。対応する field が投影に無い
/// (または `editable=false` の穴)なら `None`(呼び手はドラッグも編集も
/// 始めない)。present な成分は常に editable(Q0、2026-08-22 発注)なので、
/// キー持ち track もここを通って drag/type 編集できる。
///
/// 戻り値の第2要素は Vec2 系(Position/Scale/Anchor)の「動かさない方の成分」
/// ([`next_value`] にそのまま渡す) — scalar 系(Z/Rotation/Opacity)では未使用
/// (`[0.0, 0.0]` のダミー)。
pub fn drag_origin(
    selection: &SelectionProjection,
    field: TransformField,
) -> Option<(f64, [f64; 2])> {
    for row in &selection.transform {
        match &row.value {
            RowValue::Vector(components) => {
                if let Some(slot) = components.iter().find(|s| s.field == Some(field)) {
                    let current_vec2 = [components[0].value, components[1].value];
                    return slot.editable.then_some((slot.value, current_vec2));
                }
            }
            RowValue::Scalar(slot) => {
                if slot.field == Some(field) {
                    return slot.editable.then_some((slot.value, [0.0, 0.0]));
                }
            }
        }
    }
    // MASK section の opacity 行(scalar のみ — mask に Vec2 の値セルは無い)。
    for mask_row in &selection.masks {
        if let RowValue::Scalar(slot) = &mask_row.opacity.value {
            if slot.field == Some(field) {
                return slot.editable.then_some((slot.value, [0.0, 0.0]));
            }
        }
    }
    // EFFECTS section の param 行(scalar のみ — Glow カタログに Vec2 は無い)。
    for effect_row in &selection.effects {
        for param_row in &effect_row.params {
            if let RowValue::Scalar(slot) = &param_row.value {
                if slot.field == Some(field) {
                    return slot.editable.then_some((slot.value, [0.0, 0.0]));
                }
            }
        }
    }
    // AUDIO section の4行(scalar のみ — 4行とも per-id ではない layer 単位
    // の標準 property、`Vec` ではなく named field なのでループは固定4回)。
    if let Some(audio) = &selection.audio {
        for row in [&audio.level, &audio.pan, &audio.fade_in, &audio.fade_out] {
            if let RowValue::Scalar(slot) = &row.value {
                if slot.field == Some(field) {
                    return slot.editable.then_some((slot.value, [0.0, 0.0]));
                }
            }
        }
    }
    None
}

/// text_input へ確定的な `Id` を割る — click→type 編集へ切り替わった直後、
/// `iced::widget::operation::focus` でこのセルへフォーカスを戻すために要る
/// (mouse_area は press を own できても、フォーカスは text_input 自身の仕事
/// — click 直後にはまだ text_input が木に無いので自動フォーカスされない)。
pub fn field_input_id(field: TransformField) -> iced::widget::Id {
    // mask opacity / effect param は id が動的(枚数は静的に決まらない)。fork の
    // `Id::new` は `&'static str` 限定だが `From<String>`(Cow::Owned)がある。
    if let TransformField::MaskOpacity(mask) = field {
        return iced::widget::Id::from(format!("inspector-field-mask-{mask}-opacity"));
    }
    if let TransformField::EffectParam(effect, param) = field {
        return iced::widget::Id::from(format!(
            "inspector-field-effect-{effect}-{}",
            param.name()
        ));
    }
    let name: &'static str = match field {
        TransformField::PositionX => "inspector-field-position-x",
        TransformField::PositionY => "inspector-field-position-y",
        TransformField::PositionZ => "inspector-field-position-z",
        TransformField::ScaleX => "inspector-field-scale-x",
        TransformField::ScaleY => "inspector-field-scale-y",
        TransformField::Rotation => "inspector-field-rotation",
        TransformField::Opacity => "inspector-field-opacity",
        TransformField::AnchorX => "inspector-field-anchor-x",
        TransformField::AnchorY => "inspector-field-anchor-y",
        TransformField::Level => "inspector-field-level",
        TransformField::Pan => "inspector-field-pan",
        TransformField::FadeIn => "inspector-field-fade-in",
        TransformField::FadeOut => "inspector-field-fade-out",
        TransformField::MaskOpacity(_) | TransformField::EffectParam(_, _) => {
            unreachable!("上の early return が拾う")
        }
    };
    iced::widget::Id::new(name)
}

// ---------------------------------------------------------------------------
// drag-to-scrub、進行中の一時状態(裁定160 切片8で lib.rs から移設)。
// ---------------------------------------------------------------------------

/// Inspector 値セルの drag-to-scrub、進行中の一時状態。**Document ではない**
/// (`FieldDraft` と同じ「pane が持つ transient」の形)。値そのものの置き場は
/// `Document` の transient overlay(`Document::set_transient`)— ここは overlay
/// の宛先と、click/drag 判定・確定時の Intent 組み立てに要る最小限だけを持つ。
///
/// **置き場(`motolii_shell::Shell::inspector_drag: Option<FieldDragState>`)は
/// 移設していない**(crate doc 参照) — 型定義とこれを読み書きする自由関数
/// ([`start_field_drag`]/[`continue_field_drag`]/[`finish_field_drag`]/
/// [`cancel_field_interaction`])だけがここにある。
pub struct FieldDragState {
    field: TransformField,
    layer: LayerId,
    /// press 時点の playhead(frame)と fps。確定時のキー upsert
    /// ([`edited_value_track`])の宛先 — drag の起点値は press 時点の
    /// playhead で読んだ値なので、確定の宛先も同じ時刻に固定する。
    playhead_frame: i64,
    fps: Fps,
    /// press 時点の表示単位の値([`drag_origin`] が投影から読む)。確定
    /// Intent・Esc(overlay を外すだけで使わない)双方が参照する起点。
    start_value: f64,
    /// Vec2 系(Position/Scale/Anchor)の動かさない方の成分。scalar 系では未使用。
    current_vec2: [f64; 2],
    /// 最初の `PointerMoved` で確定する基準 x(window 座標)。`None` の間は
    /// click か drag かまだ未確定 — 確定前に値を動かすと press 直後の
    /// sub-pixel な揺れで値が動いてしまう。
    origin_x: Option<f32>,
    /// 少なくとも1回 `set_transient` を呼んだか。release 時の click/drag 判定と、
    /// Esc で overlay を外す必要があるかどうかの両方に使う。
    moved: bool,
    /// 直近の `set_transient` に渡した値。release の確定 Intent はこれをそのまま
    /// 1回 `apply` する — pointer の最終座標を release 時に持っていない
    /// (`PointerReleased` は位置を運ばない)ので、最後に計算した値をここへ
    /// 持ち回す。`moved` が `false` の間は未使用。
    last_value: Option<Value>,
}

// ---------------------------------------------------------------------------
// 書き口(裁定160 切片8で lib.rs から移設): `&mut Document`/`&mut Option<_>`
// 下書き・`&mut Option<FieldDragState>` を明示引数で受け取る自由関数。
// 呼び出し側(`motolii_shell::Shell::update_inspector` + 個々の glue メソッド)
// が `self.doc`/`self.session.selection` 等をそのまま貸す — pane crate は
// `Shell` を持てない(root → pane の一方向依存、循環禁止)ための形
// (`motolii_settings_pane` の同型セクションと同じ判断)。
// ---------------------------------------------------------------------------

/// Inspector の Transform 行 — 下書きを確定して1回の `Intent::SetTrack` を出す
/// (1 gesture = 1 undo)。数値として読めない・書き込み失敗は**黙って消さず**
/// `Err` の理由文を返す(M13、呼び出し側が status 帯へ渡す)。下書きが無い・
/// 別 field の submit・選択が無い、のいずれも `Ok(())`(何もしない)。
///
/// **書く track の意味は [`edited_value_track`]**(AE 作法、2026-08-22 発注):
/// キー無しなら静的値の書き換え、キー持ちなら playhead へのキー upsert。
/// 旧規則「2キー以上は編集拒否」は撤去した(Q0 — 値セルは常に編集可能)。
pub fn commit_inspector_field(
    doc: &mut Document,
    draft: &mut Option<FieldDraft>,
    selection: Option<LayerId>,
    playhead_frame: i64,
    fps: Fps,
    field: TransformField,
) -> Result<(), String> {
    let Some(taken) = draft.take() else {
        return Ok(());
    };
    if taken.field != field {
        // 別の field の submit(起こらないはずだが、安全側で下書きを戻す)。
        *draft = Some(taken);
        return Ok(());
    }
    let Some(layer) = selection else {
        return Ok(());
    };
    let Some(input) = parse_number(&taken.text) else {
        return Err(format!("数値として読めない: {}", taken.text));
    };
    let Ok(property) = property_id(field) else {
        return Err("property を作れない".to_owned());
    };
    let playhead_time = RationalTime::try_from_frame(playhead_frame, fps)
        .map_err(|error| format!("playhead を時刻へ写せない: {error}"))?;

    let store = doc.view();
    let track = store.track(layer, &property).ok().flatten();
    let current_vec2 = match store.value_at(layer, &property, playhead_time) {
        Ok(Some(Value::Vec2(v))) => v,
        _ => default_vec2(field),
    };
    let value = next_value(field, input, current_vec2);
    let new_track = edited_value_track(track.as_ref(), playhead_frame, fps, value)?;
    doc.apply(Intent::SetTrack {
        layer,
        property,
        track: new_track,
    })
    .map_err(|error| format!("値を書けない: {error}"))
}

/// Attrs の Name 欄 — 下書きを確定して1回の `Intent::SetAttrs` を出す。下書きが
/// 無い・選択が無い、のいずれも `Ok(())`(何もしない)。
pub fn commit_inspector_name(
    doc: &mut Document,
    draft: &mut Option<String>,
    selection: Option<LayerId>,
) -> Result<(), String> {
    let Some(text) = draft.take() else {
        return Ok(());
    };
    let Some(layer) = selection else {
        return Ok(());
    };
    let patch = LayerAttrsPatch {
        name: Some(text),
        ..Default::default()
    };
    doc.apply(Intent::SetAttrs { layer, patch })
        .map_err(|error| format!("名前を書けない: {error}"))
}

/// 値セルの press — click か drag かはまだ未確定(`FieldDragState::origin_x` が
/// `None` のまま)。選択なし・対応する field が投影に無い、のいずれも黙って
/// 無視([`commit_inspector_field`] と同じ二重の柵)。既に別の drag が進行中
/// なら多重起動しない。`playhead_frame`/`fps` は press 時点の物を捕まえて
/// 確定([`finish_field_drag`] のキー upsert)の宛先に固定する。
pub fn start_field_drag(
    drag: &mut Option<FieldDragState>,
    selection: Option<LayerId>,
    projection: Option<&SelectionProjection>,
    field: TransformField,
    playhead_frame: i64,
    fps: Fps,
) {
    if drag.is_some() {
        return; // 既に別の drag が進行中 — 多重起動しない
    }
    let Some(layer) = selection else {
        return;
    };
    let Some(selection_projection) = projection else {
        return;
    };
    let Some((start_value, current_vec2)) = drag_origin(selection_projection, field) else {
        return;
    };
    *drag = Some(FieldDragState {
        field,
        layer,
        playhead_frame,
        fps,
        start_value,
        current_vec2,
        origin_x: None,
        moved: false,
        last_value: None,
    });
}

/// window 全体の cursor 移動。drag が armed/dragging でなければ即 no-op。
/// **1px = 感度表の刻み**([`dragged_value`])。press 直後の最初の move は基準点を
/// 確定するだけで値は動かさない(そうしないと press した瞬間の sub-pixel な
/// 揺れで値が動く)。
///
/// **transient overlay(`Document::set_transient`)を毎 move 呼ぶだけ** —
/// `edit timeline` には一切触れないので、undo/redo の意味論(`revision()`)は
/// drag 中ずっと不変。
pub fn continue_field_drag(
    doc: &mut Document,
    drag: &mut Option<FieldDragState>,
    point: iced::Point,
    fine: bool,
) {
    let Some(state) = drag.as_mut() else {
        return;
    };
    let Some(origin_x) = state.origin_x else {
        state.origin_x = Some(point.x);
        return;
    };

    let delta_px = point.x - origin_x;
    if delta_px == 0.0 && !state.moved {
        return; // まだ実質的に動いていない — click 候補のまま据え置く
    }

    let field = state.field;
    let layer = state.layer;
    let start_value = state.start_value;
    let current_vec2 = state.current_vec2;

    let Ok(property) = property_id(field) else {
        return;
    };
    let new_display = dragged_value(field, start_value, delta_px, fine);
    let value = next_value(field, new_display, current_vec2);

    doc.set_transient(layer, property, value.clone());
    if let Some(state) = drag.as_mut() {
        state.moved = true;
        state.last_value = Some(value);
    }
}

/// 左クリック release(window 全体から)。**drag が実際に動いていたら確定**:
/// 最後の transient 値そのものを1回の本編集 `Intent` として `apply` してから
/// `clear_transient`(1 gesture = 1 undo、overlay を残さない)。
///
/// **`Ok(Some(field))`**: drag が動かないまま release された(click) —
/// 呼び出し側は `field` で type 編集へ切り替える
/// (`motolii_shell::Shell::enter_field_editing`、focus task の構築は crate doc
/// のとおり root 側の仕事)。**`Ok(None)`**: drag が実際に動いた(確定済み)、
/// または drag 自体が無かった — 呼び出し側の追加作業なし。**`Err`**: 確定
/// Intent の書き込みが失敗した理由文(呼び出し側が status 帯へ渡す —
/// `clear_transient` は書き込み失敗時も必ず呼ぶ、元実装と同じ、overlay を
/// 残さないため)。
pub fn finish_field_drag(
    doc: &mut Document,
    drag: &mut Option<FieldDragState>,
) -> Result<Option<TransformField>, String> {
    let Some(state) = drag.take() else {
        return Ok(None);
    };
    if !state.moved {
        return Ok(Some(state.field));
    }
    let Ok(property) = property_id(state.field) else {
        // 起こらないはず(`moved` は property_id が通った move でしか立たない)
        // だが、安全側で overlay だけは残さず抜ける実害は無い(次の press で
        // 上書きされる)。
        return Ok(None);
    };
    let mut write_error = None;
    if let Some(value) = state.last_value {
        // 確定の track も値セル Enter と同じ意味([`edited_value_track`] —
        // キー無しは静的書き換え・キー持ちは press 時点の playhead へ upsert)。
        // transient overlay は `track()` には映らないので、ここで読むのは
        // drag 開始前の本 track そのもの。
        let base_track = doc.view().track(state.layer, &property).ok().flatten();
        match edited_value_track(base_track.as_ref(), state.playhead_frame, state.fps, value) {
            Ok(track) => {
                if let Err(error) = doc.apply(Intent::SetTrack {
                    layer: state.layer,
                    property: property.clone(),
                    track,
                }) {
                    write_error = Some(format!("値を書けない: {error}"));
                }
            }
            Err(error) => write_error = Some(error),
        }
    }
    doc.clear_transient(state.layer, &property);
    match write_error {
        Some(error) => Err(error),
        None => Ok(None),
    }
}

/// Esc: 進行中の drag があれば `clear_transient` で復元し(overlay は edit
/// timeline に一切触れていないので undo/redo 履歴は最初から無傷)、無ければ
/// typing 下書き(値セル)を破棄する。
///
/// **`true`** を返したら呼び出し側はここで終わり(元の `motolii_shell::Shell::
/// cancel_inspector_interaction` の早期 return を保つ — 名前欄・Settings 下書き
/// へは踏み込まない、それらは Inspector pane の write-set 外)。
pub fn cancel_field_interaction(
    doc: &mut Document,
    drag: &mut Option<FieldDragState>,
    field_draft: &mut Option<FieldDraft>,
) -> bool {
    if let Some(state) = drag.take() {
        if state.moved {
            if let Ok(property) = property_id(state.field) {
                doc.clear_transient(state.layer, &property);
            }
        }
        return true;
    }
    field_draft.take().is_some()
}

/// 発注書の固定列グリッド `Property | X | Y | Z | Key` = `1fr + 3×value幅 + hit`。
/// scalar 行(Opacity)は mock どおり3列目(Z の位置)へ値を置き、残り2列は
/// 空箱で埋める([`blank_value_cell`] — `absent_component` の「このモデルに
/// 無い軸」とは別の意味で、単に scalar 行が3値グリッドに収まるための穴埋め)。
///
/// **裁定183(taffy 転写)は今回ここへ配線していない**([`property_row_css`]
/// の doc「FINDING」参照 — 150%実測で `motolii-taffy` の既定 rounding が
/// 既存の shell 側柵を壊すことを発見したため、部分適用として CSS 宣言+oracle
/// だけを確立した)。並べ方は引き続き旧来どおり手組みの `row_widget!`。
pub(crate) fn transform_row(
    row_projection: &TransformRowProjection,
    field_draft: Option<&FieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let label = text(row_projection.label)
        .size(dims.body_text)
        .color(colors.text_primary)
        .width(Length::Fill);

    let value_cells: Vec<Element<'static, Message>> = match &row_projection.value {
        RowValue::Vector(components) => components
            .iter()
            .map(|slot| value_cell(slot, field_draft, row_projection.decimals, dims, colors))
            .collect(),
        RowValue::Scalar(slot) => vec![
            blank_value_cell(dims, colors),
            blank_value_cell(dims, colors),
            value_cell(slot, field_draft, row_projection.decimals, dims, colors),
        ],
    };

    let content = row_widget![
        label,
        // 裁定168 施工: 値セル同士の gap は裁定167 下段(0.075×行高)へ —
        // 違反(B)「960.000540.000」と読める密着の緩衝(値セル自体の幅
        // 38px は変えない、[`value_cell`]/`inspector_pixel_fence.rs` 参照)。
        row_widget(value_cells).spacing(sibling_gap_px(dims.inspector_row_height)),
        key_glyph(row_projection.key, dims, colors), // Key 列(K1 — 結線済み)。
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    // mock `.prow{border-bottom:var(--line) solid rgba(0,0,0,.35)}` は線化 D5
    // (裁定179 文法1)が上書き — 罫線なし([`row_band_style`] doc 参照)。
    bordered_row(content.into(), dims)
}

