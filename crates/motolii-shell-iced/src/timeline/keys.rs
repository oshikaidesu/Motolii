//! property 行(`RowKind::Property`)のキー菱形 — 2026-08-19 M-6 キー編集レーン。
//!
//! **新しい描画・hit のコードはここに置く**(発注の柵)。`canvas.rs` からは
//! 「描画1行の呼び出し」と「hit 判定1行の呼び出し」だけを受ける — 構造操作
//! レーン(Group/Rename/Lock/畳み開閉)と同時に `canvas.rs` を触っても
//! 衝突しないための境界である。
//!
//! 移植元は egui 版 `crates/motolii-ui/src/timeline_editor/mod.rs` の
//! `RowKind::Property` 描画ループ(菱形: `d = 4.0`・当たり `12x12`・色は
//! `docs/mocks-ui/public/timeline-library.css:6` `.key` / `.key:hover,
//! .key.selectedKey` の正本値)。キーの列そのもの(`param_keys`)・
//! hit test(`key_hit_test`)は共用側([`motolii_ui::timeline_editor`] /
//! [`super::semantics`])にあり、ここは**絵と、hit を Message へ翻訳する所**
//! だけを持つ。

use iced::widget::canvas::{Frame, Path, Stroke, Text};
use iced::{alignment, Color, Pixels, Point, Size};

use motolii_doc::{Document, LayerId};
use motolii_ui::blitz_shell::{UiEditParam, UiKeyInterp};
use motolii_ui::timeline_editor::param_keys;
use motolii_ui::timeline_rows::{ParamRef, RowKind, TimelineRow};

use super::pane::{TimelineDrag, TimelineMsg};
use super::semantics::{key_times_match, key_ui_param, PaneGeometry, TimelineHit, ROW_H};
use crate::widgets::context_menu::MenuItem;

/// 菱形の中心から辺までの半径。egui 版 `d = 4.0`(菱形は8px四方)と同値。
const DIAMOND_R: f32 = 4.0;

// `docs/mocks-ui/public/timeline-library.css:6` `.key{border:1px solid #eee;
// background:#353535}` / `.key:hover,.key.selectedKey{border-color:#fff4bd;
// background:#e9cf72}` — egui 版 `ACCENT` / `KEY_IDLE` と同じ正本。
const KEY_IDLE: Color = Color::from_rgb(
    0x35 as f32 / 255.0,
    0x35 as f32 / 255.0,
    0x35 as f32 / 255.0,
);
const KEY_IDLE_BORDER: Color = Color::from_rgb(
    0xee as f32 / 255.0,
    0xee as f32 / 255.0,
    0xee as f32 / 255.0,
);
const KEY_ACTIVE: Color = Color::from_rgb(
    0xe9 as f32 / 255.0,
    0xcf as f32 / 255.0,
    0x72 as f32 / 255.0,
);
const KEY_ACTIVE_BORDER: Color = Color::from_rgb(
    0xff as f32 / 255.0,
    0xf4 as f32 / 255.0,
    0xbd as f32 / 255.0,
);
/// property 行のラベル字。egui 版 `Color32::from_rgb(0xa8,0xa8,0xa8)`
/// (`timeline_editor/mod.rs` の `RowKind::Property` 描画)と同値。
const PARAM_LABEL: Color = Color::from_rgb(
    0xa8 as f32 / 255.0,
    0xa8 as f32 / 255.0,
    0xa8 as f32 / 255.0,
);
/// 行下端の仕切り線。`canvas.rs` の `RULE`(`#111`)と同値 — 別 module なので
/// 定数として持ち回らず、同じ正本値をここでも直に置く。
const ROW_RULE: Color = Color::from_rgb(
    0x11 as f32 / 255.0,
    0x11 as f32 / 255.0,
    0x11 as f32 / 255.0,
);

/// `ParamRef` の種別色(チップ)。egui 版 `param_color` の簡約 — Timeline は
/// 5種類とも同じ寒色系のチップで並べているだけで、bar のような layer 色は
/// 持たない(egui 版もチップは param 種別だけに依る)。
fn param_chip_color(param: ParamRef) -> Color {
    match param {
        ParamRef::Position => Color::from_rgb(0.55, 0.72, 0.86),
        ParamRef::Anchor => Color::from_rgb(0.62, 0.62, 0.86),
        ParamRef::Scale => Color::from_rgb(0.55, 0.86, 0.62),
        ParamRef::Rotation => Color::from_rgb(0.86, 0.72, 0.55),
        ParamRef::Opacity => Color::from_rgb(0.78, 0.78, 0.78),
    }
}

