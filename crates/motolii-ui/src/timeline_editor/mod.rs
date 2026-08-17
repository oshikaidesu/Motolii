//! egui Timeline エディタ。**実際の Document を、実際の行モデルで、手で触る。**
//!
//! 旧 `examples/timeline_egui_lab.rs` の中身をそのまま module にした席で、
//! shell(`blitz_shell`)の Timeline pane と lab example の両方が同じ実装を呼ぶ。
//! example は `run_lab` を呼ぶだけの薄い起動殻である。
//!
//! ここには偽のTimeline状態を置かない。行は `timeline_rows::rows()` が
//! 実 Document から毎フレーム作ったものであり、開閉はその `TimelineFoldState` である。
//! だから**この窓で見えていることは、そのまま製品の行モデルの挙動**である。
//!
//! 触れるもの:
//!   - グループ左端の `▸`/`▾` … 子レイヤーの開閉
//!   - `◇`/`◆`            … そのレイヤーのキーパラメータ行の開閉（子とは独立）
//!   - clip bar をドラッグ … 時間方向へ移動。**キーが一緒に動く**
//!
//!   - bar の左右端6px      … トリム（in / out）
//!   - 菱形                 … 掴んでキーの時刻を変える(パラメータを問わない)
//!   - 左列を上下へドラッグ … **並べ替え。Group の中へも出し入れできる**
//!   - `M` / `S`           … mute（`visible`）/ solo の反転
//!   - 左列クリック         … 選択。`Cmd` で足し引き、`Shift` で範囲
//!   - 右クリック           … その場のメニュー。**名前の変更はここから**
//!     (ダブルクリックは使わない — 選択・並べ替え・跳ぶ が同じ場所に
//!      重なっている面では、2回目の押下が別の操作の途中と区別できない)
//!   - `Cmd/Ctrl + Z` / `Shift+Z` … Undo / Redo
//!   - `Cmd/Ctrl + D`      … 選択中の layer を複製
//!   - `Delete` / `Backspace` … 選択中の layer を削除（Group は中身ごと）
//!
//! ## 面の動かし方（AE / Premiere と同じ割り当て）
//!
//! ```text
//! ホイール / 二本指        縦スクロール
//! Shift + ホイール         横パン
//! Cmd(⌘) + ホイール        横ズーム（カーソル下の時刻が動かない）
//! ピンチ                   横ズーム
//! 下端のナビゲータ帯       掴んで横パン、両端6pxでズーム
//! ```
//!
//! **レイヤーの全体地図(minimap)は置かない。** 1 Layer = 1行なので行の一覧が
//! 既に全体図であり、その縮小版を別に持っても情報が増えない。時間方向だけは
//! 寄ると全体が見えなくなるので、そこにナビゲータ帯を置く。
//!
//! **ドラッグは Document を実際に書き換える。** 経路は
//! `DocumentWriter::prepare_* -> apply_command(gesture, command)` で、
//! 1ドラッグ = 1 `GestureId` = 1 Undo 単位である。撤去された
//! `document_edit_runtime` を経由しない — D2 が既に薄い入口を持っていた。
//!
//! 実行: `cargo run --profile fast -p motolii-ui --example timeline_egui_lab`

mod audio_seat;

use std::collections::HashMap;

use crate::timeline_rows::{rows, ParamRef, RowKind, TimelineFoldState, TimelineRow};
use audio_seat::{AudioPlayback, WallClockReason};
use motolii_audio::PcmCache;
use eframe::egui;
use egui::{Align2, Color32, CornerRadius, FontId, Rect, Sense, Stroke, StrokeKind, Vec2};
use motolii_core::{Fps, RationalTime};
use motolii_doc::{
    collect_layer_ids, find_item_location, Clip, ClipSource, Command, CommandError, DocKeyframe,
    DocKeyframeTrack, DocParam, DocValue, Document, DocumentWriter, GestureId, Group, ItemEnvelope,
    KeyframeId, LayerId, ParentLocator, Track, TrackItem, Transform2D,
};
use motolii_eval::Interp;
use std::sync::Arc;

// mock_tokens が timeline-library.html から出した値（面の大きさで動かないものだけ）
const BG: Color32 = Color32::from_rgb(0x29, 0x29, 0x29);
const HEAD_BG: Color32 = Color32::from_rgb(0x38, 0x38, 0x38);
const CELL: Color32 = Color32::from_rgb(0x36, 0x36, 0x36);
const CELL_CHILD: Color32 = Color32::from_rgb(0x30, 0x30, 0x30);
const CELL_PROP: Color32 = Color32::from_rgb(0x29, 0x29, 0x29);
const TRACK_A: Color32 = Color32::from_rgb(0x37, 0x37, 0x37);
const TRACK_B: Color32 = Color32::from_rgb(0x25, 0x25, 0x25);
/// 数字を持たない細目盛。**主目盛より弱く、下地より濃い**
const TRACK_MINOR: Color32 = Color32::from_rgb(0x2b, 0x2b, 0x2b);
/// 1区間おきの下地。**目盛と目盛のあいだを図にする**(Ableton の明暗)
const TRACK_BAND: Color32 = Color32::from_rgb(0x2f, 0x2f, 0x2f);
const RULE: Color32 = Color32::from_rgb(0x11, 0x11, 0x11);
const INK: Color32 = Color32::from_rgb(0xd4, 0xd4, 0xd4);
const DIM: Color32 = Color32::from_rgb(0x8d, 0x8d, 0x8d);
const ACCENT: Color32 = Color32::from_rgb(0xe9, 0xcf, 0x72);
const KEY_IDLE: Color32 = Color32::from_rgb(0x35, 0x35, 0x35);
// M / S が入っているときの下地。モックの採用値
const MUTE_ON: Color32 = Color32::from_rgb(0x65, 0x3b, 0x34);
const SOLO_ON: Color32 = Color32::from_rgb(0x66, 0x5b, 0x32);
/// L(編集禁止)が入っているときの下地
const LOCK_ON: Color32 = Color32::from_rgb(0x3f, 0x4e, 0x5c);
/// 親から受けているロック。**自分では外せない**ので弱く出す
const LOCK_INHERITED: Color32 = Color32::from_rgb(0x2f, 0x37, 0x3d);
/// **仮のパレット。** 値は `mock_tokens`(HTML/CSS を解いて取り出す)の担当で、
/// ここで発明した色を正本にしない — 並べて見るための当て馬である。
const LAYER_COLORS: [Color32; 8] = [
    Color32::from_rgb(0x8c, 0x6b, 0x6b),
    Color32::from_rgb(0x8c, 0x7d, 0x5c),
    Color32::from_rgb(0x7d, 0x8c, 0x5c),
    Color32::from_rgb(0x5c, 0x8c, 0x6f),
    Color32::from_rgb(0x5c, 0x7f, 0x8c),
    Color32::from_rgb(0x64, 0x66, 0x8c),
    Color32::from_rgb(0x7d, 0x5c, 0x8c),
    Color32::from_rgb(0x8c, 0x5c, 0x74),
];

/// パレットの生値(command へ入れるのは `u32`)。上の `LAYER_COLORS` と同じ並び
const LAYER_COLORS_RGB: [u32; 8] = [
    0x8c6b6b, 0x8c7d5c, 0x7d8c5c, 0x5c8c6f, 0x5c7f8c, 0x64668c, 0x7d5c8c, 0x8c5c74,
];

/// 選択の点灯。**白にする** — 将来レイヤーごとに色を散らすので、
/// 選択がその色の1つに見えてはいけない(選択は状態であって、持ち物ではない)
const SELECTED: Color32 = Color32::from_rgb(0xf2, 0xf2, 0xf2);

const RAIL_W: f32 = 196.0;
/// 行の高さ(小)。**2026-08-08 決定の「行高は固定・最小20px」の最小側**
const ROW_H: f32 = 24.0;
const PROP_H: f32 = 20.0;
/// 行の高さ(大)。名前と bar を見やすくするだけで、意味は変わらない
const ROW_H_LARGE: f32 = 34.0;
const PROP_H_LARGE: f32 = 26.0;
const HEAD_H: f32 = 34.0;
const RULER_H: f32 = 36.0;
/// ルーラが覆う秒数の**初期値**。以後は `TimelineView` が持つ。
const TIMELINE_SECONDS: f32 = 16.0;
/// これ以上は寄れない。1秒を4分割まで
const MIN_SPAN: f32 = 0.25;
/// 下端の時間ナビゲータ帯の高さ
const NAV_H: f32 = 14.0;
/// 縦のつまみの幅
const SCROLLBAR_W: f32 = 6.0;

/// 時間軸の見えている窓。**Project session が持つ状態**で、Document には入れない。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimelineView {
    /// 左端の時刻(秒)
    pub start: f32,
    /// 見えている秒数
    pub span: f32,
}

impl TimelineView {
    /// 時刻 → 面の中の x。**時間↔x の換算はこの2本しか無い。**
    pub fn time_to_x(&self, t: f32, track_left: f32, track_w: f32) -> f32 {
        track_left + (t - self.start) / self.span * track_w
    }

    /// x → 時刻。`time_to_x` の逆。
    pub fn x_to_time(&self, x: f32, track_left: f32, track_w: f32) -> f32 {
        self.start + (x - track_left) / track_w * self.span
    }

    /// `anchor` の時刻を動かさずに寄る/引く。`factor` < 1 で寄る。
    ///
    /// **カーソルの下の時刻が動かないことが、ズームの手触りそのもの**である。
    pub fn zoom_at(self, anchor: f32, factor: f32, comp: f32) -> TimelineView {
        let span = (self.span * factor).clamp(MIN_SPAN, comp.max(MIN_SPAN));
        // anchor が窓の中で占める割合を保つ
        let ratio = if self.span > 0.0 {
            (anchor - self.start) / self.span
        } else {
            0.0
        };
        TimelineView {
            start: anchor - ratio * span,
            span,
        }
        .clamped(comp)
    }

    /// 横へずらす。
    pub fn pan(self, delta_seconds: f32, comp: f32) -> TimelineView {
        TimelineView {
            start: self.start + delta_seconds,
            span: self.span,
        }
        .clamped(comp)
    }

    /// **窓は composition の外へ出ない。** 0より前も、終端より後ろも見せない。
    pub fn clamped(self, comp: f32) -> TimelineView {
        let span = self.span.clamp(MIN_SPAN, comp.max(MIN_SPAN));
        let start = self.start.clamp(0.0, (comp - span).max(0.0));
        TimelineView { start, span }
    }
}

/// ルーラの上端に置くループ帯の高さ。**掴める帯として成立する厚み**
const LOOP_H: f32 = 10.0;
/// ループ帯の下に置くロケータの段の高さ
const LOCATOR_H: f32 = 13.0;
/// ループの端を掴める幅。bar のトリム端(6px)より広い —
/// **外すと「新しい区間を引く」に落ちて、古い区間が消えてしまう**ので、
/// 端の判定は甘いほうが事故が小さい
const LOOP_GRAB: f32 = 8.0;
/// 面の端これだけ以内へポインタが入ったら、掴んだまま窓が動く
const EDGE_PAN: f32 = 28.0;
/// 端で掴み続けたときに流れる速さ(窓の幅に対する毎秒の割合)
const EDGE_PAN_RATE: f32 = 0.8;

/// ループ区間。**Project session の状態**で、Document には入れない
/// (再生の都合であって、書き出される内容ではない)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoopRegion {
    pub start: f32,
    pub end: f32,
    /// 帯は引いたまま、効きだけ切れる。**引き直さずに戻せる**
    pub on: bool,
}

/// **面の上で押しているあいだ在るもの。** 掴み物はこれ1つに畳んである。
///
/// 以前は `drag` / `loop_drag` / `nav` / `marquee` と別々の Option が並んでいて、
/// 「いま何か掴んでいるか」を聞くたびに4つ確かめる必要があった(実際、聞き忘れて
/// 再生が始まる・ロケータが1フレームごとに別の Undo になる、が起きていた)。
///
/// **Document を書くものは `GestureId` を握る** — 1ドラッグが1 Undo になるのは
/// 掴んだ瞬間に採った id を離すまで使い回すからで、毎フレーム開き直すと
/// フレーム数だけ Undo が積まれる。
#[derive(Debug, Clone)]
enum Hold {
    /// clip / キー / トリム / 並べ替え。`undo_base` は Esc で戻す判断に使う
    Item {
        grab: Grab,
        gesture: GestureId,
        undo_base: usize,
    },
    /// ロケータを時間方向へ
    Locator { index: usize, gesture: GestureId },
    /// ループ帯(Document を書かない — 区間は session の状態)
    Loop(LoopGrab),
    /// ナビゲータ帯(同上)
    Nav(NavGrab),
    /// 矩形選択(同上)
    Marquee { from: egui::Pos2, to: egui::Pos2 },
}

/// 掴んでいるあいだ、毎フレーム同じ3つを聞かれる — ポインタの時刻、端で流すか、
/// 窓の広さ。**その3つをまとめて1度だけ用意する。**
///
/// これを持ち回らないと、掴む物ごとに同じ式(`x_to_time` / `edge_pan_seconds` /
/// `track_w / span`)が写経で増えていく。実際4箇所で同じ4行が並んでいた。
#[derive(Debug, Clone, Copy)]
struct Surface {
    track_left: f32,
    track_w: f32,
    comp: f32,
    dt: f32,
}

impl Surface {
    fn time_at(&self, view: TimelineView, pos_x: f32) -> f32 {
        view.x_to_time(pos_x, self.track_left, self.track_w)
    }

    fn px_per_second(&self, view: TimelineView) -> f32 {
        self.track_w / view.span
    }

    /// 端まで運んでいるぶんの流れ。**中に居るなら 0** なので呼び手は分岐しない
    fn edge_pan(&self, view: TimelineView, pos_x: f32) -> TimelineView {
        let seconds = edge_pan_seconds(pos_x, self.track_left, self.track_w, view.span, self.dt);
        if seconds == 0.0 {
            view
        } else {
            view.pan(seconds, self.comp)
        }
    }
}

/// ループ帯のどこを掴んだか。
///
/// **端を掴んだときは反対側の端を掴んだ瞬間に控える。** 毎フレーム
/// `self.loop_region` から読み直すと、ポインタが反対の端を追い越した瞬間に
/// 区間が畳まれて1フレームの薄片になり、戻しても復元しない。
#[derive(Debug, Clone, Copy, PartialEq)]
enum LoopGrab {
    /// 何も無いところから引いた。`anchor` は掴んだ時刻で、反対側がポインタを追う
    New { anchor: f32 },
    /// 区間ごと動かす
    Move { grab_at: f32, from: (f32, f32) },
    /// 頭を動かす。`fixed` は動かないほうの端(お尻)
    In { fixed: f32 },
    /// お尻を動かす。`fixed` は動かないほうの端(頭)
    Out { fixed: f32 },
}

/// ループ帯のどこを掴んだかを決める。**端が先、中が次、外は新規**。
///
/// 端の判定を外すと `New` に落ちる = **古い区間が消える**ので、
/// 端は `LOOP_GRAB` ぶん甘く見る。短い区間で頭と尻の判定が重なったら、
/// 近いほうを採る(どちらも同じだけ近いなら尻 — 伸ばす操作のほうが多い)。
fn loop_grab_for(pos_x: f32, x0: f32, x1: f32, at: f32, region: LoopRegion) -> LoopGrab {
    let (d0, d1) = ((pos_x - x0).abs(), (pos_x - x1).abs());
    if d0 <= LOOP_GRAB || d1 <= LOOP_GRAB {
        return if d0 < d1 {
            LoopGrab::In { fixed: region.end }
        } else {
            LoopGrab::Out {
                fixed: region.start,
            }
        };
    }
    if pos_x > x0 && pos_x < x1 {
        return LoopGrab::Move {
            grab_at: at,
            from: (region.start, region.end),
        };
    }
    LoopGrab::New { anchor: at }
}

/// 引いた2点からループ区間を作る。
///
/// **右から左へ引いても同じ区間になる**(掴んだ点と離した点の順序を持たない)。
/// 時刻はフレームに乗せ、composition の外へは出さない。**最短は1フレーム** —
/// 長さ0の区間は再生が止まる場所にしかならない。
fn loop_from_drag(a: f32, b: f32, comp: f32, fps: Fps) -> (f32, f32) {
    let snap = |t: f32| {
        seconds_to_time(t.clamp(0.0, comp), fps)
            .map(|t| t.as_seconds_f64() as f32)
            .unwrap_or(t)
    };
    let frame = 1.0 / fps.as_f64() as f32;
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    let (lo, mut hi) = (snap(lo), snap(hi));
    if hi - lo < frame * 0.5 {
        hi = (lo + frame).min(comp);
    }
    (lo, hi)
}

/// ループの折り返し。**判定は「お尻に来たか」だけ。**
///
/// 区間の外から再生を始めても頭へ引き戻さない — 押した所から流れ、
/// **お尻を越えた瞬間に**頭へ戻る。だから区間より前から入れば
/// そこまで通しで聴こえ、区間より後ろから始めたなら一度も折り返さない
/// (`from` が既にお尻より後ろなら、越える瞬間が来ない)。
///
/// 行き過ぎた分は捨てずに頭へ足す — 捨てると1フレームの取りこぼしが
/// 毎周たまり、周期が伸びる。
fn wrap_playhead(from: f32, to: f32, region: LoopRegion) -> f32 {
    let length = region.end - region.start;
    if !region.on || length <= 0.0 || from >= region.end || to < region.end {
        return to;
    }
    region.start + (to - region.start) % length
}

/// 端を掴み続けているあいだ、窓が流れる秒数(このフレームぶん)。
///
/// **掴んだものを窓の外へ運べないと、長い composition では手が届かない。**
/// 端に近いほど速い。中に居るあいだは 0 で、窓は動かない。
fn edge_pan_seconds(pointer_x: f32, track_left: f32, track_w: f32, span: f32, dt: f32) -> f32 {
    let right = track_left + track_w;
    let strength = if pointer_x > right - EDGE_PAN {
        ((pointer_x - (right - EDGE_PAN)) / EDGE_PAN).min(1.5)
    } else if pointer_x < track_left + EDGE_PAN {
        -(((track_left + EDGE_PAN) - pointer_x) / EDGE_PAN).min(1.5)
    } else {
        0.0
    };
    strength * span * EDGE_PAN_RATE * dt
}

/// 並べ替えの落とし先。**行と行のあいだ**を指す。
///
/// `index` は D2 の `prepare_reparent_clip` と同じ意味、つまり
/// **外したあとの挿入位置**である。同じ親の中で下へ動かすときに1つずれるのは
/// このためで、`drop_target` がその調整を持つ。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropTarget {
    pub parent: ParentLocator,
    pub index: usize,
    /// 線を引く y。**絵のためだけに持つ**
    pub y: f32,
}

/// このフレームに来たホイールの生の量。
///
/// `smooth_scroll_delta` は egui が時間で均した値で、**指を止めても数フレーム
/// 流れ続ける**。面を掴んで動かす操作(パン・縦スクロール)では、その上乗せが
/// そのまま遅延として出る。OS 側の慣性はイベントに含まれて来るので、
/// ここで捨てているのは egui の均しだけである。
fn raw_wheel(input: &egui::InputState) -> Vec2 {
    input
        .events
        .iter()
        .filter_map(|event| match event {
            egui::Event::MouseWheel { unit, delta, .. } => Some(match unit {
                egui::MouseWheelUnit::Point => *delta,
                // 行・ページ単位で来る環境では px へ直す(macOS は Point で来る)
                egui::MouseWheelUnit::Line => *delta * 50.0,
                egui::MouseWheelUnit::Page => *delta * 400.0,
            }),
            _ => None,
        })
        .fold(Vec2::ZERO, |sum, delta| sum + delta)
}

/// 1フレームで進める上限(秒)。
///
/// **窓が隠れていた分をまとめて進めない。** eframe は見えていないと描画を間引くので、
/// 戻ってきたフレームの `dt` は数百msになりうる。そのまま足すと playhead が飛ぶ。
const MAX_STEP: f32 = 0.05;

/// `dt` 秒ぶん進んだ playhead と、**まだ再生中か**。
///
/// 終端で止まる(巻き戻さない)。頭へ戻すかどうかは押した側の判断である。
fn advance_playhead(playhead: f32, dt: f32, comp: f32) -> (f32, bool) {
    let at = playhead + dt.max(0.0);
    if at >= comp {
        (comp, false)
    } else {
        (at, true)
    }
}

/// 数字を出す目盛の間隔(秒)。**窓に16本前後入る、切りのいい値**を選ぶ。
///
/// 寄るとフレームの倍数へ、引くと秒・分の倍数へ移る。候補にしか無い値は出さない
/// — 「0.37秒ごと」のような目盛は読めないので。
fn tick_step(span: f32, fps: Fps) -> f32 {
    let frame = 1.0 / fps.as_f64() as f32;
    let target = span / 16.0;
    let mut candidates = vec![frame, frame * 2.0, frame * 5.0, frame * 10.0];
    candidates.extend_from_slice(&[
        0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0, 600.0,
    ]);
    candidates.sort_by(f32::total_cmp);
    candidates
        .into_iter()
        .find(|c| *c >= target)
        .unwrap_or(600.0)
}

/// 数字を出さない細かい目盛の間隔。**`major` を割り切り、1フレームより細かくしない。**
///
/// これが「あいだがどれくらいか」を数えられる目盛で、数字の密度を上げずに
/// 読み取れる分解能だけを上げる。フレームまで寄ったら細目盛は消える。
fn minor_step(major: f32, fps: Fps) -> Option<f32> {
    let frame = 1.0 / fps.as_f64() as f32;
    // 5等分が基本。60秒台だけは 4等分(15秒)のほうが読める
    let divisor = if (major - 60.0).abs() < 1e-3 || (major - 120.0).abs() < 1e-3 {
        4.0
    } else {
        5.0
    };
    let candidate = major / divisor;
    if candidate >= frame * 0.999 {
        Some(candidate)
    } else if frame < major * 0.5 {
        Some(frame)
    } else {
        None
    }
}

/// いま見えている窓に入る、その間隔の目盛の時刻。
///
/// **目盛は時刻に貼り付く。** 画面をN等分すると、パンしても線が動かず
/// 数字だけが変わる — 方眼が紙ではなく窓に貼られているように見えてしまう。
fn ticks_every(view: TimelineView, step: f32) -> Vec<f32> {
    let first = (view.start / step).floor() * step;
    let mut out = Vec::new();
    let mut t = first;
    // 端数で無限に回らないよう、本数で止める
    while t <= view.start + view.span + step * 0.5 && out.len() < 1024 {
        if t >= -1e-4 {
            out.push(t);
        }
        t += step;
    }
    out
}

/// 数字を出す目盛
fn ticks(view: TimelineView, fps: Fps) -> Vec<f32> {
    ticks_every(view, tick_step(view.span, fps))
}

/// その区間を暗いほうで塗るか。**絶対時刻で決める。**
///
/// 窓の中で何本目かで決めると、パンした瞬間に縞が入れ替わって画面が沸く。
/// 0秒から数えた区間の偶奇なら、窓をどう動かしても縞は動かない。
fn band_is_dark(t: f32, step: f32) -> bool {
    if step <= 0.0 {
        return false;
    }
    (t / step).round() as i64 % 2 != 0
}

/// タイムコード `M:SS:FF`。**フレーム番号まで出す** — 秒だけだと、
/// フレームに乗っているかどうかが読めない
fn timecode(seconds: f32, fps: Fps) -> String {
    let rate = fps.as_f64() as f32;
    let total_frames = (seconds * rate).round().max(0.0);
    let frame = (total_frames % rate).round() as i64;
    let whole = (total_frames / rate).floor() as i64;
    format!("{}:{:02}:{:02}", whole / 60, whole % 60, frame)
}

/// 目盛の文字。**間隔より細かい桁は出さない**
fn tick_label(t: f32, step: f32) -> String {
    let minutes = (t / 60.0).floor().max(0.0);
    let seconds = t - minutes * 60.0;
    if step >= 1.0 {
        format!("{minutes}:{seconds:04.1}")
    } else {
        format!("{minutes}:{seconds:05.2}")
    }
}

/// 右クリックのメニューから出た指示。
///
/// **メニューの中では Document を触らない。** 行を回している最中に木が変わると、
/// その場で持っている位置が全部ずれる(M/S のクリックと同じ扱い)。
#[derive(Debug, Clone, Copy, PartialEq)]
enum MenuAction {
    Group,
    Duplicate,
    Delete,
    DeleteKeys,
    ToggleMute(LayerId),
    ToggleSolo(LayerId),
    ToggleLock(LayerId),
    ToggleChildren(LayerId),
    ToggleKeys(LayerId),
    FitView,
    LoopToSelection,
    ClearLoop,
    Rename(LayerId),
    SetColor(LayerId, Option<u32>),
    ToggleColors,
    /// **右クリックした時刻に置く。** 「いま指した所」以外に置き場所は要らない
    AddLocatorAt(f32),
    RemoveLocator(usize),

    RowHeight(bool),
    SelectAll,
    Split,
    AddKey(LayerId, ParamRef),
    SetInterp(LayerId, ParamRef, KeyframeId, Interp),
}

/// メニューの下地を TimelineEditor のトンマナへ寄せる。
///
/// **egui のメニューは `visuals.window_*` と `widgets.*` で描かれる**ので、
/// 面のほうの定数(`CELL` / `INK` / `ACCENT`)をそのまま渡す。ここを既定のままに
/// すると、メニューだけ別のアプリのように見える。
fn install_lab_style(ctx: &egui::Context) {
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    let v = &mut style.visuals;
    v.dark_mode = true;
    v.window_fill = CELL;
    v.panel_fill = BG;
    v.window_stroke = Stroke::new(1.0, Color32::from_rgb(0x11, 0x11, 0x11));
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, DIM);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0x44, 0x44, 0x44));
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, INK);
    v.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    v.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    // hover は帯で示す。**触れる物と触れない物の差を色で出す**
    v.widgets.hovered.bg_fill = ACCENT;
    v.widgets.hovered.weak_bg_fill = ACCENT;
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::from_rgb(0x1c, 0x1c, 0x1c));
    v.widgets.active.bg_fill = ACCENT;
    v.widgets.active.weak_bg_fill = ACCENT;
    v.widgets.active.fg_stroke = Stroke::new(1.0, Color32::from_rgb(0x1c, 0x1c, 0x1c));
    style.spacing.button_padding = Vec2::new(8.0, 3.0);
    ctx.set_style_of(egui::Theme::Dark, style.clone());
    ctx.set_style_of(egui::Theme::Light, style);
}

/// 左列の小さな部品を1つ置く。**M / S / L も ▾ も ◇ もこれを通る。**
///
/// `Sense::click_and_drag()` なのは、**押した瞬間に自分が掴みの相手になる**ため。
/// クリック専用にすると、下に敷いてある行(選択＋並べ替え)のほうが掴みの相手に
/// なり、指が数px動いただけでボタンの `clicked()` が消える —
/// 「M/S/L がたまに効かない」の正体はこれで、ボタン側の不具合ではなかった。
fn rail_hit(ui: &mut egui::Ui, id: egui::Id, rect: Rect) -> egui::Response {
    ui.interact(rect, id, Sense::click_and_drag())
}

/// その部品が**押されたか**。
///
/// `click_and_drag` は掴みを捕まえるが、その代わり**数px動くと `clicked()` が
/// 立たない** — egui から見ると「掴んで離した」になるからである。三角や M/S/L の
/// ような小さい的では、少し動いてしまった押下も押下として扱うのが正しい
/// (押した所で離しているなら、指が揺れただけである)。
///
/// 離した場所が的の外なら押下にしない — ボタンから指をずらして逃がす、
/// あの取り消し方をそのまま残す。
fn pressed(r: &egui::Response, rect: Rect) -> bool {
    r.clicked()
        || (r.drag_stopped()
            && r.interact_pointer_pos()
                .map(|pos| rect.contains(pos))
                .unwrap_or(false))
}

/// 枠つきの四角ボタン(M / S / L)。**入っている色は呼び側が持つ**
fn rail_button(
    ui: &mut egui::Ui,
    p: &egui::Painter,
    id: egui::Id,
    rect: Rect,
    label: &str,
    on: bool,
    on_color: Color32,
) -> bool {
    let r = rail_hit(ui, id, rect);
    let hit = pressed(&r, rect);
    if on {
        p.rect_filled(rect, CornerRadius::ZERO, on_color);
    }
    p.rect_stroke(
        rect,
        CornerRadius::ZERO,
        Stroke::new(
            1.0,
            if r.hovered() {
                ACCENT
            } else {
                Color32::from_rgb(0x51, 0x51, 0x51)
            },
        ),
        StrokeKind::Inside,
    );
    p.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(9.0),
        if on {
            INK
        } else {
            Color32::from_rgb(0xaa, 0xaa, 0xaa)
        },
    );
    hit
}

/// 枠の無い記号ボタン(▾ / ◇)。開いているときは点いたままにする
fn rail_glyph(
    ui: &mut egui::Ui,
    p: &egui::Painter,
    id: egui::Id,
    rect: Rect,
    glyph: &str,
    on: bool,
) -> bool {
    let r = rail_hit(ui, id, rect);
    p.text(
        rect.center(),
        Align2::CENTER_CENTER,
        glyph,
        FontId::proportional(11.0),
        if on || r.hovered() { ACCENT } else { DIM },
    );
    pressed(&r, rect)
}

/// まだ無い操作を**席として並べる**。押せないが、どこに来るかは分かる。
///
/// 空欄にすると「この面には無い操作」に見えてしまう。灰色で置いておけば
/// 「ここに来る」と読める。実装したらここから外す。
fn seat(ui: &mut egui::Ui, label: &str) {
    ui.add_enabled(false, egui::Button::new(label));
}

/// ナビゲータ帯のどこを掴んだか
#[derive(Debug, Clone, Copy, PartialEq)]
enum NavGrab {
    Pan,
    Left,
    Right,
}

/// ポインタの y が、object 行の**何番目の上の境界**に居るか。
///
/// 行の中心で切り替える。`objects.len()` は「いちばん下」を指す
fn boundary_at(objects: &[(LayerId, f32, f32)], y: f32) -> usize {
    for (i, (_, top, h)) in objects.iter().enumerate() {
        if y < top + h * 0.5 {
            return i;
        }
    }
    objects.len()
}

/// その境界に線を引く y
fn boundary_y(objects: &[(LayerId, f32, f32)], boundary: usize) -> f32 {
    match objects.get(boundary) {
        Some((_, top, _)) => *top,
        None => objects.last().map(|(_, top, h)| top + h).unwrap_or(0.0),
    }
}

/// 行の合計高。縦スクロールの上限はここから出る
fn content_height(rows: &[TimelineRow], row_h: f32, prop_h: f32) -> f32 {
    rows.iter()
        .map(|r| match r.kind {
            RowKind::Object => row_h,
            RowKind::Property(_) => prop_h,
        })
        .sum()
}

/// **面より短い中身はスクロールしない。** 下は最後の行で止まる
fn clamp_scroll(scroll: f32, content: f32, viewport: f32) -> f32 {
    scroll.clamp(0.0, (content - viewport).max(0.0))
}

/// `layer` の subtree に `maybe` が含まれるか。**自分自身は含めない**
fn is_descendant(document: &Document, layer: LayerId, maybe: LayerId) -> bool {
    if layer == maybe {
        return false;
    }
    let Some(item) = find_item(document, layer) else {
        return false;
    };
    let mut ids = Vec::new();
    collect_layer_ids(item, &mut ids);
    ids.contains(&maybe)
}

/// 境界 `boundary`(object 行の何番目の**上**か)へ `dragged` を落とすとき、
/// D2 へ渡す `(parent, index)` を出す。
///
/// - 境界の**下にある行**の位置がそのまま挿入位置になる。開いた Group の
///   最初の子の上へ落とせば、それは「Group の中の先頭」である
/// - 末尾の境界だけは、**最後の行の次**を指す
/// - **自分自身の中へは落とせない。** Group を自分の子の中へ入れると木が壊れる
/// - 同じ親の中で下へ動かすときは1つ引く。`index` は外したあとの位置なので
fn drop_target(
    document: &Document,
    objects: &[LayerId],
    boundary: usize,
    dragged: LayerId,
) -> Option<(ParentLocator, usize)> {
    let (parent, mut index) = if boundary < objects.len() {
        let (parent, index, _) = find_item_location(document, objects[boundary])?;
        (parent, index)
    } else {
        let (parent, index, _) = find_item_location(document, *objects.last()?)?;
        (parent, index + 1)
    };
    if let ParentLocator::Group(group) = parent {
        if group == dragged || is_descendant(document, dragged, group) {
            return None;
        }
    }
    let (old_parent, old_index, _) = find_item_location(document, dragged)?;
    if old_parent == parent && old_index < index {
        index -= 1;
    }
    Some((parent, index))
}

