//! property 行(キー行、第2波 T3・裁定148/151)の draw + hit
//! **+ 第2波T4(正典 §3・§8.1・裁定146)のキー時刻ドラッグ/リタイム**。
//! **自己完結** — `input.rs`/`hit.rs`/`lane_bar.rs::hit_test` は一切呼ばない
//! (`mod.rs` doc の [`super::key_rows`] 節、`projection::layer_row_top` の
//! write-set 外 finding 参照)。
//!
//! - **描画**: 行の帯(ゼブラのリズムを layer 行と共有、EXACT TARGET 2)+
//!   rail 側の property 名 + キー菱形(描画 8×8・当たり 12×12、単一菱形 —
//!   裁定151「形状コード不採用」)
//! - **選択**: クリック=単独 / Cmd=トグル / Shift=範囲(正典 §3・§4 と同じ
//!   文法)。ここでは「どのキーを・どの操作で」までしか判定しない —
//!   `Session::selected_keys`/`key_anchor` の読み書きは唯一の書き口
//!   (`crate::Shell::update`/`apply_key_selection`)へ委ねる
//!   ([`super::KeySelectionOp`])
//! - **時刻ドラッグ/リタイム**(第2波T4): 修飾キー無しの press は
//!   [`Message::KeyGrabbed`]{retime:false} を出す(選択の差し替え+
//!   drag 開始を兼ねる、確定は `Shell` 側)。Cmd+press が「選択済み・選択が
//!   2本以上・掴んだキーがその選択の端(最小/最大 frame)」を満たせば
//!   `retime:true` で同じ Message を出す(RetimeSelection、裁定146) —
//!   満たさなければ従来どおり Cmd=トグル。**継続イベント**(press 後の
//!   move/release/右クリック)は `TimelinePane::key_drag_active`
//!   (`mod.rs` doc の [`super::key_rows`] 節参照)を見て、drag 中は
//!   ButtonPressed 以外もここで拾う — `input::Interaction` は一切触らない
//! - **Delete**: グローバルの window リスナー(`crate::inspector_pointer_event`)
//!   が Backspace/Delete を拾って `Message::DeleteSelectedKeys` を出す
//!   — ここは選択の判定だけを持ち、削除には関与しない

use iced::widget::canvas;
use iced::{mouse, Point, Rectangle, Size};

use super::projection::{frame_at_x, frame_to_x};
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

/// 今選択されているキー全員(全 property 行を横断)の frame の最小・最大・本数。
/// RetimeSelection(裁定146)の「範囲端」判定に使う — 1本しか選ばれていなければ
/// 端の概念が無い(retime 不成立)。
fn selected_frame_bounds(pane: &TimelinePane) -> Option<(i64, i64, usize)> {
    let mut min = i64::MAX;
    let mut max = i64::MIN;
    let mut count = 0usize;
    for row in &pane.property_rows {
        for key in &row.keys {
            if key.selected {
                min = min.min(key.frame);
                max = max.max(key.frame);
                count += 1;
            }
        }
    }
    (count > 0).then_some((min, max, count))
}

/// press 時点の座標だけで求める comp frame と px/frame(第2波T4、
/// `input::update` の `DragKind::Clip` 腕と同じ換算式)。
fn frame_at_position(pane: &TimelinePane, bounds: Rectangle, position: Point) -> (i64, f32) {
    let rail_width = pane.rail_width();
    let clip_width = (bounds.width - rail_width).max(0.0);
    let at_frame = frame_at_x(position.x - rail_width, clip_width, pane.duration_frames);
    let px_per_frame = if pane.duration_frames > 0 {
        clip_width / pane.duration_frames as f32
    } else {
        0.0
    };
    (at_frame, px_per_frame)
}

