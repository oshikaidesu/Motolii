//! Timeline の絵(`draw`/`draw_ruler_ticks`/`draw_hairline`/`draw_time_bands`)。
//! `TimelinePane`(`super::TimelinePane` の `canvas::Program` impl)の `draw` から
//! 委譲されるだけ — **`&mut` は1つも無い**(`draw` は `&self` でモデルを借りる
//! ので、描きながらモデルを直す道が型として無い)。

use iced::widget::canvas;
use iced::{Point, Rectangle, Size};

use super::lane_bar;
use super::projection::{frame_to_x, time_band_segment_frames, RULER_TICK_DIVISIONS};
use super::TimelinePane;

pub(crate) fn draw(
    pane: &TimelinePane,
    renderer: &iced::Renderer,
    bounds: Rectangle,
) -> Vec<canvas::Geometry> {
    let mut frame = canvas::Frame::new(renderer, bounds.size());
    let width = bounds.width;
    let ruler_height = pane.ruler_height();
    let row_height = pane.dims.row_height;
    // 罫線幅の倍数で意味を分ける — 1x: ルーラー目盛り(hairline)、1.5x: playhead、
    // 2x: マーカー(最も強い accent)。新しい寸法トークンを増やさず、単一の
    // `border_width` から比で導出する(裁定117 の「寸法は token 経由」の範囲内)。
    let hairline = pane.dims.border_width;

    // 座標シフト(裁定147・EXACT TARGET 3): レーンバー幅ぶんルーラー/クリップ面を
    // 右へ。`projection::frame_to_x`/`frame_at_x` 自体はクリップ面ローカル座標の
    // まま(純関数を汚さない)— ここ(呼び出し側)で `rail_width` を足す。
    let rail_width = pane.rail_width();
    let clip_width = (width - rail_width).max(0.0);

    // 背景。ゼブラ(裁定148)・行区切り hairline・選択ハイライトは意図して
    // rail 込みの全幅(`width`)で描く(下記) — レーンバーも同じ明暗リズムを
    // 共有する(裁定148(2))ので、この初期背景がそのまま rail の地も兼ねる
    // (mock `.thead{background:panel}` と同色、`lane_bar::draw` は塗り直さない)。
    frame.fill_rectangle(Point::ORIGIN, bounds.size(), pane.colors.surface_panel);

    // ルーラー帯 + 目盛り(クリップ面のみ — rail の corner は地のまま)。
    frame.fill_rectangle(
        Point::new(rail_width, 0.0),
        Size::new(clip_width, ruler_height),
        pane.colors.surface_raised,
    );
    draw_ruler_ticks(pane, &mut frame, rail_width, clip_width, ruler_height);

    // ルーラー帯とクリップ面の境界(裁定139: 面色の塗り分け=`surface_raised`
    // だけに頼らず hairline を足す — `.tp`/`.ruler` が border-bottom を持つ
    // mock と同じ扱い。ルーラーは「地」の第2段なので不透明な強い hairline
    // (`border_default`、`.cols`/`.ptitle` と同じロール)を使う)。**全幅**
    // (rail 込み)で引く — レーンバーの corner とクリップ面のルーラーは同じ
    // 横の区切りを共有する。
    draw_hairline(
        pane,
        &mut frame,
        0.0,
        width,
        ruler_height,
        pane.colors.border_default,
    );

    // マーカー(comp 側の名前つきロケータ)。ルーラー帯へ縦線として重ねる。
    for marker in &pane.markers {
        if let Some(frame_no) = pane.marker_frame(marker) {
            let x = rail_width + frame_to_x(frame_no, clip_width, pane.duration_frames);
            let marker_path = canvas::Path::line(Point::new(x, 0.0), Point::new(x, ruler_height));
            frame.stroke(
                &marker_path,
                canvas::Stroke::default()
                    .with_color(pane.colors.way_timeline)
                    .with_width(hairline * 2.0),
            );
        }
    }

    // 明暗のリズム(裁定148・正典 §1.6): クリップ面の「地」に2方向の読解補助を
    // 重ねる。**区切りの手段ではない**(裁定137 との両立整理) — 区切りは
    // 上の hairline と、行ごとの下 hairline([`draw`] 末尾)が担う。
    // 順序は 行方向ゼブラ → 時間方向 の順で薄い wash を積む(どちらも
    // token 経由の白 wash、raw 値直書きではない)。
    let rows_top = ruler_height;
    let rows_bottom = pane.content_height();
    for index in 0..pane.rows.len() {
        if index % 2 == 0 {
            continue; // 偶数行は地のまま(奇数行だけへ wash を乗せる)。
        }
        let row_top = rows_top + row_height * index as f32;
        frame.fill_rectangle(
            Point::new(0.0, row_top),
            Size::new(width, row_height),
            pane.colors.timeline_row_zebra,
        );
    }
    draw_time_bands(pane, &mut frame, rail_width, clip_width, rows_top, rows_bottom);

    // 層の行。
    for (index, row) in pane.rows.iter().enumerate() {
        let row_top = ruler_height + row_height * index as f32;

        if row.selected {
            // 状態: 選択(`state_selected`)。hover(`surface_hover`、中立グレー)とは
            // 別ロール — 選択は accent 味、hover は明度差だけ(意味色ロールの区別)。
            // **全幅**(rail 込み)— 選択は行そのものの状態であって、クリップ面
            // だけの状態ではない(レーンバーも同じ行に属する、裁定147)。
            frame.fill_rectangle(
                Point::new(0.0, row_top),
                Size::new(width, row_height),
                pane.colors.state_selected,
            );
        }

        let start_local = frame_to_x(row.start, clip_width, pane.duration_frames);
        let end_local = frame_to_x(row.start + row.duration, clip_width, pane.duration_frames)
            .max(start_local + 1.0);
        let start_x = rail_width + start_local;
        let end_x = rail_width + end_local;
        let bar_color = if row.hidden {
            pane.colors.text_muted
        } else {
            pane.colors.way_timeline
        };
        frame.fill_rectangle(
            Point::new(start_x, row_top + pane.dims.spacing_xs),
            Size::new(
                (end_x - start_x).max(1.0),
                (row_height - pane.dims.spacing_s).max(1.0),
            ),
            bar_color,
        );
        // **名前は描かない**(裁定147): レイヤー名の住所はレーンバー
        // (`lane_bar::draw`)へ一本化した。クリップ上の余白は将来の
        // キーフレームオーバーレイのために空けておく。

        // 行の区切り(裁定139: 面色の塗り分け=ゼブラの明暗だけに頼らず
        // hairline を足す — mock `.trow{border-bottom:...}` と同じ役目)。
        // 行同士は `.prow` と同じ弱い hairline ロール(区切り=線、
        // リズム=地の微差 — §1.6 の両立整理どおり見て区別がつく)。**全幅**
        // (rail 込み)— レーンバーも同じ行区切りを共有する(EXACT TARGET 5)。
        draw_hairline(
            pane,
            &mut frame,
            0.0,
            width,
            row_top + row_height,
            pane.colors.border_hairline_weak,
        );
    }

    // playhead(Session が正本)。クリップ面ローカル座標に rail_width を足す —
    // 結果として rail の外(x >= rail_width)にしか出ない(playhead は時間の
    // 面の物であって行ヘッダ列の物ではない)。
    let playhead_x = rail_width + frame_to_x(pane.playhead, clip_width, pane.duration_frames);
    let playhead_path = canvas::Path::line(
        Point::new(playhead_x, 0.0),
        Point::new(playhead_x, bounds.height),
    );
    frame.stroke(
        &playhead_path,
        canvas::Stroke::default()
            .with_color(pane.colors.action_active)
            .with_width(hairline * 1.5),
    );

    // レーンバー(行ヘッダ列、裁定147)— 同じ Frame の最後に重ねる。
    lane_bar::draw(pane, &mut frame, rail_width);

    vec![frame.into_geometry()]
}