/// Lab の起動殻。fixture を座らせて窓を開く(example `timeline_egui_lab` から呼ぶ)。
///
/// `shot` は従来どおり「数フレーム描いて1枚 BMP に落として閉じる」動線。
/// 撮影の都合はエディタ本体に持ち込まず、`LabHarness` が包む
/// (`blitz_shell/runner.rs` の `Harness` と同じ形)。
pub fn run_lab(shot: Option<String>) -> Result<(), crate::ShellError> {
    eframe::run_native(
        "Timeline egui Lab — 実Documentを実行モデルで",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1180.0, 460.0]),
            ..Default::default()
        },
        Box::new(move |cc| {
            crate::egui_fonts::install_symbol_fallback(&cc.egui_ctx);
            install_lab_style(&cc.egui_ctx);
            Ok(Box::new(LabHarness {
                editor: TimelineEditor::with_fixture(),
                shot,
                frame: 0,
            }))
        }),
    )
    .map_err(|error| crate::ShellError::Runtime(Box::new(error)))
}

/// エディタをそのまま包んで、`shot` があるときだけ1枚撮って閉じる。
struct LabHarness {
    editor: TimelineEditor,
    shot: Option<String>,
    frame: u32,
}

impl eframe::App for LabHarness {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.editor.show(ui);

