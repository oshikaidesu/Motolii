//! マーカーレーン(locator、B19 第1切片)の**意味の純関数+描画部品**。
//! 正典 `next/reference/timeline-grammar.md` §5「locator」:
//! **M タップ=playhead へ即置き**(名前入力に入らない・同一フレーム連打は
//! 1つに畳む・再生中でも打てる)・ドラッグ移動・クリックでジャンプ・
//! 右クリックで削除。
//!
//! ## map 行(bundle B19、`next/reference/normal-map.tsv`)
//! - **基盤として残る(時刻変更UIは未結線)**: 728(Add and Modify Marker —
//!   追加+ドラッグでの時刻変更)・738/739(Modify Marker — 同、時刻の modify
//!   のみ)。名前の modify は `timeline.marker_panel` で一覧・改名まで結線済み。
//! - **採用済の UI 入口を補強**: 717(追加)・718(削除)・743(表示)・
//!   722/723(ジャンプ — click jump は K/J ナビの補完)
//! - **見送り**: 729/730/731/733/742(フラグ — store `marker.rs` に flag が
//!   無い。store 拡張が先)・732/741(尺マーカー ⇄ In/Out 変換 — work_area
//!   との統合、B18 側と要調整)・734-737(Go to In/Out — 作業範囲ナビ、
//!   marker store 非依存)・744(クリップマーカー同期 — store は comp マーカー
//!   のみ。レイヤーマーカーは正典 §8.2 の store 設計束から)
//!
//! ## 家(裁定111/120/149 既払いの基盤の上)
//! 意味の正本は store の `Marker`(`next/core/motolii-store/src/marker.rs` —
//! Lottie `cm`/`tm`/`dr`、comp の component、順序=宣言順、id 無し)。編集は
//! `Intent::SetMarkers`(丸ごと差し替え)1本 — このモジュールは
//! 「`Vec<Marker>` を受けて新しい `Vec<Marker>` を返す」純関数だけを持ち、
//! Document には触らない(`work_area.rs` と同じ「値を受けて値を返す」型)。
//!
//! ## 別レーン走行中のための独立性(発注 EXACT TARGET)
//! 同 crate の `write.rs`/`input.rs`/`canvas.rs` とは責任を分ける — Message は
//! [`MarkerMessage`](独立 enum)で表現し、名前入力の下書きだけを `PaneState` に
//! 閉じる。確定時の `Intent::SetMarkers` は supervisor(Shell) が差す。
//!
//! ## 置き場(意匠 — 既存目盛り梯子と衝突しない位置)
//! ルーラ帯の縦の家割り(正典 §5「ルーラ帯 = ループ帯(最上段)+locator 帯+
//! 目盛+playhead」)を、**既存の梯子関数だけから導出**する(裁定167 —
//! 独自の中間値を発明しない、`canvas::loop_band_height` と同じ導出の型):
//!
//! ```text
//! y=0 ┌───────────────┐ ループ帯(canvas::loop_band_height = ruler − 大目盛り長)
//!     ├───────────────┤ ← locator_head_top
//!     │ locator 頭部  │   高さ = 大目盛り長 − 小目盛り長(小目盛りが届かない残り)
//!     ├───────────────┤ ← ruler − minor_tick_length(小目盛りの上端)
//!     │ 目盛り(下)  │
//! y=h └───────────────┘
//! ```
//!
//! 頭部(下向き三角のピン)はこの帯に住み、柄(hairline)だけが目盛り域を
//! 貫いてルーラ下端まで降りる(playhead が全高を貫くのと同じ「線は共存・
//! 面は住み分け」)。三角の底辺幅=頭部高(実窓で見てから直す型 —
//! `loop_band_height` の導出と同じ姿勢)。色は tokens のみ
//! (`way_timeline`/`text_secondary`/`border_width`/`caption_text`/`spacing_xs`)。
//! icon は不要(グリフは canvas パスで描く — フラグ見送りにより旗 icon も不要)。