fn draw_ruler_ticks(pane: &TimelinePane, frame: &mut canvas::Frame, x0: f32, width: f32, height: f32) {
    if pane.duration_frames <= 0 || width <= 0.0 {
        return;
    }
    for tick in 0..=RULER_TICK_DIVISIONS {
        let frame_no = (pane.duration_frames - 1).max(0) * tick / RULER_TICK_DIVISIONS;
        let x = x0 + frame_to_x(frame_no, width, pane.duration_frames);
        let tick_path = canvas::Path::line(
            Point::new(x, height - pane.dims.spacing_s),
            Point::new(x, height),
        );
        frame.stroke(
            &tick_path,
            canvas::Stroke::default()
                .with_color(pane.colors.border_strong)
                .with_width(pane.dims.border_width),
        );
        frame.fill_text(canvas::Text {
            content: frame_no.to_string(),
            position: Point::new(x + pane.dims.spacing_xs, 0.0),
            color: pane.colors.text_secondary,
            size: iced::Pixels(pane.dims.caption_text),
            ..Default::default()
        });
    }
}

/// 水平の hairline を1本引く(`Point`/`Size` を毎回組まずに済む共通口)。
/// `inspector_pane.rs::bordered_row` の canvas 版 — こちらは per-edge の
/// border-bottom そのもの(canvas は4辺一律の制約が無いので、Inspector側の
/// 「既知の限界」はここには適用されない)。
fn draw_hairline(
    pane: &TimelinePane,
    frame: &mut canvas::Frame,
    x0: f32,
    x1: f32,
    y: f32,
    color: iced::Color,
) {
    let path = canvas::Path::line(Point::new(x0, y), Point::new(x1, y));
    frame.stroke(
        &path,
        canvas::Stroke::default()
            .with_color(color)
            .with_width(pane.dims.border_width),
    );
}

/// 時間方向の明暗リズム(裁定148(1)・正典 §1.6)。区間幅は fps が引ければ
/// 1秒(Ableton の拍グリッド陰影と同型)、comp が無く fps が引けない時は
/// ルーラー目盛りと同じ [`RULER_TICK_DIVISIONS`] 分割へ落ちる(新しい
/// フォールバック規則を増やさない)。奇数番目の区間にだけ
/// `timeline_time_band` の薄い wash を乗せる — 偶数番目は地のまま
/// (行方向ゼブラと同じ「交互」の言葉遣い)。
fn draw_time_bands(
    pane: &TimelinePane,
    frame: &mut canvas::Frame,
    x_offset: f32,
    width: f32,
    top: f32,
    bottom: f32,
) {
    if pane.duration_frames <= 0 || width <= 0.0 || bottom <= top {
        return;
    }
    let segment_frames = time_band_segment_frames(pane.fps, pane.duration_frames);

    let mut segment_index: i64 = 0;
    let mut start_frame: i64 = 0;
    while start_frame < pane.duration_frames {
        let end_frame = (start_frame + segment_frames).min(pane.duration_frames);
        if segment_index % 2 == 1 {
            let local0 = frame_to_x(start_frame, width, pane.duration_frames);
            let local1 = frame_to_x(end_frame, width, pane.duration_frames).max(local0 + 1.0);
            let x0 = x_offset + local0;
            let x1 = x_offset + local1;
            frame.fill_rectangle(
                Point::new(x0, top),
                Size::new(x1 - x0, bottom - top),
                pane.colors.timeline_time_band,
            );
        }
        start_frame = end_frame;
        segment_index += 1;
    }
}
