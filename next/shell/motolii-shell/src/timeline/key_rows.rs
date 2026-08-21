//! property 行(キー行、第2波 T3・裁定148/151)の draw + hit。**自己完結** —
//! `input.rs`/`hit.rs`/`lane_bar.rs::hit_test` は一切呼ばない(`mod.rs` doc の
//! [`super::key_rows`] 節、`projection::layer_row_top` の write-set 外
//! finding 参照)。
//!
//! - **描画**: 行の帯(ゼブラのリズムを layer 行と共有、EXACT TARGET 2)+
//!   rail 側の property 名 + キー菱形(描画 8×8・当たり 12×12、単一菱形 —
//!   裁定151「形状コード不採用」)
//! - **選択**: クリック=単独 / Cmd=トグル / Shift=範囲(正典 §3・§4 と同じ
//!   文法)。ここでは「どのキーを・どの操作で」までしか判定しない —
//!   `Session::selected_keys`/`key_anchor` の読み書きは唯一の書き口
//!   (`crate::Shell::update`/`apply_key_selection`)へ委ねる
//!   ([`super::KeySelectionOp`])
//! - **Delete**: グローバルの window リスナー(`crate::inspector_pointer_event`)
//!   が Backspace/Delete を拾って `Message::TimelineDeleteSelectedKeys` を出す
//!   — ここは選択の判定だけを持ち、削除には関与しない

use iced::widget::canvas;
use iced::{mouse, Point, Rectangle, Size};

use super::projection::frame_to_x;
use super::{KeySelectionOp, KeySelector, TimelinePane};
use crate::Message;

/// 描画サイズ(正典 §1「文法定数」)。**当たりは絵より大きい**(bar 端と同じ
/// 思想 — 細い菱形をピクセル単位で狙わせない)。
const KEY_DIAMOND_SIZE: f32 = 8.0;
const KEY_HIT: f32 = 12.0;

/// property 行の帯が始まる y(ルーラー下相対)。選択 layer が無い/property 行が
/// 1本も無ければ `None`(帯自体が存在しない)。
fn band_top(pane: &TimelinePane) -> Option<f32> {
    if pane.property_rows.is_empty() {
        return None;
    }
    let selected = pane.selected_row_index?;
    Some(pane.ruler_height() + pane.dims.row_height * (selected as f32 + 1.0))
}

fn band_bottom(pane: &TimelinePane, top: f32) -> f32 {
    top + pane.param_row_height() * pane.property_rows.len() as f32
}

/// `y` が property 行の何行目か(0-based)。帯の外なら `None`。
fn row_at_y(pane: &TimelinePane, top: f32, y: f32) -> Option<usize> {
    if y < top {
        return None;
    }
    let index = ((y - top) / pane.param_row_height()).floor();
    if index < 0.0 {
        return None;
    }
    let index = index as usize;
    (index < pane.property_rows.len()).then_some(index)
}

fn diamond_path(cx: f32, cy: f32, half: f32) -> canvas::Path {
    canvas::Path::new(|builder| {
        builder.move_to(Point::new(cx, cy - half));
        builder.line_to(Point::new(cx + half, cy));
        builder.line_to(Point::new(cx, cy + half));
        builder.line_to(Point::new(cx - half, cy));
        builder.close();
    })
}

/// property 行の帯・rail 側の名前・キー菱形を描く(`super::canvas::draw` から
/// 委譲されるだけ — mod doc の層分担どおり)。
pub(crate) fn draw(
    pane: &TimelinePane,
    frame: &mut canvas::Frame,
    rail_width: f32,
    clip_width: f32,
    width: f32,
) {
    let Some(top) = band_top(pane) else {
        return;
    };
    let row_h = pane.param_row_height();
    for (index, row) in pane.property_rows.iter().enumerate() {
        let row_top = top + row_h * index as f32;

        // 帯(ゼブラのリズムを layer 行と共有、EXACT TARGET 2 — 区切りの手段
        // ではない、§1.6 の両立整理どおり区切りは下の hairline が担う)。
        if index % 2 == 1 {
            frame.fill_rectangle(
                Point::new(0.0, row_top),
                Size::new(width, row_h),
                pane.colors.timeline_row_zebra,
            );
        }

        // rail 側の property 名(裁定147「名前の住所はレーンバー」を property
        // 行にも延長)。layer 名より一段深く字下げして、選択 layer の子である
        // ことを示す(§1.6 グループ階層のインデント方針)。
        frame.fill_text(canvas::Text {
            content: row.property.name().to_owned(),
            position: Point::new(pane.dims.spacing_l * 2.0, row_top + row_h / 2.0),
            color: pane.colors.text_secondary,
            size: iced::Pixels(pane.dims.caption_text),
            align_y: iced::alignment::Vertical::Center,
            ..Default::default()
        });

        // 行の区切り(layer 行と同じ弱い hairline ロール、EXACT TARGET 2)。
        let hairline_path = canvas::Path::line(
            Point::new(0.0, row_top + row_h),
            Point::new(width, row_top + row_h),
        );
        frame.stroke(
            &hairline_path,
            canvas::Stroke::default()
                .with_color(pane.colors.border_hairline_weak)
                .with_width(pane.dims.border_width),
        );

        // キー菱形(描画 8×8、単一菱形 — 裁定151「形状コード不採用」)。
        for key in &row.keys {
            let cx = rail_width + frame_to_x(key.frame, clip_width, pane.duration_frames);
            let cy = row_top + row_h / 2.0;
            let color = if key.selected {
                pane.colors.action_active
            } else {
                pane.colors.way_timeline
            };
            frame.fill(&diamond_path(cx, cy, KEY_DIAMOND_SIZE / 2.0), color);
        }
    }
}

