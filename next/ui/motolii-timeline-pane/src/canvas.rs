//! Timeline の絵(`draw`/`draw_ruler_ticks`/`draw_hairline`/`draw_time_bands`)。
//! `TimelinePane`(`super::TimelinePane` の `canvas::Program` impl)の `draw` から
//! 委譲されるだけ — **`&mut` は1つも無い**(`draw` は `&self` でモデルを借りる
//! ので、描きながらモデルを直す道が型として無い)。
//!
//! **TL-arch Phase 1**(`docs/reviews/2026-08-22-timeline-canvas-widget-survey.md`
//! §6): rail(行ヘッダ列)を `super::rail` の実 widget へ切り出したことで、
//! この canvas の `bounds` はもう pane 全幅ではなく**時間場だけ**(`row![rail,
//! canvas]` の右腕 — `super::TimelinePane::view` 参照)。以前は各所で
//! `rail_width` を足し引きしていたが(x 原点が pane 左端=rail 左端だったため)、
//! 今は canvas 自身の x=0 が既に「rail の右端 = 時間場の左端」なので、その
//! 足し引きは全廃した。**意味は不変**(発注書「座標系は関数境界で吸収し、
//! 意味は不変」) — bar/目盛り/プレイヘッドの見た目・当たり判定は
//! 1px も変わらない、変わったのは「どこが原点か」という関数境界の外側の
//! 約束だけ。rail 側の見た目(スウォッチ・名前・M/S/L)はもうここでは描かない
//! (`super::rail::view` が持つ)。
//!
//! **縦スクロール発注(2026-08-22)**: この canvas の y 軸原点もルーラー分
//! 移動した — 以前は y=0 がルーラー上端で行0は `ruler_height` から始まった
//! が、ルーラー(目盛り・ループ帯・マーカー・playhead のルーラー内区間)は
//! `super::ruler::RulerHeader`(常時固定の別 canvas)へ丸ごと移設したので、
//! この canvas の y=0 は**行0の上端**そのもの。`draw`/[`super::hit::hit_test`]/
//! [`super::key_rows`] の呼び出し側は皆この1つの意味へ揃えた(x 軸の
//! TL-arch Phase 1 と同じ「座標シフトは呼び出し側だけで吸収する」手口)。
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

use motolii_store::{Fps, Marker};

use super::key_rows;
use super::projection::{frame_at_x, frame_to_x, tick_steps, time_band_segment_frames};
use super::work_area::WorkArea;
use super::TimelinePane;
use crate::tokens::{Colors, Dimensions};

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

/// ループ帯(作業範囲、正典 §5「ルーラ最上段が専用面」)の高さ。
/// mock `timeline-semantics.html` にループ帯は無い(grep 済み)ので実測転写は
/// できない — 独自比を発明する代わりに、**ルーラーの「大目盛りが届かない
/// 残り」**(`ruler高 − 大目盛り長` = 上半分)を帯に充てる(裁定167: 既存
/// 梯子からの導出。目盛り(下)と帯(上)が排他に住み分け、22px ルーラーで
/// 11px — 実窓で見てから直す型、transport 帯の置き場と同じ姿勢)。
pub fn loop_band_height(ruler_height: f32) -> f32 {
    ruler_height - major_tick_length(ruler_height)
}

