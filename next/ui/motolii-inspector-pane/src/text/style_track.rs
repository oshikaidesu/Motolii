//! SP-4(2026-08-23) 切り出し: A-1b(裁定214/217)— Size/Line Height/Tracking の
//! **track** 書き口([`super::mod`] doc 参照)。**中身は無改変** — 旧
//! `text.rs` 末尾(`TextStyleField` から `track_field_tests` まで)をそのまま
//! 移送しただけ。[`super::value`] の `default_text_style`/`default_text_document`
//! を `use super::*;` で読む。

use motolii_settings_pane::chrome::parse_number;
use motolii_store::{
    Document, Fps, Intent, LayerId, PropertyId, RationalTime, TextDocument, TextDocumentStyle,
    TextStyleId, Value,
};

use super::*;

// ---------------------------------------------------------------------------
// A-1b: Size/Line Height/Tracking の track 書き口(裁定214/217)。
//
// **まだ crate 本体(`text_section`/`crate::Message`)へ結線していない**
// (module 冒頭 doc 参照)。ここに置くのは (1) この3フィールドの `PropertyId`
// への対応 (2) 型入力(Enter)版のコミット (3) Key 列 click の意味
// (4) drag-to-scrub の3関数——**いずれも `crate::transform` の track 意味論を
// 再実装せず呼ぶだけ**(`single_hold_track`/`edited_value_track`/
// `toggled_key_track`/`key_cell_state`/`KeyCellState`、裁定215「借りる」)。
// ---------------------------------------------------------------------------

/// track へ書く3フィールド。**id 付きではない** — この section は既定行
/// (`styles[0]`、裁定98)しか編集しない現行スコープをそのまま踏襲する
/// (`TextField::Size`/`LineHeight`/`Tracking` と同じ対象)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextStyleField {
    Size,
    LineHeight,
    Tracking,
}

impl TextStyleField {
    /// この field の `PropertyId`(裁定214 の器、A-1 が `text.rs`(store crate)
    /// へ足した名前付きコンストラクタをそのまま呼ぶ)。
    pub fn property_id(self, style: TextStyleId) -> PropertyId {
        match self {
            TextStyleField::Size => PropertyId::text_style_size(style),
            TextStyleField::LineHeight => PropertyId::text_style_line_height(style),
            TextStyleField::Tracking => PropertyId::text_style_tracking(style),
        }
    }

    /// track が無い時の基準値 — **静的フィールドをそのまま読む**
    /// (`StoreView::resolved_text_document` の「track が正本、無ければ静的値」の
    /// 「無ければ」側と同じ優先順位)。Line Height の `None`(Auto)は 0.0 を
    /// 基準にする(untracked Position/Anchor の既定「未指定 = 0」と同じ判断)。
    pub fn static_value(self, style: &TextDocumentStyle) -> f64 {
        match self {
            TextStyleField::Size => style.size as f64,
            TextStyleField::LineHeight => style.line_height.unwrap_or(0.0) as f64,
            TextStyleField::Tracking => style.tracking as f64,
        }
    }
}

/// `style` を document から引く(無ければ [`default_text_style`] — `TextField`
/// 経由の既存書き口と同じ「無ければ既定」)。
fn find_text_style(document: Option<&TextDocument>, style_id: TextStyleId) -> TextDocumentStyle {
    document
        .and_then(|document| document.styles.iter().find(|style| style.id == style_id))
        .cloned()
        .unwrap_or_else(default_text_style)
}

/// track 版の下書き(`FieldDraft`/`TextFieldDraft` と同じ形 — Enter まで store に
/// 触らない)。
#[derive(Clone, Debug, PartialEq)]
pub struct TextStyleTrackDraft {
    pub style: TextStyleId,
    pub field: TextStyleField,
    pub text: String,
}

/// TEXT section の track 版コミット(`crate::transform::commit_inspector_field`
/// と同じ意味論を [`crate::transform::edited_value_track`] 経由でそのまま
/// 借りる——キー無しなら静的値書き換え、キー持ちなら playhead へのキー
/// upsert)。下書きが無い・別 field/style の submit・選択が無い、のいずれも
/// `Ok(())`。
pub fn commit_text_style_track_field(
    doc: &mut Document,
    draft: &mut Option<TextStyleTrackDraft>,
    selection: Option<LayerId>,
    playhead_frame: i64,
    fps: Fps,
    style_id: TextStyleId,
    field: TextStyleField,
) -> Result<(), String> {
    let Some(taken) = draft.take() else {
        return Ok(());
    };
    if taken.style != style_id || taken.field != field {
        // 別の欄の submit(起こらないはずだが、安全側で下書きを戻す —
        // `commit_inspector_field`/`commit_text_field` と同じ判断)。
        *draft = Some(taken);
        return Ok(());
    }
    let Some(layer) = selection else {
        return Ok(());
    };
    let value =
        parse_number(&taken.text).ok_or_else(|| format!("数値として読めない: {}", taken.text))?;
    let property = field.property_id(style_id);
    let track = doc
        .view()
        .track(layer, &property)
        .map_err(|error| format!("track を読めない: {error}"))?;
    let new_track = crate::transform::edited_value_track(
        track.as_ref(),
        playhead_frame,
        fps,
        Value::F64(value),
    )?;
    doc.apply(Intent::SetTrack {
        layer,
        property,
        track: new_track,
    })
    .map_err(|error| format!("値を書けない: {error}"))
}