/// property 行の帯内の click を判定する。**帯の内側に入った click は、菱形に
/// 当たっても外れても必ず `capture()` で吸収する** — `input.rs`/`hit.rs` は
/// 押し下げ([`super::projection::layer_row_top`])を知らないので、そのまま
/// 通すと後続層への誤爆(誤選択/誤 scrub)になる(`layer_row_top` の doc の
/// write-set 外 finding 参照)。帯の外なら `None` を返し、通常の経路
/// (`input::update`)に委ねる。
pub(crate) fn update(
    pane: &TimelinePane,
    event: &canvas::Event,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> Option<canvas::Action<Message>> {
    let canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event else {
        return None;
    };
    let position = cursor.position_in(bounds)?;
    let top = band_top(pane)?;
    let bottom = band_bottom(pane, top);
    if position.y < top || position.y >= bottom {
        return None;
    }

    let Some(row_index) = row_at_y(pane, top, position.y) else {
        return Some(canvas::Action::capture());
    };
    let row = &pane.property_rows[row_index];

    let rail_width = pane.rail_width();
    if position.x < rail_width {
        // rail 側(property 名)は今回クリック動詞を持たない — 吸収だけする。
        return Some(canvas::Action::capture());
    }
    let clip_width = (bounds.width - rail_width).max(0.0);
    let local_x = position.x - rail_width;

    let hit_key = row.keys.iter().find(|key| {
        let cx = frame_to_x(key.frame, clip_width, pane.duration_frames);
        (local_x - cx).abs() <= KEY_HIT / 2.0
    });
    let Some(key) = hit_key else {
        return Some(canvas::Action::capture()); // 行の空白 — 吸収のみ。
    };

    let clicked = KeySelector {
        layer: row.layer,
        property: row.property.clone(),
        frame: key.frame,
    };

    // 正典 §3・§4: クリック=単独 / Cmd=トグル / Shift=範囲。確定は
    // `Shell::apply_key_selection`(唯一の書き口)側 — ここは操作の種別だけ選ぶ。
    let op = if pane.modifiers.shift() {
        KeySelectionOp::Range(clicked)
    } else if pane.modifiers.command() {
        KeySelectionOp::Toggle(clicked)
    } else {
        KeySelectionOp::Single(clicked)
    };
    Some(canvas::Action::publish(Message::TimelineKeySelect(op)).and_capture())
}

/// 帯の内側にいる間だけカーソル形状を判定する(Q0: 触れそうな物には予告を
/// 出す)。帯の外なら `None` — 呼び出し側(`super::mod`)が通常の経路
/// (`input::mouse_interaction`)へ落ちる。
pub(crate) fn mouse_interaction(
    pane: &TimelinePane,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> Option<mouse::Interaction> {
    let position = cursor.position_in(bounds)?;
    let top = band_top(pane)?;
    let bottom = band_bottom(pane, top);
    if position.y < top || position.y >= bottom {
        return None;
    }
    let rail_width = pane.rail_width();
    if position.x < rail_width {
        return Some(mouse::Interaction::default());
    }
    let row_index = row_at_y(pane, top, position.y)?;
    let row = &pane.property_rows[row_index];
    let clip_width = (bounds.width - rail_width).max(0.0);
    let local_x = position.x - rail_width;
    let over_key = row.keys.iter().any(|key| {
        let cx = frame_to_x(key.frame, clip_width, pane.duration_frames);
        (local_x - cx).abs() <= KEY_HIT / 2.0
    });
    Some(if over_key {
        mouse::Interaction::Pointer
    } else {
        mouse::Interaction::default()
    })
}