fn param_label(param: ParamRef) -> &'static str {
    match param {
        ParamRef::Position => "Position",
        ParamRef::Anchor => "Anchor",
        ParamRef::Scale => "Scale",
        ParamRef::Rotation => "Rotation",
        ParamRef::Opacity => "Opacity",
    }
}

/// property 行1本を描く: レール側はチップ+ラベル(名前欄・M/S/L は object 行
/// だけの物)、トラック側はその param のキーを菱形で並べる。
///
/// `canvas.rs` の draw ループから**1回だけ**呼ぶ(呼んだら object 行の
/// 残りの描画を `continue` で飛ばす)。
#[allow(clippy::too_many_arguments)]
pub fn draw_property_row(
    frame: &mut Frame,
    document: &Document,
    view: motolii_ui::timeline_editor::TimelineView,
    geometry: &PaneGeometry,
    row: &TimelineRow,
    row_top: f32,
    width: f32,
    selected: &[(LayerId, UiEditParam, f32)],
    drag: Option<&TimelineDrag>,
    hover: Option<TimelineHit>,
) {
    let RowKind::Property(param) = row.kind else {
        return;
    };
    let cy = row_top + ROW_H * 0.5;
    let indent = 8.0 + f32::from(row.depth) * 12.0;

    // レール: チップ + ラベル(egui 版 `RowKind::Property` 描画の移植)。
    frame.fill(
        &Path::rectangle(Point::new(indent, cy - 5.5), Size::new(4.0, 11.0)),
        param_chip_color(param),
    );
    frame.fill_text(Text {
        content: param_label(param).to_owned(),
        position: Point::new(indent + 12.0, cy),
        color: PARAM_LABEL,
        size: Pixels(10.0),
        align_y: alignment::Vertical::Center.into(),
        ..Text::default()
    });

    // トラック: キーの列。ドラッグ中のキーは preview 位置で描く(Document は
    // release まで無傷 — `pane.rs` のドラッグ設計と同じ)。
    let ui_param = key_ui_param(param);
    for (_, t) in param_keys(document, row.layer, param) {
        let dragging_this = matches!(
            drag,
            Some(TimelineDrag::Key { layer, param: dparam, from_seconds, .. })
                if *layer == row.layer && ui_param == Some(*dparam) && key_times_match(*from_seconds, t)
        );
        let at = match drag {
            Some(TimelineDrag::Key { preview, .. }) if dragging_this => *preview,
            _ => t,
        };
        let x = geometry.time_to_x(view, at);
        let selected_here = ui_param.is_some_and(|p| {
            selected
                .iter()
                .any(|(l, sp, st)| *l == row.layer && *sp == p && key_times_match(*st, t))
        });
        let hovered_here = matches!(
            hover,
            Some(TimelineHit::Key { layer, param: hparam, at_seconds })
                if layer == row.layer && hparam == param && key_times_match(at_seconds, t)
        );
        draw_diamond(frame, x, cy, selected_here || dragging_this || hovered_here);
    }

    frame.fill_rectangle(
        Point::new(0.0, row_top + ROW_H - 1.0),
        Size::new(width, 1.0),
        ROW_RULE,
    );
}

fn draw_diamond(frame: &mut Frame, cx: f32, cy: f32, active: bool) {
    let path = Path::new(|p| {
        p.move_to(Point::new(cx, cy - DIAMOND_R));
        p.line_to(Point::new(cx + DIAMOND_R, cy));
        p.line_to(Point::new(cx, cy + DIAMOND_R));
        p.line_to(Point::new(cx - DIAMOND_R, cy));
        p.close();
    });
    let (fill, border) = if active {
        (KEY_ACTIVE, KEY_ACTIVE_BORDER)
    } else {
        (KEY_IDLE, KEY_IDLE_BORDER)
    };
    frame.fill(&path, fill);
    frame.stroke(&path, Stroke::default().with_color(border).with_width(1.0));
}

/// 左クリックの当たりを Message へ翻訳する。`TimelineHit::Key` で
/// `key_ui_param` が `None`(Anchor)を返したら **触れそうで触れない物を
/// 作らない**(Q0)— 選ぶ・掴む Message は出さない。
pub fn grab_message(hit: TimelineHit, additive: bool) -> Option<TimelineMsg> {
    match hit {
        TimelineHit::Key {
            layer,
            param,
            at_seconds,
        } => key_ui_param(param).map(|param| TimelineMsg::KeyGrabbed {
            layer,
            param,
            at_seconds,
            additive,
        }),
        TimelineHit::PropertyTrack { layer, param } => {
            key_ui_param(param).map(|param| TimelineMsg::KeyTrackClicked { layer, param })
        }
        _ => None,
    }
}