        let ctx = ui.ctx().clone();
        self.frame += 1;
        if let Some(path) = self.shot.clone() {
            if self.frame >= 20 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            }
            let image = ctx.input(|i| {
                i.events.iter().find_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            });
            if let Some(image) = image {
                write_bmp(&path, &image);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

/// **掴んでいる間のカーソルは、掴んでいるものが決める。**
///
/// hover から毎フレーム決め直すと、ポインタが的から少し外れた瞬間に形が戻り、
/// 「トリム中なのに掴み手のまま」のような嘘になる。掴んでいるあいだは
/// 手の形も掴んだものに固定する。
fn hold_cursor(hold: &Option<Hold>) -> Option<egui::CursorIcon> {
    match hold {
        Some(Hold::Item { grab, .. }) => Some(match grab {
            Grab::TrimIn { .. } | Grab::TrimOut { .. } => egui::CursorIcon::ResizeHorizontal,
            Grab::Move { .. } => egui::CursorIcon::Grabbing,
            Grab::KeyTime { .. } => egui::CursorIcon::Grabbing,
            Grab::Reorder { .. } => egui::CursorIcon::Grabbing,
        }),
        Some(Hold::Locator { .. }) => Some(egui::CursorIcon::Grabbing),
        Some(Hold::Loop(grab)) => Some(loop_grab_cursor(grab)),
        Some(Hold::Nav(_)) => Some(egui::CursorIcon::Grabbing),
        Some(Hold::Marquee { .. }) => Some(egui::CursorIcon::Crosshair),
        None => None,
    }
}

/// クリックの選択規則。**行にもキーにも同じものを使う。**
///
/// - 素のクリック … その1つだけにする
/// - `Cmd`      … 足し引き
/// - `Shift`    … **直前に触ったもの**からここまで(`order` の並びで数える)
///
/// 起点(anchor)は末尾に残す。続けて `Shift` を押しても基準が動かないため。
/// `order` に無いもの(畳まれて見えていない等)は範囲に入らない — **見えている
/// とおりに採れる**のが範囲選択の約束である。
fn select_click<T: Copy + PartialEq>(
    selected: &mut Vec<T>,
    item: T,
    additive: bool,
    range: bool,
    order: &[T],
) {
    if range {
        if let Some(anchor) = selected.last().copied() {
            if let (Some(a), Some(b)) = (
                order.iter().position(|x| *x == anchor),
                order.iter().position(|x| *x == item),
            ) {
                let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                let mut next: Vec<T> = order[lo..=hi]
                    .iter()
                    .copied()
                    .filter(|x| *x != anchor)
                    .collect();
                next.push(anchor);
                *selected = next;
                return;
            }
        }
    }
    if additive {
        if let Some(at) = selected.iter().position(|x| *x == item) {
            selected.remove(at);
        } else {
            selected.push(item);
        }
    } else {
        *selected = vec![item];
    }
}

/// ループ帯の掴み方に対応する手の形。**触れているときも掴んでいるときも同じ表**
fn loop_grab_cursor(grab: &LoopGrab) -> egui::CursorIcon {
    match grab {
        LoopGrab::In { .. } | LoopGrab::Out { .. } => egui::CursorIcon::ResizeHorizontal,
        LoopGrab::Move { .. } => egui::CursorIcon::Grabbing,
        LoopGrab::New { .. } => egui::CursorIcon::Crosshair,
    }
}

/// bar のどこを掴んだか
#[derive(Debug, Clone, Copy, PartialEq)]
enum BarPart {
    Body,
    TrimIn,
    TrimOut,
}

/// トリムの端の幅。モックの `.trimHandle{width:7px}` は**見た目の幅**で、
/// 掴む幅としては細い。8px にして、同じ幅の帯を絵にも出す(掴める所を見せる)
const TRIM_EDGE: f32 = 8.0;

/// bar のどこを掴んだかを決める。**端を差し出してよい場合だけ差し出す。**
///
/// - **Group の bar は端を持たない。** あれは子の範囲を写した絵で、Group 自身は
///   `clip.start` も `duration` も持たない — 端を掴ませても D2 は
///   `TrackItemNotClip` で断る。**書けない操作を差し出さない**
/// - **細い bar でも端を取らない。** 左右6pxずつ取ると幅12px以下の clip は
///   全部が端になり、**動かせない clip** ができる(掴める体を必ず残す)
/// - 端の判定は**面に映っている矩形ではなく clip 本来の矩形**で見る。
///   窓の外へ出ている端は、画面の縁であって clip の端ではない
fn classify_bar_edge(bar: Rect, pos_x: f32, is_group: bool) -> BarPart {
    if is_group || bar.width() < TRIM_EDGE * 3.0 {
        return BarPart::Body;
    }
    if pos_x - bar.left() <= TRIM_EDGE {
        BarPart::TrimIn
    } else if bar.right() - pos_x <= TRIM_EDGE {
        BarPart::TrimOut
    } else {
        BarPart::Body
    }
}

/// 掴んでいるもの。**何を掴んだかで、出す command が変わる**
#[derive(Debug, Clone)]
enum Grab {
    /// **Group を掴んだら子も同じ差分で動く。**`targets` は動かす clip と、
    /// 掴んだ瞬間の開始時刻(秒)。Group 自身は clip ではないので入らない
    Move {
        layer: LayerId,
        grab_at: f32,
        targets: Vec<(LayerId, f32, f32)>,
        /// 追従させる Position キーと、掴んだ瞬間の時刻(秒)。
        /// **絶対値で出し直すので、掴んだ瞬間の値を持ったままにする**
        /// 追従させるキー。**param ごとに出す command が違う**ので param も持つ
        keys: Vec<(LayerId, ParamRef, KeyframeId, f32)>,
        /// 追従できないキーの数。Scale/Rotation/Anchor/Opacity には
        /// 時刻を変える `prepare_*` が無い
        not_movable: usize,
    },
    /// キー1つを掴んで時刻を変える。**どのパラメータのキーかを持つ** —
    /// Position だけが専用の入口を持ち、他は `SetTransformParamKeyTime` へ行くので
    KeyTime {
        layer: LayerId,
        param: ParamRef,
        key: KeyframeId,
        grab_at: f32,
        original: f32,
    },
    TrimIn {
        layer: LayerId,
    },
    TrimOut {
        layer: LayerId,
    },
    /// 左列を掴んで上下へ。**離した瞬間に1回だけ書く。**
    /// 途中の位置は線で見せるだけで、Document は動かさない — 通り道の親へ
    /// 一度ずつ入れ直すと、1ドラッグが N 個の編集になってしまう
    Reorder {
        layer: LayerId,
    },
}

/// clip / group を掴んだ瞬間の状態を採る。
///
/// **動くもの(clip の開始時刻)と、追従するもの(その clip の Position キー)を
/// ここで一度に確定させる。** ドラッグ中は毎フレーム絶対値で出し直すので、
/// 掴んだ瞬間の値が要る。
/// 1つだけ掴む。**窓は必ず複数選択の道(`begin_move_many`)を通る**ので、
/// これを呼ぶのはテストである。
#[cfg(test)]
fn begin_move(document: &Document, layer: LayerId, grab_at: f32) -> Grab {
    begin_move_many(document, &[layer], layer, grab_at)
}

/// 複数選択のドラッグ。**選ばれている全部が同じ差分で動く。**
///
/// `layer` は掴んだ行(status に出す代表)で、`roots` が実際に動かす集合である。
/// 親と子が両方選ばれていても、**動く clip は重複しない** — `movable_clips` が
/// 子孫まで返すので、集めたあとに LayerId で畳む。
fn begin_move_many(document: &Document, roots: &[LayerId], layer: LayerId, grab_at: f32) -> Grab {
    let mut targets: Vec<(LayerId, f32, f32)> = Vec::new();
    for root in roots {
        for target in movable_clips(document, *root) {
            if !targets.iter().any(|(l, _, _)| *l == target.0) {
                targets.push(target);
            }
        }
    }
    // **キーを持つのは clip だけではない。** Group 自身の envelope にもキーがあり、
    // 掴んだ subtree の中の Group はまとめて動いたことになる。ここを clip だけに
    // していたので「Group 自体のキーが追従しない」が起きていた
    let mut keyed: Vec<LayerId> = Vec::new();
    for root in roots {
        if let Some(item) = find_item(document, *root) {
            let mut ids = Vec::new();
            collect_layer_ids(item, &mut ids);
            for id in ids {
                if !keyed.contains(&id) {
                    keyed.push(id);
                }
            }
        }
    }
    let mut keys = Vec::new();
    let mut not_movable = 0usize;
    for clip in &keyed {
        // **envelope が持つ param は全部追従できる**(2026-08-16 に D2 の
        // `SetTransformParamKeyTime` と受け付け集合の統合が入った)。
        // 追従できないのは plugin 由来(EffectParam / SourceParam)だけで、
        // TimelineEditor はまだそれを描いていないので 0 のまま
        for param in [
            ParamRef::Position,
            ParamRef::Anchor,
            ParamRef::Scale,
            ParamRef::Rotation,
            ParamRef::Opacity,
        ] {
            for (key, t) in param_keys(document, *clip, param) {
                keys.push((*clip, param, key, t));
            }
        }
    }
    let _ = &mut not_movable;
    Grab::Move {
        layer,
        grab_at,
        targets,
        keys,
        not_movable,
    }
}

/// Timeline エディタ本体。**Document の唯一の writer をここに1つだけ抱える**
/// (single writer)。外へ出て行くのは `document()` の immutable snapshot だけで、
/// 第二の writer も Document clone 編集もここから先に作らない。
pub struct TimelineEditor {
    writer: DocumentWriter,
    document: Arc<Document>,
    /// `document` を取ったときの `writer.revision`。**これが取り直しの唯一の合図**
    revision: u64,
    fold: TimelineFoldState,
    /// **いま掴んでいるもの。** 掴み物はここ1つに集める
    hold: Option<Hold>,
    names: HashMap<LayerId, String>,
    /// 選択中の layer。**Project session が持つ種類の状態**で、Document には入れない。
    /// **順序は選んだ順**で、末尾が Shift 範囲選択の起点になる
    selected: Vec<LayerId>,
    /// 見えている時間の窓。同上
    view: TimelineView,
    /// 縦スクロール(px)。行の合計高が面より高いときだけ動く。同上
    scroll_y: f32,
    /// 選択中のキー。**行の選択とは別の入れ物**で、Delete はキーが選ばれていれば
    /// キーを消す(層の選択より内側のものが勝つ)
    selected_keys: Vec<(LayerId, ParamRef, KeyframeId)>,
    /// 並べ替えのドラッグ中に、いま落とすと決まる場所。**線を描く位置でもある**
    drop: Option<DropTarget>,
    /// playhead(秒)。同上
    playhead: f32,
    /// ループ区間。同上
    loop_region: LoopRegion,
    /// 吸着するか。**Alt を押しているあいだは切れる**(押しっぱなしで自由に置ける)
    snap: bool,
    /// 行を高くするか。**意味は変わらない** — 見やすさだけ
    large_rows: bool,
    /// 色を出すか。**これは Document ではなく Workspace profile の持ち物**
    /// (白で統一して見たい人の好みが、他人の付けた色を消してはいけない)。
    /// TimelineEditor には profile が無いので、ここでは窓の状態として持つ
    colors_on: bool,
    /// 名前を編集中の layer と、編集中の文字列。**確定するまで Document は触らない**
    renaming: Option<(LayerId, String)>,
    /// メモを編集中の index と文字列。同上
    editing_locator: Option<(usize, String)>,
    /// 右クリックした時刻。**メニューの中の「ここ」がどこかを固定する**
    context_time: f32,
    /// 再生中か。**Space で入り切りする**。Document には入れない
    playing: bool,
    /// 再生中の clock の座席。`playing` のあいだだけ `Some` で、停止・pause・
    /// スクラブで drop して device を手放す。soundtrack が無い/鳴らせない時は
    /// `WallClock` に落ち、playhead は従来どおり `advance_playhead` で進む
    audio: Option<AudioPlayback>,
    /// project root(document path の親)。soundtrack の asset path 解決に使う。
    /// fixture / lab の席には無い(→ soundtrack も無いので壁時計のまま)
    project_root: Option<std::path::PathBuf>,
    /// decode 済み正準PCMの控え(`(content_hash, ordinal)`)。再生をまたいで
    /// 使い回し、再生のたびに decode し直さない
    pcm_caches: HashMap<(String, u32), Arc<PcmCache>>,
    status: String,
}

impl TimelineEditor {
    /// 実プロジェクトの writer を座らせる(shell の `--project` 経路)。
    ///
    /// fixture を読まない。行の名前は Document の台帳(`display_name`)から来る
    /// (`name()` が控えの無い layer で台帳へ落ちる)。
    pub fn new(writer: DocumentWriter) -> Self {
        Self::seated(writer, HashMap::new(), TimelineFoldState::default())
    }

    /// fixture を座らせて作る(lab と本 module のテスト動線)。
    pub(crate) fn with_fixture() -> Self {
        let (doc, names) = lab_fixture();
        let catalog =
            Arc::new(motolii_plugin::reference::reference_catalog().expect("reference catalog"));
        let writer = DocumentWriter::new(doc, catalog).expect("writer");
        let mut fold = TimelineFoldState::default();
        // 最初から階層が見えているほうが、行モデルの確認になる
        for (&layer, name) in &names {
            match name.as_str() {
                // 子の軸
                "Title scene" => fold.open_children(layer),
                // パラメータの軸。両方が同時に効くことを最初の画面で見せる
                "Shared left" => fold.open_params(layer),
                _ => {}
            }
        }
        if let Some((&layer, _)) = names.iter().find(|(_, n)| n.as_str() == "Title scene") {
            fold.open_params(layer);
        }
        Self::seated(writer, names, fold)
    }

    /// 席の共通部。cached snapshot は writer からここで1度だけ取る。
    fn seated(
        writer: DocumentWriter,
        names: HashMap<LayerId, String>,
        fold: TimelineFoldState,
    ) -> Self {
        let document = writer.snapshot();
        let revision = writer.revision;
        Self {
            writer,
            document,
            revision,
            fold,
            hold: None,
            names,
            selected: Vec::new(),
            selected_keys: Vec::new(),
            view: TimelineView {
                start: 0.0,
                span: TIMELINE_SECONDS,
            },
            scroll_y: 0.0,
            drop: None,
            playhead: 0.0,
            // 最初から引いてある帯は邪魔なので、区間だけ用意して切っておく
            loop_region: LoopRegion {
                start: 2.0,
                end: 6.0,
                on: false,
            },
            snap: true,
            large_rows: false,
            colors_on: true,
            renaming: None,
            editing_locator: None,
            context_time: 0.0,
            playing: false,
            audio: None,
            project_root: None,
            pcm_caches: HashMap::new(),
            status: "space=play  L=loop  Cmd+G=group  Del=delete  drag name=reorder".to_owned(),
        }
    }

    /// project root(document の親 dir)を座らせる。soundtrack 再生の
    /// asset path 解決(`resolve_asset_path`)がこれを使う。CLI の export と
    /// 同じ規約(`document_export.rs` の `doc_path.parent()`)。
    pub fn with_project_root(mut self, root: Option<std::path::PathBuf>) -> Self {
        self.project_root = root;
        self
    }

    /// 座っている project root(読み)。座席の配線テストが見る。
    pub fn project_root(&self) -> Option<&std::path::Path> {
        self.project_root.as_deref()
    }

    /// pane / Stage へ流す immutable snapshot(cached。取り直しは revision が合図)。
    pub fn document(&self) -> &Arc<Document> {
        &self.document
    }

    /// writer の編集世代。進んでいたら snapshot を配り直す(Stage の合図)。
    pub fn revision(&self) -> u64 {
        self.writer.revision
    }

    /// undo 台帳の深さ。1操作 = 1 gesture = 1 undo 単位。
    pub fn undo_len(&self) -> usize {
        self.writer.undo_len()
    }

    /// redo 台帳の深さ。
    pub fn redo_len(&self) -> usize {
        self.writer.redo_len()
    }

    /// Undo。通ったら cached snapshot も取り直す。
    pub fn undo(&mut self) -> Result<(), motolii_doc::UndoError> {
        self.writer.undo()?;
        refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
        Ok(())
    }

    /// Redo。同上。
    pub fn redo(&mut self) -> Result<(), motolii_doc::UndoError> {
        self.writer.redo()?;
        refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
        Ok(())
    }

    /// 入力シミュレーションの入口: clip / Group を `grab_at_seconds` で掴む。
    /// マウスのドラッグ開始と同じ経路(`begin_move_many` → `hold_item`)で、
    /// テストと統合テストがマウスの代わりに呼ぶ。
    pub fn begin_clip_move(&mut self, layer: LayerId, grab_at_seconds: f32) {
        let grab = begin_move_many(&self.document, &[layer], layer, grab_at_seconds);
        self.hold_item(grab);
    }

    /// ドラッグ中の着地(吸着なし)。`commit_drag` と同じ。
    pub fn drag_to(&mut self, at_seconds: f32) {
        self.commit_drag(at_seconds);
    }

    /// 離す。gesture は着地のたびに書き終えているので、掴みを手放すだけ。
    pub fn release(&mut self) {
        self.hold = None;
    }

    /// ポインタの時刻(秒)を、掴んでいるものに応じた D2 command にして適用する。
    ///
    /// **prepare_* が `Ok(None)` を返したら、それは「変化なし」であって失敗ではない。**
    /// 落ちた編集は status に出す。通ったことにしない。
    /// param ごとに、時刻を動かす command を選ぶ。
    ///
    /// Position だけ専用の入口があり(`SetPositionKeyTime`)、他は
    /// `SetTransformParamKeyTime` が受ける。**将来ここは1本に畳む**
    /// (台帳「キー編集APIを `ScalarPropertyId` 1本へ畳む」参照)。
    fn key_time_command(
        &self,
        layer: LayerId,
        param: ParamRef,
        key: KeyframeId,
        t: RationalTime,
    ) -> Result<Option<motolii_doc::Command>, String> {
        match param {
            ParamRef::Position => self
                .writer
                .prepare_set_position_key_time(layer, key, t)
                .map_err(|e| e.to_string()),
            other => self
                .writer
                .prepare_set_transform_param_key_time(layer, scalar_property(other), key, t)
                .map_err(|e| e.to_string()),
        }
    }

    /// ドラッグの着地時刻。**掴んでいるものが何であれ、まず候補へ吸着する。**
    /// `px_per_second` は窓の広さから来るので、寄れば吸着の間合いも細かくなる
    fn commit_drag_snapped(&mut self, at_seconds: f32, px_per_second: f32) {
        let exclude: Vec<LayerId> = self
            .item_hold()
            .map(|(grab, _, _)| vec![grab_layer(&grab)])
            .unwrap_or_default();
        let at = self.snapped(at_seconds, &exclude, px_per_second);
        self.commit_drag(at);
    }

    fn commit_drag(&mut self, at_seconds: f32) {
        let Some((grab, gesture, _)) = self.item_hold() else {
            return;
        };
        // **編集が作る時刻は全部フレーム境界へ乗る。** fps はここで1回だけ読む
        let fps = self.document.composition.fps;
        let Some(time) = seconds_to_time(at_seconds, fps) else {
            return;
        };

        // 掴んだものごとに、出す command の並びを作る。
        // 各要素は (layer, これはキーの command か, 準備結果)。
        // **Move だけが複数になりうる**(Group は子をまとめて動かす)
        let mut prepared = Vec::new();
        match &grab {
            Grab::Move {
                grab_at,
                targets,
                keys,
                ..
            } => {
                if targets.is_empty() {
                    self.status = "nothing to move (empty group)".to_owned();
                    return;
                }
                // **塊のまま動く。** 誰か1人でも端を越えるなら、全員そこで止まる。
                // 越えた者だけ置いていくと、Group の中の相対位置が壊れる
                let comp = self.document.composition.duration.as_seconds_f64() as f32;
                let floor = targets
                    .iter()
                    .map(|(_, start, _)| *start)
                    .fold(f32::INFINITY, f32::min);
                let headroom = targets
                    .iter()
                    .map(|(_, _, end)| comp - *end)
                    .fold(f32::INFINITY, f32::min);
                let delta = (at_seconds - grab_at).clamp(-floor, headroom.max(0.0));
                for (layer, start, _) in targets {
                    let Some(t) = seconds_to_time((start + delta).max(0.0), fps) else {
                        return;
                    };
                    prepared.push((
                        *layer,
                        false,
                        self.writer
                            .prepare_set_clip_start(*layer, t)
                            .map_err(|e| e.to_string()),
                    ));
                }
                // **キーは出す順が要る。** 同じ時刻に2つは置けないので、後ろへ動かす
                // ときは遅いキーから、前へ動かすときは早いキーから出す。全員が同じ
                // delta で動くので、この順なら途中でぶつからない
                let mut ordered = keys.clone();
                ordered.sort_by(|a, b| {
                    if delta >= 0.0 {
                        b.3.total_cmp(&a.3)
                    } else {
                        a.3.total_cmp(&b.3)
                    }
                });
                for (layer, param, key, original) in ordered {
                    let Some(t) = seconds_to_time((original + delta).max(0.0), fps) else {
                        return;
                    };
                    prepared.push((layer, true, self.key_time_command(layer, param, key, t)));
                }
            }
            Grab::KeyTime {
                layer,
                param,
                key,
                grab_at,
                original,
            } => {
                // Move と同じ考え方でクランプする。0秒より前・composition の
                // 終端より後ろへは出さない
                let comp = self.document.composition.duration.as_seconds_f64() as f32;
                let moved = (original + (at_seconds - grab_at)).clamp(0.0, comp.max(0.0));
                let Some(t) = seconds_to_time(moved, fps) else {
                    return;
                };
                // **Position 専用の入口へ直に行かない。** param で選ぶ1本を通す
                prepared.push((*layer, true, self.key_time_command(*layer, *param, *key, t)));
            }
            Grab::TrimIn { layer } => prepared.push((
                *layer,
                false,
                self.writer
                    .prepare_trim_clip_in(*layer, time)
                    .map_err(|e| e.to_string()),
            )),
            Grab::TrimOut { layer } => prepared.push((
                *layer,
                false,
                self.writer
                    .prepare_trim_clip_out(*layer, time)
                    .map_err(|e| e.to_string()),
            )),
            // 並べ替えは離した瞬間にしか書かない。ドラッグ中はここへ来ない
            Grab::Reorder { .. } => return,
        }

        let mut applied = 0usize;
        let mut keys_applied = 0usize;
        for (layer, is_key, result) in prepared {
            match result {
                Ok(Some(command)) => match self.writer.apply_command(gesture, command) {
                    Ok(()) => {
                        if is_key {
                            keys_applied += 1;
                        } else {
                            applied += 1;
                        }
                    }
                    Err(error) => {
                        self.status = format!("{} rejected: {error}", self.name(layer));
                        return;
                    }
                },
                Ok(None) => {}
                Err(error) => {
                    self.status = format!("{} rejected: {error}", self.name(layer));
                    return;
                }
            }
        }
        if applied + keys_applied > 0 {
            refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
            // **追従できなかったキーを黙らせない。** Position 以外は動いていない
            let detail = match &grab {
                Grab::Move { not_movable, .. } => format!(
                    "{applied} clip, {keys_applied} keys followed, {not_movable} not movable"
                ),
                Grab::KeyTime { .. } => format!("{keys_applied} key"),
                Grab::TrimIn { .. } | Grab::TrimOut { .. } => format!("{applied} clip"),
                Grab::Reorder { .. } => String::new(),
            };
            self.status = format!(
                "{} @ {at_seconds:.2}s  ({detail})  undo {}",
                self.name(grab_layer(&grab)),
                self.writer.undo_len()
            );
        }
    }

    /// ドラッグ中の Esc。**掴む前へ戻す。**
    ///
    /// 1ドラッグ = 1 `GestureId` なので、その gesture が積んだものは undo 1回で
    /// まとめて戻る。まだ何も積んでいない(掴んだだけで動かしていない)ときは
    /// **undo しない** — 掴む前の、関係ない編集を取り消してしまう。
    fn cancel_drag(&mut self) {
        let Some((grab, _, undo_base)) = self.item_hold() else {
            return;
        };
        self.hold = None;
        let layer = grab_layer(&grab);
        if self.writer.undo_len() <= undo_base {
            self.status = format!("{} cancelled (nothing to undo)", self.name(layer));
            return;
        }
        match self.writer.undo() {
            Ok(()) => {
                refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
                self.status = format!(
                    "{} cancelled  (undo {})",
                    self.name(layer),
                    self.writer.undo_len()
                );
            }
            Err(error) => self.status = format!("cancel rejected: {error}"),
        }
    }

    /// 触れているあいだの手の形。**掴んでいるときは何も言わない** —
    /// 掴んだものが決めた形を上書きしないため(`hold_cursor` が後で勝つ)。
    fn hover_cursor(&self, ctx: &egui::Context, hovered: bool, icon: egui::CursorIcon) {
        if hovered && self.hold.is_none() {
            ctx.set_cursor_icon(icon);
        }
    }

    /// いま掴んでいるのが編集(clip / キー / トリム / 並べ替え)なら、その中身
    fn item_hold(&self) -> Option<(Grab, GestureId, usize)> {
        match &self.hold {
            Some(Hold::Item {
                grab,
                gesture,
                undo_base,
            }) => Some((grab.clone(), *gesture, *undo_base)),
            _ => None,
        }
    }

    /// 掴み始める。**押した瞬間に1回だけ**
    fn hold_item(&mut self, grab: Grab) {
        let gesture = self.writer.begin_gesture();
        self.hold = Some(Hold::Item {
            grab,
            gesture,
            undo_base: self.writer.undo_len(),
        });
    }

    /// 開いてある gesture へ書く。**1ドラッグ = 1 Undo** はこれで保たれる
    fn apply_in<E: std::fmt::Display>(
        &mut self,
        gesture: GestureId,
        what: &str,
        prepared: Result<Option<Command>, E>,
    ) -> bool {
        match prepared {
            Ok(Some(command)) => match self.writer.apply_command(gesture, command) {
                Ok(()) => {
                    refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
                    true
                }
                Err(error) => {
                    self.status = format!("{what} rejected: {error}");
                    false
                }
            },
            Ok(None) => false,
            Err(error) => {
                self.status = format!("{what} rejected: {error}");
                false
            }
        }
    }

    /// **準備済みの command を1つ、1 gesture で書く。**
    ///
    /// 書けたら `true`。`Ok(None)`(変化なし)は失敗ではないので黙って `false` を返す。
    /// 落ちたら status に理由を出すので、**呼び手は成功時の言葉だけ持てばよい**。
    ///
    /// 「gesture を開く → 3通りに場合分け → 通ったら snapshot を取り直す →
    /// 落ちたら名前つきで status」は、ここへ来るまで7箇所に写経されていた。
    fn apply_one<E: std::fmt::Display>(
        &mut self,
        what: &str,
        prepared: Result<Option<Command>, E>,
    ) -> bool {
        let gesture = self.writer.begin_gesture();
        self.apply_in(gesture, what, prepared)
    }

    /// M / S / L を反転して Document へ書く。**1クリック = 1 `GestureId` = 1 Undo 単位**
    fn toggle_flag(&mut self, layer: LayerId, flag: Flag) {
        let Some((visible, solo, lock)) = item_flags(&self.document, layer) else {
            return;
        };
        let prepared = match flag {
            Flag::Mute => self.writer.prepare_set_item_visible(layer, !visible),
            Flag::Solo => self.writer.prepare_set_item_solo(layer, !solo),
            Flag::Lock => self.writer.prepare_set_item_lock(layer, !lock),
        };
        let name = self.name(layer).to_owned();
        if self.apply_one(&name, prepared) {
            let what = match (flag, visible, solo, lock) {
                (Flag::Mute, true, _, _) => "mute",
                (Flag::Mute, false, _, _) => "unmute",
                (Flag::Solo, _, false, _) => "solo",
                (Flag::Solo, _, true, _) => "unsolo",
                (Flag::Lock, _, _, false) => "lock",
                (Flag::Lock, _, _, true) => "unlock",
            };
            self.status = format!("{name} {what}  undo {}", self.writer.undo_len());
        }
    }

    /// **効いているロックか。** D2 は lock を見ない(評価・描画に影響しないフラグ
    /// なので)。触らせないのは UI 側の仕事である — ここを通さない経路を作らないこと。
    ///
    /// 親から受けている分も含める(`effective_lock`)。Group を掛けたのに中が
    /// 触れてしまうのは、ここで各行の `lock` を単体で読んでいたからだった。
    fn is_locked(&self, layer: LayerId) -> bool {
        effective_lock(&self.document, layer)
    }

    fn is_selected(&self, layer: LayerId) -> bool {
        self.selected.contains(&layer)
    }

    /// 行をクリックしたときの選択。**普通のリストと同じ3通り。**
    ///
    /// - そのまま  … その1つだけにする
    /// - `Cmd`    … 足す / 外す
    /// - `Shift`  … 直前に触った行からここまで(**見えている object 行の上で**数える。
    ///   閉じた Group の中は見えないので入らない)
    fn select(&mut self, layer: LayerId, additive: bool, range: bool, objects: &[LayerId]) {
        select_click(&mut self.selected, layer, additive, range, objects);
        self.status = match self.selected.len() {
            0 => "nothing selected".to_owned(),
            1 => format!("selected {}", self.name(self.selected[0])),
            n => format!("{n} selected"),
        };
    }

    /// キーのクリック。**行と同じ規則**(素で置き換え / `Cmd` で足し引き / `Shift` で範囲)。
    ///
    /// 並び順は画面に出ている順(行の順 → その行の中は時刻順)で、
    /// **見えているとおりに範囲が採れる**。
    fn select_key(
        &mut self,
        entry: (LayerId, ParamRef, KeyframeId),
        additive: bool,
        range: bool,
        order: &[(LayerId, ParamRef, KeyframeId)],
    ) {
        select_click(&mut self.selected_keys, entry, additive, range, order);
        self.status = format!("{} keys selected", self.selected_keys.len());
    }

    /// 選択のうち、**他の選択の子孫でないもの**だけ。
    ///
    /// 親 Group と子が同時に選ばれているとき、複製すると子が2重に増え、
    /// 削除すると親を消した時点で子が消えて `LayerNotFound` になる。
    /// **構造を触る操作は必ずここを通す。**
    fn selection_roots(&self) -> Vec<LayerId> {
        self.selected
            .iter()
            .copied()
            .filter(|layer| {
                !self
                    .selected
                    .iter()
                    .any(|other| is_descendant(&self.document, *other, *layer))
            })
            .collect()
    }

    /// 構造を触ってよい選択。**ロックされているものは外す。**
    ///
    /// 選択そのものは許す(見て確かめたいだけのことがある)。外すのは書く直前で、
    /// **1つでも外したら status で言う** — 黙って対象が減るのが一番分からない。
    fn editable_roots(&mut self) -> Vec<LayerId> {
        let roots = self.selection_roots();
        let editable: Vec<LayerId> = roots
            .iter()
            .copied()
            .filter(|layer| !self.is_locked(*layer))
            .collect();
        if editable.len() < roots.len() {
            self.status = format!("{} locked, skipped", roots.len() - editable.len());
        }
        editable
    }

    /// 選択中の layer を丸ごと複製する。**1複製 = 1 `GestureId` = 1 Undo 単位**
    ///
    /// **深いところは D2 がやる。** `prepare_duplicate_track_item` は Group の
    /// 子も、シェイプの中の入れ子(`VectorContent::Group`)も再帰して写し、
    /// LayerId / KeyframeId / EffectId を全部新しく振り直す。TimelineEditor が子を辿って
    /// 複製し直すと、その再写像を二重にしてしまう — **ここでは source を1つ渡すだけ**。
    ///
    /// 複製後は**増えたほうを選ぶ**。続けて動かすのが普通なので
    fn duplicate_selected(&mut self) {
        let roots = self.editable_roots();
        if roots.is_empty() {
            self.status = "nothing selected".to_owned();
            return;
        }
        let gesture = self.writer.begin_gesture();
        let mut made = Vec::new();
        for layer in roots {
            let name = self.name(layer).to_owned();
            let command = match self.writer.prepare_duplicate_track_item(layer) {
                Ok(command) => command,
                Err(error) => {
                    self.status = format!("{name} rejected: {error}");
                    return;
                }
            };
            // 増えたほうの LayerId は command の中にしか無い(まだ Document に無い)
            if let Command::AddTrackItem { item, .. } = &command {
                let mut ids = Vec::new();
                collect_layer_ids(item, &mut ids);
                made.extend(ids.first().copied());
            }
            if let Err(error) = self.writer.apply_command(gesture, command) {
                self.status = format!("{name} rejected: {error}");
                return;
            }
        }
        refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
        // **1つなら名前を出す。** 何が増えたか分からない状態にしない
        let what = match made.as_slice() {
            [one] => self.name(*one).to_owned(),
            many => format!("{}", many.len()),
        };
        self.status = format!("duplicated {what}  undo {}", self.writer.undo_len());
        self.selected = made;
    }

    /// 選択中のキーを消す。**1回の Delete = 1 `GestureId` = 1 Undo 単位**
    ///
    /// キーが選ばれているときは、そちらが層の削除より先に効く —
    /// **内側のものを選んでいるなら、消したいのは内側**である。
    fn delete_selected_keys(&mut self) -> bool {
        if self.selected_keys.is_empty() {
            return false;
        }
        let gesture = self.writer.begin_gesture();
        let mut removed = 0usize;
        for (layer, param, key) in self.selected_keys.clone() {
            let prepared = match param {
                ParamRef::Position => self
                    .writer
                    .prepare_remove_position_key(layer, key)
                    .map_err(|e| e.to_string()),
                other => self
                    .writer
                    .prepare_remove_transform_param_key(layer, scalar_property(other), key)
                    .map_err(|e| e.to_string()),
            };
            match prepared {
                Ok(command) => match self.writer.apply_command(gesture, command) {
                    Ok(()) => removed += 1,
                    Err(error) => {
                        self.status = format!("{} rejected: {error}", self.name(layer));
                        return true;
                    }
                },
                Err(error) => {
                    self.status = format!("{} rejected: {error}", self.name(layer));
                    return true;
                }
            }
        }
        refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
        self.selected_keys.clear();
        self.status = format!("deleted {removed} keys  undo {}", self.writer.undo_len());
        true
    }

    /// 選択中の layer を消す。**Group は中身ごと。1回の Delete = 1 Undo 単位**
    ///
    /// 消す順は関係ない — `selection_roots` が親子の重なりを外しているので、
    /// どれを先に消しても残りの `prepare` は当たる。
    fn delete_selected(&mut self) {
        let roots = self.editable_roots();
        if roots.is_empty() {
            self.status = "nothing selected".to_owned();
            return;
        }
        let gesture = self.writer.begin_gesture();
        let mut removed = 0usize;
        for layer in &roots {
            let name = self.name(*layer).to_owned();
            let command = match self.writer.prepare_remove_track_item(*layer) {
                Ok(command) => command,
                Err(error) => {
                    self.status = format!("{name} rejected: {error}");
                    return;
                }
            };
            if let Err(error) = self.writer.apply_command(gesture, command) {
                self.status = format!("{name} rejected: {error}");
                return;
            }
            removed += 1;
        }
        refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
        self.selected.clear();
        self.status = format!("deleted {removed}  undo {}", self.writer.undo_len());
    }

    /// その時刻にロケータを置く。**置いた直後から書ける状態にする**
    ///
    /// 仮の playhead のようなもので、置く場所は**指した所**である。
    fn add_locator(&mut self, at: f32) {
        let fps = self.document.composition.fps;
        let comp = self.document.composition.duration.as_seconds_f64() as f32;
        let Some(t) = seconds_to_time(at.clamp(0.0, comp), fps) else {
            return;
        };
        // 既定名は数える。**空の印を置かない** — 名前が無いと構成を指せない
        let name = format!("Locator {}", self.document.locators.len() + 1);
        let prepared: Result<Option<Command>, CommandError> =
            Ok(Some(self.writer.prepare_add_locator(t, &name)));
        if self.apply_one("locator", prepared) {
            let index = self.document.locators.len().saturating_sub(1);
            self.editing_locator = Some((index, name.clone()));
            self.status = format!("{name} at {at:.2}s  undo {}", self.writer.undo_len());
        }
    }

    /// メモの文を確定する。**空なら消す** — 空のメモは印にならない
    fn commit_locator_text(&mut self) {
        let Some((index, text)) = self.editing_locator.take() else {
            return;
        };
        let text = text.trim().to_owned();
        // 空にしたら消す。**空の印は印にならない**
        let prepared = if text.is_empty() {
            self.writer.prepare_remove_locator(index).map(Some)
        } else {
            self.writer.prepare_set_locator_text(index, &text)
        };
        if self.apply_one("locator", prepared) {
            self.status = if text.is_empty() {
                format!("locator removed  undo {}", self.writer.undo_len())
            } else {
                format!("locator: {text}  undo {}", self.writer.undo_len())
            };
        }
    }

    /// メモを外す
    fn remove_locator(&mut self, index: usize) {
        let prepared = self.writer.prepare_remove_locator(index).map(Some);
        if self.apply_one("locator", prepared) {
            self.editing_locator = None;
            self.status = format!("locator removed  undo {}", self.writer.undo_len());
        }
    }

    /// 行の色を選ぶ。`None` は「選んでいない」へ戻す(既定色に返る)。
    /// **選択が複数なら全部に付く** — 色は構成を見分けるためのものなので
    fn set_color(&mut self, layer: LayerId, rgb: Option<u32>) {
        let targets: Vec<LayerId> = if self.is_selected(layer) {
            self.editable_roots()
        } else {
            vec![layer]
        };
        let gesture = self.writer.begin_gesture();
        let mut painted = 0usize;
        for target in targets {
            match self.writer.prepare_set_item_color(target, rgb) {
                Ok(Some(command)) => {
                    if self.writer.apply_command(gesture, command).is_ok() {
                        painted += 1;
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    self.status = format!("{} rejected: {error}", self.name(target));
                    return;
                }
            }
        }
        refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
        self.status = match rgb {
            Some(_) => format!("coloured {painted}  undo {}", self.writer.undo_len()),
            None => format!("colour cleared  undo {}", self.writer.undo_len()),
        };
    }

    /// 名前の編集を確定する。**空の名前は断る**(行が読めなくなる)。
    fn commit_rename(&mut self) {
        let Some((layer, name)) = self.renaming.take() else {
            return;
        };
        let name = name.trim().to_owned();
        if name.is_empty() {
            self.status = "name cannot be empty".to_owned();
            return;
        }
        let prepared = self.writer.prepare_set_layer_name(layer, &name);
        // 同じ名前を打ち直したときは何も書かない。**失敗ではない**
        if self.apply_one("rename", prepared) {
            // TimelineEditor の控えも合わせる。**台帳が正**なので、控えは捨ててもよい
            self.names.remove(&layer);
            self.status = format!("renamed to {name}  undo {}", self.writer.undo_len());
        }
    }

    /// 名前の編集を始める。**いま見えている名前を初期値にする**
    fn begin_rename(&mut self, layer: LayerId) {
        if self.is_locked(layer) {
            self.status = format!("{} is locked", self.name(layer));
            return;
        }
        self.renaming = Some((layer, self.name(layer).to_owned()));
    }

    /// 選択をひとつの Group にまとめる。**1回 = 1 `GestureId` = 1 Undo 単位**
    ///
    /// 「空の Group を置く」+「選んだものを順に入れる」で表す。新しい意味の
    /// command は足していない(D2 `prepare_add_group` は `AddTrackItem` を返すだけ)。
    ///
    /// **親が揃っていないときは断る。** 別々の階層に居るものを1つに入れると、
    /// どの位置へ置いたのかが誰にも言えなくなる。
    fn group_selected(&mut self) {
        let roots = self.editable_roots();
        if roots.is_empty() {
            self.status = "nothing selected".to_owned();
            return;
        }
        let Some((parent, index, _)) = find_item_location(&self.document, roots[0]) else {
            self.status = "not found".to_owned();
            return;
        };
        // 位置は**いちばん上のものの場所**。まとめた結果がどこに出るかを固定する
        let mut at = index;
        for layer in &roots[1..] {
            match find_item_location(&self.document, *layer) {
                Some((p, i, _)) if p == parent => at = at.min(i),
                _ => {
                    self.status = "group: pick items in the same parent".to_owned();
                    return;
                }
            }
        }
        let name = format!("Group {}", self.document.layers.len());
        let command = match self.writer.prepare_add_group(parent, at, &name) {
            Ok(command) => command,
            Err(error) => {
                self.status = format!("group rejected: {error}");
                return;
            }
        };
        let group = match &command {
            Command::AddTrackItem { item, .. } => {
                let mut ids = Vec::new();
                collect_layer_ids(item, &mut ids);
                ids.first().copied()
            }
            _ => None,
        };
        let Some(group) = group else {
            self.status = "group: no layer".to_owned();
            return;
        };
        let gesture = self.writer.begin_gesture();
        if let Err(error) = self.writer.apply_command(gesture, command) {
            self.status = format!("group rejected: {error}");
            return;
        }
        for (i, layer) in roots.iter().enumerate() {
            let prepared =
                self.writer
                    .prepare_reparent_clip(*layer, ParentLocator::Group(group), i, None);
            match prepared {
                Ok(Some(command)) => {
                    if let Err(error) = self.writer.apply_command(gesture, command) {
                        self.status = format!("{} rejected: {error}", self.name(*layer));
                        return;
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    self.status = format!("{} rejected: {error}", self.name(*layer));
                    return;
                }
            }
        }
        refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
        // まとめたら中が見えている状態にする。**畳まれて消えたように見せない**
        self.fold.open_children(group);
        self.selected = vec![group];
        self.status = format!(
            "grouped {} into {name}  undo {}",
            roots.len(),
            self.writer.undo_len()
        );
    }

    /// 選択を playhead で切る。**1回 = 1 `GestureId` = 1 Undo 単位**
    ///
    /// 端に当たっているもの(playhead が clip の外・端ちょうど)は `Ok(None)` で
    /// 返るので、**切れなかったものは黙って飛ばす** — 失敗ではない。
    fn split_selected(&mut self) {
        let roots = self.editable_roots();
        if roots.is_empty() {
            self.status = "nothing selected".to_owned();
            return;
        }
        let fps = self.document.composition.fps;
        let Some(at) = seconds_to_time(self.playhead, fps) else {
            return;
        };
        let gesture = self.writer.begin_gesture();
        let mut split = 0usize;
        for layer in roots {
            match self.writer.prepare_split_clip(layer, at) {
                Ok(Some(command)) => match self.writer.apply_command(gesture, command) {
                    Ok(()) => split += 1,
                    Err(error) => {
                        self.status = format!("{} rejected: {error}", self.name(layer));
                        return;
                    }
                },
                Ok(None) => {}
                // Group は clip ではないので切れない。**それは断りであって失敗ではない**
                Err(_) => {}
            }
        }
        refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
        self.status = if split > 0 {
            format!(
                "split {split} at {:.2}s  undo {}",
                self.playhead,
                self.writer.undo_len()
            )
        } else {
            "nothing to split at the playhead".to_owned()
        };
    }

    /// playhead の時刻へキーを打つ。**既にキーがあるならそれを選ぶだけ。**
    fn add_key_at_playhead(&mut self, layer: LayerId, param: ParamRef) {
        let fps = self.document.composition.fps;
        let Some(at) = seconds_to_time(self.playhead, fps) else {
            return;
        };
        let gesture = self.writer.begin_gesture();
        let prepared = match param {
            ParamRef::Position => match self.writer.prepare_add_position_key(layer, at) {
                Ok(motolii_doc::AddPositionKeyPreparation::Prepared { key_id, command }) => {
                    Some((key_id, command))
                }
                Ok(motolii_doc::AddPositionKeyPreparation::AlreadyPresent { key_id }) => {
                    self.selected_keys = vec![(layer, param, key_id)];
                    self.status = "key already there".to_owned();
                    return;
                }
                Err(error) => {
                    self.status = format!("{} rejected: {error}", self.name(layer));
                    return;
                }
            },
            other => {
                match self
                    .writer
                    .prepare_add_transform_param_key(layer, scalar_property(other), at)
                {
                    Ok(motolii_doc::AddTransformParamKeyPreparation::Prepared {
                        key_id,
                        command,
                    }) => Some((key_id, command)),
                    Ok(motolii_doc::AddTransformParamKeyPreparation::AlreadyPresent { key_id }) => {
                        self.selected_keys = vec![(layer, param, key_id)];
                        self.status = "key already there".to_owned();
                        return;
                    }
                    Err(error) => {
                        self.status = format!("{} rejected: {error}", self.name(layer));
                        return;
                    }
                }
            }
        };
        let Some((key_id, command)) = prepared else {
            return;
        };
        match self.writer.apply_command(gesture, command) {
            Ok(()) => {
                refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
                // 打ったキーを選んでおく。**次の操作(値・イージング・削除)の相手になる**
                self.selected_keys = vec![(layer, param, key_id)];
                self.fold.open_params(layer);
                self.status = format!(
                    "{} {} key at {:.2}s  undo {}",
                    self.name(layer),
                    param_label(param),
                    self.playhead,
                    self.writer.undo_len()
                );
            }
            Err(error) => self.status = format!("{} rejected: {error}", self.name(layer)),
        }
    }

    /// キーの補間を変える。**入口があるのは Position だけ**である
    /// (`prepare_set_position_key_interp`)。他は D2 に無いので断る。
    fn set_key_interp(&mut self, layer: LayerId, param: ParamRef, key: KeyframeId, interp: Interp) {
        if param != ParamRef::Position {
            self.status = format!("{}: interp is Position-only in D2", param_label(param));
            return;
        }
        let gesture = self.writer.begin_gesture();
        match self
            .writer
            .prepare_set_position_key_interp(layer, key, interp)
        {
            Ok(Some(command)) => match self.writer.apply_command(gesture, command) {
                Ok(()) => {
                    refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
                    self.status = format!(
                        "{} interp  undo {}",
                        self.name(layer),
                        self.writer.undo_len()
                    );
                }
                Err(error) => self.status = format!("{} rejected: {error}", self.name(layer)),
            },
            Ok(None) => {}
            Err(error) => self.status = format!("{} rejected: {error}", self.name(layer)),
        }
    }

    /// 並べ替えを1回だけ書く。**離した瞬間に呼ぶ。**
    ///
    /// 時刻は変えない(`new_start = None`) — 上下に動かしただけで clip が
    /// 時間方向へ跳ぶと、並べ替えのつもりが編集になってしまう。
    fn commit_reorder(&mut self, layer: LayerId, to: DropTarget) {
        let name = self.name(layer).to_owned();
        let prepared = self
            .writer
            .prepare_reparent_clip(layer, to.parent, to.index, None);
        // 同じ場所へ落としたときは何も書かない。**失敗ではない**
        if self.apply_one(&name, prepared) {
            self.status = format!("moved {name}  undo {}", self.writer.undo_len());
        }
    }

    /// 行(clip / group)の右クリックメニュー。
    ///
    /// **並びは「いま効くもの → 面の状態 → まだ無いもの」**の順にした。
    /// 押せない席を上に置くと、使える操作を探すのに毎回読み飛ばすことになる。
    fn row_menu(
        ui: &mut egui::Ui,
        name: &str,
        layer: LayerId,
        has_children: bool,
        muted: bool,
        soloed: bool,
        locked: bool,
        selected: usize,
        out: &mut Option<MenuAction>,
    ) {
        ui.set_min_width(190.0);
        ui.label(egui::RichText::new(name).color(DIM).size(9.0));
        ui.separator();
        if ui
            .button(if selected > 1 {
                format!("Group {selected} layers   ⌘G")
            } else {
                "Group   ⌘G".to_owned()
            })
            .clicked()
        {
            *out = Some(MenuAction::Group);
            ui.close();
        }
        if ui.button("Duplicate   ⌘D").clicked() {
            *out = Some(MenuAction::Duplicate);
            ui.close();
        }
        if ui.button("Delete   ⌫").clicked() {
            *out = Some(MenuAction::Delete);
            ui.close();
        }
        if ui.button("Split at playhead   ⌘K").clicked() {
            *out = Some(MenuAction::Split);
            ui.close();
        }
        ui.separator();
        ui.menu_button("Add key at playhead   ▸", |ui| {
            for param in [
                ParamRef::Position,
                ParamRef::Anchor,
                ParamRef::Scale,
                ParamRef::Rotation,
                ParamRef::Opacity,
            ] {
                if ui.button(param_label(param)).clicked() {
                    *out = Some(MenuAction::AddKey(layer, param));
                    ui.close();
                }
            }
        });
        if ui
            .button(if muted { "Unmute   M" } else { "Mute   M" })
            .clicked()
        {
            *out = Some(MenuAction::ToggleMute(layer));
            ui.close();
        }
        if ui
            .button(if soloed { "Unsolo   S" } else { "Solo   S" })
            .clicked()
        {
            *out = Some(MenuAction::ToggleSolo(layer));
            ui.close();
        }
        if ui
            .button(if locked { "Unlock   L" } else { "Lock   L" })
            .clicked()
        {
            *out = Some(MenuAction::ToggleLock(layer));
            ui.close();
        }
        if ui.button("Show keys   ◇").clicked() {
            *out = Some(MenuAction::ToggleKeys(layer));
            ui.close();
        }
        if has_children && ui.button("Expand children   ▾").clicked() {
            *out = Some(MenuAction::ToggleChildren(layer));
            ui.close();
        }
        ui.separator();
        // ここから下は**席**。実装したらこの行を消して上へ移す
        seat(ui, "Cut   ⌘X");
        seat(ui, "Copy   ⌘C");
        seat(ui, "Paste   ⌘V");
        ui.menu_button("Colour   ▸", |ui| {
            ui.horizontal(|ui| {
                for rgb in LAYER_COLORS_RGB {
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::splat(16.0), Sense::click());
                    ui.painter().rect_filled(
                        rect,
                        CornerRadius::same(2),
                        Color32::from_rgb(
                            ((rgb >> 16) & 0xff) as u8,
                            ((rgb >> 8) & 0xff) as u8,
                            (rgb & 0xff) as u8,
                        ),
                    );
                    if response.hovered() {
                        ui.painter().rect_stroke(
                            rect,
                            CornerRadius::same(2),
                            Stroke::new(1.0, SELECTED),
                            StrokeKind::Outside,
                        );
                    }
                    if response.clicked() {
                        *out = Some(MenuAction::SetColor(layer, Some(rgb)));
                        ui.close();
                    }
                }
            });
            if ui.button("Default (from id)").clicked() {
                *out = Some(MenuAction::SetColor(layer, None));
                ui.close();
            }
        });
        if ui.button("Rename…   ⏎").clicked() {
            *out = Some(MenuAction::Rename(layer));
            ui.close();
        }

        seat(ui, "Reveal source");
    }

    /// キーの右クリックメニュー
    fn key_menu(
        ui: &mut egui::Ui,
        label: &str,
        layer: LayerId,
        param: ParamRef,
        key: KeyframeId,
        out: &mut Option<MenuAction>,
    ) {
        ui.set_min_width(180.0);
        ui.label(egui::RichText::new(label).color(DIM).size(9.0));
        ui.separator();
        if ui.button("Delete key   ⌫").clicked() {
            *out = Some(MenuAction::DeleteKeys);
            ui.close();
        }
        // **入口があるのは Position だけ。** 他は D2 に無いので席として置く
        if param == ParamRef::Position {
            ui.menu_button("Easing   ▸", |ui| {
                for (name, interp) in [
                    ("Hold", Interp::Hold),
                    ("Linear", Interp::Linear),
                    (
                        "Ease in-out",
                        Interp::Bezier {
                            x1: 0.42,
                            y1: 0.0,
                            x2: 0.58,
                            y2: 1.0,
                        },
                    ),
                ] {
                    if ui.button(name).clicked() {
                        *out = Some(MenuAction::SetInterp(layer, param, key, interp));
                        ui.close();
                    }
                }
            });
        } else {
            seat(ui, "Easing   (Position only in D2)");
        }
        ui.separator();
        seat(ui, "Copy key   ⌘C");
        seat(ui, "Set value…");
        seat(ui, "Snap to playhead");
    }

    /// 何も無いところの右クリックメニュー。**面そのものへの操作**
    fn surface_menu(ui: &mut egui::Ui, loop_on: bool, at: f32, out: &mut Option<MenuAction>) {
        ui.set_min_width(190.0);
        ui.label(egui::RichText::new("timeline").color(DIM).size(9.0));
        ui.separator();
        if ui.button("Fit to composition").clicked() {
            *out = Some(MenuAction::FitView);
            ui.close();
        }
        if ui.button("Loop to selection   L").clicked() {
            *out = Some(MenuAction::LoopToSelection);
            ui.close();
        }
        if loop_on && ui.button("Clear loop").clicked() {
            *out = Some(MenuAction::ClearLoop);
            ui.close();
        }
        if ui.button("Add locator here").clicked() {
            *out = Some(MenuAction::AddLocatorAt(at));
            ui.close();
        }
        if ui.button("Layer colours   on/off").clicked() {
            *out = Some(MenuAction::ToggleColors);
            ui.close();
        }
        ui.menu_button("Row height   ▸", |ui| {
            if ui.button("Small").clicked() {
                *out = Some(MenuAction::RowHeight(false));
                ui.close();
            }
            if ui.button("Large").clicked() {
                *out = Some(MenuAction::RowHeight(true));
                ui.close();
            }
        });
        ui.separator();
        seat(ui, "Paste   ⌘V");
        seat(ui, "New layer…");
        seat(ui, "Zoom to loop");
    }

    /// メニューから出た指示を1つ実行する。**行を回し終えてから呼ぶ**
    fn run_menu(&mut self, action: MenuAction, comp: f32, fps: Fps) {
        match action {
            MenuAction::Group => self.group_selected(),
            MenuAction::Duplicate => self.duplicate_selected(),
            MenuAction::Delete => self.delete_selected(),
            MenuAction::DeleteKeys => {
                self.delete_selected_keys();
            }
            MenuAction::ToggleMute(layer) => self.toggle_flag(layer, Flag::Mute),
            MenuAction::ToggleSolo(layer) => self.toggle_flag(layer, Flag::Solo),
            MenuAction::ToggleLock(layer) => self.toggle_flag(layer, Flag::Lock),
            MenuAction::ToggleChildren(layer) => {
                if self.fold.children_are_open(layer) {
                    self.fold.close_children(layer);
                } else {
                    self.fold.open_children(layer);
                }
            }
            MenuAction::ToggleKeys(layer) => {
                if self.fold.params_are_open(layer) {
                    self.fold.close_params(layer);
                } else {
                    self.fold.open_params(layer);
                }
            }
            MenuAction::FitView => {
                self.view = TimelineView {
                    start: 0.0,
                    span: comp,
                }
                .clamped(comp);
                self.status = "fit".to_owned();
            }
            MenuAction::LoopToSelection => {
                // 選んだものが占める範囲。**選択が空なら触らない**
                let mut span: Option<(f32, f32)> = None;
                for layer in &self.selected {
                    if let Some((s, e)) = clip_span(&self.document, *layer) {
                        span = Some(match span {
                            Some((a, b)) => (a.min(s), b.max(e)),
                            None => (s, e),
                        });
                    }
                }
                match span {
                    Some((s, e)) => {
                        let (start, end) = loop_from_drag(s, e, comp, fps);
                        self.loop_region = LoopRegion {
                            start,
                            end,
                            on: true,
                        };
                        self.status = format!("loop {start:.2}–{end:.2}s");
                    }
                    None => self.status = "nothing selected".to_owned(),
                }
            }
            MenuAction::ClearLoop => {
                self.loop_region.on = false;
                self.status = "loop off".to_owned();
            }
            MenuAction::Split => self.split_selected(),
            MenuAction::Rename(layer) => self.begin_rename(layer),
            MenuAction::SetColor(layer, rgb) => self.set_color(layer, rgb),
            MenuAction::ToggleColors => {
                self.colors_on = !self.colors_on;
                self.status = if self.colors_on {
                    "colours on"
                } else {
                    "colours off (white)"
                }
                .to_owned();
            }
            MenuAction::AddLocatorAt(at) => self.add_locator(at),
            MenuAction::RemoveLocator(index) => self.remove_locator(index),

            MenuAction::RowHeight(large) => {
                self.large_rows = large;
                self.status = if large { "large rows" } else { "small rows" }.to_owned();
            }
            MenuAction::SelectAll => {}
            MenuAction::AddKey(layer, param) => self.add_key_at_playhead(layer, param),
            MenuAction::SetInterp(layer, param, key, interp) => {
                self.set_key_interp(layer, param, key, interp)
            }
        }
    }

    /// 下端の時間ナビゲータ帯。**寄っているときに、いま全体のどこを見ているか。**
    ///
    /// 溝が composition 全体、明るい所が見えている窓である。
    /// **掴めば横パン、両端6pxを掴めばズーム** — ホイールを縦へ渡した分の
    /// 代わりがここにある。レイヤーの地図は置かない(行そのものが一覧なので)。
    fn navigator(
        &mut self,
        ui: &mut egui::Ui,
        p: &egui::Painter,
        full: Rect,
        track_left: f32,
        comp: f32,
    ) {
        if comp <= 0.0 {
            return;
        }
        let bar = Rect::from_min_max(
            egui::pos2(track_left, full.bottom() - NAV_H),
            egui::pos2(full.right(), full.bottom()),
        );
        p.rect_filled(
            Rect::from_min_max(egui::pos2(full.left(), bar.top()), full.max),
            CornerRadius::ZERO,
            Color32::from_rgb(0x1e, 0x1e, 0x1e),
        );
        let to_x = |t: f32| bar.left() + (t / comp) * bar.width();
        let (x0, x1) = (
            to_x(self.view.start),
            to_x(self.view.start + self.view.span),
        );
        let knob = Rect::from_min_max(
            egui::pos2(x0, bar.top() + 3.0),
            egui::pos2(x1.max(x0 + 8.0), bar.bottom() - 3.0),
        );
        let r = ui.interact(bar, ui.id().with("nav"), Sense::click_and_drag());
        if r.drag_started() {
            self.hold = r.interact_pointer_pos().map(|pos| {
                Hold::Nav(if (pos.x - knob.left()).abs() <= 6.0 {
                    NavGrab::Left
                } else if (pos.x - knob.right()).abs() <= 6.0 {
                    NavGrab::Right
                } else {
                    NavGrab::Pan
                })
            });
        }
        if r.dragged() {
            let nav_grab = match &self.hold {
                Some(Hold::Nav(mode)) => Some(*mode),
                _ => None,
            };
            if let (Some(mode), Some(pos)) = (nav_grab, r.interact_pointer_pos()) {
                let at = ((pos.x - bar.left()) / bar.width() * comp).clamp(0.0, comp);
                self.view = match mode {
                    // 掴んだ所が窓の中心へ来る
                    NavGrab::Pan => TimelineView {
                        start: at - self.view.span * 0.5,
                        span: self.view.span,
                    },
                    // 端を掴んだら、反対の端は動かさない
                    NavGrab::Left => TimelineView {
                        start: at,
                        span: (self.view.start + self.view.span - at).max(MIN_SPAN),
                    },
                    NavGrab::Right => TimelineView {
                        start: self.view.start,
                        span: (at - self.view.start).max(MIN_SPAN),
                    },
                }
                .clamped(comp);
            }
        }
        if r.drag_stopped() {
            self.hold = None;
        }
        p.rect_filled(
            knob,
            CornerRadius::same(2),
            Color32::from_rgb(0x4a, 0x4a, 0x4a),
        );
        p.rect_stroke(
            knob,
            CornerRadius::same(2),
            Stroke::new(
                1.0,
                if r.hovered() || matches!(self.hold, Some(Hold::Nav(_))) {
                    ACCENT
                } else {
                    Color32::from_rgb(0x66, 0x66, 0x66)
                },
            ),
            StrokeKind::Inside,
        );
    }

    /// 画面上の移動量(px)を、いまの窓での秒へ。**換算はここ1本**
    fn seconds_for(&self, dx: f32, track_w: f32) -> f32 {
        dx / track_w * self.view.span
    }

    /// 吸着の候補を集める。**clip の端・キー・playhead・ループの端・0・終端。**
    ///
    /// `exclude` は動かしている当人で、自分自身へは吸着しない。
    fn snap_candidates(&self, exclude: &[LayerId]) -> Vec<f32> {
        let mut out = vec![
            0.0,
            self.document.composition.duration.as_seconds_f64() as f32,
        ];
        out.push(self.playhead);
        if self.loop_region.on {
            out.push(self.loop_region.start);
            out.push(self.loop_region.end);
        }
        for row in rows(&self.document, &self.fold) {
            if exclude.contains(&row.layer) {
                continue;
            }
            match row.kind {
                RowKind::Object => {
                    if let Some((start, end)) = clip_span(&self.document, row.layer) {
                        out.push(start);
                        out.push(end);
                    }
                }
                RowKind::Property(param) => {
                    for (_, t) in param_keys(&self.document, row.layer, param) {
                        out.push(t);
                    }
                }
            }
        }
        out
    }

    /// 候補へ吸着した時刻。**間合いは画面の距離**なので、`px_per_second` が要る。
    ///
    /// 吸着しなければ元の値をそのまま返す(フレームへの丸めは後段の
    /// `seconds_to_time` がやる — **吸着はフレームより優先**である)。
    fn snapped(&self, t: f32, exclude: &[LayerId], px_per_second: f32) -> f32 {
        if !self.snap || px_per_second <= 0.0 {
            return t;
        }
        let window = SNAP_PX / px_per_second;
        let mut best: Option<(f32, f32)> = None;
        for candidate in self.snap_candidates(exclude) {
            let d = (candidate - t).abs();
            if d <= window && best.map(|(bd, _)| d < bd).unwrap_or(true) {
                best = Some((d, candidate));
            }
        }
        best.map(|(_, c)| c).unwrap_or(t)
    }

    /// 行の名前。**TimelineEditor の控えに無ければ Document の台帳を見る。**
    /// 複製で増えた layer は控えに載らないので、これが無いと "?" になる。
    fn name(&self, layer: LayerId) -> &str {
        self.names
            .get(&layer)
            .map(String::as_str)
            .or_else(|| self.document.layers.display_name(layer))
            .unwrap_or("?")
    }
}

impl TimelineEditor {
    /// audio の座席を別の時刻から開き直す(シーク・ループの折り返し)。
    /// 開き直せなければ壁時計へ**明示的に**落ち、理由を status に出す。
    /// 壁時計の座席には何もしない(壁時計にシークの追従は要らない)。
    fn reseek_audio(&mut self, at: f32, fps: Fps) {
        match self.audio.take() {
            Some(AudioPlayback::Synced(seat)) => match seat.reseek(at, fps) {
                Ok(seat) => self.audio = Some(AudioPlayback::Synced(seat)),
                Err(error) => {
                    self.status = format!("play (no audio: {error})");
                    self.audio = Some(AudioPlayback::WallClock(
                        WallClockReason::AudioUnavailable(error.to_string()),
                    ));
                }
            },
            other => self.audio = other,
        }
    }

    /// エディタ1面を `ui` の available rect いっぱいに描き、入力を受けて Document を
    /// 編集する。lab では eframe の App がそのまま、shell では Timeline pane の
    /// behavior がここを呼ぶ(旧 `impl eframe::App for Lab` の `ui` 本体)。
    pub(crate) fn show(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        ctx.request_repaint();

        // **編集の出どころを問わない。** 自分が書いたときも、Browser が別の場所で
        // シェイプを置いたときも、`revision` が進んでいれば次のフレームで拾う
        refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);

        // **毎フレーム、実Documentから行を作り直す。** 行のキャッシュを持たない
        let visible = rows(&self.document, &self.fold);

        let full = ui.available_rect_before_wrap();
        let p = ui.painter().clone();
        p.rect_filled(full, CornerRadius::ZERO, BG);

        // ---- ヘッダ ----
        let head = Rect::from_min_size(full.min, Vec2::new(full.width(), HEAD_H));
        p.rect_filled(head, CornerRadius::ZERO, HEAD_BG);
        // ---- transport ----
        // **記号は自前で描く。** ▶ や ⏮ はフォントに無くて豆腐になるので、
        // 三角と縦棒を painter で置く(M/S と同じ、chrome を painter で作る側)
        let comp_for_head = self.document.composition.duration.as_seconds_f64() as f32;
        let to_start = Rect::from_center_size(
            egui::pos2(head.left() + 18.0, head.center().y),
            Vec2::splat(18.0),
        );
        let play_hit = Rect::from_center_size(
            egui::pos2(head.left() + 42.0, head.center().y),
            Vec2::splat(18.0),
        );
        let to_start_r = ui.interact(to_start, ui.id().with("to_start"), Sense::click());
        let play_r = ui.interact(play_hit, ui.id().with("play"), Sense::click());
        {
            let c = to_start.center();
            let tint = if to_start_r.hovered() { ACCENT } else { INK };
            p.rect_filled(
                Rect::from_min_size(egui::pos2(c.x - 5.0, c.y - 5.0), Vec2::new(2.0, 10.0)),
                CornerRadius::ZERO,
                tint,
            );
            p.add(egui::Shape::convex_polygon(
                vec![
                    egui::pos2(c.x + 5.0, c.y - 5.0),
                    egui::pos2(c.x + 5.0, c.y + 5.0),
                    egui::pos2(c.x - 2.0, c.y),
                ],
                tint,
                Stroke::NONE,
            ));
            let c = play_hit.center();
            let tint = if play_r.hovered() { ACCENT } else { INK };
            if self.playing {
                for dx in [-4.0, 1.0] {
                    p.rect_filled(
                        Rect::from_min_size(egui::pos2(c.x + dx, c.y - 5.0), Vec2::new(3.0, 10.0)),
                        CornerRadius::ZERO,
                        tint,
                    );
                }
            } else {
                p.add(egui::Shape::convex_polygon(
                    vec![
                        egui::pos2(c.x - 4.0, c.y - 5.0),
                        egui::pos2(c.x + 6.0, c.y),
                        egui::pos2(c.x - 4.0, c.y + 5.0),
                    ],
                    tint,
                    Stroke::NONE,
                ));
            }
        }
        if to_start_r.clicked() {
            self.playhead = 0.0;
            self.status = "0:00:00".to_owned();
        }
        if play_r.clicked() {
            self.playing = !self.playing;
            if self.playing && self.playhead >= comp_for_head - 1e-3 {
                self.playhead = 0.0;
            }
            self.status = if self.playing { "play" } else { "pause" }.to_owned();
        }
        // **タイムコードは大きく出す。** いま何フレームに居るかは一番よく見る値
        p.text(
            egui::pos2(head.left() + 62.0, head.center().y),
            Align2::LEFT_CENTER,
            timecode(self.playhead, self.document.composition.fps),
            FontId::monospace(13.0),
            INK,
        );
        p.text(
            egui::pos2(head.right() - 10.0, head.center().y),
            Align2::RIGHT_CENTER,
            &self.status,
            FontId::proportional(10.0),
            ACCENT,
        );

        // ---- ルーラ ----
        let ruler = Rect::from_min_size(
            egui::pos2(full.left(), head.bottom()),
            Vec2::new(full.width(), RULER_H),
        );
        let track_left = full.left() + RAIL_W;
        let track_w = (full.right() - track_left).max(1.0);
        p.rect_filled(
            ruler,
            CornerRadius::ZERO,
            Color32::from_rgb(0x2a, 0x2a, 0x2a),
        );
        // **目盛は時刻に貼り付く。** ルーラも方眼もこの2本の列から引く
        let fps = self.document.composition.fps;
        let step = tick_step(self.view.span, fps);
        let ticks = ticks(self.view, fps);
        let minor = minor_step(step, fps);
        let minor_ticks: Vec<f32> = minor.map(|s| ticks_every(self.view, s)).unwrap_or_default();
        let ruler_clip = p.with_clip_rect(Rect::from_min_max(
            egui::pos2(track_left, ruler.top()),
            ruler.max,
        ));
        // 細目盛は数字を持たない。**下から短く出す** — 数字の密度を上げずに
        // 「あいだがどれだけか」を数えられるようにする
        for t in &minor_ticks {
            let x = self.view.time_to_x(*t, track_left, track_w);
            ruler_clip.line_segment(
                [
                    egui::pos2(x, ruler.bottom() - 5.0),
                    egui::pos2(x, ruler.bottom()),
                ],
                Stroke::new(1.0, Color32::from_rgb(0x3a, 0x3a, 0x3a)),
            );
        }
        for t in &ticks {
            let x = self.view.time_to_x(*t, track_left, track_w);
            ruler_clip.line_segment(
                [
                    egui::pos2(x, ruler.top() + LOOP_H + LOCATOR_H),
                    egui::pos2(x, ruler.bottom()),
                ],
                Stroke::new(1.0, Color32::from_rgb(0x55, 0x55, 0x55)),
            );
            ruler_clip.text(
                egui::pos2(x + 4.0, ruler.bottom() - 6.0),
                Align2::LEFT_BOTTOM,
                tick_label(*t, step),
                FontId::monospace(9.0),
                DIM,
            );
        }
        // いま何秒を見ているか。**窓の広さと粒**もここに出す
        p.text(
            egui::pos2(head.left() + 140.0, head.center().y),
            Align2::LEFT_CENTER,
            format!(
                "{} rows  view {:.2}–{:.2}s  grid {}  loop {}",
                visible.len(),
                self.view.start,
                self.view.start + self.view.span,
                if step >= 1.0 {
                    format!("{step:.0}s")
                } else {
                    format!("{:.0}f", step * fps.as_f64() as f32)
                },
                if self.loop_region.on {
                    format!("{:.2}–{:.2}s", self.loop_region.start, self.loop_region.end)
                } else {
                    "off".to_owned()
                }
            ),
            FontId::monospace(9.0),
            DIM,
        );
        p.line_segment(
            [ruler.left_bottom(), ruler.right_bottom()],
            Stroke::new(1.0, RULE),
        );

        // ---- 再生 ----
        // **Space で入り切り。** soundtrack がある project は音が実際に鳴り、
        // playhead は audio clock(`motolii-transport`)に同期して進む。無ければ
        // 従来どおり壁時計で playhead だけが動く。
        // 掴んでいる最中は入り切りしない — ドラッグ中に時間が流れると何が起きたか読めない
        let comp_seconds = self.document.composition.duration.as_seconds_f64() as f32;
        // **Alt を押しているあいだは吸着が切れる。** 押しっぱなしで自由に置ける
        self.snap = !ctx.input(|i| i.modifiers.alt);
        let (space, loop_key, dt) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Space),
                i.key_pressed(egui::Key::L) && !i.modifiers.command,
                i.stable_dt,
            )
        });
        // 掴み物が毎フレーム使う3つ(時刻・端流し・px/秒)を1つにまとめる
        let surface = Surface {
            track_left,
            track_w,
            comp: comp_seconds,
            dt,
        };

        if space && self.hold.is_none() {
            self.playing = !self.playing;
            // 終端で押したら頭から。止まったまま何も起きないのが一番困る
            if self.playing && self.playhead >= comp_seconds - 1e-3 {
                self.playhead = 0.0;
            }
            self.status = if self.playing { "play" } else { "pause" }.to_owned();
        }
        // `L` は帯を消さずに効きだけ切る。**引き直さずに戻せる**
        if loop_key {
            self.loop_region.on = !self.loop_region.on;
            self.status = format!(
                "loop {} ({:.2}–{:.2}s)",
                if self.loop_region.on { "on" } else { "off" },
                self.loop_region.start,
                self.loop_region.end
            );
        }
        if self.playing {
            // 再生の頭で audio の座席を開く(soundtrack 無しなら壁時計のまま)。
            // 開けない時は**黙らない** — 理由を status に出して壁時計へ落ちる
            if self.audio.is_none() {
                let playback = audio_seat::open_playback(
                    &self.document,
                    self.project_root.as_deref(),
                    &mut self.pcm_caches,
                    self.playhead,
                    fps,
                );
                match &playback {
                    AudioPlayback::Synced(_) => self.status = "play (soundtrack)".to_owned(),
                    AudioPlayback::WallClock(WallClockReason::NoSoundtrack) => {}
                    AudioPlayback::WallClock(WallClockReason::AudioUnavailable(reason)) => {
                        self.status = format!("play (no audio: {reason})");
                    }
                }
                self.audio = Some(playback);
            }
            // **手で動かした playhead に音が付いてくる。** to_start / locator で
            // 跳んだら、そこから鳴らし直す(audio が書いた値と違えば動かした印)
            let moved = matches!(
                &self.audio,
                Some(AudioPlayback::Synced(seat))
                    if audio_seat::playhead_moved(self.playhead, seat.last_synced())
            );
            if moved {
                self.reseek_audio(self.playhead, fps);
            }
            // **溜まった時間をまとめて進めない。** 窓が隠れていた分は捨てる
            // (ここだけは指摘の時点より後の修正を残した。窓が他のウィンドウの
            //  後ろにあると eframe が描画を間引き、戻った1フレームの `dt` が
            //  数百msになる — 足すと playhead が数秒ぶん飛ぶ)
            //
            // audio の座席があるときは dt を使わない — **clock の正本は
            // デバイスへ供給済みのサンプル数**(`motolii-transport`)で、
            // 隠れていた窓の分も音は正しく流れ続けている。
            let step = match self.audio.as_mut() {
                Some(AudioPlayback::Synced(seat)) => Some(seat.follow(comp_seconds)),
                _ => None,
            };
            let (at, keep) = match step {
                Some(Ok(step)) => step,
                Some(Err(error)) => {
                    // clock が壊れたら止まらずに壁時計へ明示的に落ちる
                    self.status = format!("play (no audio: {error})");
                    self.audio = Some(AudioPlayback::WallClock(
                        WallClockReason::AudioUnavailable(error.to_string()),
                    ));
                    advance_playhead(self.playhead, dt.min(MAX_STEP), comp_seconds)
                }
                None => advance_playhead(self.playhead, dt.min(MAX_STEP), comp_seconds),
            };
            // **折り返したかどうかで終端判定が変わる。** 折り返したのなら
            // composition の終わりに着いたのではない(ループのお尻が終端と
            // 同じ時刻でも、再生は続く)
            let wrapped = wrap_playhead(self.playhead, at, self.loop_region);
            let did_wrap = wrapped != at;
            self.playhead = wrapped;
            if did_wrap {
                // ループの頭から鳴らし直す(音はまっすぐにしか流れない)
                self.reseek_audio(wrapped, fps);
            }
            if !keep && !did_wrap {
                self.playing = false;
                self.status = "end".to_owned();
                // 終端で device を手放す
                self.audio = None;
            }
            // **面のほうが流れ、playhead は窓の中央に居続ける。**
            //
            // 2026-08-16: ここは一度「相対位置を保つ」へ変え、さらに DAW の
            // ページ送りへ変えたが、**利用者の指定でこの形へ戻した**。
            // 経緯は docs/reviews/2026-08-16-daw-playhead-follow-prior-art.md の
            // 「撤廃」。違和感の原因は追従ではなく目盛の明暗だった。
            //
            // 頭と終端では `clamped` が窓を止めるので、そこだけは playhead のほうが
            // 動く — 流れる物が無いところで無理に流さない
            self.view = TimelineView {
                start: self.playhead - self.view.span * 0.5,
                span: self.view.span,
            }
            .clamped(comp_seconds);
        } else if self.audio.is_some() {
            // pause / スクラブで止まったら device を手放す(握り続けない)
            self.audio = None;
        }

        // ---- ループ帯 ----
        // **ルーラの上端だけがループの面である。** 下は今までどおりスクラブなので、
        // 「掴む場所が違えば別のもの」で撃ち分けられる(bar の端6px と同じ考え方)
        let loop_lane = Rect::from_min_max(
            egui::pos2(track_left, ruler.top()),
            egui::pos2(full.right(), ruler.top() + LOOP_H),
        );
        let loop_x0 = self
            .view
            .time_to_x(self.loop_region.start, track_left, track_w);
        let loop_x1 = self
            .view
            .time_to_x(self.loop_region.end, track_left, track_w);
        let loop_hit = ui.interact(loop_lane, ui.id().with("loop"), Sense::click_and_drag());
        if let Some(pos) = loop_hit.hover_pos() {
            // **掴み判定そのものから形を出す。** 別に書くと、片方だけ直したときに
            // 手の形と起きることがずれる
            let would = loop_grab_for(pos.x, loop_x0, loop_x1, 0.0, self.loop_region);
            self.hover_cursor(&ctx, loop_hit.hovered(), loop_grab_cursor(&would));
        }
        if loop_hit.drag_started() {
            // 同じ理由で、押した場所から掴み方を決める
            if let Some(pos) = ctx
                .input(|i| i.pointer.press_origin())
                .or_else(|| loop_hit.interact_pointer_pos())
            {
                let at = self.view.x_to_time(pos.x, track_left, track_w);
                self.hold = Some(Hold::Loop(loop_grab_for(
                    pos.x,
                    loop_x0,
                    loop_x1,
                    at,
                    self.loop_region,
                )));
                // **引いたら効く。** 引いてから別のキーで入れる手順にしない
                self.loop_region.on = true;
            }
        }
        if loop_hit.dragged() {
            let loop_grab = match &self.hold {
                Some(Hold::Loop(grab)) => Some(*grab),
                _ => None,
            };
            if let (Some(grab), Some(pos)) = (loop_grab, loop_hit.interact_pointer_pos()) {
                let at = self.view.x_to_time(pos.x, track_left, track_w);
                let (start, end) = match grab {
                    LoopGrab::New { anchor } => loop_from_drag(anchor, at, comp_seconds, fps),
                    // **反対側は掴んだ瞬間の値で固定する。** 追い越しても畳まれない
                    LoopGrab::In { fixed } => loop_from_drag(at, fixed, comp_seconds, fps),
                    LoopGrab::Out { fixed } => loop_from_drag(fixed, at, comp_seconds, fps),
                    LoopGrab::Move { grab_at, from } => {
                        // 区間ごと動かすときは長さを変えない。端に当たったら止まる
                        let length = from.1 - from.0;
                        let moved = (from.0 + (at - grab_at)).clamp(0.0, comp_seconds - length);
                        loop_from_drag(moved, moved + length, comp_seconds, fps)
                    }
                };
                self.loop_region.start = start;
                self.loop_region.end = end;
                self.status = format!("loop {start:.2}–{end:.2}s");
                // 端まで引いたら窓が流れる。**窓の外までループを引ける**
                self.view = surface.edge_pan(self.view, pos.x);
            }
        }
        if loop_hit.drag_stopped() {
            self.hold = None;
        }
        // 帯を描く。**切れているときも残す** — 引いた区間は消えていない
        {
            let lane = p.with_clip_rect(loop_lane);
            let x0 = self
                .view
                .time_to_x(self.loop_region.start, track_left, track_w);
            let x1 = self
                .view
                .time_to_x(self.loop_region.end, track_left, track_w);
            lane.rect_filled(
                Rect::from_min_max(
                    egui::pos2(x0, loop_lane.top() + 1.0),
                    egui::pos2(x1, loop_lane.bottom() - 1.0),
                ),
                CornerRadius::same(2),
                if self.loop_region.on {
                    ACCENT
                } else {
                    Color32::from_rgb(0x4a, 0x4a, 0x4a)
                },
            );
            // 掴める端を示す
            if loop_hit.hovered() || matches!(self.hold, Some(Hold::Loop(_))) {
                for x in [x0, x1] {
                    lane.rect_filled(
                        Rect::from_min_max(
                            egui::pos2(x - 1.0, loop_lane.top()),
                            egui::pos2(x + 1.0, loop_lane.bottom()),
                        ),
                        CornerRadius::ZERO,
                        INK,
                    );
                }
            }
        }

        // ---- メモ(locator) ----
        // **ルーラの下端に印を置く。** 評価に入らないものなので、面の上には出さず
        // 縦線だけ薄く伸ばす(どの時刻の話かは分かるが、編集の邪魔をしない)
        let locator_lane = Rect::from_min_max(
            egui::pos2(track_left, ruler.top() + LOOP_H),
            egui::pos2(full.right(), ruler.top() + LOOP_H + LOCATOR_H),
        );
        let mut time_line_hints: Vec<f32> = Vec::new();
        let mut locator_action: Option<MenuAction> = None;
        let mut locator_clicked: Option<usize> = None;
        let mut locator_jump: Option<f32> = None;
        let mut locator_rename: Option<usize> = None;
        // **借りたまま書かない。** 行を回す間に Document が変わる形にしない
        let locators: Vec<(f32, String)> = self
            .document
            .locators
            .iter()
            .map(|m| (m.t.as_seconds_f64() as f32, m.text.clone()))
            .collect();
        for (index, (t, text)) in locators.into_iter().enumerate() {
            let t = t;
            let x = self.view.time_to_x(t, track_left, track_w);
            if x < locator_lane.left() - 8.0 || x > locator_lane.right() {
                continue;
            }
            let pin = Rect::from_center_size(
                egui::pos2(x + 1.0, locator_lane.center().y),
                Vec2::new(12.0, LOCATOR_H),
            );
            let r = ui.interact(
                pin,
                ui.id().with(("locator", index)),
                Sense::click_and_drag(),
            );
            let playhead_is_here = (self.playhead - t).abs() < 1e-3;
            self.hover_cursor(&ctx, r.hovered(), egui::CursorIcon::Grab);
            let lane_p = p.with_clip_rect(locator_lane);
            lane_p.add(egui::Shape::convex_polygon(
                vec![
                    egui::pos2(x, locator_lane.top() + 1.0),
                    egui::pos2(x + 7.0, locator_lane.center().y),
                    egui::pos2(x, locator_lane.bottom() - 1.0),
                ],
                if r.hovered() || self.editing_locator.as_ref().map(|(i, _)| *i) == Some(index) {
                    ACCENT
                } else {
                    Color32::from_rgb(0x9a, 0x8a, 0x55)
                },
                Stroke::NONE,
            ));
            if !text.is_empty() {
                lane_p.text(
                    egui::pos2(x + 10.0, locator_lane.center().y),
                    Align2::LEFT_CENTER,
                    &text,
                    FontId::proportional(9.0),
                    if playhead_is_here {
                        ACCENT
                    } else {
                        Color32::from_rgb(0xb9, 0xa9, 0x74)
                    },
                );
            }
            // 面の上へは薄い縦線だけ
            time_line_hints.push(t);
            // **押したら跳ぶ。** ロケータの本体は「そこへ行くこと」である
            if r.clicked() {
                locator_jump = Some(t);
            }
            // **掴んだ瞬間に gesture を1つ採る。** 毎フレーム開き直していたので、
            // 1回動かすとフレーム数だけ Undo が積まれていた
            if r.drag_started() {
                let gesture = self.writer.begin_gesture();
                self.hold = Some(Hold::Locator { index, gesture });
            }
            if r.dragged() {
                let held = match &self.hold {
                    Some(Hold::Locator { index, gesture }) => Some((*index, *gesture)),
                    _ => None,
                };
                if let (Some((index, gesture)), Some(pos)) = (held, r.interact_pointer_pos()) {
                    let at = surface.time_at(self.view, pos.x).clamp(0.0, surface.comp);
                    let at = self.snapped(at, &[], surface.px_per_second(self.view));
                    if let Some(time) = seconds_to_time(at, fps) {
                        let prepared = self.writer.prepare_set_locator_time(index, time);
                        if self.apply_in(gesture, "locator", prepared) {
                            self.status = format!("locator {at:.2}s");
                        }
                    }
                    self.view = surface.edge_pan(self.view, pos.x);
                }
            }
            if r.drag_stopped() {
                self.hold = None;
            }
            r.context_menu(|ui| {
                ui.set_min_width(150.0);
                ui.label(egui::RichText::new("locator").color(DIM).size(9.0));
                ui.separator();
                if ui.button("Remove locator   ⌫").clicked() {
                    locator_action = Some(MenuAction::RemoveLocator(index));
                    ui.close();
                }
            });
        }
        if let Some(at) = locator_jump {
            self.playhead = at;
            self.status = format!("{}", timecode(at, fps));
        }
        if let Some(index) = locator_clicked {
            let text = self.document.locators[index].text.clone();
            self.editing_locator = Some((index, text));
        }
        if let Some(index) = locator_rename {
            let text = self.document.locators[index].text.clone();
            self.editing_locator = Some((index, text));
        }
        if let Some(action) = locator_action {
            self.run_menu(action, comp_seconds, fps);
        }
        // 編集中は、その印の上が入力欄になる
        if let Some((index, text)) = self.editing_locator.clone() {
            if let Some(locator) = self.document.locators.get(index) {
                let x = self
                    .view
                    .time_to_x(locator.t.as_seconds_f64() as f32, track_left, track_w);
                let rect = Rect::from_min_size(
                    egui::pos2(x + 6.0, locator_lane.top()),
                    Vec2::new(120.0, locator_lane.height()),
                );
                let mut buf = text;
                let edit = ui.put(
                    rect,
                    egui::TextEdit::singleline(&mut buf)
                        .font(FontId::proportional(9.0))
                        .margin(Vec2::new(2.0, 0.0)),
                );
                edit.request_focus();
                self.editing_locator = Some((index, buf));
                if edit.lost_focus() {
                    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.editing_locator = None;
                    } else {
                        self.commit_locator_text();
                    }
                }
            } else {
                self.editing_locator = None;
            }
        }

        // ルーラのスクラブ。**Document は触らない** — playhead は session の状態。
        // **ループ帯のぶんだけ下から**(上端はループが取る)
        let ruler_track = Rect::from_min_max(
            egui::pos2(track_left, ruler.top() + LOOP_H + LOCATOR_H),
            ruler.max,
        );
        let scrub = ui.interact(ruler_track, ui.id().with("ruler"), Sense::click_and_drag());
        if scrub.is_pointer_button_down_on() {
            if let Some(pos) = scrub.interact_pointer_pos() {
                // 掴んだら再生は止まる。**手で動かしているものが勝手に進まない**
                self.playing = false;
                let at = self
                    .view
                    .x_to_time(pos.x, track_left, track_w)
                    .clamp(0.0, comp_seconds);
                // **playhead もフレームに乗る。** 編集の時刻と同じ粒でないと、
                // キーの上に置いたつもりで半端な位置に居ることになる
                self.playhead = seconds_to_time(at, fps)
                    .map(|t| t.as_seconds_f64() as f32)
                    .unwrap_or(at);
                // **掴んだまま端まで運んだら窓が付いてくる。** 窓の中にある時間しか
                // 指せないと、寄っているときに playhead を遠くへ運べない
                self.view = surface.edge_pan(self.view, pos.x);
                self.status = format!("{:.2}s", self.playhead);
            }
        }

        // ---- 行 ----
        // **行の面は、ルーラの下からナビゲータ帯の上まで。** 縦スクロールはこの中だけ
        let nav_top = (full.bottom() - NAV_H).max(ruler.bottom());
        let rows_view = Rect::from_min_max(
            egui::pos2(full.left(), ruler.bottom()),
            egui::pos2(full.right(), nav_top),
        );
        let (row_h, prop_h) = if self.large_rows {
            (ROW_H_LARGE, PROP_H_LARGE)
        } else {
            (ROW_H, PROP_H)
        };
        let content_h = content_height(&visible, row_h, prop_h);
        self.scroll_y = clamp_scroll(self.scroll_y, content_h, rows_view.height());

        // **位置を先に確定させる。** 描く順と、並べ替えの落とし先と、線の位置が
        // 同じ1つの表から出る(3箇所で y を数え直さない)
        let mut layout: Vec<(TimelineRow, f32, f32)> = Vec::with_capacity(visible.len());
        let mut y = rows_view.top() - self.scroll_y;
        for row in &visible {
            let h = match row.kind {
                RowKind::Object => row_h,
                RowKind::Property(_) => prop_h,
            };
            layout.push((*row, y, h));
            y += h;
        }
        // 並べ替えが数える単位は object 行だけ。パラメータ行のあいだへは落とせない
        let objects: Vec<(LayerId, f32, f32)> = layout
            .iter()
            .filter(|(row, _, _)| matches!(row.kind, RowKind::Object))
            .map(|(row, top, h)| (row.layer, *top, *h))
            .collect();
        let object_layers: Vec<LayerId> = objects.iter().map(|(l, _, _)| *l).collect();
        // **キーも画面に出ている順に並べる**(行の順 → その行の中は時刻順)。
        // 行と同じ規則で範囲選択できるのは、この並びがあるからである
        let key_order: Vec<(LayerId, ParamRef, KeyframeId)> = layout
            .iter()
            .filter_map(|(row, _, _)| match row.kind {
                RowKind::Property(param) => Some((row.layer, param)),
                RowKind::Object => None,
            })
            .flat_map(|(layer, param)| {
                let mut keys = param_keys(&self.document, layer, param);
                keys.sort_by(|a, b| a.1.total_cmp(&b.1));
                keys.into_iter().map(move |(key, _)| (layer, param, key))
            })
            .collect();

        let mut toggles: Vec<(LayerId, bool)> = Vec::new();
        // M / S のクリック。行を回している間は Document を触らず、回し終えてから書く
        let mut flags: Vec<(LayerId, Flag)> = Vec::new();
        let mut pick: Option<(LayerId, bool, bool)> = None;
        let mut reorder_started: Option<LayerId> = None;
        let mut reorder_released = false;
        let mut menu_action: Option<MenuAction> = None;
        let mut rename_started: Option<LayerId> = None;
        let mut rename_committed = false;
        let mut rename_cancelled = false;

        // 何も無いところの右クリック。**行より先に登録する**ので、行の上では行が勝つ
        let surface_bg = ui.interact(rows_view, ui.id().with("surface"), Sense::click_and_drag());
        // **何も無いところを押したら選択は空になる。** 押した物が選択、が通るなら
        // 「何も押していない」も通らなければ筋が合わない
        self.hover_cursor(&ctx, surface_bg.hovered(), egui::CursorIcon::Crosshair);
        if surface_bg.clicked() {
            self.selected.clear();
            self.selected_keys.clear();
            self.status = "nothing selected".to_owned();
        }
        // 矩形選択。**掴んだ範囲に bar が掛かっている行を選ぶ**
        if surface_bg.drag_started() {
            if let Some(pos) = surface_bg.interact_pointer_pos() {
                self.hold = Some(Hold::Marquee { from: pos, to: pos });
            }
        }
        if surface_bg.dragged() {
            if let (Some(Hold::Marquee { from, .. }), Some(pos)) =
                (self.hold.clone(), surface_bg.interact_pointer_pos())
            {
                self.hold = Some(Hold::Marquee { from, to: pos });
            }
        }
        {
            let loop_on = self.loop_region.on;
            // **右クリックした瞬間の時刻を控える。** メニューが開いている間に
            // ポインタが動いても、置き場所は押した所のまま
            if surface_bg.secondary_clicked() {
                if let Some(pos) = surface_bg.interact_pointer_pos() {
                    self.context_time = self.view.x_to_time(pos.x, track_left, track_w).max(0.0);
                }
            }
            let at = self.context_time;
            surface_bg
                .context_menu(|ui| TimelineEditor::surface_menu(ui, loop_on, at, &mut menu_action));
        }

        // **面からはみ出した行は描かない。** 1000行でも触るのは見えている分だけ
        let row_p = p.with_clip_rect(rows_view);
        for (row, top, h) in &layout {
            let p = &row_p;
            let (row, h) = (*row, *h);
            let y = *top;
            if y + h < rows_view.top() || y > rows_view.bottom() {
                continue;
            }
            let rect = Rect::from_min_size(egui::pos2(full.left(), y), Vec2::new(full.width(), h));
            let rail = Rect::from_min_size(rect.min, Vec2::new(RAIL_W, h));
            let track = Rect::from_min_max(egui::pos2(track_left, rect.top()), rect.max);

            // 左列
            let cell = match row.kind {
                RowKind::Property(_) => CELL_PROP,
                _ if row.depth > 0 => CELL_CHILD,
                _ => CELL,
            };
            // 選んだ行は左列も少し明るくする。**帯だけだと1クリックの手応えが薄い**
            let cell = if matches!(row.kind, RowKind::Object) && self.is_selected(row.layer) {
                Color32::from_rgb(
                    cell.r().saturating_add(0x0e),
                    cell.g().saturating_add(0x0e),
                    cell.b().saturating_add(0x0e),
                )
            } else {
                cell
            };
            p.rect_filled(rail, CornerRadius::ZERO, cell);
            // 左列のどこを押しても、その行の layer を選ぶ(ボタン類は上に載るので先に取られる)。
            // **同じ場所を掴んで上下へ引くと並べ替え**になる — AE と同じで、
            // 名前の列は「選ぶ」と「並べ替える」の両方の入口である
            if matches!(row.kind, RowKind::Object) {
                let r = ui.interact(
                    rail,
                    ui.id().with(("pick", row.layer)),
                    Sense::click_and_drag(),
                );
                self.hover_cursor(&ctx, r.hovered(), egui::CursorIcon::Grab);
                if r.clicked() {
                    let (additive, range) = ctx.input(|i| (i.modifiers.command, i.modifiers.shift));
                    pick = Some((row.layer, additive, range));
                }
                if r.drag_started() {
                    reorder_started = Some(row.layer);
                }
                if r.drag_stopped() {
                    reorder_released = true;
                }
                // **右クリックは、選んでいない行なら選び直してから開く。**
                // 選択と、メニューが効く相手が食い違うのが一番危ない
                if r.secondary_clicked() && !self.is_selected(row.layer) {
                    pick = Some((row.layer, false, false));
                }
                {
                    let name = self.name(row.layer).to_owned();
                    let (visible, solo, lock) =
                        item_flags(&self.document, row.layer).unwrap_or((true, false, false));
                    let selected = self.selected.len().max(1);
                    r.context_menu(|ui| {
                        TimelineEditor::row_menu(
                            ui,
                            &name,
                            row.layer,
                            row.has_children,
                            !visible,
                            solo,
                            lock,
                            selected,
                            &mut menu_action,
                        )
                    });
                }
            }
            if self.is_selected(row.layer) {
                // 選択の帯。**行全体ではなく左端の細い帯**(モックの inset 3px と同じ)
                p.rect_filled(
                    Rect::from_min_size(rail.left_top(), Vec2::new(3.0, rail.height())),
                    CornerRadius::ZERO,
                    SELECTED,
                );
            }
            p.line_segment(
                [rail.left_bottom(), rail.right_bottom()],
                Stroke::new(1.0, RULE),
            );
            p.line_segment(
                [rail.right_top(), rail.right_bottom()],
                Stroke::new(1.0, Color32::from_rgb(0x09, 0x09, 0x09)),
            );

            let indent = 8.0 + row.depth as f32 * 14.0;
            let cy = rail.center().y;

            // **入れ子の背骨。** 深さのぶんだけ縦線を引き、どこまでが誰の中かを見せる
            for level in 0..row.depth {
                let x = rail.left() + 8.0 + level as f32 * 14.0 + 2.0;
                p.line_segment(
                    [egui::pos2(x, rail.top()), egui::pos2(x, rail.bottom())],
                    Stroke::new(1.0, Color32::from_rgb(0x4a, 0x4a, 0x4a)),
                );
            }

            // 子の開閉（三角）— 子を持つ行だけ
            if row.has_children {
                let hit = Rect::from_center_size(
                    egui::pos2(rail.left() + indent + 2.0, cy),
                    Vec2::splat(16.0),
                );
                if rail_glyph(
                    ui,
                    p,
                    ui.id().with(("fold", row.layer)),
                    hit,
                    if row.children_open { "▾" } else { "▸" },
                    false,
                ) {
                    toggles.push((row.layer, true));
                }
            }

            match row.kind {
                RowKind::Object => {
                    let icon = Rect::from_center_size(
                        egui::pos2(rail.left() + indent + 20.0, cy),
                        Vec2::splat(9.0),
                    );
                    // 色の札。**Group もそうでないものも同じ扱い**にする
                    p.rect_filled(
                        icon,
                        CornerRadius::same(2),
                        if self.colors_on {
                            layer_color(&self.document, row.layer)
                        } else {
                            Color32::from_rgb(0x72, 0x92, 0x98)
                        },
                    );
                    // 名前。**編集中はその場が入力欄になる**(別の窓を出さない)
                    let name_rect = Rect::from_min_max(
                        egui::pos2(rail.left() + indent + 30.0, rail.top() + 2.0),
                        egui::pos2(rail.right() - 74.0, rail.bottom() - 2.0),
                    );
                    if self.renaming.as_ref().map(|(l, _)| *l) == Some(row.layer) {
                        let mut buf = self
                            .renaming
                            .as_ref()
                            .map(|(_, n)| n.clone())
                            .unwrap_or_default();
                        let edit = ui.put(
                            name_rect,
                            egui::TextEdit::singleline(&mut buf)
                                .font(FontId::proportional(11.0))
                                .margin(Vec2::new(2.0, 0.0)),
                        );
                        edit.request_focus();
                        if let Some((_, name)) = self.renaming.as_mut() {
                            *name = buf;
                        }
                        rename_committed =
                            edit.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter));
                        if edit.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                            rename_cancelled = true;
                        }
                    } else {
                        // **名前は名前の枠から出さない。** はみ出すと ◇ や M/S/L に被る
                        p.with_clip_rect(name_rect).text(
                            egui::pos2(rail.left() + indent + 32.0, cy),
                            Align2::LEFT_CENTER,
                            self.name(row.layer),
                            FontId::proportional(11.0),
                            INK,
                        );
                    }

                    // キー行の開閉（◇/◆）— キーを持つ行だけ
                    let has_keys = !visible_params(&self.document, row.layer).is_empty();
                    if has_keys {
                        let hit = Rect::from_center_size(
                            egui::pos2(rail.right() - 66.0, cy),
                            Vec2::splat(16.0),
                        );
                        if rail_glyph(
                            ui,
                            p,
                            ui.id().with(("params", row.layer)),
                            hit,
                            if row.params_open { "◆" } else { "◇" },
                            row.params_open,
                        ) {
                            toggles.push((row.layer, false));
                        }
                    }

                    // **押下状態は Document から読む。** ボタン側に状態を持たない
                    let (item_visible, item_solo, item_lock) =
                        item_flags(&self.document, row.layer).unwrap_or((true, false, false));
                    let inherited_lock = effective_lock(&self.document, row.layer) && !item_lock;
                    for (i, (label, flag)) in
                        [("M", Flag::Mute), ("S", Flag::Solo), ("L", Flag::Lock)]
                            .iter()
                            .enumerate()
                    {
                        let b = Rect::from_center_size(
                            egui::pos2(rail.right() - 48.0 + i as f32 * 18.0, cy),
                            Vec2::splat(16.0),
                        );
                        let on = match flag {
                            Flag::Mute => !item_visible,
                            Flag::Solo => item_solo,
                            // **自分が掛けた分だけを点ける。** 親から受けている分は
                            // 下で薄く出す — 押しても外れないものを点灯させない
                            Flag::Lock => item_lock,
                        };
                        if *flag == Flag::Lock && !item_lock && inherited_lock {
                            p.rect_filled(b, CornerRadius::ZERO, LOCK_INHERITED);
                        }
                        let on_color = match flag {
                            Flag::Mute => MUTE_ON,
                            Flag::Solo => SOLO_ON,
                            Flag::Lock => LOCK_ON,
                        };
                        if rail_button(
                            ui,
                            p,
                            ui.id().with(("flag", row.layer, i)),
                            b,
                            label,
                            on,
                            on_color,
                        ) {
                            flags.push((row.layer, *flag));
                        }
                    }
                }
                RowKind::Property(param) => {
                    let chip = Rect::from_center_size(
                        egui::pos2(rail.left() + indent + 10.0, cy),
                        Vec2::new(4.0, 11.0),
                    );
                    p.rect_filled(chip, CornerRadius::same(2), param_color(param));
                    p.text(
                        egui::pos2(rail.left() + indent + 22.0, cy),
                        Align2::LEFT_CENTER,
                        param_label(param),
                        FontId::proportional(10.0),
                        Color32::from_rgb(0xa8, 0xa8, 0xa8),
                    );
                }
            }

            // 時間面。**方眼はルーラと同じ目盛の上に立つ**(細目盛は薄く)
            p.rect_filled(track, CornerRadius::ZERO, TRACK_A);
            // **1区間おきに下地を変える。** 線だけだと区間が全部同じ面に見えて、
            // どこからどこまでが1目盛なのかを目で掴めない。明暗があると
            // 「区間」そのものが図になる(Ableton の Arrangement と同じ作り)。
            // 濃淡は**絶対時刻で決める** — パンしても縞が入れ替わらない
            for t in &ticks {
                if band_is_dark(*t, step) {
                    let x0 = self.view.time_to_x(*t, track_left, track_w);
                    let x1 = self.view.time_to_x(*t + step, track_left, track_w);
                    let band = Rect::from_min_max(
                        egui::pos2(x0.max(track.left()), track.top()),
                        egui::pos2(x1.min(track.right()), track.bottom()),
                    );
                    if band.width() > 0.0 {
                        p.rect_filled(band, CornerRadius::ZERO, TRACK_BAND);
                    }
                }
            }
            for t in &minor_ticks {
                let x = self.view.time_to_x(*t, track_left, track_w);
                if x < track.left() {
                    continue;
                }
                p.line_segment(
                    [egui::pos2(x, track.top()), egui::pos2(x, track.bottom())],
                    Stroke::new(1.0, TRACK_MINOR),
                );
            }
            for t in &ticks {
                let x = self.view.time_to_x(*t, track_left, track_w);
                if x < track.left() {
                    continue;
                }
                p.line_segment(
                    [egui::pos2(x, track.top()), egui::pos2(x, track.bottom())],
                    Stroke::new(1.0, TRACK_B),
                );
            }
            p.line_segment(
                [track.left_bottom(), track.right_bottom()],
                Stroke::new(1.0, RULE),
            );

            // **時間面の描画は track の中だけ。** 左列へはみ出すと M/S を覆う
            let p = p.with_clip_rect(track);
            match row.kind {
                RowKind::Object => {
                    if let Some((start, end)) = clip_span(&self.document, row.layer) {
                        let x0 = self.view.time_to_x(start, track_left, track_w);
                        let x1 = self.view.time_to_x(end, track_left, track_w);
                        let bar = Rect::from_min_max(
                            egui::pos2(x0, track.top() + 3.0),
                            egui::pos2(x1, track.bottom() - 4.0),
                        );
                        // **当たり判定も時間面の中だけ。** 寄ると clip の左端は
                        // 面の外(左レールの下)まで伸びる。描画はクリップしていたが
                        // 判定はしていなかったので、bar が左列のボタンを飲んでいた
                        // (「拡大すると開閉できない」の正体)
                        let bar_hit = bar.intersect(track);
                        let r = ui.interact(
                            bar_hit,
                            ui.id().with(("bar", row.layer)),
                            if bar_hit.width() > 0.5 {
                                Sense::click_and_drag()
                            } else {
                                Sense::hover()
                            },
                        );
                        let color = if self.colors_on {
                            layer_color(&self.document, row.layer)
                        } else if row.has_children {
                            Color32::from_rgb(0x4c, 0x49, 0x3c)
                        } else {
                            Color32::from_rgb(0x65, 0x75, 0x8c)
                        };
                        p.rect_filled(
                            bar,
                            CornerRadius::ZERO,
                            if r.dragged() { ACCENT } else { color },
                        );
                        // **ポインタの居場所は1回だけ決める。** 絵・カーソル・
                        // 掴みが別々に聞くと、答えが食い違って手の形が嘘をつく。
                        // 押している間は `interact_pointer_pos`、触れているだけの
                        // ときは `hover_pos` — どちらか一方しか来ない
                        let pointer_x = r.interact_pointer_pos().or(r.hover_pos()).map(|pos| pos.x);
                        let part = pointer_x.map(|x| classify_bar_edge(bar, x, row.has_children));
                        // **選んだ bar は光る。** 左列の帯だけだと、時間面の上で
                        // 何を選んだのかが分からない
                        let selected = self.is_selected(row.layer);
                        if selected {
                            p.rect_stroke(
                                bar,
                                CornerRadius::ZERO,
                                Stroke::new(2.0, SELECTED),
                                StrokeKind::Inside,
                            );
                        }
                        // **掴める端を絵で出す。** 触れそうな所は見えていなければ
                        // 探すことになる(hover か選択のときだけ、常時だと煩い)
                        let has_edges =
                            classify_bar_edge(bar, bar.left(), row.has_children) == BarPart::TrimIn;
                        if has_edges && (r.hovered() || selected) {
                            let hovered_part = part;
                            for (part, band) in [
                                (
                                    BarPart::TrimIn,
                                    Rect::from_min_max(
                                        bar.left_top(),
                                        egui::pos2(bar.left() + TRIM_EDGE, bar.bottom()),
                                    ),
                                ),
                                (
                                    BarPart::TrimOut,
                                    Rect::from_min_max(
                                        egui::pos2(bar.right() - TRIM_EDGE, bar.top()),
                                        bar.right_bottom(),
                                    ),
                                ),
                            ] {
                                p.rect_filled(
                                    band.intersect(track),
                                    CornerRadius::ZERO,
                                    if hovered_part == Some(part) {
                                        SELECTED
                                    } else {
                                        Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, 40)
                                    },
                                );
                            }
                        }
                        // 手の形で、掴んだら何になるかを先に言う。**同じ `part` から**
                        // **押しても何も起きない所では、掴める形を出さない。**
                        // ロック中は断るのだから、手の形も断る
                        self.hover_cursor(
                            &ctx,
                            r.hovered() && bar_hit.width() > 0.5,
                            if self.is_locked(row.layer) {
                                egui::CursorIcon::NotAllowed
                            } else {
                                match part.unwrap_or(BarPart::Body) {
                                    BarPart::Body => egui::CursorIcon::Grab,
                                    _ => egui::CursorIcon::ResizeHorizontal,
                                }
                            },
                        );
                        // **畳んである Group は、中身をその bar の中に出す。**
                        // 開けば行として見えるものが、閉じると消えてしまうと、
                        // 何が入っているのか掴めない(説明の文字を足さずに済ませる)
                        if row.has_children && !row.children_open {
                            for (child, start, end) in movable_clips(&self.document, row.layer) {
                                let cx0 = self.view.time_to_x(start, track_left, track_w);
                                let cx1 = self.view.time_to_x(end, track_left, track_w);
                                let inner = Rect::from_min_max(
                                    egui::pos2(cx0, bar.top() + bar.height() * 0.55),
                                    egui::pos2(cx1, bar.bottom() - 1.0),
                                );
                                if inner.width() > 0.5 {
                                    p.rect_filled(
                                        inner,
                                        CornerRadius::ZERO,
                                        if self.colors_on {
                                            layer_color(&self.document, child)
                                        } else {
                                            Color32::from_rgb(0x65, 0x75, 0x8c)
                                        },
                                    );
                                }
                            }
                        }
                        p.rect_stroke(
                            bar,
                            CornerRadius::ZERO,
                            Stroke::new(1.0, Color32::from_rgb(0x17, 0x17, 0x17)),
                            StrokeKind::Inside,
                        );
                        // **クリックしただけで選ぶ。** 掴んで動かすまで選択が
                        // 変わらないのは、押した手応えが無いのと同じである
                        if r.clicked() {
                            let (additive, range) =
                                ctx.input(|i| (i.modifiers.command, i.modifiers.shift));
                            pick = Some((row.layer, additive, range));
                        }
                        if r.drag_started() && self.is_locked(row.layer) {
                            self.status = format!("{} is locked", self.name(row.layer));
                        } else if r.drag_started() {
                            // **判定は押した場所で行う。** egui は数px動いてから
                            // ドラッグ開始を報せるので、そのときのポインタは既に
                            // 動いている — 右端を掴んで左へ引くと、報せが来た時点で
                            // 端の外に出ており、トリムのつもりが移動になっていた
                            let press = ctx
                                .input(|i| i.pointer.press_origin())
                                .or_else(|| r.interact_pointer_pos());
                            if let Some(pos) = press {
                                let grab = match classify_bar_edge(bar, pos.x, row.has_children) {
                                    BarPart::TrimIn => Grab::TrimIn { layer: row.layer },
                                    BarPart::TrimOut => Grab::TrimOut { layer: row.layer },
                                    BarPart::Body => {
                                        // **選ばれているものは一緒に動く。** 選ばれて
                                        // いない bar を掴んだら、その1つを選び直す
                                        if !self.is_selected(row.layer) {
                                            self.selected = vec![row.layer];
                                        }
                                        let roots = self.selection_roots();
                                        begin_move_many(
                                            &self.document,
                                            &roots,
                                            row.layer,
                                            surface.time_at(self.view, pos.x),
                                        )
                                    }
                                };
                                self.hold_item(grab);
                            }
                        }
                        if r.dragged() {
                            if let Some(pos) = r.interact_pointer_pos() {
                                let at = surface.time_at(self.view, pos.x).max(0.0);
                                self.commit_drag_snapped(at, surface.px_per_second(self.view));
                                // 掴んだまま端まで運んだら窓が流れる(playhead と同じ)
                                self.view = surface.edge_pan(self.view, pos.x);
                            }
                        }
                        if r.drag_stopped() {
                            self.hold = None;
                        }
                    }
                }
                RowKind::Property(param) => {
                    // **時刻を動かせるのは Position だけ。** 他は掴んでも動かない
                    let movable = param == ParamRef::Position;
                    for (key, t) in param_keys(&self.document, row.layer, param) {
                        let x = self.view.time_to_x(t, track_left, track_w);
                        let c = egui::pos2(x, track.center().y);
                        let d = 4.0;
                        // 掴む的は菱形の中心から6px。bar の端と同じ寸法
                        let hit = Rect::from_center_size(c, Vec2::splat(12.0)).intersect(track);
                        let r = ui.interact(
                            hit,
                            ui.id()
                                .with(("key", row.layer, param_label(param), key.get())),
                            if hit.width() > 0.5 {
                                Sense::click_and_drag()
                            } else {
                                Sense::hover()
                            },
                        );
                        p.add(egui::Shape::convex_polygon(
                            vec![
                                egui::pos2(c.x, c.y - d),
                                egui::pos2(c.x + d, c.y),
                                egui::pos2(c.x, c.y + d),
                                egui::pos2(c.x - d, c.y),
                            ],
                            if self.selected_keys.contains(&(row.layer, param, key))
                                || r.dragged()
                                || r.hovered()
                            {
                                ACCENT
                            } else {
                                KEY_IDLE
                            },
                            Stroke::new(1.0, Color32::from_rgb(0xee, 0xee, 0xee)),
                        ));
                        self.hover_cursor(
                            &ctx,
                            r.hovered() && hit.width() > 0.5,
                            if self.is_locked(row.layer) {
                                egui::CursorIcon::NotAllowed
                            } else {
                                egui::CursorIcon::Grab
                            },
                        );
                        // キーもクリックで選ぶ。**Delete の対象がここで決まる**
                        if r.clicked() {
                            let (additive, range) =
                                ctx.input(|i| (i.modifiers.command, i.modifiers.shift));
                            self.select_key((row.layer, param, key), additive, range, &key_order);
                        }
                        if r.drag_started() && self.is_locked(row.layer) {
                            self.status = format!("{} is locked", self.name(row.layer));
                        } else if r.drag_started() {
                            // **どのパラメータのキーも掴める。** D2 に
                            // `SetTransformParamKeyTime` が入った時点で Position 縛りの
                            // 理由は消えていたのに、掴む側だけ残っていた。
                            // 掴んだ時刻も**押した場所**から採る — ドラッグ開始の報せは
                            // 数px動いた後に来るので、そこを起点にすると最初の1手で跳ぶ
                            if let Some(pos) = ctx
                                .input(|i| i.pointer.press_origin())
                                .or_else(|| r.interact_pointer_pos())
                            {
                                self.hold_item(Grab::KeyTime {
                                    layer: row.layer,
                                    param,
                                    key,
                                    grab_at: surface.time_at(self.view, pos.x),
                                    original: t,
                                });
                            }
                        }
                        if r.dragged() {
                            if let Some(pos) = r.interact_pointer_pos() {
                                let at = surface.time_at(self.view, pos.x).max(0.0);
                                self.commit_drag_snapped(at, surface.px_per_second(self.view));
                                self.view = surface.edge_pan(self.view, pos.x);
                            }
                        }
                        if r.drag_stopped() {
                            self.hold = None;
                        }
                        if r.secondary_clicked() && !self.is_selected(row.layer) {
                            pick = Some((row.layer, false, false));
                        }
                        {
                            let name = self.name(row.layer).to_owned();
                            let (visible, solo, lock) = item_flags(&self.document, row.layer)
                                .unwrap_or((true, false, false));
                            let selected = self.selected.len().max(1);
                            r.context_menu(|ui| {
                                TimelineEditor::row_menu(
                                    ui,
                                    &name,
                                    row.layer,
                                    row.has_children,
                                    !visible,
                                    solo,
                                    lock,
                                    selected,
                                    &mut menu_action,
                                )
                            });
                        }
                    }
                }
            }
        }

        // 掴み終わったら、矩形に掛かった行を選ぶ
        if surface_bg.drag_stopped() {
            let swept = match self.hold.take() {
                Some(Hold::Marquee { from, to }) => Some((from, to)),
                other => {
                    self.hold = other;
                    None
                }
            };
            if let Some((from, to)) = swept {
                let rect = Rect::from_two_pos(from, to);
                let mut hit = Vec::new();
                for (layer, top, h) in &objects {
                    let row_band = (*top, top + h);
                    if rect.bottom() < row_band.0 || rect.top() > row_band.1 {
                        continue;
                    }
                    // 時間方向は bar と重なっているか。**行に掛かるだけでは選ばない** —
                    // 空の時間を囲っただけで選ばれると、掃くたびに全部拾ってしまう
                    if let Some((start, end)) = clip_span(&self.document, *layer) {
                        let x0 = self.view.time_to_x(start, track_left, track_w);
                        let x1 = self.view.time_to_x(end, track_left, track_w);
                        if rect.right() >= x0 && rect.left() <= x1 {
                            hit.push(*layer);
                        }
                    }
                }
                // **掃いた中のキーも拾う。** bar と同じ掃き方で、同じように選べる
                let keys: Vec<(LayerId, ParamRef, KeyframeId)> = layout
                    .iter()
                    .filter_map(|(row, top, h)| match row.kind {
                        RowKind::Property(param) => Some((row.layer, param, *top, *h)),
                        RowKind::Object => None,
                    })
                    .filter(|(_, _, top, h)| rect.bottom() >= *top && rect.top() <= top + h)
                    .flat_map(|(layer, param, _, _)| {
                        param_keys(&self.document, layer, param)
                            .into_iter()
                            .map(move |(key, t)| (layer, param, key, t))
                    })
                    .filter(|(_, _, _, t)| {
                        let x = self.view.time_to_x(*t, track_left, track_w);
                        rect.left() <= x && x <= rect.right()
                    })
                    .map(|(layer, param, key, _)| (layer, param, key))
                    .collect();

                let swept_keys = !keys.is_empty();
                if swept_keys {
                    self.selected_keys = keys;
                    self.status = format!("{} keys selected", self.selected_keys.len());
                }
                if !hit.is_empty() {
                    self.selected = hit;
                    // キーを掃いていないときだけ、キーの選択を落とす
                    if !swept_keys {
                        self.selected_keys.clear();
                    }
                    self.status = format!("{} selected", self.selected.len());
                }
            }
        }

        // ---- 並べ替え ----
        // **落とし先は境界で決まる。** どの行の上に居るかではなく、
        // どの行と行のあいだに居るか
        if let Some(layer) = reorder_started {
            if !self.is_selected(layer) {
                self.selected = vec![layer];
            }
            self.hold_item(Grab::Reorder { layer });
        }
        if let Some((Grab::Reorder { layer }, _, _)) = self.item_hold() {
            self.drop = ctx.input(|i| i.pointer.latest_pos()).and_then(|pos| {
                let boundary = boundary_at(&objects, pos.y);
                let y = boundary_y(&objects, boundary);
                drop_target(&self.document, &object_layers, boundary, layer)
                    .map(|(parent, index)| DropTarget { parent, index, y })
            });
            if let Some(to) = self.drop {
                // 落とす前に線で見せる。**書くのは離した瞬間だけ**
                row_p.line_segment(
                    [
                        egui::pos2(full.left() + 4.0, to.y),
                        egui::pos2(full.right() - 4.0, to.y),
                    ],
                    Stroke::new(2.0, ACCENT),
                );
            }
            if reorder_released {
                if let Some(to) = self.drop.take() {
                    self.commit_reorder(layer, to);
                }
                self.hold = None;
            }
        }

        // ---- 縦スクロール / 横ズーム / 横パン ----
        // **割り当ては AE / Premiere と同じ。** 素のホイールは縦、Cmd で横ズーム
        let comp = self.document.composition.duration.as_seconds_f64() as f32;
        // **運ぶ量は生のまま取る。** `smooth_scroll_delta` は egui が均した値で、
        // 指を止めても数フレーム流れ続ける — 面を掴んで動かしている感触にならない
        // (OS 側の慣性はそのまま来るので、失うのは egui の上乗せ分だけ)。
        // ズームだけは均した値を使う: 倍率は1フレームの差が指数で効くので、
        // 生値だと段が見える
        let (scroll, smooth, shift, command, pinch, pointer) = ctx.input(|i| {
            (
                raw_wheel(i),
                i.smooth_scroll_delta,
                i.modifiers.shift,
                i.modifiers.command,
                i.zoom_delta(),
                i.pointer.latest_pos(),
            )
        });
        if let Some(pos) = pointer.filter(|p| full.contains(*p)) {
            // ズームの起点は時間面の中に留める。左列の上でピンチしても暴れない
            let anchor = self
                .view
                .x_to_time(pos.x.max(track_left), track_left, track_w);
            if (pinch - 1.0).abs() > 1e-3 {
                // ピンチ。**trackpad はこれが本命**
                self.view = self.view.zoom_at(anchor, 1.0 / pinch, comp);
            } else if command && smooth.y != 0.0 {
                // **カーソルの下の時刻は動かない。** それがズームの手触りそのもの
                self.view = self
                    .view
                    .zoom_at(anchor, 0.9_f32.powf(smooth.y / 50.0), comp);
            } else if shift && scroll.y != 0.0 {
                // マウスの Shift + ホイール。横しか無いので、そのまま横へ
                self.view = self.view.pan(-self.seconds_for(scroll.y, track_w), comp);
            } else {
                // **二本指はどちらの軸も同時に効く。** 以前は「x が少しでも動いたら
                // 横パンだけ」にしていたので、縦へ振ったつもりの僅かな横ブレで
                // 面が滑っていた(素直でない、の正体)。指の動きをそのまま2軸へ渡す
                if scroll.x != 0.0 {
                    self.view = self.view.pan(-self.seconds_for(scroll.x, track_w), comp);
                }
                if scroll.y != 0.0 {
                    self.scroll_y =
                        clamp_scroll(self.scroll_y - scroll.y, content_h, rows_view.height());
                }
            }
        }

        // ---- 縦のつまみ ----
        // 中身が面より高いときだけ出る。**掴んで動かせる**
        if content_h > rows_view.height() {
            let track_rect = Rect::from_min_max(
                egui::pos2(full.right() - SCROLLBAR_W, rows_view.top()),
                rows_view.max,
            );
            let ratio = rows_view.height() / content_h;
            let knob_h = (track_rect.height() * ratio).max(24.0);
            let travel = track_rect.height() - knob_h;
            let at = self.scroll_y / (content_h - rows_view.height());
            let knob = Rect::from_min_size(
                egui::pos2(track_rect.left(), track_rect.top() + travel * at),
                Vec2::new(SCROLLBAR_W, knob_h),
            );
            let r = ui.interact(track_rect, ui.id().with("vscroll"), Sense::click_and_drag());
            if r.dragged() && travel > 0.0 {
                let per_px = (content_h - rows_view.height()) / travel;
                self.scroll_y = clamp_scroll(
                    self.scroll_y + r.drag_delta().y * per_px,
                    content_h,
                    rows_view.height(),
                );
            }
            p.rect_filled(
                track_rect,
                CornerRadius::ZERO,
                Color32::from_rgb(0x22, 0x22, 0x22),
            );
            p.rect_filled(
                knob,
                CornerRadius::same(2),
                if r.hovered() || r.dragged() {
                    ACCENT
                } else {
                    Color32::from_rgb(0x55, 0x55, 0x55)
                },
            );
        }

        // ---- 時間のナビゲータ帯 ----
        self.navigator(ui, &p, full, track_left, comp);

        // playhead は行を描き終えてから、面の上に1本。
        //
        // **時間面の中だけに描く。** 窓の左端より前の時刻に居ると x が
        // レールの中へ入るので、クリップしないと**レイヤー名の列を縦に貫く**。
        // 時刻を持たない列に時刻の線が出るのは、面の意味が壊れて見える
        let time_p = p.with_clip_rect(Rect::from_min_max(
            egui::pos2(track_left, ruler.top()),
            egui::pos2(full.right(), rows_view.bottom()),
        ));
        // メモの縦線。**薄く、面の上だけ**(印そのものはルーラに置く)
        for t in &time_line_hints {
            let x = self.view.time_to_x(*t, track_left, track_w);
            time_p.line_segment(
                [
                    egui::pos2(x, rows_view.top()),
                    egui::pos2(x, rows_view.bottom()),
                ],
                Stroke::new(1.0, Color32::from_rgb(0x4a, 0x44, 0x33)),
            );
        }

        // ループの境目を面の上まで伸ばす。**どこで折り返すかが行の上で分かる**
        if self.loop_region.on {
            for t in [self.loop_region.start, self.loop_region.end] {
                let x = self.view.time_to_x(t, track_left, track_w);
                time_p.line_segment(
                    [
                        egui::pos2(x, ruler.top() + LOOP_H),
                        egui::pos2(x, rows_view.bottom()),
                    ],
                    Stroke::new(1.0, Color32::from_rgb(0x6b, 0x60, 0x3a)),
                );
            }
        }
        // 矩形選択の枠
        if let Some(Hold::Marquee { from, to }) = self.hold {
            let rect = Rect::from_two_pos(from, to);
            time_p.rect_filled(
                rect,
                CornerRadius::ZERO,
                Color32::from_rgba_unmultiplied(0xe9, 0xcf, 0x72, 24),
            );
            time_p.rect_stroke(
                rect,
                CornerRadius::ZERO,
                Stroke::new(1.0, ACCENT),
                StrokeKind::Inside,
            );
        }

        let playhead_x = self.view.time_to_x(self.playhead, track_left, track_w);
        time_p.line_segment(
            [
                egui::pos2(playhead_x, ruler.top()),
                egui::pos2(playhead_x, rows_view.bottom()),
            ],
            Stroke::new(1.0, Color32::from_rgb(0xe8, 0xe8, 0xe8)),
        );
        time_p.add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(playhead_x - 5.0, ruler.top()),
                egui::pos2(playhead_x + 5.0, ruler.top()),
                egui::pos2(playhead_x, ruler.top() + 7.0),
            ],
            Color32::from_rgb(0xe8, 0xe8, 0xe8),
            Stroke::NONE,
        ));

        if let Some((layer, additive, range)) = pick {
            self.select(layer, additive, range, &object_layers);
            // **行を選んだらキーの選択は落とす。** Delete の対象が2つあると読めない
            self.selected_keys.clear();
        }

        for (layer, is_children) in toggles {
            if is_children {
                if self.fold.children_are_open(layer) {
                    self.fold.close_children(layer);
                } else {
                    self.fold.open_children(layer);
                }
            } else if self.fold.params_are_open(layer) {
                self.fold.close_params(layer);
            } else {
                self.fold.open_params(layer);
            }
        }

        for (layer, flag) in flags {
            // 親から受けているロックは、自分の L を触っても外れない。**黙らせない**
            if flag == Flag::Lock
                && effective_lock(&self.document, layer)
                && !item_flags(&self.document, layer)
                    .map(|(_, _, l)| l)
                    .unwrap_or(false)
            {
                self.status = format!("{} is locked by a parent", self.name(layer));
                continue;
            }
            self.toggle_flag(layer, flag);
        }

        // **掴んでいるあいだ、時刻を指の近くに出す。** status 行まで目を運ばせない
        if self.hold.is_some() || scrub.is_pointer_button_down_on() {
            if let Some(pos) = ctx.input(|i| i.pointer.latest_pos()) {
                let at = self.view.x_to_time(pos.x, track_left, track_w);
                let label = format!("{}", timecode(at.max(0.0), fps));
                let anchor = egui::pos2(pos.x + 12.0, pos.y - 18.0);
                let size = Vec2::new(74.0, 16.0);
                let box_rect = Rect::from_min_size(anchor, size);
                p.rect_filled(
                    box_rect,
                    CornerRadius::same(2),
                    Color32::from_rgb(0x1c, 0x1c, 0x1c),
                );
                p.rect_stroke(
                    box_rect,
                    CornerRadius::same(2),
                    Stroke::new(1.0, Color32::from_rgb(0x55, 0x55, 0x55)),
                    StrokeKind::Inside,
                );
                p.text(
                    box_rect.center(),
                    Align2::CENTER_CENTER,
                    label,
                    FontId::monospace(9.0),
                    ACCENT,
                );
            }
        }

        // 名前の編集。**始める・確定する・やめる**を行の外で1回ずつ
        if let Some(layer) = rename_started {
            self.begin_rename(layer);
        }
        if rename_committed {
            self.commit_rename();
        }
        if rename_cancelled {
            self.renaming = None;
            self.status = "rename cancelled".to_owned();
        }

        // **掴んでいるあいだの手の形は、掴んだものが決める。** hover の答えより後に
        // 置いて上書きする — 途中でポインタが的から外れても形が変わらない
        if let Some(icon) = hold_cursor(&self.hold) {
            ctx.set_cursor_icon(icon);
        }

        // メニューから出た指示は、行を回し終えてから1つだけ実行する
        if let Some(action) = menu_action {
            self.run_menu(action, comp_seconds, fps);
        }

        // Undo / Redo。1ドラッグ = 1 GestureId なので、掴んで動かした分がまとめて戻る
        let (undo, redo, escape, duplicate, delete, group, split, select_all) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Z) && i.modifiers.command && !i.modifiers.shift,
                i.key_pressed(egui::Key::Z) && i.modifiers.command && i.modifiers.shift,
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::D) && i.modifiers.command,
                i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace),
                i.key_pressed(egui::Key::G) && i.modifiers.command,
                i.key_pressed(egui::Key::K) && i.modifiers.command,
                i.key_pressed(egui::Key::A) && i.modifiers.command,
            )
        });
        // **掴んでいる最中の Esc は、その gesture ごと取り消す。**
        // 掴んでいないときの Esc は何もしない — Undo の代わりではない
        if escape && self.hold.is_some() {
            self.cancel_drag();
        } else if undo {
            match self.writer.undo() {
                Ok(()) => {
                    refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
                    self.status = format!("undo  ({} left)", self.writer.undo_len());
                }
                Err(error) => self.status = format!("undo rejected: {error}"),
            }
        } else if redo {
            match self.writer.redo() {
                Ok(()) => {
                    refresh_if_stale(&self.writer, &mut self.document, &mut self.revision);
                    self.status = format!("redo  ({} left)", self.writer.redo_len());
                }
                Err(error) => self.status = format!("redo rejected: {error}"),
            }
        }

        // Cmd/Ctrl + D。**選択が無いときは何もしない** — 複製する対象が無い
        if duplicate {
            self.duplicate_selected();
        }

        // Delete / Backspace。**キーが選ばれていればキーが先**、無ければ層を消す
        // (Group は中身ごと。D2 の RemoveTrackItem)。
        // ドラッグ中は効かせない — 掴んだものが消えると gesture の行き先が無くなる
        if delete && self.hold.is_none() && !self.delete_selected_keys() {
            self.delete_selected();
        }

        // Cmd/Ctrl + G。**選択をひとつの Group にまとめる**
        if group && self.hold.is_none() {
            self.group_selected();
        }

        // Cmd/Ctrl + K。**playhead で切る**
        if split && self.hold.is_none() {
            self.split_selected();
        }

        // `M`。**playhead にメモを置く**(置いた直後から書ける)
        if ctx.input(|i| i.key_pressed(egui::Key::M) && !i.modifiers.command)
            && self.renaming.is_none()
            && self.editing_locator.is_none()
        {
            self.add_locator(self.playhead);
        }

        // Enter。**1つだけ選んでいるときに名前を編集する**(AE と同じ)
        if ctx.input(|i| i.key_pressed(egui::Key::Enter))
            && self.renaming.is_none()
            && self.selected.len() == 1
        {
            let layer = self.selected[0];
            self.begin_rename(layer);
        }

        // Cmd/Ctrl + A。**見えている行を全部選ぶ**(閉じた Group の中は見えていない)
        if select_all {
            self.selected = object_layers.clone();
            self.selected_keys.clear();
            self.status = format!("{} selected", self.selected.len());
        }
    }
}

