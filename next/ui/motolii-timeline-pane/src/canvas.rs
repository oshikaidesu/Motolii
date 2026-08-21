//! Timeline の絵(`draw`/`draw_ruler_ticks`/`draw_hairline`/`draw_time_bands`)。
//! `TimelinePane`(`super::TimelinePane` の `canvas::Program` impl)の `draw` から
//! 委譲されるだけ — **`&mut` は1つも無い**(`draw` は `&self` でモデルを借りる
//! ので、描きながらモデルを直す道が型として無い)。
//!
//! ## 比率の出典(裁定172 §1/§2、転写元 `next/reference/mocks/timeline-semantics.html`)
//!
//! bar/ruler/目盛りの寸法は「梯子」(裁定165(1)・裁定167)— 独自の中間値を
//! 発明せず、mock の実測比をそのまま使う。この節の関数群がその唯一の出典
//! ([`ruler_height`]/[`bar_inset`]/[`bar_corner_radius`]/[`minor_tick_length`]/
//! [`major_tick_length`])。全て「比率×分母」の丸め(`.round()`)— mock の
//! 実測 px と比較する pure fn テストが末尾にある。

use iced::widget::canvas;
use iced::{Point, Rectangle, Size};

use super::key_rows;
use super::lane_bar;
use super::projection::{frame_at_x, frame_to_x, tick_steps, time_band_segment_frames};
use super::TimelinePane;

/// ルーラー帯の高さ(裁定172 §2: `0.846×行高`)。mock `.ruler{height:22px}` /
/// `.row{height:26px}`(`22/26`)の実測。第1波の「行高をそのまま流用」
/// (裁定167 が禁じる、比率梯子を無視した1.0倍という名の中間値)を廃した —
/// `TimelinePane::ruler_height`(内部の private メソッド)がこの1関数だけを呼ぶ。
/// `pub`(クレート外に見える)なのは他の比率と同じ理由(`lib.rs` の
/// `pub use canvas::{...}` doc 参照 — `motolii-shell::screenshot` cross-crate 用)。
pub fn ruler_height(row_height: f32) -> f32 {
    (0.846 * row_height).round()
}

/// bar の縦 inset(裁定172 §2: `0.154×行高` — 「梯子中段」)。mock
/// `.bar{top:4px}` / `.row{height:26px}`(`4/26`)の実測。現行の
/// `spacing_xs`(0.10 相当)流用は裁定167(比率梯子の中間値禁止)違反だった。
pub fn bar_inset(row_height: f32) -> f32 {
    (0.154 * row_height).round()
}

/// bar の角丸半径(裁定172 §2: `0.111×bar高` — bar は inset 適用後の高さが
/// 分母)。mock `.bar{height:18px;border-radius:2px}`(`2/18`)の実測。
pub fn bar_corner_radius(bar_height: f32) -> f32 {
    (0.111 * bar_height).round()
}

/// ルーラー小目盛りの長さ(裁定172 §1: `0.227×ruler高`)。mock `.tick{height:5px}`
/// / `.ruler{height:22px}`(`5/22`)の実測 — 裁定165(1)「独自比を発明しない」の
/// 素直な適用。現行の `spacing_s` 引き算式(ruler高からの天引き)を廃した。
pub fn minor_tick_length(ruler_height: f32) -> f32 {
    (0.227 * ruler_height).round()
}

/// ルーラー大目盛りの長さ(裁定172 §1: `0.5×ruler高`)。mock
/// `.tick.major{height:11px}` / `.ruler{height:22px}`(`11/22`)の実測。
pub fn major_tick_length(ruler_height: f32) -> f32 {
    (0.5 * ruler_height).round()
}