/// Multi-selection variant of [`commit_text_style_track_field`]. The draft is
/// consumed once, compatible text layers receive one `SetTrack` each, and the
/// shared bulk boundary commits the whole gesture as one undo step.
pub fn commit_text_style_track_field_for_layers(
    doc: &mut Document,
    draft: &mut Option<TextStyleTrackDraft>,
    selected_layers: &[LayerId],
    playhead_frame: i64,
    fps: Fps,
    style_id: TextStyleId,
    field: TextStyleField,
) -> Result<(), String> {
    let Some(taken) = draft.take() else {
        return Ok(());
    };
    if taken.style != style_id || taken.field != field {
        *draft = Some(taken);
        return Ok(());
    }
    let value =
        parse_number(&taken.text).ok_or_else(|| format!("数値として読めない: {}", taken.text))?;
    let property = field.property_id(style_id);

    crate::bulk::apply_to_selected_text_layers(doc, selected_layers, |layer, store| {
        let track = store
            .track(layer, &property)
            .map_err(|error| format!("track を読めない: {error}"))?;
        let new_track = crate::transform::edited_value_track(
            track.as_ref(),
            playhead_frame,
            fps,
            Value::F64(value),
        )?;
        if track.as_ref() == Some(&new_track) {
            Ok(None)
        } else {
            Ok(Some(Intent::SetTrack {
                layer,
                property: property.clone(),
                track: new_track,
            }))
        }
    })
}

/// Key 列 click の意味(`crate::transform::toggled_key_track` をそのまま
/// 呼ぶ)。track が無い時の基準値は [`TextStyleField::static_value`]。
pub fn toggle_text_style_key(
    doc: &mut Document,
    selection: Option<LayerId>,
    playhead_frame: i64,
    fps: Fps,
    style_id: TextStyleId,
    field: TextStyleField,
) -> Result<(), String> {
    let Some(layer) = selection else {
        return Ok(());
    };
    let property = field.property_id(style_id);
    let store = doc.view();
    let track = store
        .track(layer, &property)
        .map_err(|error| format!("track を読めない: {error}"))?;
    let document = store
        .text_document(layer)
        .map_err(|error| format!("text document を読めない: {error}"))?;
    let style = find_text_style(document.as_ref(), style_id);
    let current_value = Value::F64(field.static_value(&style));
    let new_track = crate::transform::toggled_key_track(
        track.as_ref(),
        playhead_frame,
        fps,
        current_value,
    )?;
    doc.apply(Intent::SetTrack {
        layer,
        property,
        track: new_track,
    })
    .map_err(|error| format!("キーを打てない: {error}"))
}

/// Inspector 値セルの drag-to-scrub、進行中の一時状態(`crate::transform::
/// FieldDragState` と同型 — Vec2 系を持たない分、text-style は常に scalar
/// なので `current_vec2`/`field: TransformField` は要らない)。
pub struct TextStyleDragState {
    field: TextStyleField,
    style: TextStyleId,
    layer: LayerId,
    playhead_frame: i64,
    fps: Fps,
    start_value: f64,
    origin_x: Option<f32>,
    moved: bool,
    last_value: Option<f64>,
}

/// drag 感度(px→単位、そのまま1:1)。**発明ではなく write-set の都合による
/// 複製** — `crate::transform::drag_step_per_pixel` は `TransformField` 専用の
/// private 関数なので再利用できず、text-style 用の値をここで独立に持つ
/// (RETURN 参照: 感度チューニング自体は実窓 φ 期の仕事、次発注)。
const TEXT_STYLE_DRAG_STEP: f64 = 1.0;