/// **外で入った編集を、次のフレームで拾う。**
///
/// Browser が別の場所でシェイプを置いたとき、Timeline はその編集を自分では
/// 知らない。編集の出どころを問わず `writer.revision` だけを見る。
///
/// `snapshot()` は Document を丸ごと clone するので、**毎フレーム呼ばない** —
/// revision が進んだフレームだけ取り直す。取り直したら `true`。
fn refresh_if_stale(
    writer: &DocumentWriter,
    cached: &mut Arc<Document>,
    cached_revision: &mut u64,
) -> bool {
    if writer.revision == *cached_revision {
        return false;
    }
    *cached = writer.snapshot();
    *cached_revision = writer.revision;
    true
}

/// TimelineEditor の `ParamRef` を D2 の property セレクタへ。
fn scalar_property(param: ParamRef) -> motolii_doc::ScalarPropertyId {
    use motolii_doc::ScalarPropertyId as S;
    match param {
        ParamRef::Position => S::Position,
        ParamRef::Anchor => S::Anchor,
        ParamRef::Scale => S::Scale,
        ParamRef::Rotation => S::Rotation,
        ParamRef::Opacity => S::Opacity,
    }
}

fn grab_layer(grab: &Grab) -> LayerId {
    match grab {
        Grab::Move { layer, .. }
        | Grab::KeyTime { layer, .. }
        | Grab::TrimIn { layer }
        | Grab::TrimOut { layer }
        | Grab::Reorder { layer } => *layer,
    }
}