pub(crate) fn draw(
    pane: &TimelinePane,
    renderer: &iced::Renderer,
    bounds: Rectangle,
    cursor: iced::mouse::Cursor,
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
        // 選択 layer の下に property 行が挿入されている間、後続の層行は押し下がる
        // (`TimelinePane::layer_row_top`、EXACT TARGET 1)。
        //
        // x は rail の右から(時間帯 `draw_time_bands` と同じ起点)。**rail は
        // 時間カメラの外**(利用者知覚モデル 2026-08-21: 横スケールのジェスチャーは
        // rail に効かず、縦スクロールだけが通る — rail は時間場の上に乗る別レイヤー
        // であって、時間場の wash(ゼブラ・時間帯)を受けない)。
        let row_top = rows_top + pane.layer_row_top(index);
        frame.fill_rectangle(
            Point::new(rail_width, row_top),
            Size::new(clip_width, row_height),
            pane.colors.timeline_row_zebra,
        );
    }
    draw_time_bands(pane, &mut frame, rail_width, clip_width, rows_top, rows_bottom);

    // 時間方向の縦線(利用者裁定 2026-08-21 夜・σ EXACT TARGET 2、mock
    // `timeline-semantics.html` の `bands()` 第2ループが出典)。帯(面・粗い
    // リズム)とは別の周波数(線・全目盛の細かいリズム) — 描画順は
    // 帯→縦線→bar(地の上・内容物の下)なので、bar を描く層の行ループより
    // 先にここへ置く。
    draw_tick_lines(pane, &mut frame, rail_width, clip_width, rows_top, rows_bottom);

    // 層の行。
    for (index, row) in pane.rows.iter().enumerate() {
        let row_top = ruler_height + pane.layer_row_top(index);

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
        // ドラッグ中の bar は ACCENT(第2波T5、`row.dragging` は
        // `projection::apply_clip_preview` が掴んでいる1行にだけ立てる —
        // R1 egui版実測「ドラッグ中のbarはACCENT色に変わる」を踏襲)。
        // hidden との優先順位: 掴んで動かしている最中は見えていることの方が
        // 重要なので dragging が hidden より優先。レイヤー差し色(第1波)は
        // 通常時だけ効く — dragging/hidden の優先順位はそのまま(既存テスト
        // が守る)、`label_color` が `Some` で初めて `way_timeline` の代わりに
        // パレット色を使う。index が万一パレット長を超えていたら(将来の
        // パレット縮小等)既定色へ落ちる(panic しない、M16 と同じ姿勢)。
        let bar_color = if row.dragging {
            pane.colors.action_active
        } else if row.hidden {
            pane.colors.text_muted
        } else if let Some(color) = row
            .label_color
            .and_then(|index| pane.colors.label_palette.get(index as usize))
        {
            *color
        } else {
            pane.colors.way_timeline
        };
        // 縦 inset・角丸(裁定172 §2)— 比率の出典は [`bar_inset`]/
        // [`bar_corner_radius`](モジュール冒頭「比率の出典」節)。`fill_rectangle`
        // ではなく丸角 `Path::rounded_rectangle` を使う(iced canvas に既にある
        // ネイティブ API — 近似ではない)。
        let inset = bar_inset(row_height);
        let bar_height = (row_height - inset * 2.0).max(1.0);
        let radius = bar_corner_radius(bar_height);
        let bar_path = canvas::Path::rounded_rectangle(
            Point::new(start_x, row_top + inset),
            Size::new((end_x - start_x).max(1.0), bar_height),
            radius.into(),
        );
        frame.fill(&bar_path, bar_color);
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

    // property 行(キー行、第2波 T3・裁定148/151) — 選択 layer の下に挿入する。
    // 帯・キー菱形・rail 側のラベルは `key_rows.rs` が自己完結で描く(mod doc
    // 参照)。
    key_rows::draw(pane, &mut frame, rail_width, clip_width, width);

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

    // ポインタ近くのタイムコードミニラベル(第2波T5、R1 egui版実測「掴んでいる
    // 間ポインタ近くにタイムコードのミニラベルを出す」を踏襲)。drag 中
    // (`pane.preview_active`、clip/key どちらでも)だけ・クリップ面上でだけ出す
    // — rail 側(行ヘッダ列)は時間の面ではないので出さない。既存の
    // `fill_text`(ルーラー目盛りと同じ描画手段)をそのまま使う、新しい描画
    // 語彙は増やさない。
    if pane.preview_active {
        if let Some(position) = cursor.position_in(bounds) {
            if position.x >= rail_width {
                let frame_no = frame_at_x(position.x - rail_width, clip_width, pane.duration_frames);
                frame.fill_text(canvas::Text {
                    content: frame_no.to_string(),
                    position: Point::new(position.x + pane.dims.spacing_xs, position.y - pane.dims.spacing_l),
                    color: pane.colors.action_active,
                    size: iced::Pixels(pane.dims.caption_text),
                    ..Default::default()
                });
            }
        }
    }

    vec![frame.into_geometry()]
}