use iced::widget::{button, canvas, column, container, row, scrollable, text, text_input};
use iced::{Element, Length, Point};

use motolii_store::{Fps, Marker, RationalTime};

use crate::canvas::{loop_band_height, major_tick_length, minor_tick_length};
use crate::projection::frame_to_x;
use crate::tokens::{Colors, Dimensions};
use crate::{Message, TimelinePane};

/* motolii-component
id = "timeline.marker_panel"
kind = "semantic"
weight = "core_edit"
maps = [1014]
entry = ["MarkerMessage", "marker_panel"]
meaning = ["Marker", "renamed"]
evaluation = ["marker_label", "renamed"]
render = ["marker_panel"]
observable = ["marker_panel_shows_named_markers", "marker_panel_rename_commits_and_undoes_in_one_step"]
*/

// ---------------------------------------------------------------------------
// 縦の家割り(梯子からの導出のみ — 新しい寸法 token は増やさない)
// ---------------------------------------------------------------------------

/// locator 頭部の上端(ルーラ帯ローカル y)。ループ帯の下端そのもの
/// (`canvas::loop_band_height` — 面の住み分けで隣接、隙間も重なりも無い)。
pub fn locator_head_top(ruler_height: f32) -> f32 {
    loop_band_height(ruler_height)
}

/// locator 頭部の高さ。**大目盛り長 − 小目盛り長** = 小目盛りが届かない
/// 中間帯(22px ルーラで 11−5 = 6px)。大目盛りの線だけがこの帯を横切るが、
/// 線と面の共存は playhead と同じ扱い(モジュール doc の家割り図参照)。
pub fn locator_head_height(ruler_height: f32) -> f32 {
    major_tick_length(ruler_height) - minor_tick_length(ruler_height)
}

/// locator の掴み許容(px)。`work_area::LOOP_GRAB`/`hit::TRIM_EDGE` と同じ
/// 「正典 §1 の文法定数はコード定数」の型 — 点 locator は線なので両側に開く。
pub const LOCATOR_GRAB: f32 = 8.0;

// ---------------------------------------------------------------------------
// 投影(store → 画面の純関数)
// ---------------------------------------------------------------------------

/// マーカー1枚の画面向け投影。`RowProjection` と同じ役割分担で **frame を
/// 持ち px を持たない**(px への換算は draw/hit 時に `frame_to_x` — 窓幅は
/// 毎フレーム変わりうる)。`index` は store の `Vec<Marker>` の添字
/// (マーカーに安定 id は無い — 宣言順が正本、store `marker.rs` doc)。
#[derive(Clone, Debug, PartialEq)]
pub struct LocatorProjection {
    /// store の `Vec<Marker>` 内の添字(編集 Message はこれで名指す)。
    pub index: usize,
    /// ロケータの comp フレーム位置(`tm` の floor — 既存
    /// `TimelinePane::marker_frame` と同じ丸め)。
    pub frame: i64,
    /// 範囲ロケータの尺(フレーム)。0 = 単発(`dr` = 0、store doc)。
    pub duration_frames: i64,
    /// ラベル(`cm`)。空 = 無名(M タップ即置きの既定 — 正典 §5)。
    pub name: String,
}

/// マーカーの comp フレーム位置。fps が引けない(comp が無い)時は `None` —
/// 黙って誤った位置に描くより描かない(M13、`TimelinePane::marker_frame` と
/// 同じ理由・同じ floor 丸め)。
pub fn locator_frame(marker: &Marker, fps: Fps) -> Option<i64> {
    marker.time.try_to_frame_floor(fps).ok()
}

