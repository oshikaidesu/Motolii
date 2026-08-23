//! ドラッグ中のライブプレビュー(SP-2 分割、`projection.rs` 773-976行を移設)。
//! **中身は無改変** — 可視性のみ `pub(super)` → `pub(crate)`(元は
//! `projection` 直下の兄弟モジュールで `pub(super)` = crate root まで見えて
//! いたが、`preview` は `projection` の子になったので同じ到達範囲を
//! `pub(crate)` で保つ。呼び手(`lib.rs`)は `projection::apply_clip_preview`
//! のまま — `mod.rs` の re-export 参照)。

use super::*;
#[cfg(test)]
use super::tree::row_selected;

// ---------------------------------------------------------------------------
// ドラッグ中のライブプレビュー(第2波T5、正典 §5.5「プレビューは毎フレーム」)。
// ---------------------------------------------------------------------------

/// クリップ drag のプレビューを行の投影へ焼き込む(`TimelinePane::
/// with_clip_preview` から呼ばれる純関数)。**`layer` が一致する1行だけ**
/// `start`/`duration` を置き換え、[`RowProjection::dragging`] を立てる —
/// 一致する行が無ければ黙って素通り(発明しない)。`preview` が `None` なら
/// `rows` をそのまま返す(通常描画、呼び出し側で分岐を増やさない)。
///
/// `TimelineDragState`(`crate::lib` 側の pane-local transient)を直接は
/// 知らない — 呼び出し側(`Shell::build_timeline_pane`)が `(layer,
/// drag.preview)` へ薄く写して渡す。EXACT TARGET 1 の「プレビュー後timing」。
pub(crate) fn apply_clip_preview(
    rows: Vec<RowProjection>,
    preview: Option<(LayerId, LayerTiming)>,
) -> Vec<RowProjection> {
    let Some((layer, timing)) = preview else {
        return rows;
    };
    rows.into_iter()
        .map(|mut row| {
            if row.id == layer {
                row.start = timing.start;
                row.duration = timing.duration;
                row.dragging = true;
            }
            row
        })
        .collect()
}

/// キー drag/リタイムのプレビューを property 行へ焼き込む(`TimelinePane::
/// with_key_preview` から呼ばれる純関数)。`preview` は「掴んだ瞬間の
/// selector(layer/property/**旧**frame) → 新 frame」のペア列
/// (`TimelineKeyDragState::origins` と `preview` を呼び出し側が index で
/// ゆわえて渡す — この関数自体は `TimelineKeyDragState` を知らない)。
/// 一致する `(layer, property, frame)` の key だけ frame を置き換える —
/// 一致しなければ黙って素通り。`preview` が `None`(非ドラッグ中)なら
/// `rows` をそのまま返す。
///
/// リタイム中は選択キー全部が `origins`/`preview` に並ぶので、この1関数で
/// move/retime どちらのプレビューも同じ経路を通る(EXACT TARGET 4)。
pub(crate) fn apply_key_preview(
    rows: Vec<PropertyRowProjection>,
    preview: Option<&[(KeySelector, i64)]>,
) -> Vec<PropertyRowProjection> {
    let Some(preview) = preview else {
        return rows;
    };
    rows.into_iter()
        .map(|mut row| {
            for key in &mut row.keys {
                if let Some(&(_, new_frame)) = preview.iter().find(|(selector, _)| {
                    selector.layer == row.layer
                        && selector.property == row.property
                        && selector.frame == key.frame
                }) {
                    key.frame = new_frame;
                }
            }
            row
        })
        .collect()
}

#[cfg(test)]
mod preview_tests {
    use super::*;
    use motolii_store::Speed;

    fn row(id: u64, start: i64, duration: i64) -> RowProjection {
        RowProjection {
            id: LayerId(id),
            name: String::new(),
            hidden: false,
            solo: false,
            locked: false,
            label_color: None,
            start,
            duration,
            selected: false,
            dragging: false,
            depth: 0,
            has_children: false,
            children_open: true,
        }
    }

    fn timing(start: i64, duration: i64) -> LayerTiming {
        LayerTiming { start, duration, source_in: 0, speed: Speed::NORMAL }
    }

    /// **オラクル(赤→緑)**: `preview` の layer に一致する行だけ start/duration が
    /// 置き換わり `dragging` が立つ。他の行は無傷。
    #[test]
    fn apply_clip_preview_replaces_only_the_matching_layer() {
        let rows = vec![row(1, 0, 50), row(2, 60, 20)];
        let out = apply_clip_preview(rows, Some((LayerId(2), timing(40, 10))));

        assert_eq!(out[0], row(1, 0, 50), "掴んでいない行が動いている");
        assert_eq!(out[1].start, 40);
        assert_eq!(out[1].duration, 10);
        assert!(out[1].dragging, "掴んでいる行の dragging が立っていない");
        assert!(!out[0].dragging);
    }