/// その layer を動かしたとき、実際に開始時刻が変わる clip を集める。
///
/// **Group 自身は clip ではないので、動くのは子孫の clip である。**
/// モックの「Group barを動かすと子barも同じ差分で追従する」をここで満たす。
fn movable_clips(document: &Document, layer: LayerId) -> Vec<(LayerId, f32, f32)> {
    fn collect(item: &TrackItem, out: &mut Vec<(LayerId, f32, f32)>) {
        match item {
            TrackItem::Clip(clip) => {
                let start = clip.start.as_seconds_f64() as f32;
                out.push((
                    clip.envelope.layer_id,
                    start,
                    start + clip.duration.as_seconds_f64() as f32,
                ));
            }
            TrackItem::Group(group) => {
                for child in &group.children {
                    collect(child, out);
                }
            }
        }
    }
    let mut out = Vec::new();
    if let Some(item) = find_item(document, layer) {
        collect(item, &mut out);
    }
    out
}

/// 吸着の間合い(px)。**画面の距離で決める** — 寄れば時間としては細かくなる
const SNAP_PX: f32 = 7.0;

/// 秒 → **最寄りのフレーム境界**の時刻。
///
/// 普通のタイムラインはフレーム単位で動く。clip の頭が 2.0334 秒に来ると、
/// 揃うべきキーが揃わなくなる。丸めは `motolii-core` の変換に任せ、
/// **ここで `(x * fps).round()` を自前で書かない**(fps は有理数で、
/// 30000/1001 のような値がある)。
fn seconds_to_time(seconds: f32, fps: Fps) -> Option<RationalTime> {
    let raw = RationalTime::try_new((f64::from(seconds) * 1000.0).round() as i64, 1000).ok()?;
    let frame = raw.try_to_frame_round(fps).ok()?;
    RationalTime::try_from_frame(frame, fps).ok()
}