/// 右クリックの当たりを、補間メニューを開く Message へ翻訳する。
/// 菱形の上だけが対象(Anchor は `key_ui_param` が `None` を返すので黙る)。
pub fn menu_request(hit: TimelineHit, anchor: iced::Point) -> Option<TimelineMsg> {
    match hit {
        TimelineHit::Key {
            layer,
            param,
            at_seconds,
        } => key_ui_param(param).map(|param| TimelineMsg::KeyMenuRequested {
            layer,
            param,
            at_seconds,
            anchor,
        }),
        _ => None,
    }
}

/// 補間メニューの id ⇄ `UiKeyInterp`。egui 版 `key_menu` の
/// "Hold" / "Linear" / "Ease in-out" と同じ3択(2026-08-19)。
const INTERP_HOLD: u32 = 0;
const INTERP_LINEAR: u32 = 1;
const INTERP_SMOOTH: u32 = 2;

pub fn interp_for_menu_id(id: u32) -> Option<UiKeyInterp> {
    match id {
        INTERP_HOLD => Some(UiKeyInterp::Hold),
        INTERP_LINEAR => Some(UiKeyInterp::Linear),
        INTERP_SMOOTH => Some(UiKeyInterp::Smooth),
        _ => None,
    }
}

/// 補間メニューの項目。**入口があるのは Position だけ**(D2 に
/// `prepare_set_position_key_interp` しか無い — egui 版 `key_menu` と同じ穴)。
/// 他の param は無効な1項目で理由を言う(Q0: 死んだ見た目ではなく文脈 disabled)。
pub fn interp_menu_items(param: UiEditParam) -> Vec<MenuItem> {
    if param != UiEditParam::Position {
        return vec![MenuItem {
            id: u32::MAX,
            label: "Easing (Position only in D2)".to_owned(),
            enabled: false,
        }];
    }
    vec![
        MenuItem {
            id: INTERP_HOLD,
            label: "Hold".to_owned(),
            enabled: true,
        },
        MenuItem {
            id: INTERP_LINEAR,
            label: "Linear".to_owned(),
            enabled: true,
        },
        MenuItem {
            id: INTERP_SMOOTH,
            label: "Ease in-out".to_owned(),
            enabled: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_never_produces_a_grab_or_menu_message() {
        let hit = TimelineHit::Key {
            layer: LayerId::from_raw(1),
            param: ParamRef::Anchor,
            at_seconds: 1.0,
        };
        assert_eq!(grab_message(hit, false), None, "Anchor は掴めない(Q0)");
        assert_eq!(
            menu_request(hit, iced::Point::ORIGIN),
            None,
            "Anchor は補間メニューも開かない"
        );
    }

    #[test]
    fn a_key_hit_maps_to_key_grabbed() {
        let hit = TimelineHit::Key {
            layer: LayerId::from_raw(7),
            param: ParamRef::Position,
            at_seconds: 2.5,
        };
        assert_eq!(
            grab_message(hit, true),
            Some(TimelineMsg::KeyGrabbed {
                layer: LayerId::from_raw(7),
                param: UiEditParam::Position,
                at_seconds: 2.5,
                additive: true,
            })
        );
    }

    #[test]
    fn empty_track_maps_to_key_track_clicked() {
        let hit = TimelineHit::PropertyTrack {
            layer: LayerId::from_raw(3),
            param: ParamRef::Scale,
        };
        assert_eq!(
            grab_message(hit, false),
            Some(TimelineMsg::KeyTrackClicked {
                layer: LayerId::from_raw(3),
                param: UiEditParam::Scale,
            })
        );
    }

    #[test]
    fn non_position_interp_menu_is_a_single_disabled_note() {
        let items = interp_menu_items(UiEditParam::Scale);
        assert_eq!(items.len(), 1);
        assert!(!items[0].enabled);
    }

    #[test]
    fn position_interp_menu_has_three_enabled_choices() {
        let items = interp_menu_items(UiEditParam::Position);
        assert_eq!(items.len(), 3);
        assert!(items.iter().all(|item| item.enabled));
        assert_eq!(interp_for_menu_id(items[0].id), Some(UiKeyInterp::Hold));
        assert_eq!(interp_for_menu_id(items[1].id), Some(UiKeyInterp::Linear));
        assert_eq!(interp_for_menu_id(items[2].id), Some(UiKeyInterp::Smooth));
    }
}