    /// `preview == None`(非ドラッグ中)は素通り — 呼び出しが増えても通常描画を
    /// 汚さない。
    #[test]
    fn apply_clip_preview_none_is_a_passthrough() {
        let rows = vec![row(1, 0, 50)];
        let out = apply_clip_preview(rows.clone(), None);
        assert_eq!(out, rows);
    }

    fn key_row(layer: LayerId, property: PropertyId, frames: &[i64]) -> PropertyRowProjection {
        PropertyRowProjection {
            layer,
            property,
            keys: frames
                .iter()
                .map(|&frame| PropertyKeyProjection { frame, selected: true })
                .collect(),
        }
    }

    /// **オラクル(赤→緑)**: 一致する selector(旧 frame)の key だけ新 frame へ
    /// 置き換わる。同じ property の他 key は無傷。
    #[test]
    fn apply_key_preview_replaces_only_the_matching_selector() {
        let layer = LayerId(1);
        let property = PropertyId::new("opacity").expect("opacity は予約語ではない");
        let rows = vec![key_row(layer, property.clone(), &[10, 20])];
        let pairs = [(KeySelector { layer, property: property.clone(), frame: 10 }, 15)];

        let out = apply_key_preview(rows, Some(&pairs));

        assert_eq!(out[0].keys[0].frame, 15, "一致した key の frame が置き換わっていない");
        assert_eq!(out[0].keys[1].frame, 20, "一致していない key まで動いている");
    }

    /// リタイムのように複数 key を同時にプレビューする形(EXACT TARGET 4)。
    #[test]
    fn apply_key_preview_moves_every_paired_key_in_one_pass() {
        let layer = LayerId(1);
        let property = PropertyId::new("opacity").expect("opacity は予約語ではない");
        let rows = vec![key_row(layer, property.clone(), &[0, 20, 70, 90])];
        let pairs = [
            (KeySelector { layer, property: property.clone(), frame: 20 }, 20),
            (KeySelector { layer, property: property.clone(), frame: 70 }, 41),
            (KeySelector { layer, property: property.clone(), frame: 90 }, 50),
        ];

        let out = apply_key_preview(rows, Some(&pairs));

        assert_eq!(
            out[0].keys.iter().map(|k| k.frame).collect::<Vec<_>>(),
            vec![0, 20, 41, 50],
            "選択キー全部が比例位置でプレビューされていない"
        );
    }

    /// 一致しない selector(別 layer/property/frame)は黙って無視 — 発明しない。
    #[test]
    fn apply_key_preview_ignores_a_selector_that_does_not_match_any_key() {
        let layer = LayerId(1);
        let other_layer = LayerId(9);
        let property = PropertyId::new("opacity").expect("opacity は予約語ではない");
        let rows = vec![key_row(layer, property.clone(), &[10])];
        let pairs = [(KeySelector { layer: other_layer, property: property.clone(), frame: 10 }, 99)];

        let out = apply_key_preview(rows, Some(&pairs));

        assert_eq!(out[0].keys[0].frame, 10, "一致しない selector で動いてしまった");
    }

    /// `preview == None`(非ドラッグ中)は素通り。
    #[test]
    fn apply_key_preview_none_is_a_passthrough() {
        let layer = LayerId(1);
        let property = PropertyId::new("opacity").expect("opacity は予約語ではない");
        let rows = vec![key_row(layer, property, &[10, 20])];
        let out = apply_key_preview(rows.clone(), None);
        assert_eq!(out, rows);
    }

    /// **オラクル(U1 finding「multi-select のハイライト未配線」の根治)**:
    /// `selected_layers` の一員は focus(`selection`)でなくても行が選択扱いに
    /// なる。focus 単独・非選択も従来どおり。
    #[test]
    fn row_selected_includes_multi_selection_members() {
        let mut session = Session::default();
        session.selection = Some(LayerId(1));
        session.selected_layers = vec![LayerId(1), LayerId(2)];

        assert!(row_selected(&session, LayerId(1)), "focus 行が選択扱いでない");
        assert!(
            row_selected(&session, LayerId(2)),
            "selected_layers の一員(非 focus)がハイライトされない — U1 finding の未配線"
        );
        assert!(!row_selected(&session, LayerId(3)), "非選択行まで選択扱いになっている");
    }
}