fn param_label(p: ParamRef) -> &'static str {
    match p {
        ParamRef::Position => "Position",
        ParamRef::Anchor => "Anchor",
        ParamRef::Scale => "Scale",
        ParamRef::Rotation => "Rotation",
        ParamRef::Opacity => "Opacity",
    }
}

fn param_color(p: ParamRef) -> Color32 {
    match p {
        ParamRef::Position => Color32::from_rgb(0xcf, 0x75, 0x6d),
        ParamRef::Scale => Color32::from_rgb(0x75, 0xa9, 0x78),
        ParamRef::Rotation => Color32::from_rgb(0x77, 0x9b, 0xd0),
        ParamRef::Opacity => Color32::from_rgb(0xb1, 0x8e, 0xc0),
        ParamRef::Anchor => Color32::from_rgb(0xe1, 0xb8, 0x66),
    }
}

/// Document を歩いて、その layer の item を探す
fn find_item(document: &Document, layer: LayerId) -> Option<&TrackItem> {
    fn walk(items: &[TrackItem], layer: LayerId) -> Option<&TrackItem> {
        for item in items {
            let env = match item {
                TrackItem::Clip(c) => &c.envelope,
                TrackItem::Group(g) => &g.envelope,
            };
            if env.layer_id == layer {
                return Some(item);
            }
            if let TrackItem::Group(g) = item {
                if let Some(found) = walk(&g.children, layer) {
                    return Some(found);
                }
            }
        }
        None
    }
    document.tracks.iter().find_map(|t| walk(&t.items, layer))
}

fn visible_params(document: &Document, layer: LayerId) -> Vec<ParamRef> {
    find_item(document, layer)
        .map(crate::timeline_rows::keyed_params)
        .unwrap_or_default()
}

fn clip_span(document: &Document, layer: LayerId) -> Option<(f32, f32)> {
    match find_item(document, layer)? {
        TrackItem::Clip(c) => {
            let start = c.start.as_seconds_f64() as f32;
            Some((start, start + c.duration.as_seconds_f64() as f32))
        }
        // Group は子の範囲を包む
        TrackItem::Group(g) => {
            let mut span: Option<(f32, f32)> = None;
            for child in &g.children {
                if let TrackItem::Clip(c) = child {
                    let s = c.start.as_seconds_f64() as f32;
                    let e = s + c.duration.as_seconds_f64() as f32;
                    span = Some(match span {
                        Some((a, b)) => (a.min(s), b.max(e)),
                        None => (s, e),
                    });
                }
            }
            span
        }
    }
}

/// その layer の色。**選ばれていれば Document の値、無ければ id から導く。**
///
/// 導出を `LayerId` にするのは、**採番後に変わらず再利用もされない**唯一の値だから。
/// 行番号から導くと並べ替えた瞬間に総入れ替えになり、色記憶にならない。
/// ただし導出は複製で色が変わる(新しい id が付く)ので、**選んだ色は Document へ**
/// 置く — envelope ごと写るので複製にも付いていく。
fn layer_color(document: &Document, layer: LayerId) -> Color32 {
    let chosen = find_item(document, layer).and_then(|item| match item {
        TrackItem::Clip(c) => c.envelope.color,
        TrackItem::Group(g) => g.envelope.color,
    });
    match chosen {
        Some(rgb) => Color32::from_rgb(
            ((rgb >> 16) & 0xff) as u8,
            ((rgb >> 8) & 0xff) as u8,
            (rgb & 0xff) as u8,
        ),
        None => LAYER_COLORS[(layer.get() as usize) % LAYER_COLORS.len()],
    }
}

/// 掴める3つのフラグ。**押下状態は Document から読む**(ボタン側に状態を持たない)
#[derive(Debug, Clone, Copy, PartialEq)]
enum Flag {
    Mute,
    Solo,
    Lock,
}

/// **効いているロック。** 自分が掛かっているか、**親のどれかが掛かっていれば掛かる**。
///
/// Group を掛けたのに中が触れてしまうのは、`ItemEnvelope.lock` を各行で
/// 単体で読んでいたからである。ロックは「この枝には触るな」という意味なので、
/// 木を下りながら受け継ぐ。
fn effective_lock(document: &Document, layer: LayerId) -> bool {
    fn walk(items: &[TrackItem], layer: LayerId, inherited: bool) -> Option<bool> {
        for item in items {
            let env = match item {
                TrackItem::Clip(c) => &c.envelope,
                TrackItem::Group(g) => &g.envelope,
            };
            let locked = inherited || env.lock;
            if env.layer_id == layer {
                return Some(locked);
            }
            if let TrackItem::Group(group) = item {
                if let Some(found) = walk(&group.children, layer, locked) {
                    return Some(found);
                }
            }
        }
        None
    }
    document
        .tracks
        .iter()
        .find_map(|track| walk(&track.items, layer, false))
        .unwrap_or(false)
}

/// その layer の `visible` / `solo` / `lock`
fn item_flags(document: &Document, layer: LayerId) -> Option<(bool, bool, bool)> {
    let env = match find_item(document, layer)? {
        TrackItem::Clip(c) => &c.envelope,
        TrackItem::Group(g) => &g.envelope,
    };
    Some((env.visible, env.solo, env.lock))
}

/// キーを `(KeyframeId, 時刻(秒))` で返す。**掴んだ物を追い続けるには id が要る** —
/// 時刻は編集で変わるが `KeyframeId` は変わらない
fn param_keys(document: &Document, layer: LayerId, param: ParamRef) -> Vec<(KeyframeId, f32)> {
    let Some(item) = find_item(document, layer) else {
        return Vec::new();
    };
    let env = match item {
        TrackItem::Clip(c) => &c.envelope,
        TrackItem::Group(g) => &g.envelope,
    };
    let doc_param = match param {
        ParamRef::Position => &env.transform.position,
        ParamRef::Anchor => &env.transform.anchor,
        ParamRef::Scale => &env.transform.scale,
        ParamRef::Rotation => &env.transform.rotation,
        ParamRef::Opacity => &env.opacity,
    };
    match doc_param {
        DocParam::Keyframes(track) => track
            .keys()
            .iter()
            .map(|k| (k.id, k.t.as_seconds_f64() as f32))
            .collect(),
        _ => Vec::new(),
    }
}

// ---- fixture: Group ひとつ、子3枚、兄弟2枚 ----

fn time(ms: i64) -> RationalTime {
    RationalTime::try_new(ms, 1000).expect("fixture time")
}

// single-writer-exempt の理由は引数行に同居(guard は同一行コメントだけを honor する)。
fn keys_at(
    document: &mut Document, // single-writer-exempt: fixture が Document を所有している(writer より前)
    seconds: &[f64],
    v: DocValue,
) -> DocParam {
    let mut track = DocKeyframeTrack::new();
    for s in seconds {
        let id = KeyframeId::from_raw(document.next_stable_id.allocate().expect("key id"));
        track.insert(DocKeyframe {
            id,
            t: time((s * 1000.0) as i64),
            value: v.clone(),
            interp: Interp::Hold,
        });
    }
    DocParam::Keyframes(track)
}

fn make_clip(
    document: &mut Document, // single-writer-exempt: fixture が Document を所有している(writer より前)
    name: &str,
    start_s: f64,
    dur_s: f64,
    positions: &[f64],
    scales: &[f64],
) -> (TrackItem, LayerId, String) {
    let asset = document
        .assets
        .allocate(name, "video/mp4", &format!("{name}-hash"))
        .expect("asset");
    let layer = document.layers.allocate(name).expect("layer");
    let mut envelope = ItemEnvelope::new(layer);
    let mut transform = Transform2D::identity();
    if !positions.is_empty() {
        transform.position = keys_at(document, positions, DocValue::Vec2([0.0, 0.0]));
    }
    if !scales.is_empty() {
        transform.scale = keys_at(document, scales, DocValue::Vec2([1.0, 1.0]));
    }
    envelope.transform = transform;
    (
        TrackItem::Clip(Clip {
            envelope,
            start: time((start_s * 1000.0) as i64),
            duration: time((dur_s * 1000.0) as i64),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        }),
        layer,
        name.to_owned(),
    )
}

/// lab の fixture Document(層・Group・キー入り)。lab の起動と本 module のテスト、
/// shell 統合テスト(`tests/blitz_shell_editor_seat.rs`)が同じものを使う。
pub fn lab_fixture() -> (Document, HashMap<LayerId, String>) {
    let mut document = Document::new_current();
    // ルーラが 0:00〜0:16 なので、composition もそこまで伸ばす
    document.composition.duration = time(16_000);
    let track = document.track_ids.allocate("V1").expect("track");
    let mut names = HashMap::new();

    let (a, la, na) = make_clip(
        &mut document,
        "Shared left",
        0.6,
        6.0,
        &[1.2, 5.0],
        &[2.4, 5.8],
    );
    let (b, lb, nb) = make_clip(&mut document, "Reference text", 5.4, 6.8, &[6.4, 11.2], &[]);
    let (c, lc, nc) = make_clip(
        &mut document,
        "Shared right",
        11.0,
        4.2,
        &[11.4, 13.1, 14.7],
        &[],
    );
    names.insert(la, na);
    names.insert(lb, nb);
    names.insert(lc, nc);

    let group_layer = document.layers.allocate("Title scene").expect("layer");
    names.insert(group_layer, "Title scene".to_owned());
    let mut group_envelope = ItemEnvelope::new(group_layer);
    group_envelope.transform = Transform2D {
        position: keys_at(&mut document, &[2.8, 10.1], DocValue::Vec2([0.0, 0.0])),
        ..Transform2D::identity()
    };
    group_envelope.opacity = keys_at(&mut document, &[1.9, 6.9, 12.5], DocValue::F64(1.0));
    let group = TrackItem::Group(Group {
        envelope: group_envelope,
        children: vec![a, b, c],
    });

    let (bg, lbg, nbg) = make_clip(&mut document, "Background", 0.0, 13.4, &[], &[]);
    let (audio, laudio, naudio) = make_clip(&mut document, "starter-tone.wav", 1.1, 12.5, &[], &[]);
    names.insert(lbg, nbg);
    names.insert(laudio, naudio);

    document.tracks.push(Track {
        id: track,
        items: vec![group, bg, audio],
    });
    document.validate().expect("valid fixture");
    (document, names)
}