/// press — click か drag かはまだ未確定(`start_field_drag` と同じ形)。選択
/// なし、のいずれも黙って無視。press 時点の値は `resolved_text_document`
/// 相当(track があればその評価値、無ければ静的値)を読む——drag の起点が
/// 既にキー付き track の値と食い違う事故を防ぐ。
pub fn start_text_style_drag(
    drag: &mut Option<TextStyleDragState>,
    doc: &Document,
    selection: Option<LayerId>,
    style_id: TextStyleId,
    field: TextStyleField,
    playhead_frame: i64,
    fps: Fps,
) {
    if drag.is_some() {
        return; // 既に別の drag が進行中 — 多重起動しない
    }
    let Some(layer) = selection else {
        return;
    };
    let Ok(playhead_time) = RationalTime::try_from_frame(playhead_frame, fps) else {
        return;
    };
    let property = field.property_id(style_id);
    let store = doc.view();
    let start_value = match store.value_at(layer, &property, playhead_time) {
        Ok(Some(Value::F64(v))) => v,
        // track が無い/型不一致 — 静的値へ(裁定20 の応用)。
        _ => {
            let document = store.text_document(layer).ok().flatten();
            field.static_value(&find_text_style(document.as_ref(), style_id))
        }
    };
    *drag = Some(TextStyleDragState {
        field,
        style: style_id,
        layer,
        playhead_frame,
        fps,
        start_value,
        origin_x: None,
        moved: false,
        last_value: None,
    });
}

/// window 全体の cursor 移動(`continue_field_drag` と同じ形 — transient
/// overlay だけを毎 move 書き換える、edit timeline には触れない)。
pub fn continue_text_style_drag(
    doc: &mut Document,
    drag: &mut Option<TextStyleDragState>,
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
        return;
    }

    let factor = if fine {
        crate::transform::DRAG_SHIFT_FACTOR
    } else {
        1.0
    };
    let new_value = state.start_value + f64::from(delta_px) * TEXT_STYLE_DRAG_STEP * factor;
    let property = state.field.property_id(state.style);
    doc.set_transient(state.layer, property, Value::F64(new_value));
    if let Some(state) = drag.as_mut() {
        state.moved = true;
        state.last_value = Some(new_value);
    }
}