pub(crate) fn draw(
    pane: &TimelinePane,
    renderer: &iced::Renderer,
    bounds: Rectangle,
    cursor: iced::mouse::Cursor,
) -> Vec<canvas::Geometry> {
    let mut frame = canvas::Frame::new(renderer, bounds.size());
    let width = bounds.width;
    let row_height = pane.dims.row_height;
    // 罫線幅の倍数で意味を分ける — 1x: ルーラー目盛り(hairline)、1.5x: playhead、
    // 2x: マーカー(最も強い accent)。新しい寸法トークンを増やさず、単一の
    // `border_width` から比で導出する(裁定117 の「寸法は token 経由」の範囲内)。
    let hairline = pane.dims.border_width;

    // **TL-arch Phase 1**: 座標シフトの足し引きは撤去した(モジュール doc
    // 参照) — `width`(= `bounds.width`)は既に時間場だけの幅、`rail_width` を
    // 足す/引く必要はもう無い(`projection::frame_to_x`/`frame_at_x` 自体は
    // 元から時間場ローカル座標のまま — ここが呼び出し側でオフセットを足す
    // 責任を持っていたが、その責任自体が rail 分離で不要になった)。

    // 背景。ゼブラ(裁定148)・行区切り hairline・選択ハイライトはこの canvas
    // の全幅(= 時間場だけ、rail は含まない)で描く — rail 側の同じ状態
    // (選択ハイライト等)は `super::rail::view` が rail 側の container 背景で
    // 独立に描く(2箇所で同じ `row.selected` を読むが、描画先の座標系が別なので
    // 複製ではなく分担)。
    frame.fill_rectangle(Point::ORIGIN, bounds.size(), pane.colors.surface_panel);

    // **縦スクロール発注(2026-08-22)**: ルーラー帯(目盛り・ループ帯・
    // マーカー・playhead のルーラー内区間)はこの canvas から丸ごと撤去した
    // — `super::ruler::RulerHeader` が常時固定の別 canvas として持つ
    // (`TimelinePane::view` が `column![header, scrollable(row![rail, this
    // canvas])]` を組む、`super::ruler` モジュール doc 参照)。この canvas の
    // y=0 はもう「ルーラー下」ではなく「行0の上端」そのもの —
    // `projection::layer_row_top`/`hit::hit_test` の呼び出し側(この関数・
    // `key_rows.rs`・`input.rs`)は皆この1つの意味へ揃えた(ヘッダー分離の
    // 座標シフトは呼び出し側だけで吸収し、`layer_row_top`/`layer_row_at_y`
    // 自体は無改造 — TL-arch Phase 1 の x 軸版と同じ手口)。

    // 明暗のリズム(裁定148・正典 §1.6): クリップ面の「地」に2方向の読解補助を
    // 重ねる。**区切りの手段ではない**(裁定137 との両立整理) — 区切りは
    // 行ごとの下 hairline([`draw`] 末尾)が担う。
    // 順序は 行方向ゼブラ → 時間方向 の順で薄い wash を積む(どちらも
    // token 経由の白 wash、raw 値直書きではない)。
    let rows_top = 0.0;
    let rows_bottom = bounds.height;
    for index in 0..pane.rows.len() {
        if index % 2 == 0 {
            continue; // 偶数行は地のまま(奇数行だけへ wash を乗せる)。
        }
        // 選択 layer の下に property 行が挿入されている間、後続の層行は押し下がる
        // (`TimelinePane::layer_row_top`、EXACT TARGET 1)。
        //
        // x=0 は既にこの canvas の左端(= 時間場の左端、TL-arch Phase 1 で
        // rail が分離された)。**rail は時間カメラの外**(利用者知覚モデル
        // 2026-08-21: 横スケールのジェスチャーは rail に効かず、縦スクロール
        // だけが通る — rail は時間場の上に乗る別レイヤーであって、時間場の
        // wash(ゼブラ・時間帯)を受けない、という意味は canvas が rail 領域を
        // 描かなくなった今も不変)。
        let row_top = rows_top + pane.layer_row_top(index);
        frame.fill_rectangle(
            Point::new(0.0, row_top),
            Size::new(width, row_height),
            pane.colors.timeline_row_zebra,
        );
    }
    draw_time_bands(pane, &mut frame, 0.0, width, rows_top, rows_bottom);

    // 時間方向の縦線(利用者裁定 2026-08-21 夜・σ EXACT TARGET 2、mock
    // `timeline-semantics.html` の `bands()` 第2ループが出典)。帯(面・粗い
    // リズム)とは別の周波数(線・全目盛の細かいリズム) — 描画順は
    // 帯→縦線→bar(地の上・内容物の下)なので、bar を描く層の行ループより
    // 先にここへ置く。
    draw_tick_lines(pane, &mut frame, 0.0, width, rows_top, rows_bottom);

    // 層の行。y=0 は行0の上端(ヘッダー分離済み、上のモジュール doc 参照)。
    for (index, row) in pane.rows.iter().enumerate() {
        let row_top = pane.layer_row_top(index);

        if row.selected {
            // 状態: 選択(`state_selected`)。hover(`surface_hover`、中立グレー)とは
            // 別ロール — 選択は accent 味、hover は明度差だけ(意味色ロールの区別)。
            // **この canvas の全幅**(= 時間場のみ、rail は含まない)— rail 側の
            // 同じ選択ハイライトは `super::rail::view` が独立に描く(canvas.rs
            // 冒頭のモジュール doc「分担」節参照、選択は行そのものの状態
            // であって、レーンバーも同じ行に属する、裁定147)。
            frame.fill_rectangle(
                Point::new(0.0, row_top),
                Size::new(width, row_height),
                pane.colors.state_selected,
            );
        }

        let start_x = frame_to_x(row.start, width, pane.duration_frames);
        let end_x = frame_to_x(row.start + row.duration, width, pane.duration_frames)
            .max(start_x + 1.0);
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
        // (`super::rail::view`)へ一本化した。クリップ上の余白は将来の
        // キーフレームオーバーレイのために空けておく。

        // 音声波形(TL7 統合手順1: 「canvas.rs の bar 描画ループから
        // waveform_segments/waveform_ink を呼ぶ」)。`pane.waveforms` に
        // 何も無い(=波形取得が計画されていない/`Ready` でない)layer は
        // 何も描かない — `waveform_state_segments` 自身が
        // `NotRequested`/`Loading` を空列に落とす(`waveform_view.rs` の
        // 3状態オラクル参照)ので、ここでは呼ぶだけで分岐を増やさない。
        if let Some(state) = pane.waveforms.get(&row.id) {
            let clip_rect = Rectangle {
                x: start_x,
                y: row_top + inset,
                width: (end_x - start_x).max(1.0),
                height: bar_height,
            };
            let segments = crate::waveform_view::waveform_state_segments(state, clip_rect);
            if !segments.is_empty() {
                let ink = crate::waveform_view::waveform_ink(pane.colors.data, bar_color);
                for segment in &segments {
                    let path = canvas::Path::line(
                        Point::new(segment.x, segment.y_top),
                        Point::new(segment.x, segment.y_bottom),
                    );
                    frame.stroke(
                        &path,
                        canvas::Stroke::default().with_color(ink).with_width(pane.dims.border_width),
                    );
                }
            }
        }

        // 行の区切り(裁定139: 面色の塗り分け=ゼブラの明暗だけに頼らず
        // hairline を足す — mock `.trow{border-bottom:...}` と同じ役目)。
        // 行同士は `.prow` と同じ弱い hairline ロール(区切り=線、
        // リズム=地の微差 — §1.6 の両立整理どおり見て区別がつく)。この
        // canvas の全幅(= 時間場のみ)— rail 側の同じ行区切りは
        // `super::rail::view` が container の border で独立に描く
        // (EXACT TARGET 5 の意味は不変、描画先が分かれただけ)。
        draw_hairline(
            hairline,
            &mut frame,
            0.0,
            width,
            row_top + row_height,
            pane.colors.border_hairline_weak,
        );
    }

    // property 行(キー行、第2波 T3・裁定148/151) — 選択 layer の下に挿入する。
    // 帯・キー菱形は `key_rows.rs` が描く(rail 側の property 名ラベルは
    // TL-arch Phase 1 で `super::rail::view` へ移設済み — mod doc 参照)。
    key_rows::draw(pane, &mut frame, width);

    // playhead(Session が正本)。この canvas はもう時間場の行だけなので、
    // オフセットを足す必要は無い(TL-arch Phase 1、モジュール doc 参照)。
    // ルーラー内区間は `super::ruler::RulerHeader` が同じ x から別途描く
    // (常時固定ヘッダー、[`draw_playhead_line`] が唯一の出典 — 2箇所で
    // 別の式にしない、縦スクロール発注 EXACT TARGET 4「playhead は固定」)。
    let playhead_x = frame_to_x(pane.playhead, width, pane.duration_frames);
    draw_playhead_line(pane.colors, hairline, playhead_x, 0.0, bounds.height, &mut frame);

    // レーンバー(行ヘッダ列、裁定147)は TL-arch Phase 1 で実 widget へ移設
    // 済み(`super::rail::view`、`super::TimelinePane::view` が `row![rail,
    // canvas]` として組む) — この canvas はもう rail を描かない。

    // ポインタ近くのタイムコードミニラベル(第2波T5、R1 egui版実測「掴んでいる
    // 間ポインタ近くにタイムコードのミニラベルを出す」を踏襲)。drag 中
    // (`pane.preview_active`、clip/key どちらでも)だけ出す。この canvas は
    // もう時間場だけなので、rail 上かどうかの判定(旧 `position.x >=
    // rail_width`)は不要になった — `cursor.position_in(bounds)` が返す座標は
    // 常に時間場ローカル。
    if pane.preview_active {
        if let Some(position) = cursor.position_in(bounds) {
            let frame_no = frame_at_x(position.x, width, pane.duration_frames);
            frame.fill_text(canvas::Text {
                content: frame_no.to_string(),
                position: Point::new(position.x + pane.dims.spacing_xs, position.y - pane.dims.spacing_l),
                color: pane.colors.action_active,
                size: iced::Pixels(pane.dims.caption_text),
                ..Default::default()
            });
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
/// **`pub(crate)`(縦スクロール発注 2026-08-22)**: 本体は `&TimelinePane` では
/// なく明示引数を取る — `super::ruler::RulerHeader`(常時固定ヘッダー、
/// `TimelinePane` を持たない使い捨てスナップショット)もこの1関数から同じ
/// 目盛りを描くため(2箇所で別の刻み方を持たない、モジュール冒頭「比率の
/// 出典」節と同じ精神)。呼び出し元は今のところ本体の `draw`(削除済み —
/// ルーラーはヘッダーへ移設)と `ruler::RulerHeader::draw` の1箇所のみ。
pub(crate) fn draw_ruler_ticks(
    fps: Option<Fps>,
    duration_frames: i64,
    dims: Dimensions,
    colors: Colors,
    frame: &mut canvas::Frame,
    x0: f32,
    width: f32,
    height: f32,
) {
    if duration_frames <= 0 || width <= 0.0 {
        return;
    }
    let (minor, major) = tick_steps(fps, duration_frames, width, dims.row_height);
    let last_frame = (duration_frames - 1).max(0);
    let mut frame_no = 0i64;
    while frame_no <= last_frame {
        let is_major = frame_no % major == 0;
        let x = x0 + frame_to_x(frame_no, width, duration_frames);
        let top = if is_major {
            height - major_tick_length(height)
        } else {
            height - minor_tick_length(height)
        };
        let color = if is_major {
            colors.border_strong
        } else {
            colors.border_hairline_weak
        };
        let tick_path = canvas::Path::line(Point::new(x, top), Point::new(x, height));
        frame.stroke(
            &tick_path,
            canvas::Stroke::default().with_color(color).with_width(dims.border_width),
        );
        if is_major {
            frame.fill_text(canvas::Text {
                content: frame_no.to_string(),
                position: Point::new(x + dims.spacing_xs, 0.0),
                color: colors.text_secondary,
                size: iced::Pixels(dims.caption_text),
                ..Default::default()
            });
        }
        frame_no += minor;
    }
}

/// 水平の hairline を1本引く(`Point`/`Size` を毎回組まずに済む共通口)。
/// `inspector_pane.rs::bordered_row` の canvas 版 — こちらは per-edge の
/// border-bottom そのもの(canvas は4辺一律の制約が無いので、Inspector側の
/// 「既知の限界」はここには適用されない)。**`pub(crate)`**: `super::ruler`
/// も同じ hairline を引く(`border_width` だけの依存へ縮めた — `draw_ruler_ticks`
/// と同じ理由)。
pub(crate) fn draw_hairline(
    border_width: f32,
    frame: &mut canvas::Frame,
    x0: f32,
    x1: f32,
    y: f32,
    color: iced::Color,
) {
    let path = canvas::Path::line(Point::new(x0, y), Point::new(x1, y));
    frame.stroke(
        &path,
        canvas::Stroke::default().with_color(color).with_width(border_width),
    );
}

/// ループ帯(作業範囲、B21+B18 第1切片・正典 §5)の塗り。ルーラ最上段の
/// 専用面 — 目盛り(下半分)と住み分ける([`loop_band_height`] の導出参照)。
/// ink は状態の器(裁定179: on = accent、off = 静かな gray — 帯は消えない
/// (正典 §5)ので off でも「引いてある」ことは読める)。**縦スクロール発注**:
/// ルーラー内区間の専用面なので `super::ruler::RulerHeader::draw` だけが呼ぶ
/// (本体の行キャンバスはもうルーラーを持たない、モジュール冒頭 doc 参照)。
pub(crate) fn draw_loop_band(
    area: WorkArea,
    loop_enabled: bool,
    duration_frames: i64,
    colors: Colors,
    frame: &mut canvas::Frame,
    width: f32,
    ruler_height: f32,
) {
    let band_height = loop_band_height(ruler_height);
    let x0 = frame_to_x(area.start, width, duration_frames);
    let x1 = frame_to_x(area.end, width, duration_frames).max(x0 + 1.0);
    let band_color = if loop_enabled { colors.action_active } else { colors.border_strong };
    frame.fill_rectangle(Point::new(x0, 0.0), Size::new(x1 - x0, band_height), band_color);
}

/// マーカーの comp フレーム位置。fps が引けない(comp が無い)時は `None` —
/// 黙って誤った位置に描くより、描かない方がまし(M13 と同じ理由)。**唯一の
/// 出典**(旧 `TimelinePane::marker_frame` を吸収 — `super::ruler` も同じ関数を
/// 呼ぶ、2箇所で別の丸めを持たない)。
pub(crate) fn marker_frame(marker: &Marker, fps: Option<Fps>) -> Option<i64> {
    let fps = fps?;
    marker.time.try_to_frame_floor(fps).ok()
}

/// マーカー1本の縦線(ルーラー帯へ重ねる)。**縦スクロール発注**: ルーラー内
/// 区間の専用面なので `super::ruler::RulerHeader::draw` だけが呼ぶ。
pub(crate) fn draw_marker_line(
    color: iced::Color,
    hairline: f32,
    x: f32,
    ruler_height: f32,
    frame: &mut canvas::Frame,
) {
    let marker_path = canvas::Path::line(Point::new(x, 0.0), Point::new(x, ruler_height));
    frame.stroke(
        &marker_path,
        canvas::Stroke::default().with_color(color).with_width(hairline * 2.0),
    );
}

/// playhead の縦線(1本)。**唯一の出典**(縦スクロール発注 EXACT TARGET 4
/// 「playhead は固定」— 本体の行 canvas(`draw` 末尾)と
/// `super::ruler::RulerHeader::draw` の両方が同じ x をこの1関数へ渡すので、
/// 2箇所の playhead 線が同じ式からずれずに描かれる)。
pub(crate) fn draw_playhead_line(
    colors: Colors,
    hairline: f32,
    x: f32,
    y0: f32,
    y1: f32,
    frame: &mut canvas::Frame,
) {
    let playhead_path = canvas::Path::line(Point::new(x, y0), Point::new(x, y1));
    frame.stroke(
        &playhead_path,
        canvas::Stroke::default().with_color(colors.action_active).with_width(hairline * 1.5),
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

    /// ループ帯はルーラーの「大目盛りが届かない残り」(上半分)— 目盛りと
    /// 排他に住み分ける(重なる帯を作らない)。
    #[test]
    fn loop_band_and_major_ticks_partition_the_ruler() {
        let ruler = ruler_height(MOCK_ROW_HEIGHT);
        assert_eq!(loop_band_height(ruler), 11.0);
        assert_eq!(loop_band_height(ruler) + major_tick_length(ruler), ruler, "帯+大目盛=ルーラー全高");
    }
}