fn write_bmp(path: &str, image: &egui::ColorImage) {
    let (w, h) = (image.size[0] as u32, image.size[1] as u32);
    let row_bytes = (w * 3).next_multiple_of(4);
    let mut out = Vec::with_capacity(54 + (row_bytes * h) as usize);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(54 + row_bytes * h).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(w as i32).to_le_bytes());
    out.extend_from_slice(&(h as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    for _ in 0..6 {
        out.extend_from_slice(&0u32.to_le_bytes());
    }
    for y in (0..h).rev() {
        let mut written = 0;
        for x in 0..w {
            let c = image.pixels[(y * w + x) as usize];
            out.push(c.b());
            out.push(c.g());
            out.push(c.r());
            written += 3;
        }
        while written < row_bytes {
            out.push(0);
            written += 1;
        }
    }
    std::fs::write(path, out).expect("write bmp");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer_named(names: &HashMap<LayerId, String>, want: &str) -> LayerId {
        *names
            .iter()
            .find(|(_, n)| n.as_str() == want)
            .map(|(l, _)| l)
            .expect("fixture layer")
    }

    /// マウス無しで、ドラッグが出すのと同じ command を通す。
    /// **手で触る前に、編集が Document へ届くことをここで確かめる。**
    #[test]
    fn dragging_a_clip_moves_it_in_the_document_and_undo_puts_it_back() {
        let (doc, names) = lab_fixture();
        let catalog =
            Arc::new(motolii_plugin::reference::reference_catalog().expect("reference catalog"));
        let mut writer = DocumentWriter::new(doc, catalog).expect("writer");
        let fps = writer.snapshot().composition.fps;
        let layer = layer_named(&names, "Background");

        let before = clip_span(&writer.snapshot(), layer).expect("span");
        assert_eq!(before.0, 0.0, "fixture の Background は 0s 始まり");

        let gesture = writer.begin_gesture();
        let command = writer
            .prepare_set_clip_start(layer, seconds_to_time(2.0, fps).expect("time"))
            .expect("prepare")
            .expect("変化があるので command が出る");
        writer.apply_command(gesture, command).expect("apply");

        let after = clip_span(&writer.snapshot(), layer).expect("span");
        assert!((after.0 - 2.0).abs() < 1e-3, "clip が動いた: {after:?}");
        assert_eq!(
            after.1 - after.0,
            before.1 - before.0,
            "move は尺を変えない"
        );

        writer.undo().expect("undo");
        let restored = clip_span(&writer.snapshot(), layer).expect("span");
        assert!((restored.0 - before.0).abs() < 1e-3, "1ドラッグ = 1 Undo");
    }

    /// **Group を掴むと、子が同じ差分で動く。** モックで決めた挙動。
    #[test]
    fn dragging_a_group_moves_every_child_by_the_same_delta() {
        let (doc, names) = lab_fixture();
        let catalog =
            Arc::new(motolii_plugin::reference::reference_catalog().expect("reference catalog"));
        let mut writer = DocumentWriter::new(doc, catalog).expect("writer");
        let fps = writer.snapshot().composition.fps;
        let group = layer_named(&names, "Title scene");

        let targets = movable_clips(&writer.snapshot(), group);
        assert_eq!(targets.len(), 3, "Group の子 clip は3枚");

        // Shared right が 15.2s で終わり、composition は 16s。
        // **塊で動ける余地は 0.8s しかない** — それを超える delta は頭打ちになる
        let delta = 0.5_f32;
        let gesture = writer.begin_gesture();
        for (layer, start, _) in &targets {
            let command = writer
                .prepare_set_clip_start(*layer, seconds_to_time(start + delta, fps).expect("time"))
                .expect("prepare")
                .expect("command");
            writer.apply_command(gesture, command).expect("apply");
        }

        let after = writer.snapshot();
        for (layer, start, _) in &targets {
            let moved = clip_span(&after, *layer).expect("span").0;
            assert!(
                (moved - (start + delta)).abs() < 1e-3,
                "子は同じ差分で動く: {moved} vs {}",
                start + delta
            );
        }

        // Group 行の bar は子を包むので、まとめて動いたように見える
        let group_span = clip_span(&after, group).expect("group span");
        assert!((group_span.0 - (0.6 + delta)).abs() < 1e-3);

        writer.undo().expect("undo");
        let restored = writer.snapshot();
        for (layer, start, _) in &targets {
            assert!((clip_span(&restored, *layer).expect("span").0 - start).abs() < 1e-3);
        }
    }

    /// **clip が動いたら、その Position キーも一緒に動く。**
    /// Scale は時刻を変える `prepare_*` が無いので置いていかれる — それも押さえる。
    #[test]
    fn moving_a_clip_carries_its_position_keys() {
        let mut lab = TimelineEditor::with_fixture();
        let layer = layer_named(&lab.names, "Shared left");

        let start_before = clip_span(&lab.document, layer).expect("span").0;
        let position_before = param_keys(&lab.document, layer, ParamRef::Position);
        let scale_before = param_keys(&lab.document, layer, ParamRef::Scale);
        assert_eq!(
            position_before.len(),
            2,
            "fixture の Shared left は Position キー2つ"
        );
        assert_eq!(scale_before.len(), 2, "Scale も2つ。こちらも追従する");

        lab.hold_item(begin_move(&lab.document, layer, 3.0));
        lab.commit_drag(4.0); // +1.0s

        let after = lab.writer.snapshot();
        assert!(
            (clip_span(&after, layer).expect("span").0 - (start_before + 1.0)).abs() < 1e-3,
            "clip が +1.0s 動いた"
        );
        let position_after = param_keys(&after, layer, ParamRef::Position);
        for (key, before) in &position_before {
            let now = position_after
                .iter()
                .find(|(id, _)| id == key)
                .expect("KeyframeId は時刻編集で不変")
                .1;
            assert!(
                (now - (before + 1.0)).abs() < 1e-3,
                "Position キーが clip に追従する: {now} vs {}",
                before + 1.0
            );
        }
        // Scale キーも同じ delta で追従する(2026-08-16 に D2 の入口ができた)。
        // **守るべきは「時刻が動く」ことではなく「KeyframeId が変わらない」こと** —
        // remove+add で代用すると id が変わり、Undo と同一性が壊れる
        let scale_after = param_keys(&after, layer, ParamRef::Scale);
        assert_eq!(
            scale_after.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            scale_before.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            "KeyframeId は時刻編集で不変。remove+add で代用しない"
        );
        for ((_, before), (_, now)) in scale_before.iter().zip(scale_after.iter()) {
            assert!(
                (now - (before + 1.0)).abs() < 1e-3,
                "Scale キーも clip に追従する: {now} vs {}",
                before + 1.0
            );
        }

        lab.writer.undo().expect("undo");
        let restored = lab.writer.snapshot();
        assert!(
            (clip_span(&restored, layer).expect("span").0 - start_before).abs() < 1e-3,
            "1ドラッグ = 1 Undo"
        );
        assert_eq!(
            param_keys(&restored, layer, ParamRef::Position),
            position_before,
            "キーも同じ Undo でまとめて戻る"
        );
    }

    /// **キーを掴んで動かすと、そのキーだけが動く。** clip も他のキーも巻き込まない。
    #[test]
    fn dragging_a_position_key_changes_only_that_key() {
        let mut lab = TimelineEditor::with_fixture();
        let layer = layer_named(&lab.names, "Shared right");

        let before = param_keys(&lab.document, layer, ParamRef::Position);
        assert_eq!(
            before.len(),
            3,
            "fixture の Shared right は Position キー3つ"
        );
        let start_before = clip_span(&lab.document, layer).expect("span").0;
        let (key, t0) = before[0];

        lab.hold_item(Grab::KeyTime {
            layer,
            param: ParamRef::Position,
            key,
            grab_at: t0,
            original: t0,
        });
        lab.commit_drag(t0 + 1.0);

        let after = param_keys(&lab.writer.snapshot(), layer, ParamRef::Position);
        let moved = after.iter().find(|(id, _)| *id == key).expect("key").1;
        assert!(
            (moved - (t0 + 1.0)).abs() < 1e-3,
            "掴んだキーだけ +1.0s: {moved}"
        );
        for (id, t) in &before[1..] {
            let now = after.iter().find(|(i, _)| i == id).expect("key").1;
            assert_eq!(now, *t, "他のキーは動かない");
        }
        assert!(
            (clip_span(&lab.writer.snapshot(), layer).expect("span").0 - start_before).abs() < 1e-3,
            "clip.start は変わらない"
        );
    }

    /// **M / S / L は Document を書き換える。** 枠と文字だけではない。
    #[test]
    fn muting_a_layer_writes_through_to_the_document() {
        let mut lab = TimelineEditor::with_fixture();
        let layer = layer_named(&lab.names, "Background");
        assert_eq!(
            item_flags(&lab.document, layer),
            Some((true, false, false)),
            "既定は表示・非solo・非ロック"
        );

        lab.toggle_flag(layer, Flag::Mute);
        assert_eq!(
            item_flags(&lab.document, layer),
            Some((false, false, false)),
            "M で visible=false"
        );

        lab.toggle_flag(layer, Flag::Solo);
        assert_eq!(
            item_flags(&lab.document, layer),
            Some((false, true, false)),
            "S で solo=true"
        );

        lab.writer.undo().expect("undo");
        lab.document = lab.writer.snapshot();
        assert_eq!(
            item_flags(&lab.document, layer),
            Some((false, false, false)),
            "1クリック = 1 Undo"
        );
        lab.writer.undo().expect("undo");
        lab.document = lab.writer.snapshot();
        assert_eq!(item_flags(&lab.document, layer), Some((true, false, false)));
    }

    /// **Scale のキーも clip について来る。** 2026-08-16 に D2 側の入口ができるまで
    /// できなかったこと。Position だけが動いて Scale が取り残される状態を潰す。
    #[test]
    fn moving_a_clip_carries_scale_keys_too_not_just_position() {
        let (doc, names) = lab_fixture();
        let catalog =
            Arc::new(motolii_plugin::reference::reference_catalog().expect("reference catalog"));
        let mut writer = DocumentWriter::new(doc, catalog).expect("writer");
        let fps = writer.snapshot().composition.fps;
        let layer = layer_named(&names, "Shared left");

        let scale_before = param_keys(&writer.snapshot(), layer, ParamRef::Scale);
        assert_eq!(
            scale_before.len(),
            2,
            "fixture の Shared left は Scale キー2つ"
        );

        let delta = 0.5_f32;
        let gesture = writer.begin_gesture();
        // clip 本体
        let start = clip_span(&writer.snapshot(), layer).expect("span").0;
        let command = writer
            .prepare_set_clip_start(layer, seconds_to_time(start + delta, fps).expect("time"))
            .expect("prepare")
            .expect("command");
        writer.apply_command(gesture, command).expect("apply");
        // Scale キー(後ろから出す。同時刻の占有を避ける)
        for (key, t) in scale_before.iter().rev() {
            let command = writer
                .prepare_set_transform_param_key_time(
                    layer,
                    motolii_doc::ScalarPropertyId::Scale,
                    *key,
                    seconds_to_time(t + delta, fps).expect("time"),
                )
                .expect("prepare")
                .expect("command");
            writer.apply_command(gesture, command).expect("apply");
        }

        let after = param_keys(&writer.snapshot(), layer, ParamRef::Scale);
        for ((_, before_t), (_, after_t)) in scale_before.iter().zip(after.iter()) {
            assert!(
                (after_t - (before_t + delta)).abs() < 1e-3,
                "Scale キーが追従する: {after_t} vs {}",
                before_t + delta
            );
        }

        writer.undo().expect("undo");
        let restored = param_keys(&writer.snapshot(), layer, ParamRef::Scale);
        for ((_, before_t), (_, back_t)) in scale_before.iter().zip(restored.iter()) {
            assert!((back_t - before_t).abs() < 1e-3, "1ドラッグ = 1 Undo");
        }
    }

    // ---- 以下は未実装。実装者はこれを通すこと ----

    const COMP: f32 = 16.0;

    fn view() -> TimelineView {
        TimelineView {
            start: 0.0,
            span: 16.0,
        }
    }

    #[test]
    fn zoom_keeps_the_anchor_time_under_the_cursor() {
        let v = view();
        let (left, w) = (100.0_f32, 900.0_f32);
        let anchor = 6.0_f32;
        let x_before = v.time_to_x(anchor, left, w);

        let zoomed = v.zoom_at(anchor, 0.5, COMP);
        let x_after = zoomed.time_to_x(anchor, left, w);

        assert!(
            (x_before - x_after).abs() < 0.5,
            "カーソルの下の時刻は動かない: {x_before} vs {x_after}"
        );
        assert!((zoomed.span - 8.0).abs() < 1e-3, "span は半分になる");
    }

    #[test]
    fn zoom_clamps_at_the_whole_composition_and_at_a_quarter_second() {
        let out = view().zoom_at(8.0, 100.0, COMP);
        assert!(
            (out.span - COMP).abs() < 1e-3,
            "composition より広く引けない"
        );
        assert!(out.start.abs() < 1e-3);

        let mut deep = view();
        for _ in 0..20 {
            deep = deep.zoom_at(8.0, 0.5, COMP);
        }
        assert!((deep.span - MIN_SPAN).abs() < 1e-3, "0.25秒より寄れない");
    }

    #[test]
    fn pan_cannot_scroll_before_zero_or_past_the_end() {
        let v = view().zoom_at(8.0, 0.25, COMP); // span 4s
        assert!((v.span - 4.0).abs() < 1e-3);

        let left = v.pan(-999.0, COMP);
        assert!(left.start.abs() < 1e-3, "0より前へは行かない");

        let right = v.pan(999.0, COMP);
        assert!(
            (right.start - (COMP - 4.0)).abs() < 1e-3,
            "終端より後ろは見せない: {}",
            right.start
        );
    }

    #[test]
    fn x_to_time_is_the_inverse_of_time_to_x() {
        let v = TimelineView {
            start: 3.5,
            span: 4.0,
        };
        let (left, w) = (196.0_f32, 800.0_f32);
        for t in [3.5_f32, 4.0, 5.25, 7.5] {
            let back = v.x_to_time(v.time_to_x(t, left, w), left, w);
            assert!((back - t).abs() < 1e-3, "{t} -> {back}");
        }
    }

    #[test]
    fn a_document_edit_made_elsewhere_shows_up_on_the_next_frame() {
        // Browser がシェイプを置いたときに Timeline がすぐ出す、の最小形。
        // **TimelineEditor が自分で編集していないのに、行が増えること**を見る
        let (doc, names) = lab_fixture();
        let catalog =
            Arc::new(motolii_plugin::reference::reference_catalog().expect("reference catalog"));
        let mut writer = DocumentWriter::new(doc, catalog).expect("writer");

        let mut cached = writer.snapshot();
        let mut cached_revision = writer.revision;
        let rows_before = rows(&cached, &TimelineFoldState::default()).len();

        // TimelineEditor の外で編集する(ここでは M を落とすだけ。行数は変えない編集でも revision は進む)
        let layer = layer_named(&names, "Background");
        let gesture = writer.begin_gesture();
        let command = writer
            .prepare_set_item_visible(layer, false)
            .expect("prepare")
            .expect("command");
        writer.apply_command(gesture, command).expect("apply");

        assert_ne!(writer.revision, cached_revision, "編集で revision が進む");

        // 次のフレームで拾う
        let refreshed = refresh_if_stale(&writer, &mut cached, &mut cached_revision);
        assert!(refreshed, "revision が変わったら取り直す");
        assert_eq!(
            rows(&cached, &TimelineFoldState::default()).len(),
            rows_before
        );
        assert!(
            !item_flags(&cached, layer).expect("flags").0,
            "外からの編集が cached snapshot へ反映されている"
        );

        // 変化が無ければ取り直さない(毎フレーム clone しない)
        assert!(!refresh_if_stale(
            &writer,
            &mut cached,
            &mut cached_revision
        ));
    }

    /// **Esc は掴む前へ戻す。** ドラッグは毎フレーム出し直すが、1ドラッグ =
    /// 1 `GestureId` なので、undo 1回でその gesture の編集がまとめて消える。
    #[test]
    fn escape_during_a_drag_restores_the_original_start() {
        let mut lab = TimelineEditor::with_fixture();
        let layer = layer_named(&lab.names, "Background");
        let before = clip_span(&lab.document, layer).expect("span").0;

        let undo_base = lab.writer.undo_len();
        lab.hold_item(begin_move(&lab.document, layer, 1.0));
        // 実際のドラッグと同じく、動かしている間に何度も出す
        lab.commit_drag(2.0);
        lab.commit_drag(3.0);
        assert!(
            (clip_span(&lab.document, layer).expect("span").0 - (before + 2.0)).abs() < 1e-3,
            "掴んでいる間は動いている: {:?}",
            clip_span(&lab.document, layer)
        );

        lab.cancel_drag();

        assert!(lab.hold.is_none(), "取り消したら、もう掴んでいない");
        assert!(
            (clip_span(&lab.document, layer).expect("span").0 - before).abs() < 1e-3,
            "Esc で掴む前の開始時刻へ戻る: {:?}",
            clip_span(&lab.document, layer)
        );
        assert_eq!(
            lab.writer.undo_len(),
            undo_base,
            "取り消した gesture は履歴に残らない"
        );
        assert!(
            lab.status.contains("cancelled"),
            "status に出す: {}",
            lab.status
        );
    }

    /// **複製の深さは D2 が持っている。** Group を1つ渡すだけで、子3枚が
    /// 新しい `LayerId` を持って一緒に来る。TimelineEditor 側で子を辿って複製し直さない。
    #[test]
    fn duplicating_a_group_copies_its_children_with_fresh_ids() {
        let mut lab = TimelineEditor::with_fixture();
        let group = layer_named(&lab.names, "Title scene");

        let items_before = lab.document.tracks[0].items.len();
        let children_before: Vec<LayerId> = movable_clips(&lab.document, group)
            .into_iter()
            .map(|(layer, _, _)| layer)
            .collect();
        assert_eq!(children_before.len(), 3, "fixture の Title scene は子3枚");

        lab.selected = vec![group];
        lab.duplicate_selected();

        // (a) TrackItem が1つ増えている
        assert_eq!(
            lab.document.tracks[0].items.len(),
            items_before + 1,
            "複製が1つ増える: {}",
            lab.status
        );
        assert!(
            lab.status.contains("Title scene"),
            "status に複製した名前を出す: {}",
            lab.status
        );

        let copy = lab.document.tracks[0]
            .items
            .iter()
            .filter_map(|item| match item {
                TrackItem::Group(g) => Some(g.envelope.layer_id),
                _ => None,
            })
            .find(|layer| *layer != group)
            .expect("複製された Group");
        let children_after: Vec<LayerId> = movable_clips(&lab.document, copy)
            .into_iter()
            .map(|(layer, _, _)| layer)
            .collect();

        // (c) 子の枚数は3枚のまま。D2 が再帰して写している
        assert_eq!(children_after.len(), 3, "子も一緒に複製される");
        // (b) 複製された子の LayerId は元と全部違う
        for layer in &children_after {
            assert!(
                !children_before.contains(layer),
                "複製された子は新しい LayerId: {layer:?} が元と重なる"
            );
        }

        // (d) Undo で元へ戻る。1複製 = 1 GestureId
        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(
            lab.document.tracks[0].items.len(),
            items_before,
            "1複製 = 1 Undo"
        );
        assert_eq!(
            movable_clips(&lab.document, group)
                .into_iter()
                .map(|(layer, _, _)| layer)
                .collect::<Vec<_>>(),
            children_before,
            "元の Group は触られていない"
        );
    }

    /// **ドラッグの結果はフレーム境界に乗る。** 半端な位置に置けると、
    /// 揃うべきキーが揃わなくなる。
    #[test]
    fn dragging_lands_on_a_frame_boundary() {
        let (doc, names) = lab_fixture();
        let catalog =
            Arc::new(motolii_plugin::reference::reference_catalog().expect("reference catalog"));
        let mut writer = DocumentWriter::new(doc, catalog).expect("writer");
        let fps = writer.snapshot().composition.fps;
        let layer = layer_named(&names, "Background");

        // フレームの途中に相当する秒数(30fps なら 1/30 = 0.0333.. の非整数倍)
        let ragged = 2.0334_f32;
        let snapped = seconds_to_time(ragged, fps).expect("time");

        let frame = snapped.try_to_frame_round(fps).expect("frame");
        assert_eq!(
            snapped,
            RationalTime::try_from_frame(frame, fps).expect("time"),
            "丸めた時刻はフレーム境界と往復で一致する"
        );

        // 実際に適用しても境界のまま
        let gesture = writer.begin_gesture();
        let command = writer
            .prepare_set_clip_start(layer, snapped)
            .expect("prepare")
            .expect("command");
        writer.apply_command(gesture, command).expect("apply");

        let after = clip_span(&writer.snapshot(), layer).expect("span").0;
        let back = seconds_to_time(after, fps).expect("time");
        assert_eq!(back, snapped, "適用後も境界に乗ったまま: {after}");
    }

    // ---- 見えている行の並び。並べ替えと範囲選択が数える単位 ----
    fn objects_of(lab: &TimelineEditor) -> Vec<LayerId> {
        rows(&lab.document, &lab.fold)
            .into_iter()
            .filter(|r| matches!(r.kind, RowKind::Object))
            .map(|r| r.layer)
            .collect()
    }

    fn location(lab: &TimelineEditor, layer: LayerId) -> (ParentLocator, usize) {
        let (parent, index, _) = find_item_location(&lab.document, layer).expect("location");
        (parent, index)
    }

    /// **面より短い中身はスクロールしない。下は最後の行で止まる。**
    #[test]
    fn scrolling_stops_at_the_top_and_at_the_last_row() {
        // 中身(200) が面(300) より低い: どちらへ回しても 0
        assert_eq!(clamp_scroll(-40.0, 200.0, 300.0), 0.0);
        assert_eq!(clamp_scroll(80.0, 200.0, 300.0), 0.0);
        // 中身(500) が面(300) より高い: 下限は 0、上限は差の 200
        assert_eq!(clamp_scroll(-1.0, 500.0, 300.0), 0.0);
        assert_eq!(clamp_scroll(120.0, 500.0, 300.0), 120.0);
        assert_eq!(clamp_scroll(999.0, 500.0, 300.0), 200.0);

        // 行の合計高は object 24 / property 20 の実寸から出る
        let lab = TimelineEditor::with_fixture();
        let visible = rows(&lab.document, &lab.fold);
        let objects = visible
            .iter()
            .filter(|r| matches!(r.kind, RowKind::Object))
            .count() as f32;
        let props = visible.len() as f32 - objects;
        assert_eq!(
            content_height(&visible, ROW_H, PROP_H),
            objects * ROW_H + props * PROP_H
        );
        assert!(
            content_height(&visible, ROW_H_LARGE, PROP_H_LARGE)
                > content_height(&visible, ROW_H, PROP_H),
            "大きい行のほうが高い"
        );
    }

    /// **選ばれているものは、掴んだ1つだけでなく全部が同じ差分で動く。**
    #[test]
    fn moving_one_of_several_selected_clips_moves_them_all() {
        let mut lab = TimelineEditor::with_fixture();
        let background = layer_named(&lab.names, "Background");
        let tone = layer_named(&lab.names, "starter-tone.wav");
        let untouched = layer_named(&lab.names, "Shared left");

        let before = |lab: &TimelineEditor, l| clip_span(&lab.document, l).expect("span").0;
        let (bg0, tone0, other0) = (
            before(&lab, background),
            before(&lab, tone),
            before(&lab, untouched),
        );

        lab.selected = vec![background, tone];
        let roots = lab.selection_roots();
        lab.hold_item(begin_move_many(&lab.document, &roots, background, 3.0));
        lab.commit_drag(3.5); // +0.5s

        assert!(
            (before(&lab, background) - (bg0 + 0.5)).abs() < 1e-3,
            "掴んだほうが動く"
        );
        assert!(
            (before(&lab, tone) - (tone0 + 0.5)).abs() < 1e-3,
            "**もう一方も同じ差分で動く**"
        );
        assert!(
            (before(&lab, untouched) - other0).abs() < 1e-3,
            "選んでいないものは動かない"
        );

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert!(
            (before(&lab, background) - bg0).abs() < 1e-3,
            "1ドラッグ = 1 Undo"
        );
        assert!((before(&lab, tone) - tone0).abs() < 1e-3);
    }

    /// 親 Group と子を同時に選んでも、**動くのは1回分の差分**である。
    #[test]
    fn selecting_a_group_and_its_child_moves_the_child_once() {
        let mut lab = TimelineEditor::with_fixture();
        let group = layer_named(&lab.names, "Title scene");
        let child = layer_named(&lab.names, "Shared left");
        let start0 = clip_span(&lab.document, child).expect("span").0;

        lab.selected = vec![group, child];
        assert_eq!(
            lab.selection_roots(),
            vec![group],
            "子は親の子孫なので構造操作からは外れる"
        );

        let roots = lab.selection_roots();
        let Grab::Move { targets, .. } = begin_move_many(&lab.document, &roots, group, 0.0) else {
            panic!("Move を掴んだはず");
        };
        assert_eq!(targets.len(), 3, "同じ clip を2度数えない");

        lab.hold_item(begin_move_many(&lab.document, &roots, group, 1.0));
        lab.commit_drag(1.3); // +0.3s
        assert!(
            (clip_span(&lab.document, child).expect("span").0 - (start0 + 0.3)).abs() < 1e-3,
            "子は 0.3s だけ動く(0.6s ではない)"
        );
    }

    /// **Shift クリックは、見えている行の上で範囲を採る。**
    #[test]
    fn shift_click_selects_the_range_between_two_rows() {
        let mut lab = TimelineEditor::with_fixture();
        let objects = objects_of(&lab);
        assert!(objects.len() >= 4, "fixture の行数");

        lab.select(objects[0], false, false, &objects);
        assert_eq!(lab.selected, vec![objects[0]]);

        lab.select(objects[2], false, true, &objects);
        assert_eq!(lab.selected.len(), 3, "0..2 の3行");
        for layer in &objects[0..=2] {
            assert!(lab.is_selected(*layer));
        }
        assert_eq!(
            lab.selected.last(),
            Some(&objects[0]),
            "起点は末尾に残る。続けて Shift を押しても基準が動かない"
        );

        // Cmd は足し引き
        lab.select(objects[3], true, false, &objects);
        assert!(lab.is_selected(objects[3]));
        lab.select(objects[3], true, false, &objects);
        assert!(
            !lab.is_selected(objects[3]),
            "同じ行をもう一度 Cmd クリックで外れる"
        );

        // 素のクリックは1つに戻す
        lab.select(objects[1], false, false, &objects);
        assert_eq!(lab.selected, vec![objects[1]]);
    }

    /// **行を上へ落とすと、Document の並びが変わる。** 時刻は変わらない。
    #[test]
    fn dropping_a_row_above_another_reorders_the_document() {
        let mut lab = TimelineEditor::with_fixture();
        let objects = objects_of(&lab);
        let background = layer_named(&lab.names, "Background");
        let start_before = clip_span(&lab.document, background).expect("span").0;
        assert_eq!(location(&lab, background).1, 1, "はじめは Group の次");

        // 境界0 = いちばん上の行の上
        let (parent, index) =
            drop_target(&lab.document, &objects, 0, background).expect("落とせる");
        lab.commit_reorder(
            background,
            DropTarget {
                parent,
                index,
                y: 0.0,
            },
        );

        assert_eq!(location(&lab, background), (parent, 0), "先頭へ来た");
        assert!(
            (clip_span(&lab.document, background).expect("span").0 - start_before).abs() < 1e-3,
            "**並べ替えは時刻を動かさない**"
        );

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(location(&lab, background).1, 1, "1回の Undo で戻る");
    }

    /// 下へ動かすときは**外したあとの位置**になる。1つずれるのを埋める。
    #[test]
    fn dropping_a_row_at_the_end_lands_after_the_last_one() {
        let mut lab = TimelineEditor::with_fixture();
        let objects = objects_of(&lab);
        let background = layer_named(&lab.names, "Background");
        let tone = layer_named(&lab.names, "starter-tone.wav");
        assert_eq!(location(&lab, tone).1, 2, "はじめは最後");

        let (parent, index) =
            drop_target(&lab.document, &objects, objects.len(), background).expect("落とせる");
        assert_eq!(index, 2, "3つのうち自分を外したので、末尾は 2 である");
        lab.commit_reorder(
            background,
            DropTarget {
                parent,
                index,
                y: 0.0,
            },
        );

        assert_eq!(location(&lab, background).1, 2, "最後へ来た");
        assert_eq!(location(&lab, tone).1, 1, "追い越された側は1つ上がる");
    }

    /// **開いた Group の中へも落とせる。** 出し入れは同じ1本の command で表す。
    #[test]
    fn dropping_a_row_into_an_open_group_reparents_it() {
        let mut lab = TimelineEditor::with_fixture();
        let objects = objects_of(&lab);
        let group = layer_named(&lab.names, "Title scene");
        let background = layer_named(&lab.names, "Background");
        assert_eq!(objects[0], group, "先頭は Group で、子が開いている");

        // 境界1 = Group の最初の子の上 = 「Group の中の先頭」
        let (parent, index) =
            drop_target(&lab.document, &objects, 1, background).expect("落とせる");
        assert_eq!(parent, ParentLocator::Group(group));
        lab.commit_reorder(
            background,
            DropTarget {
                parent,
                index,
                y: 0.0,
            },
        );

        assert_eq!(location(&lab, background), (ParentLocator::Group(group), 0));
        assert_eq!(
            movable_clips(&lab.document, group).len(),
            4,
            "Group を動かすと、入れたものも一緒に動く"
        );

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(
            location(&lab, background),
            (ParentLocator::Track(lab.document.tracks[0].id), 1),
            "1回の Undo で元の親へ戻る"
        );
    }

    /// **自分の中へは落とせない。** Group を自分の子の中へ入れると木が壊れる。
    #[test]
    fn a_group_cannot_be_dropped_inside_itself() {
        let lab = TimelineEditor::with_fixture();
        let objects = objects_of(&lab);
        let group = layer_named(&lab.names, "Title scene");

        for boundary in 1..=3 {
            assert!(
                drop_target(&lab.document, &objects, boundary, group).is_none(),
                "境界 {boundary} は Group の中なので落とせない"
            );
        }
        // 子の外(いちばん上・いちばん下)へは動かせる
        assert!(drop_target(&lab.document, &objects, 0, group).is_some());
        assert!(drop_target(&lab.document, &objects, objects.len(), group).is_some());
    }

    /// **Delete は Group を中身ごと消し、1回の Undo で全員が戻る。**
    #[test]
    fn deleting_a_group_takes_its_children_and_one_undo_puts_them_back() {
        let mut lab = TimelineEditor::with_fixture();
        let group = layer_named(&lab.names, "Title scene");
        let child = layer_named(&lab.names, "Shared left");
        let items_before = lab.document.tracks[0].items.len();

        lab.selected = vec![group, child]; // 親と子を同時に選んでも壊れない
        lab.delete_selected();

        assert_eq!(
            lab.document.tracks[0].items.len(),
            items_before - 1,
            "status: {}",
            lab.status
        );
        assert!(find_item(&lab.document, child).is_none(), "子も消える");
        assert!(lab.selected.is_empty(), "消したものを選んだままにしない");

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(lab.document.tracks[0].items.len(), items_before);
        assert!(
            find_item(&lab.document, child).is_some(),
            "**同じ LayerId で戻る**。id を振り直さない"
        );
        assert_eq!(lab.name(child), "Shared left", "表示名も戻る");
    }

    /// 複数選んで消すのも**1回の Undo 単位**である。
    #[test]
    fn deleting_two_selected_layers_is_one_undo() {
        let mut lab = TimelineEditor::with_fixture();
        let background = layer_named(&lab.names, "Background");
        let tone = layer_named(&lab.names, "starter-tone.wav");
        let undo_before = lab.writer.undo_len();

        lab.selected = vec![background, tone];
        lab.delete_selected();
        assert_eq!(lab.document.tracks[0].items.len(), 1);
        assert_eq!(
            lab.writer.undo_len(),
            undo_before + 1,
            "1 gesture にまとまる"
        );

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(lab.document.tracks[0].items.len(), 3, "2枚とも戻る");
    }

    /// 複数選んだ Cmd+D は**選んだ数だけ増え、増えたほうが選ばれる**。
    #[test]
    fn duplicating_two_selected_layers_makes_two_and_selects_them() {
        let mut lab = TimelineEditor::with_fixture();
        let background = layer_named(&lab.names, "Background");
        let tone = layer_named(&lab.names, "starter-tone.wav");
        let items_before = lab.document.tracks[0].items.len();

        lab.selected = vec![background, tone];
        lab.duplicate_selected();

        assert_eq!(lab.document.tracks[0].items.len(), items_before + 2);
        assert_eq!(lab.selected.len(), 2, "増えたほうが選ばれている");
        assert!(
            !lab.selected.contains(&background) && !lab.selected.contains(&tone),
            "選ばれているのは元ではなく複製: {:?}",
            lab.selected
        );

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(
            lab.document.tracks[0].items.len(),
            items_before,
            "2枚まとめて1回の Undo で戻る"
        );
    }

    /// **目盛は時刻に貼り付く。** 窓を N 等分すると、パンしても線が動かない。
    #[test]
    fn ticks_are_multiples_of_the_step_not_divisions_of_the_window() {
        let fps = Fps::try_new(30, 1).expect("fps");
        let view = TimelineView {
            start: 3.3,
            span: 16.0,
        };
        let step = tick_step(view.span, fps);
        let list = ticks(view, fps);

        assert!(list.len() >= 6, "窓に何本か入る: {list:?}");
        for t in &list {
            let n = t / step;
            assert!(
                (n - n.round()).abs() < 1e-3,
                "目盛は {step} の倍数である: {t}"
            );
        }
        // パンすると、目盛は同じ倍数の列のまま**時刻ごと動く**
        let panned = ticks(
            TimelineView {
                start: 5.3,
                span: 16.0,
            },
            fps,
        );
        assert_ne!(list, panned, "窓が動けば見える目盛も変わる");
        for t in &panned {
            let n = t / step;
            assert!((n - n.round()).abs() < 1e-3);
        }
        // 0 より前は出さない
        assert!(ticks(
            TimelineView {
                start: 0.0,
                span: 16.0
            },
            fps
        )
        .iter()
        .all(|t| *t >= 0.0));
    }

    /// 寄るほど細かい目盛になり、**最後はフレームの倍数**になる。
    #[test]
    fn tick_step_gets_finer_as_you_zoom_in() {
        let fps = Fps::try_new(30, 1).expect("fps");
        let frame = 1.0 / 30.0;

        let wide = tick_step(600.0, fps);
        let mid = tick_step(16.0, fps);
        let close = tick_step(MIN_SPAN, fps);
        assert!(wide > mid && mid > close, "{wide} > {mid} > {close}");
        assert!(
            close >= frame - 1e-6,
            "1フレームより細かくはしない: {close}"
        );
        let n = close / frame;
        assert!(
            (n - n.round()).abs() < 1e-3,
            "寄ったらフレームの倍数: {close}"
        );

        // 文字は間隔より細かい桁を出さない
        assert_eq!(tick_label(64.5, 1.0), "1:04.5");
        assert_eq!(tick_label(64.5, frame), "1:04.50");
    }

    /// **Space の再生は終端で止まる。** 巻き戻さない。
    ///
    /// `advance_playhead` は **soundtrack が無い project の壁時計 fallback** である
    /// (soundtrack がある project の playhead は audio clock に同期して進む —
    /// `audio_seat::follow_audio_clock` のテストがそちらの正本)。
    #[test]
    fn wall_clock_fallback_advances_and_stops_at_the_end() {
        let (at, playing) = advance_playhead(0.0, 0.5, 16.0);
        assert!((at - 0.5).abs() < 1e-6);
        assert!(playing);

        let (at, playing) = advance_playhead(15.9, 0.5, 16.0);
        assert_eq!(at, 16.0, "終端を越えない");
        assert!(!playing, "終端で止まる");

        // 止まったフレームで dt が来ても進まない
        let (at, playing) = advance_playhead(16.0, 0.016, 16.0);
        assert_eq!(at, 16.0);
        assert!(!playing);
    }

    /// **細目盛は主目盛を割り切り、1フレームより細かくならない。**
    #[test]
    fn minor_ticks_divide_the_labelled_ones_and_stop_at_a_frame() {
        let fps = Fps::try_new(30, 1).expect("fps");
        let frame = 1.0 / 30.0;

        for span in [600.0_f32, 120.0, 16.0, 4.0, 1.0, MIN_SPAN] {
            let major = tick_step(span, fps);
            let Some(minor) = minor_step(major, fps) else {
                assert!(
                    major <= frame * 2.0,
                    "細目盛を消していいのはフレームまで寄ったときだけ: major={major}"
                );
                continue;
            };
            assert!(minor >= frame * 0.999, "1フレームより細かくしない: {minor}");
            assert!(minor < major, "主目盛より細かい: {minor} < {major}");
            let n = major / minor;
            assert!(
                (n - n.round()).abs() < 1e-3,
                "主目盛を割り切る: {major} / {minor}"
            );
        }
    }

    /// **再生中は面が流れ、playhead は窓の中央に居続ける。**
    /// 頭と終端だけは窓が止まり、playhead のほうが窓の中を動く。
    ///
    /// 2026-08-16: 相対位置保持・ページ送りを経て、**利用者の指定でこの形に戻した**。
    #[test]
    fn playback_keeps_the_playhead_centred_and_stops_scrolling_at_both_ends() {
        let comp = 16.0_f32;
        let span = 4.0_f32;
        let centred = |playhead: f32| {
            TimelineView {
                start: playhead - span * 0.5,
                span,
            }
            .clamped(comp)
        };

        // 真ん中あたり: playhead は窓の中央
        let view = centred(8.0);
        assert!((view.start - 6.0).abs() < 1e-4);
        assert!(((8.0 - view.start) / view.span - 0.5).abs() < 1e-4);

        // 頭: 窓は0より前へ行かない = playhead は窓の左寄りに居る
        let view = centred(0.5);
        assert_eq!(view.start, 0.0);
        assert!((0.5 - view.start) / view.span < 0.5);

        // 終端: 窓はcompより後ろへ行かない = playhead は窓の右寄りに居る
        let view = centred(15.5);
        assert!((view.start - (comp - span)).abs() < 1e-4);
        assert!((15.5 - view.start) / view.span > 0.5);
    }

    /// 窓が隠れていた分を**まとめて進めない**(壁時計 fallback の規律。
    /// audio clock 側は供給済みサンプル数が正本なので、この上限は要らない)。
    #[test]
    fn a_long_frame_does_not_teleport_the_playhead() {
        // 800ms 止まっていた次のフレーム
        let (at, playing) = advance_playhead(2.0, (0.8_f32).min(MAX_STEP), 16.0);
        assert!(playing);
        assert!(
            (at - 2.05).abs() < 1e-6,
            "1フレームで進むのは MAX_STEP まで: {at}"
        );
    }

    /// **縞は時刻に貼り付く。** 窓を動かしても入れ替わらない。
    ///
    /// 窓の中で何本目かで濃淡を決めると、パンした瞬間に全部の縞が反転して
    /// 画面が沸く。0秒から数えた区間の偶奇なら、窓の位置は式に入らない。
    #[test]
    fn the_bands_are_nailed_to_time_not_to_the_window() {
        assert!(!band_is_dark(0.0, 1.0), "0–1s は明");
        assert!(band_is_dark(1.0, 1.0), "1–2s は暗");
        assert!(!band_is_dark(2.0, 1.0));

        // 目盛が2秒粒になっても、偶奇は0秒から数える
        assert!(!band_is_dark(0.0, 2.0));
        assert!(band_is_dark(2.0, 2.0));
        assert!(!band_is_dark(4.0, 2.0));

        // **窓の位置は式に入らない** — どの窓から見ても同じ時刻は同じ濃さ
        let fps = Fps::try_new(30, 1).expect("fps");
        let step = tick_step(16.0, fps);
        let shade = band_is_dark(4.0, step);
        for start in [0.0_f32, 0.7, 3.3, 9.9] {
            let view = TimelineView { start, span: 16.0 };
            if let Some(t) = ticks(view, fps)
                .into_iter()
                .find(|t| (*t - 4.0).abs() < 1e-3)
            {
                assert_eq!(band_is_dark(t, step), shade, "start={start} で反転した");
            }
        }
    }

    /// **右から左へ引いても同じ区間になる。** フレームに乗り、最短は1フレーム。
    #[test]
    fn a_loop_drag_reads_the_same_in_either_direction() {
        let fps = Fps::try_new(30, 1).expect("fps");
        let comp = 16.0_f32;

        let forward = loop_from_drag(2.0, 6.0, comp, fps);
        let backward = loop_from_drag(6.0, 2.0, comp, fps);
        assert_eq!(forward, backward, "掴んだ順序を持たない");
        assert!((forward.0 - 2.0).abs() < 1e-3 && (forward.1 - 6.0).abs() < 1e-3);

        // 端は必ずフレーム境界
        let (start, end) = loop_from_drag(2.0334, 5.9876, comp, fps);
        for t in [start, end] {
            let snapped = seconds_to_time(t, fps).expect("time").as_seconds_f64() as f32;
            assert!((t - snapped).abs() < 1e-4, "フレームに乗る: {t}");
        }

        // 点を突いただけでも長さ0にはしない(**止まる場所しか作らない区間を作らない**)
        let (start, end) = loop_from_drag(4.0, 4.0, comp, fps);
        assert!(end > start, "最短でも1フレーム: {start}–{end}");
        assert!((end - start - 1.0 / 30.0).abs() < 1e-3);

        // composition の外へは出ない
        let (start, end) = loop_from_drag(-3.0, 99.0, comp, fps);
        assert!(start >= 0.0 && end <= comp, "{start}–{end}");
    }

    /// **判定は「お尻を越えたか」だけ。** 入り口へ引き戻さない。
    #[test]
    fn playback_wraps_when_it_reaches_the_loop_end_not_when_it_starts_outside() {
        let region = LoopRegion {
            start: 2.0,
            end: 6.0,
            on: true,
        };

        // 区間の中: お尻に来るまで何もしない
        assert_eq!(wrap_playhead(2.9, 3.0, region), 3.0);
        assert!(
            (wrap_playhead(5.99, 6.02, region) - 2.02).abs() < 1e-4,
            "行き過ぎた分が残る"
        );
        assert!(
            (wrap_playhead(5.99, 6.0, region) - 2.0).abs() < 1e-4,
            "お尻ちょうどで頭へ"
        );

        // **区間より前から始めても引き戻さない。** そこまで通しで流れる
        assert_eq!(wrap_playhead(0.0, 0.5, region), 0.5, "入り口へ強制しない");
        assert_eq!(wrap_playhead(1.9, 1.95, region), 1.95);
        assert!(
            (wrap_playhead(5.9, 6.01, region) - 2.01).abs() < 1e-4,
            "前から入っても、お尻では折り返す"
        );

        // **区間より後ろから始めたら一度も折り返さない。** 越える瞬間が来ない
        assert_eq!(wrap_playhead(9.0, 9.1, region), 9.1);
        assert_eq!(wrap_playhead(6.0, 6.1, region), 6.1);

        // 1フレームで何周ぶんも飛んでいても剰余で収まる
        assert!((wrap_playhead(5.9, 15.0, region) - 3.0).abs() < 1e-4);

        // 切れているなら折り返さない
        let off = LoopRegion {
            on: false,
            ..region
        };
        assert_eq!(wrap_playhead(5.9, 9.0, off), 9.0);
    }

    /// **端まで運んだときだけ窓が流れる。** 面の中では動かない。
    #[test]
    fn dragging_to_the_edge_scrolls_and_the_middle_does_not() {
        let (track_left, track_w, span, dt) = (196.0_f32, 800.0_f32, 8.0_f32, 1.0 / 60.0);

        assert_eq!(
            edge_pan_seconds(track_left + track_w * 0.5, track_left, track_w, span, dt),
            0.0,
            "真ん中では動かない"
        );

        let right = edge_pan_seconds(track_left + track_w - 2.0, track_left, track_w, span, dt);
        assert!(right > 0.0, "右端では先へ流れる: {right}");
        let left = edge_pan_seconds(track_left + 2.0, track_left, track_w, span, dt);
        assert!(left < 0.0, "左端では戻る: {left}");

        // 端に近いほど速い
        let near = edge_pan_seconds(track_left + track_w - 20.0, track_left, track_w, span, dt);
        assert!(right > near && near > 0.0, "{right} > {near}");

        // 窓の外まで出しても、1フレームで窓の幅を超えて飛ばない
        let far = edge_pan_seconds(track_left + track_w + 200.0, track_left, track_w, span, dt);
        assert!(far < span * 0.5, "1フレームで飛びすぎない: {far}");
    }

    /// **お尻を掴んだつもりが新規作成になる、が一番効く事故。**
    ///
    /// 端の判定を外すと `New` に落ち、そこまで作った区間が消える。
    /// だから端は甘く見る。
    #[test]
    fn the_ends_of_a_loop_are_grabbable_and_a_near_miss_is_not_a_new_region() {
        let region = LoopRegion {
            start: 2.0,
            end: 6.0,
            on: true,
        };
        let (x0, x1) = (300.0_f32, 700.0_f32);

        // 端そのもの
        assert_eq!(
            loop_grab_for(x0, x0, x1, 2.0, region),
            LoopGrab::In { fixed: 6.0 },
            "頭を掴む"
        );
        assert_eq!(
            loop_grab_for(x1, x0, x1, 6.0, region),
            LoopGrab::Out { fixed: 2.0 },
            "お尻を掴む"
        );
        // 少し外しても端のまま(**新規作成に落ちない**)
        for dx in [-LOOP_GRAB, -3.0, 3.0, LOOP_GRAB] {
            assert_eq!(
                loop_grab_for(x1 + dx, x0, x1, 6.0, region),
                LoopGrab::Out { fixed: 2.0 },
                "お尻から {dx}px は掴めたまま"
            );
        }
        // 内側は移動、完全に外は新規
        assert!(matches!(
            loop_grab_for((x0 + x1) * 0.5, x0, x1, 4.0, region),
            LoopGrab::Move { .. }
        ));
        assert!(matches!(
            loop_grab_for(x1 + 40.0, x0, x1, 9.0, region),
            LoopGrab::New { .. }
        ));
    }

    /// **端を掴んで反対側を追い越しても、区間が畳まれない。**
    ///
    /// 反対側を毎フレーム `loop_region` から読み直すと、追い越した瞬間に
    /// 長さ0 → 最短1フレームへ潰れ、戻しても復元しない。
    #[test]
    fn dragging_an_end_past_the_other_keeps_the_region_anchored() {
        let fps = Fps::try_new(30, 1).expect("fps");
        let comp = 16.0_f32;
        let region = LoopRegion {
            start: 2.0,
            end: 6.0,
            on: true,
        };

        // お尻を頭より前まで引く: 固定した頭(2.0)との区間になる
        let LoopGrab::Out { fixed } = loop_grab_for(700.0, 300.0, 700.0, 6.0, region) else {
            panic!("お尻を掴んだはず");
        };
        let (start, end) = loop_from_drag(fixed, 1.0, comp, fps);
        assert!(
            (start - 1.0).abs() < 1e-3 && (end - 2.0).abs() < 1e-3,
            "{start}–{end}"
        );

        // そのまま戻せば元の長さに戻る(潰れていない)
        let (start, end) = loop_from_drag(fixed, 6.0, comp, fps);
        assert!(
            (start - 2.0).abs() < 1e-3 && (end - 6.0).abs() < 1e-3,
            "{start}–{end}"
        );
    }

    /// **Group を動かすと、Group 自身のキーも一緒に動く。**
    ///
    /// 2026-08-16: 「Group は `clip.start` を持たないので動いた事実が Document に
    /// 無い」として保留していた点。利用者裁定で追従させる — プリコンポを掴んだら
    /// 中身ごと動く、が期待される挙動である。
    #[test]
    fn moving_a_group_carries_the_groups_own_keys_too() {
        let mut lab = TimelineEditor::with_fixture();
        let group = layer_named(&lab.names, "Title scene");
        let child = layer_named(&lab.names, "Shared left");

        let group_position = param_keys(&lab.document, group, ParamRef::Position);
        let group_opacity = param_keys(&lab.document, group, ParamRef::Opacity);
        let child_start = clip_span(&lab.document, child).expect("span").0;
        assert_eq!(
            group_position.len(),
            2,
            "fixture の Group は Position キー2つ"
        );
        assert_eq!(group_opacity.len(), 3, "Opacity キー3つ");

        lab.selected = vec![group];
        let roots = lab.selection_roots();
        lab.hold_item(begin_move_many(&lab.document, &roots, group, 1.0));
        lab.commit_drag(1.4); // +0.4s

        let after = lab.writer.snapshot();
        assert!(
            (clip_span(&after, child).expect("span").0 - (child_start + 0.4)).abs() < 1e-3,
            "子は動く(今までどおり)"
        );
        for (param, before) in [
            (ParamRef::Position, &group_position),
            (ParamRef::Opacity, &group_opacity),
        ] {
            let now = param_keys(&after, group, param);
            assert_eq!(
                now.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
                before.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
                "KeyframeId は変わらない"
            );
            for ((_, was), (_, is)) in before.iter().zip(now.iter()) {
                assert!(
                    (is - (was + 0.4)).abs() < 1e-3,
                    "**Group 自身の {} キーも追従する**: {is} vs {}",
                    param_label(param),
                    was + 0.4
                );
            }
        }

        lab.writer.undo().expect("undo");
        let restored = lab.writer.snapshot();
        assert_eq!(
            param_keys(&restored, group, ParamRef::Opacity),
            group_opacity,
            "1ドラッグ = 1 Undo"
        );
    }

    /// **選択をひとつの Group にまとめる。** 1回の Undo で元の並びへ戻る。
    #[test]
    fn grouping_two_layers_puts_them_under_one_new_group() {
        let mut lab = TimelineEditor::with_fixture();
        let background = layer_named(&lab.names, "Background");
        let tone = layer_named(&lab.names, "starter-tone.wav");
        let items_before = lab.document.tracks[0].items.len();

        lab.selected = vec![background, tone];
        lab.group_selected();

        // Group 1つに置き換わる(2枚が中へ入るので、トップレベルは1つ減る)
        assert_eq!(
            lab.document.tracks[0].items.len(),
            items_before - 1,
            "status: {}",
            lab.status
        );
        let group = lab.selected[0];
        assert_eq!(lab.selected.len(), 1, "まとめた Group が選ばれている");
        let (parent, _, _) = find_item_location(&lab.document, background).expect("location");
        assert_eq!(parent, ParentLocator::Group(group), "中へ入った");
        assert_eq!(
            movable_clips(&lab.document, group).len(),
            2,
            "Group を動かせば2枚とも動く"
        );
        assert!(lab.fold.children_are_open(group), "中が見えている");

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(
            lab.document.tracks[0].items.len(),
            items_before,
            "**1回の Undo で元の並びへ**(Group を置く + 入れる が1 gesture)"
        );
        assert!(
            find_item(&lab.document, group).is_none(),
            "Group ごと消える"
        );
    }

    /// 親が揃っていない選択は**まとめない**。位置が言えなくなる。
    #[test]
    fn grouping_refuses_a_selection_that_spans_parents() {
        let mut lab = TimelineEditor::with_fixture();
        let inside = layer_named(&lab.names, "Shared left"); // Group の中
        let outside = layer_named(&lab.names, "Background"); // トップレベル
        let items_before = lab.document.tracks[0].items.len();

        lab.selected = vec![outside, inside];
        lab.group_selected();

        assert_eq!(
            lab.document.tracks[0].items.len(),
            items_before,
            "何も起きない"
        );
        assert!(
            lab.status.contains("same parent"),
            "理由を出す: {}",
            lab.status
        );
    }

    /// **キーが選ばれていれば、Delete はキーを消す。** 層は消えない。
    #[test]
    fn delete_removes_the_selected_keys_and_leaves_the_layer() {
        let mut lab = TimelineEditor::with_fixture();
        let layer = layer_named(&lab.names, "Shared right");
        let before = param_keys(&lab.document, layer, ParamRef::Position);
        assert_eq!(before.len(), 3);

        lab.selected = vec![layer];
        lab.selected_keys = vec![(layer, ParamRef::Position, before[1].0)];
        assert!(lab.delete_selected_keys(), "キーの削除が効いた");

        let now = param_keys(&lab.document, layer, ParamRef::Position);
        assert_eq!(now.len(), 2, "1つ消えた");
        assert!(
            !now.iter().any(|(id, _)| *id == before[1].0),
            "消えたのは選んだキー"
        );
        assert!(
            find_item(&lab.document, layer).is_some(),
            "**層は消えない**"
        );
        assert!(
            lab.selected_keys.is_empty(),
            "消したものを選んだままにしない"
        );

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(
            param_keys(&lab.document, layer, ParamRef::Position).len(),
            3,
            "1回の Undo で戻る"
        );

        // キーを選んでいなければ、Delete は層のほうへ回る
        lab.selected_keys.clear();
        assert!(!lab.delete_selected_keys(), "キーが無いなら何もしない");
    }

    /// **playhead で切ると、clip が2枚になる。** 端では切れない(断りであって失敗ではない)。
    #[test]
    fn splitting_at_the_playhead_makes_two_clips() {
        let mut lab = TimelineEditor::with_fixture();
        let layer = layer_named(&lab.names, "Background");
        let (start, end) = clip_span(&lab.document, layer).expect("span");
        let items_before = lab.document.tracks[0].items.len();

        lab.selected = vec![layer];
        lab.playhead = (start + end) * 0.5;
        lab.split_selected();

        assert_eq!(
            lab.document.tracks[0].items.len(),
            items_before + 1,
            "1枚増える: {}",
            lab.status
        );
        let left = clip_span(&lab.document, layer).expect("span");
        assert!((left.0 - start).abs() < 1e-3 && (left.1 - lab.playhead).abs() < 1e-3);

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(
            lab.document.tracks[0].items.len(),
            items_before,
            "1回の Undo で戻る"
        );

        // 端では切れない。**何も起きないが status で言う**
        lab.playhead = start;
        lab.split_selected();
        assert_eq!(lab.document.tracks[0].items.len(), items_before);
        assert!(lab.status.contains("nothing to split"), "{}", lab.status);
    }

    /// **playhead へキーを打てる。** 既にあるなら打たずにそれを選ぶ。
    #[test]
    fn adding_a_key_at_the_playhead_puts_one_key_and_selects_it() {
        let mut lab = TimelineEditor::with_fixture();
        let layer = layer_named(&lab.names, "Shared left");
        let before = param_keys(&lab.document, layer, ParamRef::Scale);
        assert_eq!(before.len(), 2);

        lab.playhead = 3.0;
        lab.add_key_at_playhead(layer, ParamRef::Scale);

        let now = param_keys(&lab.document, layer, ParamRef::Scale);
        assert_eq!(now.len(), 3, "1つ増えた: {}", lab.status);
        assert!(
            now.iter().any(|(_, t)| (t - 3.0).abs() < 1e-3),
            "playhead の時刻にある"
        );
        assert_eq!(lab.selected_keys.len(), 1, "打ったキーが選ばれている");
        assert!(
            lab.fold.params_are_open(layer),
            "キー行が開く(打った物が見える)"
        );

        // 2回目は増えない。**同じ時刻には1つしか置かない**
        lab.add_key_at_playhead(layer, ParamRef::Scale);
        assert_eq!(
            param_keys(&lab.document, layer, ParamRef::Scale).len(),
            3,
            "{}",
            lab.status
        );

        // **id は使い回されない**(2026-08-16 に D2 側を直した点)
        let mut lab2 = TimelineEditor::with_fixture();
        let layer2 = layer_named(&lab2.names, "Shared left");
        lab2.playhead = 3.0;
        lab2.add_key_at_playhead(layer2, ParamRef::Scale);
        lab2.playhead = 4.0;
        lab2.add_key_at_playhead(layer2, ParamRef::Scale);
        let ids: Vec<_> = param_keys(&lab2.document, layer2, ParamRef::Scale)
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        let mut unique = ids.clone();
        unique.sort_by_key(|id| id.get());
        unique.dedup();
        assert_eq!(ids.len(), unique.len(), "KeyframeId が重複しない: {ids:?}");
    }

    /// **吸着は「近くの候補へ寄る」。** 間合いは画面の距離で決まる。
    #[test]
    fn dragging_snaps_to_nearby_edges_and_keys_but_not_to_itself() {
        let mut lab = TimelineEditor::with_fixture();
        let moving = layer_named(&lab.names, "Background");
        let neighbour = layer_named(&lab.names, "Shared left");
        let (target_start, _) = clip_span(&lab.document, neighbour).expect("span");
        let px_per_second = 100.0_f32; // 1秒=100px。SNAP_PX=7 なので 0.07s の間合い

        // 近ければ隣の clip の頭へ吸着する
        let near = target_start + 0.05;
        assert!(
            (lab.snapped(near, &[moving], px_per_second) - target_start).abs() < 1e-4,
            "隣の端へ寄る"
        );
        // 遠ければ動かない
        let far = target_start + 0.5;
        assert!((lab.snapped(far, &[moving], px_per_second) - far).abs() < 1e-4);

        // **自分自身へは吸着しない**(掴んでいる当人は候補から外す)。
        // 0秒始まりの clip では composition の頭(0.0)と区別が付かないので、
        // 途中から始まる clip で見る
        let own = layer_named(&lab.names, "starter-tone.wav");
        let (own_start, _) = clip_span(&lab.document, own).expect("span");
        let near_own = own_start + 0.02;
        assert!(
            (lab.snapped(near_own, &[own], px_per_second) - near_own).abs() < 1e-4,
            "自分の端には吸わない: {}",
            lab.snapped(near_own, &[own], px_per_second)
        );

        // playhead も候補
        lab.playhead = 9.0;
        assert!((lab.snapped(9.03, &[moving], px_per_second) - 9.0).abs() < 1e-4);

        // Alt 相当(切ってあるとき)は素通し
        lab.snap = false;
        assert!((lab.snapped(near, &[moving], px_per_second) - near).abs() < 1e-4);
    }

    /// **タイムコードはフレーム番号まで出す。** 秒だけだと乗っているか読めない。
    #[test]
    fn timecode_counts_frames_not_decimals() {
        let fps = Fps::try_new(30, 1).expect("fps");
        assert_eq!(timecode(0.0, fps), "0:00:00");
        assert_eq!(timecode(1.0, fps), "0:01:00");
        assert_eq!(timecode(1.0 + 6.0 / 30.0, fps), "0:01:06");
        assert_eq!(timecode(61.0, fps), "1:01:00");
        // 半端な秒は最寄りのフレームとして出る(表示が実体より細かくならない)
        assert_eq!(timecode(2.0334, fps), "0:02:01");
    }

    /// **ロックは触らせないだけ。** 評価にも描画にも影響しない(D2 の B④)。
    #[test]
    fn a_locked_layer_keeps_its_place_and_its_keys() {
        let mut lab = TimelineEditor::with_fixture();
        let locked = layer_named(&lab.names, "Background");
        let free = layer_named(&lab.names, "starter-tone.wav");

        lab.toggle_flag(locked, Flag::Lock);
        assert!(lab.is_locked(locked), "status: {}", lab.status);
        assert!(!lab.is_locked(free));

        // 消えない。**選んでいても構造操作から外れる**
        let items_before = lab.document.tracks[0].items.len();
        lab.selected = vec![locked, free];
        lab.delete_selected();
        assert!(
            find_item(&lab.document, locked).is_some(),
            "ロックは消えない"
        );
        assert!(
            find_item(&lab.document, free).is_none(),
            "ロックでないほうは消える"
        );
        assert_eq!(lab.document.tracks[0].items.len(), items_before - 1);

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);

        // 複製もされない
        let items_before = lab.document.tracks[0].items.len();
        lab.selected = vec![locked];
        lab.duplicate_selected();
        assert_eq!(
            lab.document.tracks[0].items.len(),
            items_before,
            "ロックは複製されない: {}",
            lab.status
        );

        // 切れない
        lab.playhead = 4.0;
        lab.split_selected();
        assert_eq!(
            lab.document.tracks[0].items.len(),
            items_before,
            "ロックは切れない"
        );

        // **外せば元どおり触れる**
        lab.toggle_flag(locked, Flag::Lock);
        assert!(!lab.is_locked(locked));
        lab.selected = vec![locked];
        lab.duplicate_selected();
        assert_eq!(
            lab.document.tracks[0].items.len(),
            items_before + 1,
            "外したら複製できる"
        );
    }

    /// M / S / L は**押下状態を Document から読む**。ボタン側に状態を持たない。
    #[test]
    fn the_three_flags_write_through_and_read_back() {
        let mut lab = TimelineEditor::with_fixture();
        let layer = layer_named(&lab.names, "Reference text");
        assert_eq!(item_flags(&lab.document, layer), Some((true, false, false)));

        for (flag, expect) in [
            (Flag::Mute, (false, false, false)),
            (Flag::Solo, (false, true, false)),
            (Flag::Lock, (false, true, true)),
        ] {
            lab.toggle_flag(layer, flag);
            assert_eq!(
                item_flags(&lab.document, layer),
                Some(expect),
                "{flag:?} を入れた後: {}",
                lab.status
            );
        }

        // 1クリック = 1 Undo
        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(item_flags(&lab.document, layer), Some((false, true, false)));
    }

    /// **名前を変えても、動くのは台帳だけ。** 行も、キーも、選択も動かない。
    #[test]
    fn renaming_a_layer_only_moves_the_ledger_entry() {
        let mut lab = TimelineEditor::with_fixture();
        let layer = layer_named(&lab.names, "Reference text");
        let rows_before = rows(&lab.document, &lab.fold).len();
        let span_before = clip_span(&lab.document, layer).expect("span");

        lab.begin_rename(layer);
        assert_eq!(
            lab.renaming.as_ref().map(|(l, n)| (*l, n.as_str())),
            Some((layer, "Reference text")),
            "**いま見えている名前が初期値**"
        );
        lab.renaming = Some((layer, "  Caption  ".to_owned()));
        lab.commit_rename();

        assert_eq!(
            lab.name(layer),
            "Caption",
            "前後の空白は落とす: {}",
            lab.status
        );
        assert_eq!(
            rows(&lab.document, &lab.fold).len(),
            rows_before,
            "行は増減しない"
        );
        assert_eq!(clip_span(&lab.document, layer).expect("span"), span_before);
        assert!(lab.renaming.is_none(), "編集は終わっている");

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(lab.name(layer), "Reference text", "1回の Undo で戻る");
    }

    /// **空の名前は断る。** 行が読めなくなるので、編集も終わらせない。
    #[test]
    fn an_empty_name_is_refused_and_keeps_the_editor_open() {
        let mut lab = TimelineEditor::with_fixture();
        let layer = layer_named(&lab.names, "Background");

        lab.begin_rename(layer);
        lab.renaming = Some((layer, "   ".to_owned()));
        lab.commit_rename();

        assert_eq!(lab.name(layer), "Background", "名前は変わらない");
        assert!(lab.status.contains("empty"), "理由を出す: {}", lab.status);
    }

    /// ロックされた行は名前も変えられない。**編集禁止は名前も含む。**
    #[test]
    fn a_locked_layer_cannot_be_renamed() {
        let mut lab = TimelineEditor::with_fixture();
        let layer = layer_named(&lab.names, "Background");
        lab.toggle_flag(layer, Flag::Lock);

        lab.begin_rename(layer);
        assert!(lab.renaming.is_none(), "編集が始まらない");
        assert!(lab.status.contains("locked"), "{}", lab.status);
    }

    /// **ロケータは指した所に置き、すぐ書ける。** 空にすると消える。
    #[test]
    fn a_locator_is_placed_at_the_playhead_and_empty_text_removes_it() {
        let mut lab = TimelineEditor::with_fixture();
        assert!(lab.document.locators.is_empty());

        lab.add_locator(4.0);
        assert_eq!(lab.document.locators.len(), 1, "{}", lab.status);
        assert!((lab.document.locators[0].t.as_seconds_f64() as f32 - 4.0).abs() < 1e-3);
        assert_eq!(
            lab.editing_locator.as_ref().map(|(i, _)| *i),
            Some(0),
            "**置いた直後から書ける**"
        );

        // 書く
        lab.editing_locator = Some((0, "  intro  ".to_owned()));
        lab.commit_locator_text();
        assert_eq!(lab.document.locators[0].text, "intro", "前後の空白は落とす");
        assert!(lab.editing_locator.is_none());

        // 空にすると消える
        lab.editing_locator = Some((0, "   ".to_owned()));
        lab.commit_locator_text();
        assert!(lab.document.locators.is_empty(), "{}", lab.status);

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(lab.document.locators.len(), 1, "1回の Undo で戻る");
        assert_eq!(lab.document.locators[0].text, "intro");
    }

    /// ロケータは**時刻と名前だけ**。ツリーにも選択にも触らない。
    #[test]
    fn locators_do_not_touch_the_tree() {
        let mut lab = TimelineEditor::with_fixture();
        let rows_before = rows(&lab.document, &lab.fold).len();
        let items_before = lab.document.tracks[0].items.len();
        let layer = layer_named(&lab.names, "Background");
        let span_before = clip_span(&lab.document, layer).expect("span");

        lab.add_locator(2.0);
        lab.editing_locator = Some((0, "verse".to_owned()));
        lab.commit_locator_text();
        lab.add_locator(7.0);
        lab.editing_locator = Some((1, "chorus".to_owned()));
        lab.commit_locator_text();

        assert_eq!(lab.document.locators.len(), 2);
        assert_eq!(
            rows(&lab.document, &lab.fold).len(),
            rows_before,
            "行は増えない"
        );
        assert_eq!(lab.document.tracks[0].items.len(), items_before);
        assert_eq!(clip_span(&lab.document, layer).expect("span"), span_before);

        // 外すのは index で。**保持順が宛先**なので、後ろを外しても前は動かない
        lab.remove_locator(1);
        assert_eq!(lab.document.locators.len(), 1);
        assert_eq!(lab.document.locators[0].text, "verse");
    }

    /// **色は選ぶまで Document に載らない。** 既定は id から導き、並べ替えでは動かない。
    #[test]
    fn a_layer_colour_is_derived_until_it_is_chosen() {
        let mut lab = TimelineEditor::with_fixture();
        let layer = layer_named(&lab.names, "Background");
        let other = layer_named(&lab.names, "starter-tone.wav");

        // 既定: Document には何も載っていない
        let stored = |lab: &TimelineEditor, l: LayerId| match find_item(&lab.document, l) {
            Some(TrackItem::Clip(c)) => c.envelope.color,
            Some(TrackItem::Group(g)) => g.envelope.color,
            None => None,
        };
        assert_eq!(stored(&lab, layer), None, "選ぶまでは載らない");
        let derived = layer_color(&lab.document, layer);
        assert_eq!(
            derived,
            LAYER_COLORS[(layer.get() as usize) % LAYER_COLORS.len()],
            "id から導く"
        );
        assert_ne!(
            layer_color(&lab.document, other),
            derived,
            "隣とは違う色になる(id が違うので)"
        );

        // **並べ替えても色は動かない** — 行番号ではなく id から来ているので
        let objects = objects_of(&lab);
        let (parent, index) = drop_target(&lab.document, &objects, 0, layer).expect("落とせる");
        lab.commit_reorder(
            layer,
            DropTarget {
                parent,
                index,
                y: 0.0,
            },
        );
        assert_eq!(
            layer_color(&lab.document, layer),
            derived,
            "並べ替えで変わらない"
        );

        // 選ぶと載る。**複製にも付いていく**(envelope ごと写るので)
        lab.selected = vec![layer];
        lab.set_color(layer, Some(0x5c8c6f));
        assert_eq!(stored(&lab, layer), Some(0x5c8c6f), "{}", lab.status);
        assert_eq!(
            layer_color(&lab.document, layer),
            Color32::from_rgb(0x5c, 0x8c, 0x6f)
        );

        lab.duplicate_selected();
        let copy = lab.selected[0];
        assert_eq!(
            stored(&lab, copy),
            Some(0x5c8c6f),
            "**複製は色を持って生まれる**"
        );

        // 既定へ戻す
        lab.selected = vec![layer];
        lab.set_color(layer, None);
        assert_eq!(stored(&lab, layer), None);
        assert_eq!(layer_color(&lab.document, layer), derived, "既定色へ返る");

        // 1回の Undo で戻る
        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(stored(&lab, layer), Some(0x5c8c6f));
    }

    /// **「色を出すか」は Document に入れない。** 窓の側の好みである。
    #[test]
    fn turning_colours_off_does_not_touch_the_document() {
        let mut lab = TimelineEditor::with_fixture();
        let layer = layer_named(&lab.names, "Background");
        lab.selected = vec![layer];
        lab.set_color(layer, Some(0x8c6b6b));
        let before = lab.document.clone();

        lab.run_menu(MenuAction::ToggleColors, 16.0, lab.document.composition.fps);
        assert!(!lab.colors_on, "{}", lab.status);
        assert_eq!(
            lab.document.as_ref(),
            before.as_ref(),
            "**他人の付けた色は消えない**。表示の好みが Document を書き換えない"
        );
    }

    /// **Group を掛けると中も掛かる。** ロックは「この枝に触るな」である。
    #[test]
    fn locking_a_group_locks_everything_inside_it() {
        let mut lab = TimelineEditor::with_fixture();
        let group = layer_named(&lab.names, "Title scene");
        let child = layer_named(&lab.names, "Shared left");
        let outside = layer_named(&lab.names, "Background");

        assert!(!lab.is_locked(child));
        lab.toggle_flag(group, Flag::Lock);

        assert!(lab.is_locked(group), "{}", lab.status);
        assert!(lab.is_locked(child), "**子も掛かる**");
        assert!(!lab.is_locked(outside), "外は掛からない");

        // 子は自分では掛けていない(押しても外せない状態)
        assert_eq!(
            item_flags(&lab.document, child).map(|(_, _, l)| l),
            Some(false),
            "自分の lock は false のまま — 受けているだけ"
        );

        // 子を消せない・動かせない
        let items_before = movable_clips(&lab.document, group).len();
        lab.selected = vec![child];
        lab.delete_selected();
        assert!(find_item(&lab.document, child).is_some(), "子は消えない");
        assert_eq!(movable_clips(&lab.document, group).len(), items_before);

        // 親を外せば子も触れる
        lab.toggle_flag(group, Flag::Lock);
        assert!(!lab.is_locked(child), "親を外せば子も自由");
    }

    /// **ロケータのドラッグは1回の Undo。**
    ///
    /// 掴んだ瞬間の gesture を離すまで使い回すからで、毎フレーム開き直すと
    /// フレーム数だけ Undo が積まれる(掴み物を1つに畳むまで、実際そうなっていた)。
    #[test]
    fn dragging_a_locator_is_one_undo_step() {
        let mut lab = TimelineEditor::with_fixture();
        lab.add_locator(3.0);
        lab.editing_locator = None;
        let undo_before = lab.writer.undo_len();

        // 掴んで、動かして、離す — 途中の適用は同じ gesture へ入る
        let gesture = lab.writer.begin_gesture();
        lab.hold = Some(Hold::Locator { index: 0, gesture });
        for at in [4.0_f32, 5.0, 6.0] {
            let time = seconds_to_time(at, lab.document.composition.fps).expect("time");
            let prepared = lab.writer.prepare_set_locator_time(0, time);
            assert!(lab.apply_in(gesture, "locator", prepared));
        }
        lab.hold = None;

        assert!(
            (lab.document.locators[0].t.as_seconds_f64() as f32 - 6.0).abs() < 1e-3,
            "最後の位置に居る"
        );
        assert_eq!(
            lab.writer.undo_len(),
            undo_before + 1,
            "**3回動かしても Undo は1つ**"
        );

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert!(
            (lab.document.locators[0].t.as_seconds_f64() as f32 - 3.0).abs() < 1e-3,
            "1回で掴む前へ戻る: {}",
            lab.document.locators[0].t.as_seconds_f64()
        );
    }

    /// **掴み物は1つの入れ物に入っている。** 「何か掴んでいるか」を1箇所で聞ける。
    #[test]
    fn everything_held_lives_in_one_place() {
        let mut lab = TimelineEditor::with_fixture();
        let layer = layer_named(&lab.names, "Background");
        assert!(lab.hold.is_none());

        lab.hold_item(begin_move(&lab.document, layer, 1.0));
        assert!(lab.hold.is_some(), "掴んでいる");
        assert!(lab.item_hold().is_some(), "編集の掴みとして取り出せる");

        // 編集以外の掴みは、編集としては取り出せない
        lab.hold = Some(Hold::Nav(NavGrab::Pan));
        assert!(lab.hold.is_some());
        assert!(lab.item_hold().is_none(), "ナビゲータは編集ではない");

        lab.hold = Some(Hold::Marquee {
            from: egui::pos2(0.0, 0.0),
            to: egui::pos2(10.0, 10.0),
        });
        assert!(lab.item_hold().is_none());

        lab.hold = None;
        assert!(lab.item_hold().is_none());
    }

    /// **Position 以外のキーも掴める。** D2 の入口ができた日から可能だったのに、
    /// 掴む側が Position 縛りのままだった(2026-08-17 に外した)。
    #[test]
    fn a_scale_key_can_be_dragged_like_a_position_one() {
        let mut lab = TimelineEditor::with_fixture();
        let layer = layer_named(&lab.names, "Shared left");
        let before = param_keys(&lab.document, layer, ParamRef::Scale);
        assert_eq!(before.len(), 2, "fixture の Shared left は Scale キー2つ");
        let (key, t0) = before[0];
        let others: Vec<_> = param_keys(&lab.document, layer, ParamRef::Position);
        let clip_before = clip_span(&lab.document, layer).expect("span");

        lab.hold_item(Grab::KeyTime {
            layer,
            param: ParamRef::Scale,
            key,
            grab_at: t0,
            original: t0,
        });
        lab.commit_drag(t0 + 0.5);

        let after = param_keys(&lab.document, layer, ParamRef::Scale);
        let moved = after
            .iter()
            .find(|(id, _)| *id == key)
            .expect("id は不変")
            .1;
        assert!(
            (moved - (t0 + 0.5)).abs() < 1e-3,
            "掴んだキーが動く: {moved}"
        );
        assert_eq!(
            after.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            before.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            "KeyframeId は変わらない"
        );
        assert_eq!(
            param_keys(&lab.document, layer, ParamRef::Position),
            others,
            "他のパラメータのキーは動かない"
        );
        assert_eq!(
            clip_span(&lab.document, layer).expect("span"),
            clip_before,
            "clip も動かない"
        );

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        assert_eq!(
            param_keys(&lab.document, layer, ParamRef::Scale),
            before,
            "1回で戻る"
        );
    }

    /// **時間面の外へはみ出した clip は、左列のボタンを飲まない。**
    ///
    /// 寄ると clip の左端は面の左(レールの下)まで伸びる。描画はクリップして
    /// いたのに判定はしていなかったので、bar が三角や M/S/L の上に乗っていた。
    #[test]
    fn a_bar_that_starts_before_the_window_does_not_reach_into_the_rail() {
        let track_left = 196.0_f32;
        let track_w = 800.0_f32;
        let track = Rect::from_min_max(
            egui::pos2(track_left, 100.0),
            egui::pos2(track_left + track_w, 124.0),
        );
        let view = TimelineView {
            start: 4.0,
            span: 8.0,
        };

        // 0秒から始まる clip を 4秒地点から見ている: x0 は面の左の外
        let x0 = view.time_to_x(0.0, track_left, track_w);
        let x1 = view.time_to_x(6.0, track_left, track_w);
        assert!(x0 < track_left, "左端は面の外: {x0}");

        let bar = Rect::from_min_max(egui::pos2(x0, track.top()), egui::pos2(x1, track.bottom()));
        let hit = bar.intersect(track);
        assert!(
            hit.left() >= track_left - 1e-3,
            "判定は面の中から: {}",
            hit.left()
        );
        assert!(hit.width() > 0.5, "見えている分は掴める");

        // 完全に左へ出た clip は掴む的が残らない
        let gone = Rect::from_min_max(
            egui::pos2(view.time_to_x(0.0, track_left, track_w), track.top()),
            egui::pos2(view.time_to_x(1.0, track_left, track_w), track.bottom()),
        );
        assert!(
            gone.intersect(track).width() <= 0.5,
            "面の外にしか無いなら的も無い"
        );
    }

    /// **書けない操作を差し出さない。** Group の bar には端が無い。
    ///
    /// Group の bar は子の範囲を写した絵で、Group 自身は `clip.start` も
    /// `duration` も持たない。掴ませても D2 は `TrackItemNotClip` で断るので、
    /// 端として振る舞わせない(掴めるのに何も起きない、を作らない)。
    #[test]
    fn a_group_bar_has_no_trim_edges() {
        let bar = Rect::from_min_max(egui::pos2(100.0, 0.0), egui::pos2(400.0, 20.0));

        for x in [100.0_f32, 103.0, 250.0, 397.0, 400.0] {
            assert_eq!(
                classify_bar_edge(bar, x, true),
                BarPart::Body,
                "Group はどこを掴んでも body: x={x}"
            );
        }
        // clip なら端がある
        assert_eq!(classify_bar_edge(bar, 102.0, false), BarPart::TrimIn);
        assert_eq!(classify_bar_edge(bar, 398.0, false), BarPart::TrimOut);
        assert_eq!(classify_bar_edge(bar, 250.0, false), BarPart::Body);
    }

    /// **細い bar は全部が体。** 端を取ると動かせない clip ができる。
    #[test]
    fn a_thin_bar_keeps_a_body_to_grab() {
        // 幅18px未満: 左右6pxずつ取ると掴める体が残らない
        let thin = Rect::from_min_max(egui::pos2(100.0, 0.0), egui::pos2(112.0, 20.0));
        for x in [100.0_f32, 104.0, 108.0, 112.0] {
            assert_eq!(
                classify_bar_edge(thin, x, false),
                BarPart::Body,
                "細い bar は動かせるほうを残す: x={x}"
            );
        }

        // ちょうど体が残る幅なら端が出る
        let wide = Rect::from_min_max(egui::pos2(100.0, 0.0), egui::pos2(140.0, 20.0));
        assert_eq!(classify_bar_edge(wide, 101.0, false), BarPart::TrimIn);
        assert_eq!(classify_bar_edge(wide, 139.0, false), BarPart::TrimOut);
        assert_eq!(classify_bar_edge(wide, 120.0, false), BarPart::Body);
    }

    /// 端の判定は**clip 本来の矩形**で見る。窓の外へ出ている端は clip の端ではない。
    #[test]
    fn the_window_edge_is_not_the_clips_edge() {
        // 面が 196..996 で、clip は面の左外(0px)から中(500px)まで伸びている
        let bar = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(500.0, 20.0));
        // 面の左端すぐの位置は、clip の中(体)である
        assert_eq!(classify_bar_edge(bar, 200.0, false), BarPart::Body);
        // clip 本来の右端では端が出る
        assert_eq!(classify_bar_edge(bar, 497.0, false), BarPart::TrimOut);
    }

    /// **掴んでいるあいだの手の形は、掴んだものが決める。**
    ///
    /// hover から毎フレーム決め直すと、ポインタが的から少し外れた瞬間に形が戻る
    /// (トリム中なのに掴み手、など)。掴みの種類と形の対応をここで固定する。
    #[test]
    fn the_cursor_follows_what_is_held_not_what_is_under_the_pointer() {
        use egui::CursorIcon;

        assert_eq!(hold_cursor(&None), None, "掴んでいなければ何も言わない");

        let held = |grab: Grab| {
            hold_cursor(&Some(Hold::Item {
                grab,
                gesture: GestureId::from_raw(1),
                undo_base: 0,
            }))
        };
        let layer = LayerId::from_raw(1);
        assert_eq!(
            held(Grab::TrimIn { layer }),
            Some(CursorIcon::ResizeHorizontal),
            "端を引いているあいだは横矢印"
        );
        assert_eq!(
            held(Grab::TrimOut { layer }),
            Some(CursorIcon::ResizeHorizontal)
        );
        assert_eq!(
            held(Grab::Reorder { layer }),
            Some(CursorIcon::Grabbing),
            "並べ替えは掴み手"
        );
        assert_eq!(
            hold_cursor(&Some(Hold::Marquee {
                from: egui::pos2(0.0, 0.0),
                to: egui::pos2(1.0, 1.0),
            })),
            Some(CursorIcon::Crosshair),
            "掃いているあいだは十字"
        );
        assert_eq!(
            hold_cursor(&Some(Hold::Loop(LoopGrab::In { fixed: 1.0 }))),
            Some(CursorIcon::ResizeHorizontal),
            "ループの端も横矢印"
        );
        assert_eq!(
            hold_cursor(&Some(Hold::Loop(LoopGrab::Move {
                grab_at: 0.0,
                from: (0.0, 1.0)
            }))),
            Some(CursorIcon::Grabbing),
            "ループごと動かすなら掴み手"
        );
    }

    /// **横矢印はトリムのときだけ。** 掴み判定と同じ表から出す。
    #[test]
    fn the_resize_arrow_only_appears_where_something_resizes() {
        use egui::CursorIcon;
        let region = LoopRegion {
            start: 2.0,
            end: 6.0,
            on: true,
        };
        let (x0, x1) = (300.0_f32, 700.0_f32);

        // ループ帯: 端だけが横矢印。中は掴み手、外は十字(新しく引く)
        let icon_at = |x: f32| loop_grab_cursor(&loop_grab_for(x, x0, x1, 0.0, region));
        assert_eq!(icon_at(x0), CursorIcon::ResizeHorizontal);
        assert_eq!(icon_at(x1), CursorIcon::ResizeHorizontal);
        assert_eq!(icon_at((x0 + x1) * 0.5), CursorIcon::Grabbing, "中は動かす");
        assert_eq!(icon_at(x1 + 60.0), CursorIcon::Crosshair, "外は引く");

        // clip: 端だけが横矢印。Group と細い clip には端が無いので出ない
        let bar = Rect::from_min_max(egui::pos2(100.0, 0.0), egui::pos2(400.0, 20.0));
        let bar_icon = |x: f32, is_group: bool| match classify_bar_edge(bar, x, is_group) {
            BarPart::Body => CursorIcon::Grab,
            _ => CursorIcon::ResizeHorizontal,
        };
        assert_eq!(bar_icon(101.0, false), CursorIcon::ResizeHorizontal);
        assert_eq!(bar_icon(250.0, false), CursorIcon::Grab);
        assert_eq!(
            bar_icon(101.0, true),
            CursorIcon::Grab,
            "**Group の端では横矢印を出さない**(伸ばせないので)"
        );

        let thin = Rect::from_min_max(egui::pos2(100.0, 0.0), egui::pos2(112.0, 20.0));
        assert_eq!(
            match classify_bar_edge(thin, 101.0, false) {
                BarPart::Body => CursorIcon::Grab,
                _ => CursorIcon::ResizeHorizontal,
            },
            CursorIcon::Grab,
            "**細い clip の端でも出さない**(伸ばす代わりに動かせる)"
        );
    }

    /// **行とキーは同じ選択規則を通る。** 素で置き換え / `Cmd` で足し引き / `Shift` で範囲。
    #[test]
    fn rows_and_keys_share_one_selection_rule() {
        let order = ['a', 'b', 'c', 'd', 'e'];
        let mut sel: Vec<char> = Vec::new();

        select_click(&mut sel, 'b', false, false, &order);
        assert_eq!(sel, vec!['b'], "素のクリックは1つだけにする");

        select_click(&mut sel, 'd', false, true, &order);
        assert_eq!(sel.len(), 3, "b..d の3つ: {sel:?}");
        for c in ['b', 'c', 'd'] {
            assert!(sel.contains(&c));
        }
        assert_eq!(sel.last(), Some(&'b'), "起点は末尾に残る");

        // 続けて Shift を押しても起点は動かない(b から数え直す)
        select_click(&mut sel, 'a', false, true, &order);
        assert_eq!(sel.len(), 2, "a..b: {sel:?}");
        assert!(sel.contains(&'a') && sel.contains(&'b'));

        // Cmd は足し引き
        select_click(&mut sel, 'e', true, false, &order);
        assert!(sel.contains(&'e'));
        select_click(&mut sel, 'e', true, false, &order);
        assert!(!sel.contains(&'e'), "同じものをもう一度で外れる");

        // 並びに無いものは範囲に入らない(畳まれて見えていない行など)
        let mut sel = vec!['b'];
        select_click(&mut sel, 'z', false, true, &['a', 'b', 'c']);
        assert_eq!(
            sel,
            vec!['z'],
            "見えていないなら範囲にならず、素の選択へ落ちる"
        );
    }

    /// キーの範囲選択は**画面に出ている順**(行の順 → 時刻順)で採る。
    #[test]
    fn a_range_of_keys_follows_what_is_on_screen() {
        let mut lab = TimelineEditor::with_fixture();
        let layer = layer_named(&lab.names, "Shared left");
        lab.fold.open_params(layer);
        let _ = layer;

        // 行の順 → 時刻順に並べた列を、画面と同じ手順で作る
        let visible = rows(&lab.document, &lab.fold);
        let key_order: Vec<(LayerId, ParamRef, KeyframeId)> = visible
            .iter()
            .filter_map(|row| match row.kind {
                RowKind::Property(param) => Some((row.layer, param)),
                RowKind::Object => None,
            })
            .flat_map(|(l, param)| {
                let mut keys = param_keys(&lab.document, l, param);
                keys.sort_by(|a, b| a.1.total_cmp(&b.1));
                keys.into_iter().map(move |(key, _)| (l, param, key))
            })
            .collect();
        assert!(
            key_order.len() >= 4,
            "Position 2つ + Scale 2つ: {}",
            key_order.len()
        );

        lab.select_key(key_order[0], false, false, &key_order);
        assert_eq!(lab.selected_keys.len(), 1);

        // **Shift で、間のキーがパラメータをまたいで入る**
        lab.select_key(key_order[3], false, true, &key_order);
        assert_eq!(lab.selected_keys.len(), 4, "{:?}", lab.selected_keys);
        for entry in &key_order[0..=3] {
            assert!(lab.selected_keys.contains(entry));
        }

        // 選んだキーはまとめて消える(既に通っている道を、複数でも通る)
        let doomed = lab.selected_keys.clone();
        assert!(lab.delete_selected_keys());
        assert!(lab.selected_keys.is_empty());
        for (l, param, key) in &doomed {
            assert!(
                !param_keys(&lab.document, *l, *param)
                    .iter()
                    .any(|(id, _)| id == key),
                "選んだキーが消えた: {} {}  status={}",
                lab.name(*l),
                param_label(*param),
                lab.status
            );
        }

        lab.writer.undo().expect("undo");
        refresh_if_stale(&lab.writer, &mut lab.document, &mut lab.revision);
        for (l, param, key) in &doomed {
            assert!(
                param_keys(&lab.document, *l, *param)
                    .iter()
                    .any(|(id, _)| id == key),
                "**まとめて消しても Undo は1回**で全部戻る"
            );
        }
    }
}
