//! SP-4(2026-08-23) 切り出し: `super`([`super::TransformField`] 等の field/Key
//! 識別・oracle・値の意味)を土台にした、drag-to-scrub の感度表・transient 状態
//! ([`FieldDragState`])・書き口の自由関数(`commit_inspector_field`/
//! `start_field_drag`/…)・TRANSFORM/APPEARANCE 行の view([`transform_row`])。
//! **中身は無改変** — 旧 `transform.rs` 後半(drag-to-scrub 以降)をそのまま
//! この位置へ移送しただけ(`super::mod` doc 参照)。

use motolii_core::{Fps, RationalTime};
use motolii_settings_pane::chrome::parse_number;
use motolii_store::{Document, Intent, LayerAttrsPatch, LayerId, Value};
use motolii_tokens_rs::{Colors, Dimensions};

use iced::widget::{row as row_widget, text};
use iced::{Element, Length};

use crate::projection::{RowValue, SelectionProjection, TransformRowProjection};
use crate::chrome::{bordered_row, blank_value_cell, key_glyph, sibling_gap_px, value_cell};
use crate::Message;

use super::{default_vec2, edited_value_track, next_value, property_id, FieldDraft, TransformField};

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
        TransformField::Opacity
        | TransformField::MaskOpacity(_)
        | TransformField::MaskExpansion(_) => 1.0,
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
    if let TransformField::MaskExpansion(mask) = field {
        return iced::widget::Id::from(format!("inspector-field-mask-{mask}-expansion"));
    }
    if let TransformField::EffectParam(effect, param) = field {
        return iced::widget::Id::from(format!(
            "inspector-field-effect-{effect}-{}",
            param.id
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
        TransformField::MaskOpacity(_)
        | TransformField::MaskExpansion(_)
        | TransformField::EffectParam(_, _) => {
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
        .size(dims.theme().text.body)
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
    .spacing(dims.theme().space.xs)
    .align_y(iced::alignment::Vertical::Center);

    // mock `.prow{border-bottom:var(--line) solid rgba(0,0,0,.35)}` は線化 D5
    // (裁定179 文法1)が上書き — 罫線なし([`row_band_style`] doc 参照)。
    bordered_row(content.into(), dims)
}