/// store 投影の純関数(発注やること2)。fps 無し(comp 無し)なら空 —
/// 位置が出せないロケータは1枚も並べない。順序は宣言順のまま。
pub fn project(markers: &[Marker], fps: Option<Fps>) -> Vec<LocatorProjection> {
    let Some(fps) = fps else {
        return Vec::new();
    };
    markers
        .iter()
        .enumerate()
        .filter_map(|(index, marker)| {
            let frame = locator_frame(marker, fps)?;
            let duration_frames = marker.duration.try_to_frame_floor(fps).unwrap_or(0).max(0);
            Some(LocatorProjection {
                index,
                frame,
                duration_frames,
                name: marker.name.clone(),
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// 当たり(押した瞬間の座標1回だけ — 正典 §1)
// ---------------------------------------------------------------------------

/// `y`(ルーラ帯ローカル)が locator の縦域か。頭部上端からルーラ下端まで —
/// 柄も掴める(Q0「触れそうで触れない物は不合格」: 見えている線の全長が
/// 触れる)。ループ帯(`y < locator_head_top`)とは排他で住み分け。
pub fn in_locator_lane(y: f32, ruler_height: f32) -> bool {
    y >= locator_head_top(ruler_height) && y < ruler_height
}

/// `x`(時間場ローカル px)に最も近い locator の添字。±[`LOCATOR_GRAB`] の
/// 外は `None`(→ 既存の scrub 経路へ)。範囲ロケータは span 内=距離0。
/// **同距離なら後の添字**(後から描いた方が上に見える — 絵と当たりの一致)。
pub fn locator_at(
    x: f32,
    width: f32,
    duration_frames: i64,
    locators: &[LocatorProjection],
) -> Option<usize> {
    let mut best: Option<(f32, usize)> = None;
    for locator in locators {
        let x0 = frame_to_x(locator.frame, width, duration_frames);
        let x1 = if locator.duration_frames > 0 {
            frame_to_x(locator.frame + locator.duration_frames, width, duration_frames).max(x0)
        } else {
            x0
        };
        let distance = if x < x0 {
            x0 - x
        } else if x > x1 {
            x - x1
        } else {
            0.0
        };
        if distance <= LOCATOR_GRAB && best.map_or(true, |(d, _)| distance <= d) {
            best = Some((distance, locator.index));
        }
    }
    best.map(|(_, index)| index)
}

// ---------------------------------------------------------------------------
// 意味の純関数(Vec<Marker> → Vec<Marker> — Document 不接触)
// ---------------------------------------------------------------------------

/// M タップ=playhead へ即置き(map 717 の UI 入口・728 の add 側、正典 §5)。
/// [`added_at_frame`] の playhead 特殊形(2入口目の右クリック追加
/// (`AddMarkerAt`/S2 発注 #22)と意味を共有するための一般化)。
pub fn added_at_playhead(markers: &[Marker], playhead: i64, fps: Fps) -> Option<Vec<Marker>> {
    added_at_frame(markers, playhead, fps)
}

/// 任意フレームへ即置き(`added_at_playhead` の一般化 — S2 発注 #22
/// 「マーカー追加 UI が無い」の穴埋めで、ルーラ locator lane 右クリック
/// (`Message::AddMarkerAt`)が playhead 以外の座標を渡せるようにした)。
/// 名前は空(名前入力に入らない — メニュー経由の即リネームは supervisor の
/// TextEdit 結線)・尺 0(単発)。**同一フレームに既存なら `None`**(連打は
/// 1つに畳む — 正典 §5 の明文なので、M13 の「黙る」違反ではない)。
/// fps 変換が立たない時も `None`(壊れた時刻を書き込まない)。
pub fn added_at_frame(markers: &[Marker], frame: i64, fps: Fps) -> Option<Vec<Marker>> {
    let time = RationalTime::try_from_frame(frame, fps).ok()?;
    if markers.iter().any(|m| locator_frame(m, fps) == Some(frame)) {
        return None; // 同一フレーム連打は1つに畳む(正典 §5)。
    }
    let zero = RationalTime::try_from_frame(0, fps).ok()?;
    let mut next = markers.to_vec();
    next.push(Marker {
        name: String::new(),
        time,
        duration: zero,
    });
    Some(next)
}

/// ドラッグ移動の1回分(map 728/738/739 の時刻 modify、正典 §5)。時刻だけを
/// `new_frame`(comp 内へ clamp)へ差し替え、名前・尺は保つ。添字が外れて
/// いる・fps 変換が立たない時は `None`(壊さない)。
pub fn moved(
    markers: &[Marker],
    index: usize,
    new_frame: i64,
    fps: Fps,
    duration_frames: i64,
) -> Option<Vec<Marker>> {
    let target = markers.get(index)?;
    let cap = if duration_frames > 0 { duration_frames - 1 } else { i64::MAX };
    let clamped = new_frame.clamp(0, cap.max(0));
    let time = RationalTime::try_from_frame(clamped, fps).ok()?;
    let mut next = markers.to_vec();
    next[index] = Marker {
        name: target.name.clone(),
        time,
        duration: target.duration.clone(),
    };
    Some(next)
}

/// 右クリック削除(map 718 の UI 入口、正典 §5)。添字が外れていれば `None`。
pub fn removed(markers: &[Marker], index: usize) -> Option<Vec<Marker>> {
    if index >= markers.len() {
        return None;
    }
    let mut next = markers.to_vec();
    next.remove(index);
    Some(next)
}

/// マーカー名の確定。入力欄の下書きは UI 側に置き、ここでは確定した名前だけを
/// 既存の `Marker` へ適用する。空名は許す(名前を消して無名のビートへ戻すため)。
/// 同名・範囲外は no-op として undo を汚さない。
pub fn renamed(markers: &[Marker], index: usize, name: &str) -> Option<Vec<Marker>> {
    let target = markers.get(index)?;
    let name = name.trim().to_owned();
    if target.name == name {
        return None;
    }
    let mut next = markers.to_vec();
    next[index] = Marker {
        name,
        time: target.time.clone(),
        duration: target.duration.clone(),
    };
    Some(next)
}

/// 一覧で見せる名前。無名マーカーも順番を失わないよう「Marker N」と表示し、
/// 時刻は常にフレームで添える(クリックで同じ位置へ戻れることを観測できる顔)。
pub fn marker_label(marker: &Marker, index: usize, fps: Option<Fps>) -> String {
    let name = if marker.name.trim().is_empty() {
        format!("Marker {}", index + 1)
    } else {
        marker.name.clone()
    };
    let frame = fps
        .and_then(|fps| locator_frame(marker, fps))
        .map_or_else(|| "?".to_owned(), |frame| frame.to_string());
    format!("{name}  ·  {frame}f")
}

// ---------------------------------------------------------------------------
// pane-local Message(独立 enum — 統合は supervisor)
// ---------------------------------------------------------------------------

/// マーカーレーンの Message(`crate::Message` へは**合流させていない** —
/// 発注 EXACT TARGET「独立 enum」。supervisor が `crate::Message` に
/// `Marker(MarkerMessage)` の1腕を足して畳むか、腕ごと写すかは統合判断)。
#[derive(Debug, Clone)]
pub enum MarkerMessage {
    /// M タップ / メニュー「Add Marker」(map 717/728)。playhead 位置へ即置き。
    AddAtPlayhead,
    /// ルーラ locator lane 右クリック(S2 発注 #22 の2入口目、S6 併存
    /// 裁定195)。クリック位置のフレームへ即置き([`added_at_frame`] 参照)。
    AddAtFrame(i64),
    /// locator を掴んだ瞬間(正典 §1「判定は押した瞬間の座標」)。
    Grabbed { index: usize, at_frame: i64 },
    /// ドラッグ中のポインタ移動。絶対値で出し直す([`MarkerDrag::dragged`])。
    DragMoved { at_frame: i64 },
    /// release = 確定(未移動なら no-op — [`MarkerDrag::finish`])。
    DragReleased,
    /// Esc / 右クリック = 破棄(裁定151「キャンセルの一般化」)。
    DragCancelled,
    /// クリックでジャンプ(正典 §5、map 722/723 の K/J ナビの補完)。
    /// playhead の書き込みは shell の `ScrubTo` 系と同じ経路へ写る。
    JumpTo(i64),
    /// 右クリック削除(map 718 の UI 入口)。
    Remove(usize),
    /// 一覧の名前編集を開始する。Document にはまだ触れない。
    RenameBegin(usize),
    /// 一覧の名前下書き。確定まで Document/undo には触れない。
    RenameEdited(String),
    /// 一覧の名前を1回の `Intent::SetMarkers` として確定する。
    RenameCommit,
    /// 一覧の名前編集を取り消す。
    RenameCancel,
    /// 名前編集を確定した値。Shell の意味更新へ渡す最終形。
    Rename { index: usize, name: String },
}

// ---------------------------------------------------------------------------
// ドラッグの一時状態(transient — Document は release まで不接触)
// ---------------------------------------------------------------------------

/// locator ドラッグ移動の進行中状態。`write.rs` の `TimelineDragState` と同じ
/// 「掴んだ瞬間の値を控え、毎回**絶対値で出し直す**(delta 蓄積禁止、正典
/// §2)・release で1回だけ確定(1 gesture = 1 undo)・キャンセルは捨てるだけ」
/// の型。置き場(`PaneState` への同居)は supervisor — この struct 自身は
/// どこに置かれても完結する。
#[derive(Clone)]
pub struct MarkerDrag {
    index: usize,
    /// 掴んだ瞬間の `Vec<Marker>` 丸ごと(`Intent::SetMarkers` が丸ごと
    /// 差し替えなので、確定形も丸ごと持つのが素直)。
    origin: Vec<Marker>,
    /// 掴んだ locator の掴んだ瞬間のフレーム(絶対値計算の基準)。
    origin_frame: i64,
    /// 掴んだ瞬間のポインタ位置(comp frame)。
    grab_at_frame: i64,
    preview: Vec<Marker>,
}

impl MarkerDrag {
    /// 掴んだ瞬間([`MarkerMessage::Grabbed`])。添字が外れている・fps で
    /// 位置が出せない locator は掴めない(`None` — 安全側)。
    pub fn start(markers: &[Marker], index: usize, at_frame: i64, fps: Fps) -> Option<Self> {
        let origin_frame = locator_frame(markers.get(index)?, fps)?;
        Some(Self {
            index,
            origin: markers.to_vec(),
            origin_frame,
            grab_at_frame: at_frame,
            preview: markers.to_vec(),
        })
    }

    /// ドラッグ中のポインタ移動([`MarkerMessage::DragMoved`])。掴んだ瞬間の
    /// 値を基準に**絶対値で出し直す**(delta 蓄積禁止)。
    pub fn dragged(&mut self, at_frame: i64, fps: Fps, duration_frames: i64) {
        let new_frame = self.origin_frame + (at_frame - self.grab_at_frame);
        if let Some(next) = moved(&self.origin, self.index, new_frame, fps, duration_frames) {
            self.preview = next;
        }
    }

    /// 描画用のプレビュー(ドラッグ中はこちらを `project` へ渡す — 正典 §5.5
    /// 「プレビューは毎フレーム、Document 書き込みは確定1回」)。
    pub fn preview(&self) -> &[Marker] {
        &self.preview
    }

    /// release = 確定([`MarkerMessage::DragReleased`])。**未移動なら `None`**
    /// (no-op — 履歴を汚さない)。`Some` は `Intent::SetMarkers { markers }` へ
    /// そのまま渡す1回分(1 gesture = 1 undo)。キャンセルは finish を呼ばず
    /// この struct を捨てるだけ(Document 不接触なので完全に無傷)。
    pub fn finish(self) -> Option<Vec<Marker>> {
        if self.preview == self.origin {
            return None;
        }
        Some(self.preview)
    }
}

// ---------------------------------------------------------------------------
// 絵(canvas 部品 — `canvas::draw` への実配線は supervisor)
// ---------------------------------------------------------------------------

/// マーカーレーンを描く(発注やること2「表示・ラベル表示」)。既存
/// `canvas::draw` のマーカー全高2x線ループの**置き換え先**(供給は supervisor:
/// `project(&pane.markers, pane.fps)` の結果と `pane.ruler_height()` を渡す)。
///
/// - 範囲ロケータ: 頭部帯の高さpremで `[frame, frame+dr)` の面
/// - 頭部: 下向き三角(底辺=頭部上端、頂点=頭部下端 — ピン)
/// - 柄: 頭部下端 → ルーラ下端の hairline(1x — playhead 1.5x より弱く、
///   旧マーカー線の2x を廃してルーラ内の線の強さ順を playhead > 柄 = 目盛りへ)
/// - ラベル: 頭部の右 `spacing_xs`、`caption_text`/`text_secondary`
///   (大目盛りラベル(上端)とは縦域が別 — 家割り図参照)
pub fn draw_locators(
    frame: &mut canvas::Frame,
    locators: &[LocatorProjection],
    width: f32,
    duration_frames: i64,
    ruler_height: f32,
    dims: &Dimensions,
    colors: &Colors,
) {
    let head_top = locator_head_top(ruler_height);
    let head_height = locator_head_height(ruler_height);
    let head_bottom = head_top + head_height;
    let half_width = head_height * 0.5; // 底辺幅=頭部高(モジュール doc「実窓で見てから直す型」)。

    for locator in locators {
        let x = frame_to_x(locator.frame, width, duration_frames);

        // 範囲ロケータの面(dr > 0)— 頭部帯に住む(目盛りと衝突しない)。
        if locator.duration_frames > 0 {
            let x1 = frame_to_x(locator.frame + locator.duration_frames, width, duration_frames)
                .max(x + 1.0);
            frame.fill_rectangle(
                Point::new(x, head_top),
                iced::Size::new(x1 - x, head_height),
                colors.way_timeline,
            );
        }

        // 頭部(下向き三角のピン)。
        let head = canvas::Path::new(|builder| {
            builder.move_to(Point::new(x - half_width, head_top));
            builder.line_to(Point::new(x + half_width, head_top));
            builder.line_to(Point::new(x, head_bottom));
            builder.close();
        });
        frame.fill(&head, colors.way_timeline);

        // 柄(目盛り域を貫いてルーラ下端まで — 線は共存、playhead と同じ扱い)。
        let stem = canvas::Path::line(Point::new(x, head_bottom), Point::new(x, ruler_height));
        frame.stroke(
            &stem,
            canvas::Stroke::default()
                .with_color(colors.way_timeline)
                .with_width(dims.border_width),
        );

        // ラベル(cm)。無名(M タップ既定)は描かない。
        if !locator.name.is_empty() {
            frame.fill_text(canvas::Text {
                content: locator.name.clone(),
                position: Point::new(x + half_width + dims.spacing_xs, head_top),
                color: colors.text_secondary,
                size: iced::Pixels(dims.caption_text),
                ..Default::default()
            });
        }
    }
}

// ---------------------------------------------------------------------------
// 一覧パネル(マーカーの名前・時刻・戻り先を1箇所に集める)
// ---------------------------------------------------------------------------

/// タイムライン上部のコンパクトなマーカー一覧。描画線と同じ `Marker` を読む
/// だけで、別のマーカー台帳や再生位置を持たない。名前編集中だけ下書きを受け、
/// 確定は `MarkerMessage::RenameCommit` で Shell に返す。
pub fn marker_panel(pane: &TimelinePane) -> Element<'static, Message> {
    let count = pane.markers.len();
    let header = row![
        text("Markers").size(pane.dims.caption_text),
        text(format!("{count}"))
            .size(pane.dims.caption_text)
            .color(pane.colors.text_secondary),
    ]
    .spacing(pane.dims.spacing_s)
    .align_y(iced::alignment::Vertical::Center);

    let mut rows = Vec::with_capacity(pane.markers.len().max(1));
    if pane.markers.is_empty() {
        rows.push(
            text("M でビート/歌詞位置を置く")
                .size(pane.dims.caption_text)
                .color(pane.colors.text_secondary)
                .into(),
        );
    } else {
        for (index, marker) in pane.markers.iter().enumerate() {
            let frame = pane
                .fps
                .and_then(|fps| locator_frame(marker, fps))
                .unwrap_or(0);
            let name: Element<'static, Message> = match pane.marker_rename.as_ref() {
                Some((editing, draft)) if *editing == index => text_input("marker name", draft.clone())
                    .on_input(|value| Message::Marker(MarkerMessage::RenameEdited(value)))
                    .on_submit(Message::Marker(MarkerMessage::RenameCommit))
                    .size(pane.dims.caption_text)
                    .padding([0.0, pane.dims.spacing_xs])
                    .width(Length::Fill)
                    .into(),
                _ => text(marker_label(marker, index, pane.fps))
                    .size(pane.dims.caption_text)
                    .color(pane.colors.text_primary)
                    .width(Length::Fill)
                    .into(),
            };

            let jump = button(text(format!("{frame}f")).size(pane.dims.caption_text))
                .on_press(Message::Marker(MarkerMessage::JumpTo(frame)))
                .padding([0.0, pane.dims.spacing_xs]);
            let action = match pane.marker_rename.as_ref() {
                Some((editing, _)) if *editing == index => {
                    button(text("Cancel").size(pane.dims.caption_text))
                        .on_press(Message::Marker(MarkerMessage::RenameCancel))
                }
                _ => button(text("Rename").size(pane.dims.caption_text))
                    .on_press(Message::Marker(MarkerMessage::RenameBegin(index))),
            };
            let remove = button(text("Delete").size(pane.dims.caption_text))
                .on_press(Message::Marker(MarkerMessage::Remove(index)))
                .padding([0.0, pane.dims.spacing_xs]);

            rows.push(
                row![name, jump, action, remove]
                    .spacing(pane.dims.spacing_xs)
                    .align_y(iced::alignment::Vertical::Center)
                    .height(Length::Fixed(pane.dims.row_height))
                    .into(),
            );
        }
    }

    container(
        column![
            header,
            scrollable(column(rows)).height(Length::Fixed(pane.dims.row_height * 4.0))
        ]
        .spacing(pane.dims.spacing_xs)
        .padding([pane.dims.spacing_xs, pane.dims.spacing_s]),
    )
    .width(Length::Fill)
    .into()
}

#[cfg(test)]
mod panel_tests {
    use super::*;

    fn marker(name: &str, frame: i64) -> Marker {
        let fps = Fps::try_new(30, 1).expect("valid fps");
        Marker {
            name: name.to_owned(),
            time: RationalTime::try_from_frame(frame, fps).expect("valid time"),
            duration: RationalTime::try_from_frame(0, fps).expect("valid duration"),
        }
    }

    #[test]
    fn renamed_changes_only_the_requested_marker() {
        let markers = vec![marker("Verse", 30), marker("", 60)];
        let next = renamed(&markers, 1, "Chorus").expect("name changed");
        assert_eq!(next[0], markers[0]);
        assert_eq!(next[1].name, "Chorus");
        assert!(renamed(&next, 1, " Chorus ").is_none(), "trimmed same name is a no-op");
    }

    #[test]
    fn marker_panel_shows_named_markers() {
        let fps = Fps::try_new(30, 1).expect("valid fps");
        assert_eq!(marker_label(&marker("Chorus", 45), 0, Some(fps)), "Chorus  ·  45f");
        assert_eq!(marker_label(&marker("", 45), 2, Some(fps)), "Marker 3  ·  45f");
    }
}