/// 小目盛/大目盛の階層(利用者裁定 2026-08-21 夜、`projection::tick_steps` が
/// 唯一の出典)。旧・全尺 `RULER_TICK_DIVISIONS` 等分を撤去 — 目盛りは常に
/// 時刻へ絶対整列する(0, step, 2*step, ...)。
///
/// 小目盛= 短い線だけ(`border_hairline_weak` 級 — 既存 token のみ、S4)・
/// ラベル無し(密度が上がるので毎ステップへラベルを置くと読めなくなる)。
/// 大目盛= 長い線(`border_strong`)+ フレーム番号ラベル(旧実装の踏襲、
/// NON-GOALS「ルーラーへの timecode 文字」は新設しないという意味なので、
/// 既存のプレーンなフレーム番号表示はそのまま残す)。
///
/// 目盛りの長さは裁定172 §1: `spacing_m`/`spacing_s` の天引き式(token 経由
/// だが mock 比とは無関係)を廃し、[`minor_tick_length`]/[`major_tick_length`]
/// (ruler 高からの比率、モジュール冒頭「比率の出典」節)に一本化した。
fn draw_ruler_ticks(pane: &TimelinePane, frame: &mut canvas::Frame, x0: f32, width: f32, height: f32) {
    if pane.duration_frames <= 0 || width <= 0.0 {
        return;
    }
    let (minor, major) = tick_steps(pane.fps, pane.duration_frames, width, pane.dims.row_height);
    let last_frame = (pane.duration_frames - 1).max(0);
    let mut frame_no = 0i64;
    while frame_no <= last_frame {
        let is_major = frame_no % major == 0;
        let x = x0 + frame_to_x(frame_no, width, pane.duration_frames);
        let top = if is_major {
            height - major_tick_length(height)
        } else {
            height - minor_tick_length(height)
        };
        let color = if is_major {
            pane.colors.border_strong
        } else {
            pane.colors.border_hairline_weak
        };
        let tick_path = canvas::Path::line(Point::new(x, top), Point::new(x, height));
        frame.stroke(
            &tick_path,
            canvas::Stroke::default().with_color(color).with_width(pane.dims.border_width),
        );
        if is_major {
            frame.fill_text(canvas::Text {
                content: frame_no.to_string(),
                position: Point::new(x + pane.dims.spacing_xs, 0.0),
                color: pane.colors.text_secondary,
                size: iced::Pixels(pane.dims.caption_text),
                ..Default::default()
            });
        }
        frame_no += minor;
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

/// 時間方向の明暗リズム(裁定148(1)・正典 §1.6)。区間幅=**大目盛の周期**
/// (`projection::tick_steps` の第2要素、利用者裁定 2026-08-21 夜 — 旧・固定
/// 1秒/`RULER_TICK_DIVISIONS` 等分は撤去。目盛りの階層と同じラダーから出す
/// ので、明暗帯は常に大目盛と同じ場所で切り替わる)。奇数番目の区間にだけ
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
    let segment_frames =
        time_band_segment_frames(pane.fps, pane.duration_frames, width, pane.dims.row_height);

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

/// 時間方向の縦線 — 全目盛の投影(利用者裁定 2026-08-21 夜・σ EXACT TARGET 2、
/// mock `timeline-semantics.html` の `bands()` 第2ループが出典)。時間方向は
/// 周波数で役割分担する: [`draw_time_bands`](面、大目盛周期の粗いリズム)に
/// 対して、この関数は**線**(全目盛の細かいリズム)を描く — [`tick_steps`]
/// (唯一の出典、`draw_ruler_ticks`/screenshot 器具と共有)から小目盛ごとに
/// 1本、大目盛の位置だけ `timeline_grid_major`(わずかに強い「帯の境界の
/// 確認線」)、他は `timeline_grid_minor`(弱)。
///
/// **時間場のみ**(rail は時間カメラの外 — `draw_time_bands` と同じ
/// `x_offset`)。mock は f=0(rail 境界と重なる位置)を引かない —
/// `frame_no = minor` から開始してその踏襲。
fn draw_tick_lines(
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
    let (minor, major) = tick_steps(pane.fps, pane.duration_frames, width, pane.dims.row_height);
    let last_frame = (pane.duration_frames - 1).max(0);
    let mut frame_no = minor;
    while frame_no <= last_frame {
        let is_major = frame_no % major == 0;
        let x = x_offset + frame_to_x(frame_no, width, pane.duration_frames);
        let color = if is_major {
            pane.colors.timeline_grid_major
        } else {
            pane.colors.timeline_grid_minor
        };
        let path = canvas::Path::line(Point::new(x, top), Point::new(x, bottom));
        frame.stroke(
            &path,
            canvas::Stroke::default().with_color(color).with_width(pane.dims.border_width),
        );
        frame_no += minor;
    }
}

#[cfg(test)]
mod ratio_tests {
    use super::*;

    /// mock `timeline-semantics.html` `.row{height:26px}` を分母に、各比率が
    /// mock の実測 px とそのまま一致することを固定する(裁定172 §1/§2 — ORACLE
    /// 「比率の純関数テスト4本」)。梯子の途中(0.10 等)へ丸めていないことの
    /// 検収 — これらは裁定172 施工前は red だった。
    const MOCK_ROW_HEIGHT: f32 = 26.0;

    #[test]
    fn ruler_height_matches_mock_22_of_26() {
        assert_eq!(ruler_height(MOCK_ROW_HEIGHT), 22.0);
    }

    #[test]
    fn bar_inset_matches_mock_4_of_26() {
        assert_eq!(bar_inset(MOCK_ROW_HEIGHT), 4.0);
    }

    #[test]
    fn bar_corner_radius_matches_mock_2_of_18() {
        // mock は inset=4 を row 高 26 から2回引いて bar 高 18 を得る
        // (`.bar{top:4px;height:18px}`)— 角丸の分母はこの bar 高であって
        // 行高ではない。
        let inset = bar_inset(MOCK_ROW_HEIGHT);
        let bar_height = MOCK_ROW_HEIGHT - inset * 2.0;
        assert_eq!(bar_height, 18.0, "梯子の中間値: mock の bar 高は 18px");
        assert_eq!(bar_corner_radius(bar_height), 2.0);
    }

    #[test]
    fn tick_lengths_match_mock_5_and_11_of_22() {
        let ruler = ruler_height(MOCK_ROW_HEIGHT);
        assert_eq!(ruler, 22.0);
        assert_eq!(minor_tick_length(ruler), 5.0, "mock `.tick{{height:5px}}`");
        assert_eq!(major_tick_length(ruler), 11.0, "mock `.tick.major{{height:11px}}`");
    }
}
