//! property 行(キー行、第2波 T3・裁定148/151)の draw + hit
//! **+ 第2波T4(正典 §3・§8.1・裁定146)のキー時刻ドラッグ/リタイム**。
//! **自己完結** — `input.rs`/`hit.rs`/`lane_bar.rs::hit_test` は一切呼ばない
//! (`mod.rs` doc の [`super::key_rows`] 節、`projection::layer_row_top` の
//! write-set 外 finding 参照)。
//!
//! - **描画**: 行の帯(ゼブラのリズムを layer 行と共有、EXACT TARGET 2)+
//!   キー菱形(描画 8×8・当たり 12×12、単一菱形 — 裁定151「形状コード
//!   不採用」)。**rail 側の property 名は描かない**(TL-arch Phase 1、
//!   2026-08-22 — `super::rail::view` が実 widget text として持つ。この
//!   ファイルは時間場側の帯・菱形だけを描く)
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

use super::projection::{frame_at_x, frame_to_x, PropertyKeyProjection};
use super::{KeySelectionOp, KeySelector, TimelinePane};
use crate::Message;

/// property 行の帯が始まる y。**縦スクロール発注(2026-08-22)**: ルーラーは
/// `super::ruler::RulerHeader` へ移設され、この canvas(body)の y=0 は
/// もう「ルーラー下」ではなく「行0の上端」そのもの(`super::canvas`
/// モジュール冒頭 doc 参照) — 旧 `ruler_height` の加算はここでは不要になった。
/// 選択 layer が無い/property 行が1本も無ければ `None`(帯自体が存在しない)。
fn band_top(pane: &TimelinePane) -> Option<f32> {
    if pane.property_rows.is_empty() {
        return None;
    }
    let selected = pane.selected_row_index?;
    Some(pane.dims.row_height * (selected as f32 + 1.0))
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

/// **時間軸カリング**(AX-4「計算量で壊れる」finding、`key_rows.rs:92,118`)。
/// 10分尺(5,400〜9,000フレーム)で1property に数百キーが乗ると、狭い
/// canvas 幅(数百〜千数百px)へ写した時に**複数キーが同じ画面ピクセル列へ
/// 重なる**運用が普通にある。重なった列は「後に描く方(= `row.keys` の
/// 並びで後ろ)が完全に同形・不透明の菱形で前を覆う」ため、手前の物を
/// 間引いても最終ラスタは1px も変わらない(菱形は `frame_to_x` が返す
/// 同じ x へ同じ `KEY_DIAMOND_SIZE` で描かれる — 塗り重ねの結果は最後の
/// 1回だけが効く、canvas の paint 順そのもの)。
///
/// **前提**: `row.keys` は frame 昇順(`property_rows` が `track.keys()` の
/// 並びをそのまま filter_map するだけ — `motolii-eval::KeyframeTrack::eval`
/// の `binary_search_by` が要求する昇順ソートを継承)。`frame_to_x` は
/// frame について単調非減少なので、同じ画面ピクセルへ重なるキーは**必ず
/// 連続区間**になる — 隣(次)と比べるだけで判定できる(全体を2回舐める
/// 必要も、バケット表も要らない)。
///
/// **端は必ず残す**(境界の1本を落とすと絵が変わる、という一般原則の
/// 適用): 各連続区間の**最後の1本**(= 最終的に画面へ乗る色を決める本人)
/// だけを残す。区間が長さ1(隣と重ならない)なら、その1本がそのまま残る
/// ので疎な運用(重なりが無い通常時)では退化して「全キーを描く」に一致する
/// — 絵はどんな入力でも不変。
fn keys_for_draw(
    keys: &[PropertyKeyProjection],
    width: f32,
    duration_frames: i64,
) -> impl Iterator<Item = &PropertyKeyProjection> {
    keys.iter().enumerate().filter_map(move |(index, key)| {
        let is_last_in_run = match keys.get(index + 1) {
            Some(next) => {
                let cx = frame_to_x(key.frame, width, duration_frames).round();
                let next_cx = frame_to_x(next.frame, width, duration_frames).round();
                cx != next_cx
            }
            None => true,
        };
        is_last_in_run.then_some(key)
    })
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

/// property 行の帯・キー菱形を描く(`super::canvas::draw` から委譲される
/// だけ — mod doc の層分担どおり)。`width` はこの canvas 自身の幅
/// (= 時間場のみ、TL-arch Phase 1 で rail が分離されたので `super::canvas`
/// と同じ意味 — オフセットは要らない、`super::canvas` 冒頭のモジュール doc
/// 参照)。rail 側の property 名ラベルは `super::rail::view` が持つ。
pub(crate) fn draw(pane: &TimelinePane, frame: &mut canvas::Frame, width: f32) {
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

        // 行の区切り(layer 行と同じ弱い hairline ロール、EXACT TARGET 2)。
        let hairline_path = canvas::Path::line(
            Point::new(0.0, row_top + row_h),
            Point::new(width, row_top + row_h),
        );
        frame.stroke(
            &hairline_path,
            canvas::Stroke::default()
                .with_color(pane.colors.border_hairline_weak)
                .with_width(pane.dims.theme().stroke.hairline),
        );

        // キー菱形(描画寸は JSON 正本の key_diamond_size、単一菱形 —
        // 裁定151「形状コード不採用」)。
        // **時間軸カリング**([`keys_for_draw`] doc 参照) — 同じ画面ピクセル
        // 列に重なるキーは最後の1本だけ描く、絵は不変。
        for key in keys_for_draw(&row.keys, width, pane.duration_frames) {
            let cx = frame_to_x(key.frame, width, pane.duration_frames);
            let cy = row_top + row_h / 2.0;
            let color = if key.selected {
                pane.colors.action_active
            } else {
                pane.colors.way_timeline
            };
            frame.fill(
                &diamond_path(
                    cx,
                    cy,
                    pane.dims.components.timeline.key_diamond_size / 2.0,
                ),
                color,
            );
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
/// `input::update` の `DragKind::Clip` 腕と同じ換算式)。この canvas は
/// TL-arch Phase 1 で時間場だけになった(`super::canvas` 冒頭のモジュール
/// doc 参照)ので `bounds.width` がそのままクリップ幅、オフセットは不要。
fn frame_at_position(pane: &TimelinePane, bounds: Rectangle, position: Point) -> (i64, f32) {
    let width = bounds.width;
    let at_frame = frame_at_x(position.x, width, pane.duration_frames);
    let px_per_frame = if pane.duration_frames > 0 {
        width / pane.duration_frames as f32
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

    // rail 側(property 名)はもう TL-arch Phase 1 でこの canvas の外
    // (`super::rail::view` が実 widget として持つ、`x < rail_width` の分岐は
    // 撤去 — この canvas の bounds はもう時間場だけなので、その分岐自体が
    // 意味を失った)。
    let clip_width = bounds.width;
    let local_x = position.x;

    let hit_key = row.keys.iter().find(|key| {
        let cx = frame_to_x(key.frame, clip_width, pane.duration_frames);
        (local_x - cx).abs() <= pane.dims.components.timeline.key_hit / 2.0
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
    // rail 側の分岐は撤去(`update` と同じ理由 — この canvas の bounds は
    // もう時間場だけ、上のコメント参照)。
    let row_index = row_at_y(pane, top, position.y)?;
    let row = &pane.property_rows[row_index];
    let clip_width = bounds.width;
    let local_x = position.x;
    let over_key = row.keys.iter().any(|key| {
        let cx = frame_to_x(key.frame, clip_width, pane.duration_frames);
        (local_x - cx).abs() <= pane.dims.components.timeline.key_hit / 2.0
    });
    Some(if over_key {
        mouse::Interaction::Pointer
    } else {
        mouse::Interaction::default()
    })
}

#[cfg(test)]
mod culling_tests {
    use super::*;

    fn key(frame: i64, selected: bool) -> PropertyKeyProjection {
        PropertyKeyProjection { frame, selected }
    }

    /// **絵が変わらないことのオラクル**(canvas を通さない層 — `iced_test`
    /// の Simulator は canvas を構造的に見られないため、`draw` が実際に
    /// 呼ぶ塗り操作を「最終的にどのピクセル列がどの色になるか」という
    /// pure な写像へ落として比較する)。
    ///
    /// `draw` は `row.keys` を並び順に塗るだけなので、任意の画面ピクセル
    /// 列の最終色は「その列に重なる最後の(=配列内で最も後ろの)キーの
    /// `selected`」で決まる(canvas の paint 順そのもの、キー菱形は同形・
    /// 不透明)。この関数は**フル描画(間引き無し)を仮定した**その写像
    /// (pixel → 最終 selected)を計算する — [`keys_for_draw`] が返す間引き
    /// 済み列から同じ写像を計算し、両者が一致すれば「間引いても最終ラスタは
    /// 変わらない」ことの直接証明になる。
    fn final_pixel_colors(
        keys: impl Iterator<Item = PropertyKeyProjection>,
        width: f32,
        duration_frames: i64,
    ) -> std::collections::BTreeMap<i32, bool> {
        let mut out = std::collections::BTreeMap::new();
        for k in keys {
            let px = frame_to_x(k.frame, width, duration_frames).round() as i32;
            out.insert(px, k.selected); // 後着が前を上書き = draw の paint 順。
        }
        out
    }

    fn assert_same_picture(keys: Vec<PropertyKeyProjection>, width: f32, duration_frames: i64) {
        let full = final_pixel_colors(keys.iter().cloned(), width, duration_frames);
        let culled = final_pixel_colors(
            keys_for_draw(&keys, width, duration_frames).cloned(),
            width,
            duration_frames,
        );
        assert_eq!(
            full, culled,
            "間引き後の最終ピクセル色がフル描画と一致しない(絵が変わった)"
        );
    }

    #[test]
    fn sparse_keys_are_all_kept_unchanged() {
        // 重ならない疎な配置(通常運用)では退化して「全キーを描く」に一致する。
        let keys = vec![key(0, false), key(100, false), key(200, true), key(299, false)];
        let kept: Vec<_> = keys_for_draw(&keys, 800.0, 300).collect();
        assert_eq!(kept.len(), keys.len(), "疎な配置なのに間引かれた");
        assert_same_picture(keys, 800.0, 300);
    }

    #[test]
    fn dense_run_on_the_same_pixel_collapses_to_the_last_key() {
        // 800px 幅に 9000 フレームを写すと 1px あたり 11 フレーム強 — 隣接
        // フレームが同じ画面ピクセルへ何本も重なるのは普通に起こる。
        let keys: Vec<_> = (0..40).map(|f| key(f, f == 39)).collect(); // 全員 x≈0
        let kept: Vec<_> = keys_for_draw(&keys, 800.0, 9000).collect();
        assert!(kept.len() < keys.len(), "密集しているのに間引かれていない");
        assert_eq!(
            kept.last().unwrap().frame,
            39,
            "区間の最後(最終的に見える色を決める本人)が残っていない"
        );
        assert_same_picture(keys, 800.0, 9000);
    }

    #[test]
    fn selection_color_of_a_culled_key_is_not_silently_lost_if_it_is_the_last_in_its_run() {
        // 選択済みキーが重なり区間の最後(=一番上に乗る)なら、選択色は
        // 間引き後も必ず見える(端の1本は必ず残る、という一般原則)。
        let keys = vec![key(0, false), key(1, false), key(2, true)];
        assert_same_picture(keys.clone(), 4000.0, 9000);
        let kept: Vec<_> = keys_for_draw(&keys, 4000.0, 9000).collect();
        assert!(kept.iter().any(|k| k.selected), "選択済みキーの色が失われた");
    }

    #[test]
    fn ten_thousand_dense_keys_cull_down_to_at_most_the_pixel_width() {
        // 実尺(9,000フレーム級)より広い密度でも、間引き後の描画本数は
        // 画面幅(≈px 数)を超えない — O(property行×keyframe数) だった draw
        // 呼び出しあたりの実描画数が O(width_px) 級へ落ちることの実測。
        let width = 1200.0;
        let keys: Vec<_> = (0..10_000).map(|f| key(f, false)).collect();
        let kept: Vec<_> = keys_for_draw(&keys, width, 10_000).collect();
        assert!(
            kept.len() as f32 <= width + 2.0,
            "間引き後もピクセル幅を大きく超えて描いている: {}",
            kept.len()
        );
        assert_same_picture(keys, width, 10_000);
    }

    #[test]
    fn degenerate_zero_width_collapses_to_a_single_key_and_matches_full_draw() {
        // width<=0(pane 未計測・崩壊直後)では frame_to_x が常に 0.0 を返す
        // ので、フル描画でも全キーが厳密に同じ座標へ重なる — 間引きは
        // 「本物の重複」を消しているだけで、この場合も絵は不変。
        let keys = vec![key(0, false), key(50, true), key(99, false)];
        assert_same_picture(keys, 0.0, 100);
    }
}