/// property 行の帯内の click を判定する、**及び**進行中のキー drag の継続
/// イベント(第2波T4)。**帯の内側に入った click は、菱形に当たっても外れても
/// 必ず `capture()` で吸収する** — `input.rs`/`hit.rs` は押し下げ
/// ([`super::projection::layer_row_top`])を知らないので、そのまま通すと
/// 後続層への誤爆(誤選択/誤 scrub)になる(`layer_row_top` の doc の
/// write-set 外 finding 参照)。帯の外・drag 非進行中なら `None` を返し、
/// 通常の経路(`input::update`)に委ねる。
pub(crate) fn update(
    pane: &TimelinePane,
    event: &canvas::Event,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> Option<canvas::Action<Message>> {
    // 進行中のキー drag(第2波T4) — 種類(move/retime)を問わず、bounds 内の
    // 継続イベントはここで拾う。press した位置がどこだったかは関係ない
    // (clip drag と同じ「掴んだ後は canvas 全体が対象」— `input::update` の
    // `DragKind::Clip` の move/release 腕と同じ形)。
    if pane.key_drag_active {
        return match event {
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let position = cursor.position_in(bounds)?;
                let (at_frame, px_per_frame) = frame_at_position(pane, bounds, position);
                Some(
                    canvas::Action::publish(Message::KeyDragMoved { at_frame, px_per_frame })
                        .and_capture(),
                )
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                Some(canvas::Action::publish(Message::KeyDragReleased).and_capture())
            }
            // 右クリック = キャンセル(裁定151「キャンセルの一般化」、正典 §2 を
            // キーへ延長)。Esc は window 全体の subscription から別経路で届く。
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                Some(canvas::Action::publish(Message::KeyDragCancelled).and_capture())
            }
            _ => None,
        };
    }

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
    let (at_frame, _) = frame_at_position(pane, bounds, position);

    // 正典 §3・§4: クリック=単独 / Cmd=トグル / Shift=範囲。第2波T4:
    // 修飾キー無しの press は選択の差し替え+drag 開始を兼ねる
    // (`Message::KeyGrabbed`、確定/選択の実際の読み書きは
    // `Shell::update` 側 — ここは操作の種別だけ選ぶのは変わらない)。
    if pane.modifiers.shift() {
        // Shift 範囲選択は drag を伴わない(範囲選択そのものが動詞)。
        return Some(
            canvas::Action::publish(Message::KeySelect(KeySelectionOp::Range(clicked))).and_capture(),
        );
    }
    if pane.modifiers.command() {
        // RetimeSelection(裁定146): 選択済み・選択2本以上・掴んだキーがその
        // 選択の端(最小/最大 frame)なら Cmd+drag は retime。それ以外は従来の
        // Cmd=トグル(クリックのみで動かなければ `Shell::finish_timeline_key_drag`
        // が Toggle へ安全側で倒す — click/drag 判定は `inspector_drag` と同じ形)。
        let is_retime_edge = key.selected
            && selected_frame_bounds(pane)
                .is_some_and(|(min, max, count)| count >= 2 && (key.frame == min || key.frame == max));
        if is_retime_edge {
            return Some(
                canvas::Action::publish(Message::KeyGrabbed {
                    key: clicked,
                    at_frame,
                    retime: true,
                })
                .and_capture(),
            );
        }
        return Some(
            canvas::Action::publish(Message::KeySelect(KeySelectionOp::Toggle(clicked))).and_capture(),
        );
    }
    Some(
        canvas::Action::publish(Message::KeyGrabbed {
            key: clicked,
            at_frame,
            retime: false,
        })
        .and_capture(),
    )
}

/// 帯の内側にいる間だけカーソル形状を判定する(Q0: 触れそうな物には予告を
/// 出す)。帯の外なら `None` — 呼び出し側(`super::mod`)が通常の経路
/// (`input::mouse_interaction`)へ落ちる。**進行中のキー drag(第2波T4)は
/// 帯の外でも `Grabbing`**(`input::mouse_interaction` の `state.drag.is_some()`
/// 腕と同じ「掴んでいる間はどこでも Grabbing」)。
pub(crate) fn mouse_interaction(
    pane: &TimelinePane,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> Option<mouse::Interaction> {
    if pane.key_drag_active {
        return Some(mouse::Interaction::Grabbing);
    }
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