/// release(`finish_field_drag` と同じ形): 実際に動いていたら最後の transient
/// 値を1回の `Intent::SetTrack`(`edited_value_track` 経由)として確定し、
/// `clear_transient` で overlay を必ず外す。
pub fn finish_text_style_drag(
    doc: &mut Document,
    drag: &mut Option<TextStyleDragState>,
) -> Result<(), String> {
    let Some(state) = drag.take() else {
        return Ok(());
    };
    let property = state.field.property_id(state.style);
    if !state.moved {
        return Ok(());
    }
    let mut write_error = None;
    if let Some(value) = state.last_value {
        let base_track = doc.view().track(state.layer, &property).ok().flatten();
        match crate::transform::edited_value_track(
            base_track.as_ref(),
            state.playhead_frame,
            state.fps,
            Value::F64(value),
        ) {
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
        None => Ok(()),
    }
}

#[cfg(test)]
mod track_field_tests {
    use super::*;
    use motolii_store::{Composition, LayerMeta, LayerSource, LayerTiming};

    fn doc_with_text_layer() -> (Document, LayerId) {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 640,
            height: 360,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Text,
                    order: 0,
                    timing: LayerTiming {
                        start: 0,
                        duration: 100,
                        source_in: 0,
                        ..Default::default()
                    },
                },
            },
            Intent::SetTextDocument {
                layer,
                document: default_text_document(),
            },
        ])
        .unwrap();
        (doc, layer)
    }

    #[test]
    fn commit_writes_a_static_hold_track_when_no_track_exists_yet() {
        let (mut doc, layer) = doc_with_text_layer();
        let mut draft = Some(TextStyleTrackDraft {
            style: TextStyleId(0),
            field: TextStyleField::Size,
            text: "200".to_owned(),
        });
        commit_text_style_track_field(
            &mut doc,
            &mut draft,
            Some(layer),
            0,
            Fps::try_new(30, 1).unwrap(),
            TextStyleId(0),
            TextStyleField::Size,
        )
        .expect("書けるはず");
        assert!(draft.is_none());

        let value = doc
            .view()
            .value_at(layer, &PropertyId::text_style_size(TextStyleId(0)), RationalTime::ZERO)
            .unwrap();
        assert_eq!(value, Some(Value::F64(200.0)));
    }

    #[test]
    fn multi_selection_size_writes_text_layers_in_one_undo_and_skips_unsupported_layers() {
        let (mut doc, first) = doc_with_text_layer();
        let second = LayerId(2);
        let solid = LayerId(3);
        doc.apply_all([
            Intent::AddLayer(second),
            Intent::SetMeta {
                layer: second,
                meta: LayerMeta {
                    source: LayerSource::Text,
                    order: 1,
                    timing: LayerTiming::place(0, None, 100),
                },
            },
            Intent::SetTextDocument {
                layer: second,
                document: default_text_document(),
            },
            Intent::AddLayer(solid),
            Intent::SetMeta {
                layer: solid,
                meta: LayerMeta {
                    source: LayerSource::Solid {
                        rgba: [0, 0, 0, 255],
                        width: 64,
                        height: 64,
                    },
                    order: 2,
                    timing: LayerTiming::place(0, None, 100),
                },
            },
        ])
        .unwrap();
        let before = doc.edit_head();
        let mut draft = Some(TextStyleTrackDraft {
            style: TextStyleId(0),
            field: TextStyleField::Size,
            text: "200".to_owned(),
        });

        commit_text_style_track_field_for_layers(
            &mut doc,
            &mut draft,
            &[first, second, solid],
            0,
            Fps::try_new(30, 1).unwrap(),
            TextStyleId(0),
            TextStyleField::Size,
        )
        .unwrap();

        let property = PropertyId::text_style_size(TextStyleId(0));
        assert_eq!(doc.edit_head(), before + 1, "複数 layer の Size は1 undoに束ねる");
        assert_eq!(doc.view().value_at(first, &property, RationalTime::ZERO).unwrap(), Some(Value::F64(200.0)));
        assert_eq!(doc.view().value_at(second, &property, RationalTime::ZERO).unwrap(), Some(Value::F64(200.0)));
        assert_eq!(doc.view().track(solid, &property).unwrap(), None, "非TextへSizeを書かない");

        assert!(doc.undo(), "一括 Size は undo できる");
        assert_eq!(doc.view().track(first, &property).unwrap(), None);
        assert_eq!(doc.view().track(second, &property).unwrap(), None);
        assert_eq!(doc.view().track(solid, &property).unwrap(), None);
    }

    /// Line Height は `None`(Auto)の間、track が無ければ 0.0 を基準にする
    /// (`TextStyleField::static_value` の doc 参照)。
    #[test]
    fn line_height_commit_uses_zero_as_the_base_while_auto() {
        let (mut doc, layer) = doc_with_text_layer();
        assert_eq!(
            default_text_style().line_height,
            None,
            "既定 style は Auto(None)のはず(この試験の前提)"
        );
        toggle_text_style_key(
            &mut doc,
            Some(layer),
            0,
            Fps::try_new(30, 1).unwrap(),
            TextStyleId(0),
            TextStyleField::LineHeight,
        )
        .expect("キーを打てるはず");

        let value = doc
            .view()
            .value_at(
                layer,
                &PropertyId::text_style_line_height(TextStyleId(0)),
                RationalTime::ZERO,
            )
            .unwrap();
        assert_eq!(value, Some(Value::F64(0.0)), "Auto の基準値は 0.0 のはず");
    }

    #[test]
    fn drag_round_trip_writes_the_dragged_value_as_a_track() {
        let (mut doc, layer) = doc_with_text_layer();
        let fps = Fps::try_new(30, 1).unwrap();
        let mut drag: Option<TextStyleDragState> = None;
        start_text_style_drag(&mut drag, &doc, Some(layer), TextStyleId(0), TextStyleField::Size, 0, fps);
        assert!(drag.is_some());

        continue_text_style_drag(&mut doc, &mut drag, iced::Point::new(0.0, 0.0), false);
        continue_text_style_drag(&mut doc, &mut drag, iced::Point::new(40.0, 0.0), false);

        finish_text_style_drag(&mut doc, &mut drag).expect("確定できるはず");
        assert!(drag.is_none());

        let value = doc
            .view()
            .value_at(layer, &PropertyId::text_style_size(TextStyleId(0)), RationalTime::ZERO)
            .unwrap();
        let expected = default_text_style().size as f64 + 40.0 * TEXT_STYLE_DRAG_STEP;
        assert_eq!(value, Some(Value::F64(expected)));
    }

    #[test]
    fn drag_that_never_moves_leaves_no_track_and_no_transient_overlay() {
        let (mut doc, layer) = doc_with_text_layer();
        let fps = Fps::try_new(30, 1).unwrap();
        let mut drag: Option<TextStyleDragState> = None;
        start_text_style_drag(&mut drag, &doc, Some(layer), TextStyleId(0), TextStyleField::Size, 0, fps);
        finish_text_style_drag(&mut doc, &mut drag).expect("no-op のはず");

        assert_eq!(
            doc.view()
                .track(layer, &PropertyId::text_style_size(TextStyleId(0)))
                .unwrap(),
            None,
            "動かない drag が track を作ってしまった"
        );
    }
}
