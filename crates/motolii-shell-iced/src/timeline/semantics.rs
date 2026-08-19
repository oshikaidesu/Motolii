//! Timeline の**意味関数** — egui 版(`motolii_ui::timeline_editor`)からの移植。
//!
//! ここに在るのは全部 **純関数**である。toolkit 型(iced / egui)を持たず、
//! Document snapshot と数値だけを受けて答えを返す。描画(`canvas.rs`)と
//! ジェスチャ(`pane.rs`)とテストが同じ関数を呼ぶので、
//! 「カーソルが trim の形なのに掴むと移動する」類のズレが構造的に起きない
//! (spike の `hit_test` 1本方式と同じ判断)。
//!
//! ## 移植元の対応表(egui 版 `timeline_editor/mod.rs`)
//!
//! | ここ | 移植元 | 意味 |
//! |---|---|---|
//! | `bar_span` | `clip_span` | clip の範囲。Group は子の範囲を包む |
//! | `classify_bar_zone` | `classify_bar_edge` | 端 8px。Group と細い bar は端を持たない |
//! | `snap_candidates` / `snapped` | 同名 | 吸着候補は clip の端・playhead・0・終端。間合いは画面 7px |
//! | `clamped_move_delta` | `commit_drag` の Move 腕 | 塊のまま動き、誰か1人でも端を越えるなら全員止まる |
//! | `clamped_trim` | (D2 の validate を先回りする preview 用クランプ) | 1フレーム未満に潰さない・composition の外へ出さない |
//! | `move_targets` | `selection_roots` + `movable_clips` | 親子の重なりを外し、Group は子ごと動く |
//! | `frame_snapped` | `seconds_to_time` | 編集が作る時刻はフレーム境界へ乗る |
//!
//! 時間 ↔ x の換算そのものは移植ではなく**共用**である
//! (`motolii_ui::timeline_editor::TimelineView` を直接使う。zoom / pan / clamp も同じ)。

use motolii_core::{Fps, RationalTime};
use motolii_doc::{Document, LayerId, TrackItem};
use motolii_ui::blitz_shell::{UiEditParam, UiItemFlag};
use motolii_ui::timeline_editor::{TimelineView, TrimEdge};
use motolii_ui::timeline_rows::{keyed_params, ParamRef, RowKind, TimelineRow};

/// 左レール(名前の列)の幅。
///
/// 196→210→234 の経緯(egui 版に寄せる過程で M/S・L ボタン分を足していった)
/// は 2026-08-19 第2波利用者裁定(「タイムラインから下がかなりどデカい」、
/// campaign 文書の追記節)で 196 へ差し戻した — css(`.columnHead`)/ egui
/// (`RAIL_W`)と同値に復帰。M/S/L が `FLAG_BTN_W` 16px 化(旧 22px)で3個分
/// 18px 縮んだため、名前欄はボタン膨張前の余裕へ戻る(`row_name_right_edge`
/// 参照。長い名前は `truncate_name` の省略記号に任せる)。
pub const RAIL_W: f32 = 196.0;
/// transport 帯(playhead 読み・`N rows`・`view a-bs`・`grid n`)の高さ。
/// 元は `/tmp/egui-same-doc.png` の最上段(30px) — 再生ボタンや `space=play`
/// 等の**対応する intent が無い項目は描かない**(2026-08-19 裁定、Q0)。
/// 2026-08-19 第2波利用者裁定(密度圧縮、campaign 文書の追記節)で 24 へ詰めた
/// — skia(`HEADER_HEIGHT` 22)と egui(`HEAD_H` 34)の間、css(`.timelineHead`
/// 34px)とは意味の違う帯のまま食い違う(oracle 参照)。
pub const TRANSPORT_H: f32 = 24.0;
/// ARRANGEMENT 俯瞰帯の高さ。元は `/tmp/egui-same-doc.png` の 22px(他の
/// object 行より低い、灰色の丸みを帯びた1本の帯 — セグメント表示ではない)。
/// 2026-08-19 第2波利用者裁定(密度圧縮)で egui 版 `NAV_H`(14)と同値へ。
pub const OVERVIEW_H: f32 = 14.0;
/// ルーラの高さ。元は egui 版 `RULER_H`(36)と同値。2026-08-19 第2波利用者
/// 裁定(密度圧縮)で 20 へ詰めた — skia(`RULER_HEIGHT` 18)と egui(36)の間、
/// 目盛ラベル(font size 10)が入る最小。
pub const RULER_H: f32 = 20.0;
/// object 行の高さ。元は egui 版 `ROW_H`(小、24)と同値。2026-08-08 決定
/// 「行高は固定・最小 20px」の下限まで、2026-08-19 第2波利用者裁定(密度圧縮)
/// で詰めた。
pub const ROW_H: f32 = 20.0;
/// M / S / L ボタン1枚の幅。モック `.ms`(`docs/mocks-ui/public/timeline-library.css`)
/// と同値の 16px — 2026-08-19 第2波利用者裁定(密度圧縮)で 22→16。
const FLAG_BTN_W: f32 = 16.0;
/// M / S / L ボタンの高さ。同上、18→16(モック `.ms` と同値の正方形)。
const FLAG_BTN_H: f32 = 16.0;
/// M / S ボタン間の隙間。
const FLAG_BTN_GAP: f32 = 4.0;
/// レール右端からの余白。
const FLAG_BTN_MARGIN: f32 = 8.0;
/// transport の2ボタン(to_start / play-pause)1枚の一辺。egui 版(18px, `HEAD_H`
/// 34px の帯)より詰める。2026-08-19 第2波利用者裁定で `TRANSPORT_H` が
/// 24px へ詰まった後も、値そのものは変えていない(24px 帯に上下 4px ずつの
/// 余白が残り、既存の値のままで収まる)。
const TRANSPORT_BTN: f32 = 16.0;
/// transport 帯の左端から to_start ボタンの左端までの余白。
const TRANSPORT_BTN_MARGIN: f32 = 8.0;
/// to_start と play/pause のあいだの隙間。
const TRANSPORT_BTN_GAP: f32 = 6.0;
/// ARRANGEMENT 帯の右余白。
const OVERVIEW_MARGIN: f32 = 10.0;
/// trim の端の幅。egui 版 `TRIM_EDGE` と同値(spike も同じ 8.0 を採った)。
pub const TRIM_EDGE: f32 = 8.0;
/// 吸着の間合い(px)。egui 版 `SNAP_PX` と同値。
pub const SNAP_PX: f32 = 7.0;
/// ルーラが覆う秒数の初期値。egui 版 `TIMELINE_SECONDS` と同値。
pub const TIMELINE_SECONDS: f32 = 16.0;
/// Shift 付き矢印のコマ数。egui 版 `COARSE_FRAME_STEP` と同値。
pub const COARSE_FRAME_STEP: i32 = 10;

/// 面の寸法。**canvas とテストが同じ数字を使う**ための1枚。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaneGeometry {
    pub width: f32,
    pub height: f32,
    /// soundtrack の波形帯の高さ。無ければ 0(帯ごと出ない)。
    pub wave_h: f32,
}

impl PaneGeometry {
    pub fn track_left(&self) -> f32 {
        RAIL_W
    }

    pub fn track_w(&self) -> f32 {
        (self.width - RAIL_W).max(1.0)
    }

    /// transport 帯(playhead 読み・`N rows`・`view a-bs`・`grid n`)の下端。
    pub fn transport_bottom(&self) -> f32 {
        TRANSPORT_H
    }

    /// ARRANGEMENT 帯の下端(= ルーラの上端)。
    pub fn overview_bottom(&self) -> f32 {
        TRANSPORT_H + OVERVIEW_H
    }

    /// ルーラの上端。ARRANGEMENT 帯の下。
    pub fn ruler_top(&self) -> f32 {
        TRANSPORT_H + OVERVIEW_H
    }

    /// ルーラの下端(= 波形帯 / 行の面の上端の手前)。
    pub fn ruler_bottom(&self) -> f32 {
        TRANSPORT_H + OVERVIEW_H + RULER_H
    }

    /// 行の面の上端(transport + ARRANGEMENT + ルーラ + 波形帯の下)。
    pub fn rows_top(&self) -> f32 {
        TRANSPORT_H + OVERVIEW_H + RULER_H + self.wave_h
    }

    /// ARRANGEMENT 帯のトラック部分(レールの右)の x 範囲。ルーラ/行と同じ
    /// track_left で揃える(この行だけ列が割れているわけではない)。
    pub fn overview_track_x(&self) -> (f32, f32) {
        let x0 = self.track_left();
        let x1 = (self.width - OVERVIEW_MARGIN).max(x0 + 1.0);
        (x0, x1)
    }

    /// 最下段の横スクロールバー(表示専用)の y 範囲。
    pub fn scrollbar_y(&self) -> (f32, f32) {
        let h = 6.0;
        ((self.height - h - 4.0).max(0.0), h)
    }

    pub fn row_top(&self, index: usize, scroll_y: f32) -> f32 {
        self.rows_top() + index as f32 * ROW_H - scroll_y
    }

    /// y が指している行の index。行の面の外なら `None`。
    pub fn row_at(&self, y: f32, scroll_y: f32, row_count: usize) -> Option<usize> {
        if y < self.rows_top() || y > self.height {
            return None;
        }
        let local = y - self.rows_top() + scroll_y;
        if local < 0.0 {
            return None;
        }
        let index = (local / ROW_H).floor() as usize;
        (index < row_count).then_some(index)
    }

    pub fn time_to_x(&self, view: TimelineView, t: f32) -> f32 {
        view.time_to_x(t, self.track_left(), self.track_w())
    }

    pub fn x_to_time(&self, view: TimelineView, x: f32) -> f32 {
        view.x_to_time(x, self.track_left(), self.track_w())
    }

    pub fn px_per_second(&self, view: TimelineView) -> f32 {
        self.track_w() / view.span.max(f32::EPSILON)
    }
}

/// L(ロック)ボタンの x 範囲。**レール右端に一番近い**(egui 版 `[M, S, L]` の
/// 並びで L が `i=2` = 一番右と同じ位置関係)。2026-08-19 構造操作レーンで追加 —
/// M / S はここから左へ1ピッチずつ詰める(`solo_button_x` / `mute_button_x` を参照)。
pub fn row_lock_button_x() -> (f32, f32) {
    let x1 = RAIL_W - FLAG_BTN_MARGIN;
    (x1 - FLAG_BTN_W, x1)
}

/// S ボタンの x 範囲。L の左隣(`hit_test` と描画(`canvas.rs`)が同じこれを呼ぶ)。
pub fn solo_button_x() -> (f32, f32) {
    let (lock_x0, _) = row_lock_button_x();
    let x1 = lock_x0 - FLAG_BTN_GAP;
    (x1 - FLAG_BTN_W, x1)
}

/// M ボタンの x 範囲。S の左隣。
pub fn mute_button_x() -> (f32, f32) {
    let (solo_x0, _) = solo_button_x();
    let x1 = solo_x0 - FLAG_BTN_GAP;
    (x1 - FLAG_BTN_W, x1)
}

/// 行の入れ子インデント(px)。egui 版は `8.0 + depth*14.0`、この canvas は
/// 手描きの寸法に合わせて `8.0 + depth*12.0`(既存の draw ループが使っていた
/// 値をそのまま定数化 — 見た目を変えない)。fold 矢印・swatch・名前の位置は
/// 全部ここを基準にする。
pub fn row_indent(depth: u16) -> f32 {
    8.0 + f32::from(depth) * 12.0
}

/// 子の開閉(▶)矢印の当たり判定・描画の x 範囲。`row_indent` の左寄りに
/// 幅 `ROW_FOLD_W` を確保する — has_children が無い行では描かないだけで、
/// 列そのものは全行で同じ位置に居る(名前の縦位置が行ごとにぶれない)。
pub const ROW_FOLD_W: f32 = 14.0;

pub fn row_fold_arrow_x(indent: f32) -> (f32, f32) {
    (indent, indent + ROW_FOLD_W)
}

/// swatch(色札)の左端。fold 矢印の右。
pub fn row_swatch_x(indent: f32) -> f32 {
    indent + ROW_FOLD_W
}

/// 名前の左端。swatch の右。
pub fn row_name_x(indent: f32) -> f32 {
    row_swatch_x(indent) + 20.0
}

/// param 行の開閉(◇/◆)の当たり判定・描画の x 範囲。M/S/L の左、名前との間。
pub fn row_params_toggle_x() -> (f32, f32) {
    let (mute_x0, _) = mute_button_x();
    let x1 = mute_x0 - FLAG_BTN_GAP - 6.0;
    (x1 - 16.0, x1)
}

/// 名前が伸びてよい右端。◇/◆ が出る行はその手前まで、出ない行は M ボタンの手前まで。
/// **egui 版は名前を `with_clip_rect` で枠から出さない**(「はみ出すと ◇ や
/// M/S/L に被る」)— この canvas に clip rect の口が無いので、代わりに描く前の
/// 文字数で切る(`structure::truncate_name` と対で使う)。
pub fn row_name_right_edge(has_params_toggle: bool) -> f32 {
    let x0 = if has_params_toggle {
        row_params_toggle_x().0
    } else {
        mute_button_x().0
    };
    x0 - 6.0
}

/// M / S ボタンの高さ・行内の上端オフセット。描画がボタン矩形を作るのに使う。
pub fn flag_button_rect_y(row_top: f32) -> (f32, f32) {
    (row_top + (ROW_H - FLAG_BTN_H) * 0.5, FLAG_BTN_H)
}

/// transport 帯の「先頭へ戻る」ボタンの矩形(x0, x1, y0, y1)。egui 版
/// `to_start`(`timeline_editor/mod.rs`)と同じ並び — 左端いちばん。
/// `hit_test` と描画(`canvas.rs`)が同じこれを呼ぶ(2026-08-19 再生機構移植レーン)。
pub fn to_start_button_rect(geometry: &PaneGeometry) -> (f32, f32, f32, f32) {
    let cy = geometry.transport_bottom() * 0.5;
    let half = TRANSPORT_BTN * 0.5;
    let x0 = TRANSPORT_BTN_MARGIN;
    (x0, x0 + TRANSPORT_BTN, cy - half, cy + half)
}

/// transport 帯の play/pause ボタンの矩形。to_start の右隣(egui 版と同じ並び)。
pub fn play_pause_button_rect(geometry: &PaneGeometry) -> (f32, f32, f32, f32) {
    let (_, to_start_x1, y0, y1) = to_start_button_rect(geometry);
    let x0 = to_start_x1 + TRANSPORT_BTN_GAP;
    (x0, x0 + TRANSPORT_BTN, y0, y1)
}

/// bar のどこに居るか。egui 版 `BarPart` の対応物(名前は zone にして、
/// intent の `TrimEdge` と別物であることを保つ)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarZone {
    Body,
    In,
    Out,
}

/// canvas ローカル座標が何の上に居るか。**純関数 `hit_test` の戻り値**であって状態ではない。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimelineHit {
    /// ARRANGEMENT 俯瞰帯のトラックの上(押下 / 引きずり = その時刻へ view を寄せる)。
    Overview { at_seconds: f32 },
    /// ルーラの上(スクラブ)。`at_seconds` は指した時刻。
    Ruler { at_seconds: f32 },
    /// 行の M / S ボタンの上(押下 = `ToggleItemFlag`)。
    FlagButton { layer: LayerId, flag: UiItemFlag },
    /// 行の L ボタンの上(押下 = `UiIntent::SetLayerLock`)。
    LockButton { layer: LayerId },
    /// 子の開閉(▶)矢印の上(押下 = `FoldToggled { params: false }`)。
    /// `has_children` の行だけがここを返す。
    FoldArrow { layer: LayerId },
    /// param 行の開閉(◇/◆)の上(押下 = `FoldToggled { params: true }`)。
    /// キーを持つ行だけがここを返す。
    ParamsToggle { layer: LayerId },
    /// 左レールの行の上(クリック = 選択)。
    Rail { layer: LayerId },
    /// bar の上。`at_seconds` は掴んだ時刻(スナップ前)。
    Bar {
        layer: LayerId,
        zone: BarZone,
        at_seconds: f32,
    },
    /// property 行(`RowKind::Property`)の菱形の上。`at_seconds` は**その
    /// キーの実時刻**(Document から読んだ値そのもの — クリック位置ではない)。
    /// `param` は `ParamRef` のまま(Anchor も含む)なので、掴める/選べるかの
    /// 判断は消費側(`key_ui_param`)に任せる(2026-08-19 M-6 キー編集レーン)。
    Key {
        layer: LayerId,
        param: ParamRef,
        at_seconds: f32,
    },
    /// property 行の、菱形の外側(時間トラック面)。**Timeline から playhead へ
    /// キーを打つ**入口(左クリック = `KeyTrackClicked`)。
    PropertyTrack { layer: LayerId, param: ParamRef },
    /// transport 帯の「先頭へ戻る」ボタン(2026-08-19 再生機構移植レーン)。
    ToStart,
    /// transport 帯の play/pause ボタン。同上。
    PlayPause,
    /// 何も無い所。
    Empty,
}

/// 菱形の当たり判定の半径(px)。egui 版 `Rect::from_center_size(c, 12x12)` と同値
/// (`timeline_editor/mod.rs` の `RowKind::Property` 描画ループ)。
pub const KEY_HIT_HALF: f32 = 6.0;

/// その行の bar の範囲(秒)と「Group か」。egui 版 `clip_span` の移植
/// (Group は**直接の clip 子**の範囲を包む — 移植元と同じ規則)。
pub fn bar_span(document: &Document, layer: LayerId) -> Option<(f32, f32, bool)> {
    match find_item(document, layer)? {
        TrackItem::Clip(c) => {
            let start = c.start.as_seconds_f64() as f32;
            Some((start, start + c.duration.as_seconds_f64() as f32, false))
        }
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
            span.map(|(a, b)| (a, b, true))
        }
    }
}

/// bar のどこを掴んだか。egui 版 `classify_bar_edge` の移植:
/// **Group の bar は端を持たない**(D2 が `TrackItemNotClip` で断る操作を差し出さない)。
/// **細い bar も端を取らない**(幅 24px 未満で全部が端になり、動かせない clip を作らない)。
pub fn classify_bar_zone(x0: f32, x1: f32, pos_x: f32, is_group: bool) -> BarZone {
    if is_group || x1 - x0 < TRIM_EDGE * 3.0 {
        return BarZone::Body;
    }
    if pos_x - x0 <= TRIM_EDGE {
        BarZone::In
    } else if x1 - pos_x <= TRIM_EDGE {
        BarZone::Out
    } else {
        BarZone::Body
    }
}

/// canvas ローカル座標 → 何の上に居るか。
///
/// 副作用が無いので、押下の翻訳(`canvas.rs` の update)もカーソル形状
/// (`mouse_interaction`)も絵(trim 帯のハイライト)も同じこれを呼ぶ。
pub fn hit_test(
    document: &Document,
    rows: &[TimelineRow],
    view: TimelineView,
    geometry: &PaneGeometry,
    scroll_y: f32,
    x: f32,
    y: f32,
) -> TimelineHit {
    // transport 帯: `N rows`・`view a-bs`・`grid n`・playhead の時計は表示専用のまま
    // (対応する intent が無い、2026-08-19 裁定、Q0)。to_start / play-pause の
    // 2ボタンだけは口が着いた(`ShellGateway::toggle_playing` /
    // `UiIntent::SetPlayhead` — 2026-08-19 再生機構移植レーン)。
    if y < geometry.transport_bottom() {
        let (x0, x1, y0, y1) = to_start_button_rect(geometry);
        if x >= x0 && x <= x1 && y >= y0 && y <= y1 {
            return TimelineHit::ToStart;
        }
        let (x0, x1, y0, y1) = play_pause_button_rect(geometry);
        if x >= x0 && x <= x1 && y >= y0 && y <= y1 {
            return TimelineHit::PlayPause;
        }
        return TimelineHit::Empty;
    }
    // ARRANGEMENT 俯瞰帯はレールの右のトラック部分だけ(composition 全体が相手 —
    // view の span/start ではなく comp の 0..duration に写す)。
    if y < geometry.overview_bottom() {
        let (x0, x1) = geometry.overview_track_x();
        if x < x0 || x > x1 {
            return TimelineHit::Empty;
        }
        let comp = document.composition.duration.as_seconds_f64() as f32;
        if comp <= 0.0 {
            return TimelineHit::Empty;
        }
        let ratio = ((x - x0) / (x1 - x0)).clamp(0.0, 1.0);
        return TimelineHit::Overview {
            at_seconds: ratio * comp,
        };
    }
    // ルーラはトラック面の上だけ(レールの頭は何でもない)。
    if y < geometry.ruler_bottom() {
        if x >= geometry.track_left() {
            return TimelineHit::Ruler {
                at_seconds: geometry.x_to_time(view, x),
            };
        }
        return TimelineHit::Empty;
    }
    let Some(index) = geometry.row_at(y, scroll_y, rows.len()) else {
        return TimelineHit::Empty;
    };
    let row = rows[index];
    // property 行(`RowKind::Property`)は object 行と別の当たり判定を持つ:
    // 名前/M/S/L のレールは object 行だけの物なので、property 行では
    // トラック面(菱形・空白)しか当たらない(2026-08-19 M-6 キー編集レーン)。
    if let RowKind::Property(param) = row.kind {
        let row_top = geometry.row_top(index, scroll_y);
        return key_hit_test(document, row.layer, param, view, geometry, row_top, x, y);
    }
    // Object 以外の行(将来の行種)は掴ませない。**Property 行は上で処理済み** —
    // `row.layer` は親 object の物を流用しているので(`timeline_rows::rows` の規則)、
    // bar / M / S / L / fold の当たりを親と取り違えないためのガードである。
    if row.kind != RowKind::Object {
        return TimelineHit::Empty;
    }
    if x < geometry.track_left() {
        let row_top = geometry.row_top(index, scroll_y);
        let (btn_y0, btn_h) = flag_button_rect_y(row_top);
        if y >= btn_y0 && y <= btn_y0 + btn_h {
            let (mute_x0, mute_x1) = mute_button_x();
            let (solo_x0, solo_x1) = solo_button_x();
            let (lock_x0, lock_x1) = row_lock_button_x();
            if x >= mute_x0 && x <= mute_x1 {
                return TimelineHit::FlagButton {
                    layer: row.layer,
                    flag: UiItemFlag::Mute,
                };
            }
            if x >= solo_x0 && x <= solo_x1 {
                return TimelineHit::FlagButton {
                    layer: row.layer,
                    flag: UiItemFlag::Solo,
                };
            }
            if x >= lock_x0 && x <= lock_x1 {
                return TimelineHit::LockButton { layer: row.layer };
            }
            if row_has_keys(document, row.layer) {
                let (pt_x0, pt_x1) = row_params_toggle_x();
                if x >= pt_x0 && x <= pt_x1 {
                    return TimelineHit::ParamsToggle { layer: row.layer };
                }
            }
        }
        if row.has_children {
            let indent = row_indent(row.depth);
            let (ax0, ax1) = row_fold_arrow_x(indent);
            if x >= ax0 && x <= ax1 {
                return TimelineHit::FoldArrow { layer: row.layer };
            }
        }
        return TimelineHit::Rail { layer: row.layer };
    }
    let Some((start, end, is_group)) = bar_span(document, row.layer) else {
        return TimelineHit::Empty;
    };
    let x0 = geometry.time_to_x(view, start);
    let x1 = geometry.time_to_x(view, end);
    if x < x0 || x > x1 {
        return TimelineHit::Empty;
    }
    TimelineHit::Bar {
        layer: row.layer,
        zone: classify_bar_zone(x0, x1, x, is_group),
        at_seconds: geometry.x_to_time(view, x),
    }
}

/// 吸着の候補。egui 版 `snap_candidates` の移植:
/// **clip の端・playhead・0・終端**。`exclude` は動かしている当人
/// (自分自身へは吸着しない)。ループ帯とキー行はこの pane にまだ無いので候補に出ない。
pub fn snap_candidates(
    document: &Document,
    rows: &[TimelineRow],
    playhead: f32,
    exclude: &[LayerId],
) -> Vec<f32> {
    let mut out = vec![
        0.0,
        document.composition.duration.as_seconds_f64() as f32,
        playhead,
    ];
    for row in rows {
        if exclude.contains(&row.layer) {
            continue;
        }
        if let Some((start, end, _)) = bar_span(document, row.layer) {
            out.push(start);
            out.push(end);
        }
    }
    out
}

/// 候補へ吸着した時刻。egui 版 `snapped` の移植 — **間合いは画面の距離**
/// (寄れば時間としては細かくなる)。吸着しなければ元の値をそのまま返す。
/// フレームへの丸めは後段(`commit_drag` の `seconds_to_time`)がやる —
/// **吸着はフレームより優先**である。
pub fn snapped(t: f32, candidates: &[f32], px_per_second: f32) -> f32 {
    if px_per_second <= 0.0 {
        return t;
    }
    let window = SNAP_PX / px_per_second;
    let mut best: Option<(f32, f32)> = None;
    for &candidate in candidates {
        let d = (candidate - t).abs();
        if d <= window && best.map(|(bd, _)| d < bd).unwrap_or(true) {
            best = Some((d, candidate));
        }
    }
    best.map(|(_, c)| c).unwrap_or(t)
}

/// **塊のまま動く**移動差分。egui 版 `commit_drag` の Move 腕のクランプの移植:
/// 誰か1人でも 0 より前・composition の終端より後ろへ出るなら、全員そこで止まる
/// (置いていくと選択の相対配置が壊れる)。
pub fn clamped_move_delta(targets: &[(LayerId, f32, f32)], comp: f32, raw_delta: f32) -> f32 {
    if targets.is_empty() {
        return 0.0;
    }
    let floor = targets
        .iter()
        .map(|(_, start, _)| *start)
        .fold(f32::INFINITY, f32::min);
    let headroom = targets
        .iter()
        .map(|(_, _, end)| comp - *end)
        .fold(f32::INFINITY, f32::min);
    raw_delta.clamp(-floor, headroom.max(0.0))
}

/// trim の preview クランプ。D2 の validate(1フレーム未満・composition の外)を
/// **先回りして絵に出す**ためのもので、最終判断は intent の後ろの
/// `prepare_trim_clip_in` / `prepare_trim_clip_out` が持つ。
pub fn clamped_trim(edge: TrimEdge, span: (f32, f32), comp: f32, fps: Fps, at: f32) -> f32 {
    let one_frame = RationalTime::try_from_frame(1, fps)
        .map(|t| t.as_seconds_f64() as f32)
        .unwrap_or(1.0 / 30.0);
    match edge {
        TrimEdge::In => at.clamp(0.0, span.1 - one_frame),
        TrimEdge::Out => at.clamp(span.0 + one_frame, comp),
    }
}

/// 選択から実際に動く clip の集合を出す。egui 版
/// `selection_roots`(親子の重なりを外す)+ `begin_move_many` の `targets` 集め
/// (Group は子の clip を全部、重複は LayerId で畳む)の移植。
pub fn move_targets(document: &Document, selected: &[LayerId]) -> Vec<(LayerId, f32, f32)> {
    let roots: Vec<LayerId> = selected
        .iter()
        .copied()
        .filter(|layer| {
            !selected
                .iter()
                .any(|other| is_descendant(document, *other, *layer))
        })
        .collect();
    let mut targets: Vec<(LayerId, f32, f32)> = Vec::new();
    for root in roots {
        for target in movable_clips(document, root) {
            if !targets.iter().any(|(layer, _, _)| *layer == target.0) {
                targets.push(target);
            }
        }
    }
    targets
}

/// 時刻をフレーム境界へ。egui 版 `seconds_to_time` の移植(scrub の preview 用)。
pub fn frame_snapped(seconds: f32, fps: Fps) -> f32 {
    let Ok(raw) = RationalTime::try_new((f64::from(seconds) * 1000.0).round() as i64, 1000) else {
        return seconds;
    };
    let Ok(frame) = raw.try_to_frame_round(fps) else {
        return seconds;
    };
    RationalTime::try_from_frame(frame, fps)
        .map(|t| t.as_seconds_f64() as f32)
        .unwrap_or(seconds)
}

/// 押されている矢印をコマ数へ。egui 版 `frame_step` の移植
/// (←→ = ±1、Shift = ±10、左右同時は 0)。text focus の腕は
/// この pane にまだ text 入力が無いので持たない。
pub fn frame_step(left: bool, right: bool, shift: bool) -> i32 {
    if left == right {
        return 0;
    }
    let magnitude = if shift { COARSE_FRAME_STEP } else { 1 };
    if right {
        magnitude
    } else {
        -magnitude
    }
}

/// 起動直後の窓。egui 版と同じ「0 から 16 秒」を composition へクランプしたもの。
pub fn initial_view(comp: f32) -> TimelineView {
    TimelineView {
        start: 0.0,
        span: TIMELINE_SECONDS,
    }
    .clamped(comp)
}

/// egui 版 `find_item` の移植(Group の中も辿る)。
pub fn find_item(document: &Document, layer: LayerId) -> Option<&TrackItem> {
    fn walk(items: &[TrackItem], layer: LayerId) -> Option<&TrackItem> {
        for item in items {
            let envelope = match item {
                TrackItem::Clip(c) => &c.envelope,
                TrackItem::Group(g) => &g.envelope,
            };
            if envelope.layer_id == layer {
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

/// その layer の `(muted, solo)`。egui 版 `item_flags`(private)の移植 —
/// **押下状態の正本は Document**(`ItemEnvelope.visible` / `.solo`)で、
/// M ボタンの点灯は `!visible` の裏返し(Mute に専用の bool は無い)。
pub fn item_mute_solo(document: &Document, layer: LayerId) -> (bool, bool) {
    match find_item(document, layer) {
        Some(TrackItem::Clip(c)) => (!c.envelope.visible, c.envelope.solo),
        Some(TrackItem::Group(g)) => (!g.envelope.visible, g.envelope.solo),
        None => (false, false),
    }
}

/// その layer 自身の lock。egui 版 `item_flags`(private)の lock 成分の移植 —
/// **親から受けている分は含まない**(L ボタンの点灯そのもの。トグルの向きを
/// 決めるのに要る。継承ぶんの薄い塗りは `row_effective_lock` を使う)。
pub fn row_own_lock(document: &Document, layer: LayerId) -> bool {
    match find_item(document, layer) {
        Some(TrackItem::Clip(c)) => c.envelope.lock,
        Some(TrackItem::Group(g)) => g.envelope.lock,
        None => false,
    }
}

/// 効いているロックか(親から受けている分も含む)。egui 版 `effective_lock`
/// (private)の移植 — ジェスチャの拒否は D2 dispatch 側(`TimelineEditor::is_locked`)
/// が最終判断を持つので、ここは **描画(薄い塗り)専用の先読み**である。
pub fn row_effective_lock(document: &Document, layer: LayerId) -> bool {
    fn walk(items: &[TrackItem], layer: LayerId, inherited: bool) -> Option<bool> {
        for item in items {
            let (id, own_lock) = match item {
                TrackItem::Clip(c) => (c.envelope.layer_id, c.envelope.lock),
                TrackItem::Group(g) => (g.envelope.layer_id, g.envelope.lock),
            };
            let locked = inherited || own_lock;
            if id == layer {
                return Some(locked);
            }
            if let TrackItem::Group(g) = item {
                if let Some(found) = walk(&g.children, layer, locked) {
                    return Some(found);
                }
            }
        }
        None
    }
    document
        .tracks
        .iter()
        .find_map(|t| walk(&t.items, layer, false))
        .unwrap_or(false)
}

/// キーを持つパラメータが1つでもあるか。◇/◆ トグルを描くかどうかの判定
/// (egui 版 `visible_params` の移植 — `timeline_rows::keyed_params` を共用する)。
pub fn row_has_keys(document: &Document, layer: LayerId) -> bool {
    match find_item(document, layer) {
        Some(item) => !keyed_params(item).is_empty(),
        None => false,
    }
}

/// egui 版 `movable_clips` の移植(Group は子の clip を全部返す)。
pub fn movable_clips(document: &Document, layer: LayerId) -> Vec<(LayerId, f32, f32)> {
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

/// egui 版 `is_descendant` の移植(自分自身は含めない)。
pub fn is_descendant(document: &Document, layer: LayerId, maybe: LayerId) -> bool {
    if layer == maybe {
        return false;
    }
    let Some(item) = find_item(document, layer) else {
        return false;
    };
    let mut ids = Vec::new();
    motolii_doc::collect_layer_ids(item, &mut ids);
    ids.contains(&maybe)
}

/// ルーラの目盛の刻み(秒)。目盛の間隔が 64px を下回らない最小の刻み
/// (spike の `grid_step` と同じ選び方を、フレームでなく秒の系列で)。
pub fn ruler_step_seconds(px_per_second: f32) -> f32 {
    const SERIES: [f32; 12] = [
        0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0, 600.0,
    ];
    for step in SERIES {
        if step * px_per_second >= 64.0 {
            return step;
        }
    }
    *SERIES.last().expect("series is not empty")
}

// ---------------------------------------------------------------------------
// キー行(RowKind::Property)の意味関数 — 2026-08-19 M-6 キー編集レーン
//
// 描画([`crate::timeline::keys`])と hit test(この module の `hit_test`)が
// 同じこれを呼ぶ(egui 版 `param_keys` を1本だけ持ち回る — 複製しない)。
// ---------------------------------------------------------------------------

/// 秒どうしが「同じキー」と見なせるか。**画面座標ではなく秒の絶対誤差**
/// (µs 往復・f32 丸めの桁より充分粗い — フレーム間隔よりずっと狭いので
/// 隣のキーとは取り違えない)。
pub const KEY_TIME_EPS: f32 = 1e-4;

pub fn key_times_match(a: f32, b: f32) -> bool {
    (a - b).abs() <= KEY_TIME_EPS
}

/// `ParamRef` → wire(`UiEditParam`)。Inspector が Anchor に行を持たない
/// 2026-08-18 の決定をそのまま引くので、Anchor は `None`(見えるが、この版では
/// 掴めない — 選択・drag・削除を許さない。触れそうで触れない物を作らない
/// ため、hit test 側でこの判定を通す)。
pub fn key_ui_param(param: ParamRef) -> Option<UiEditParam> {
    match param {
        ParamRef::Position => Some(UiEditParam::Position),
        ParamRef::Scale => Some(UiEditParam::Scale),
        ParamRef::Rotation => Some(UiEditParam::Rotation),
        ParamRef::Opacity => Some(UiEditParam::Opacity),
        ParamRef::Anchor => None,
    }
}

/// property 行の上での当たり。菱形なら `Key`、トラック面の余白なら
/// `PropertyTrack`、行の外(レール側)なら `Empty`(egui 版はレールに
/// チップ+ラベルしか出さない — M/S/名前は object 行だけの物)。
#[allow(clippy::too_many_arguments)]
pub fn key_hit_test(
    document: &Document,
    layer: LayerId,
    param: ParamRef,
    view: TimelineView,
    geometry: &PaneGeometry,
    row_top: f32,
    x: f32,
    y: f32,
) -> TimelineHit {
    if x < geometry.track_left() {
        return TimelineHit::Empty;
    }
    let cy = row_top + ROW_H * 0.5;
    for (_, t) in motolii_ui::timeline_editor::param_keys(document, layer, param) {
        let kx = geometry.time_to_x(view, t);
        if (x - kx).abs() <= KEY_HIT_HALF && (y - cy).abs() <= KEY_HIT_HALF {
            return TimelineHit::Key {
                layer,
                param,
                at_seconds: t,
            };
        }
    }
    TimelineHit::PropertyTrack { layer, param }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_ui::timeline_editor::lab_fixture;

    fn geometry() -> PaneGeometry {
        PaneGeometry {
            width: 1024.0,
            height: 700.0,
            wave_h: 0.0,
        }
    }

    fn fixture_layer(name: &str) -> (Document, LayerId) {
        let (document, names) = lab_fixture();
        let layer = *names
            .iter()
            .find(|(_, n)| n.as_str() == name)
            .map(|(l, _)| l)
            .expect("fixture layer");
        (document, layer)
    }

    /// **Group の bar は端を持たない・細い bar も端を持たない。**
    /// egui 版 `a_group_bar_has_no_trim_edges` と同じ主張。
    #[test]
    fn groups_and_thin_bars_have_no_trim_edges() {
        assert_eq!(classify_bar_zone(0.0, 100.0, 3.0, true), BarZone::Body);
        assert_eq!(classify_bar_zone(0.0, 20.0, 1.0, false), BarZone::Body);
        assert_eq!(classify_bar_zone(0.0, 100.0, 3.0, false), BarZone::In);
        assert_eq!(classify_bar_zone(0.0, 100.0, 97.0, false), BarZone::Out);
        assert_eq!(classify_bar_zone(0.0, 100.0, 50.0, false), BarZone::Body);
    }

    /// カーソル下の時刻を固定したまま寄れる(共用している `TimelineView` が
    /// この面の寸法でもその意味を保つことの確認)。
    #[test]
    fn zooming_keeps_the_time_under_the_anchor() {
        let geometry = geometry();
        let view = initial_view(16.0);
        let anchor_x = 500.0;
        let anchor_t = geometry.x_to_time(view, anchor_x);
        let zoomed = view.zoom_at(anchor_t, 0.5, 16.0);
        let after = geometry.x_to_time(zoomed, anchor_x);
        assert!(
            (after - anchor_t).abs() < 1e-3,
            "anchor の時刻が動いた: {anchor_t} -> {after}"
        );
        assert!(zoomed.span < view.span, "factor < 1 で寄る");
    }

    /// **塊のまま動く。** 一番左の clip が 0 に着いたら全員そこで止まる。
    #[test]
    fn a_selection_stops_together_at_the_left_edge() {
        let targets = vec![
            (LayerId::from_raw(1), 2.0, 5.0),
            (LayerId::from_raw(2), 6.0, 9.0),
        ];
        assert_eq!(clamped_move_delta(&targets, 16.0, -10.0), -2.0);
        assert_eq!(clamped_move_delta(&targets, 16.0, 3.0), 3.0);
        // 右端も同じ: 9.0 の clip が 16.0 に着くまでしか動けない。
        assert_eq!(clamped_move_delta(&targets, 16.0, 10.0), 7.0);
    }

    /// 吸着は**画面の距離**で決まり、候補が無ければ元の値のまま。
    #[test]
    fn snapping_uses_screen_distance() {
        let candidates = vec![0.0, 5.0, 16.0];
        // 100px/s なら 7px = 0.07s の間合い。
        assert_eq!(snapped(5.05, &candidates, 100.0), 5.0);
        assert_eq!(snapped(5.2, &candidates, 100.0), 5.2);
        // 10px/s なら 0.7s の間合いに広がる。
        assert_eq!(snapped(5.5, &candidates, 10.0), 5.0);
    }

    /// fixture の行モデルの上で hit が egui 版と同じ答えを出す:
    /// Background(行 index 1, 0.0..13.4s)の本体と端。
    #[test]
    fn hit_test_resolves_bar_zones_on_the_fixture() {
        let (document, background) = fixture_layer("Background");
        let rows = motolii_ui::timeline_rows::rows(
            &document,
            &motolii_ui::timeline_rows::TimelineFoldState::default(),
        );
        assert_eq!(rows.len(), 3, "fixture は畳んだ状態で 3 object 行");
        assert_eq!(rows[1].layer, background, "行 1 が Background");

        let geometry = geometry();
        let view = initial_view(16.0);
        let y = geometry.row_top(1, 0.0) + ROW_H / 2.0;

        // 本体(2.0s あたり)。
        let x_body = geometry.time_to_x(view, 2.0);
        match hit_test(&document, &rows, view, &geometry, 0.0, x_body, y) {
            TimelineHit::Bar { layer, zone, .. } => {
                assert_eq!(layer, background);
                assert_eq!(zone, BarZone::Body);
            }
            other => panic!("本体を外した: {other:?}"),
        }

        // 右端(13.4s の内側 4px)。
        let x_out = geometry.time_to_x(view, 13.4) - 4.0;
        match hit_test(&document, &rows, view, &geometry, 0.0, x_out, y) {
            TimelineHit::Bar { zone, .. } => assert_eq!(zone, BarZone::Out),
            other => panic!("右端を外した: {other:?}"),
        }

        // ルーラ(ARRANGEMENT 帯の下)。
        match hit_test(
            &document,
            &rows,
            view,
            &geometry,
            0.0,
            400.0,
            geometry.ruler_top() + 10.0,
        ) {
            TimelineHit::Ruler { .. } => {}
            other => panic!("ルーラを外した: {other:?}"),
        }

        // ARRANGEMENT 俯瞰帯(ラベルの右のトラック部分)。
        let (ov_x0, ov_x1) = geometry.overview_track_x();
        match hit_test(
            &document,
            &rows,
            view,
            &geometry,
            0.0,
            (ov_x0 + ov_x1) / 2.0,
            (geometry.transport_bottom() + geometry.overview_bottom()) / 2.0,
        ) {
            TimelineHit::Overview { .. } => {}
            other => panic!("ARRANGEMENT 帯を外した: {other:?}"),
        }

        // レール。
        match hit_test(&document, &rows, view, &geometry, 0.0, 10.0, y) {
            TimelineHit::Rail { layer } => assert_eq!(layer, background),
            other => panic!("レールを外した: {other:?}"),
        }

        // M ボタン。
        let (btn_y0, btn_h) = flag_button_rect_y(geometry.row_top(1, 0.0));
        let (mute_x0, mute_x1) = mute_button_x();
        match hit_test(
            &document,
            &rows,
            view,
            &geometry,
            0.0,
            (mute_x0 + mute_x1) / 2.0,
            btn_y0 + btn_h / 2.0,
        ) {
            TimelineHit::FlagButton { layer, flag } => {
                assert_eq!(layer, background);
                assert_eq!(flag, UiItemFlag::Mute);
            }
            other => panic!("M ボタンを外した: {other:?}"),
        }
    }

    /// transport 帯の to_start / play-pause ボタン(2026-08-19 再生機構移植
    /// レーン)。egui 版と同じ「並び順」を、hit の種類で確かめる —
    /// to_start が左、play/pause がその右。
    #[test]
    fn hit_test_resolves_the_transport_buttons() {
        let (document, _background) = fixture_layer("Background");
        let rows = motolii_ui::timeline_rows::rows(
            &document,
            &motolii_ui::timeline_rows::TimelineFoldState::default(),
        );
        let geometry = geometry();
        let view = initial_view(16.0);

        let (x0, x1, y0, y1) = to_start_button_rect(&geometry);
        let center = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
        match hit_test(&document, &rows, view, &geometry, 0.0, center.0, center.1) {
            TimelineHit::ToStart => {}
            other => panic!("to_start ボタンを外した: {other:?}"),
        }

        let (x0, x1, y0, y1) = play_pause_button_rect(&geometry);
        let center = ((x0 + x1) * 0.5, (y0 + y1) * 0.5);
        match hit_test(&document, &rows, view, &geometry, 0.0, center.0, center.1) {
            TimelineHit::PlayPause => {}
            other => panic!("play/pause ボタンを外した: {other:?}"),
        }

        // to_start は play/pause より左(egui 版と同じ並び)。
        let (to_start_x0, _, _, _) = to_start_button_rect(&geometry);
        let (play_x0, _, _, _) = play_pause_button_rect(&geometry);
        assert!(
            to_start_x0 < play_x0,
            "to_start が play/pause より右に来てはいけない(egui 版と並びが逆)"
        );

        // transport 帯の他の場所(ボタンの外)は引き続き表示専用のまま。
        match hit_test(&document, &rows, view, &geometry, 0.0, 400.0, 5.0) {
            TimelineHit::Empty => {}
            other => panic!("transport 帯のボタン以外がクリック可になっている: {other:?}"),
        }
    }

    /// 親 Group と子が同時に選ばれていても、**動く clip は重複しない**。
    #[test]
    fn move_targets_fold_parent_and_child_selections() {
        let (document, names) = lab_fixture();
        let group = *names
            .iter()
            .find(|(_, n)| n.as_str() == "Title scene")
            .map(|(l, _)| l)
            .expect("group");
        let child = *names
            .iter()
            .find(|(_, n)| n.as_str() == "Shared left")
            .map(|(l, _)| l)
            .expect("child");
        let both = move_targets(&document, &[group, child]);
        let only_group = move_targets(&document, &[group]);
        assert_eq!(both, only_group, "子は親の subtree に畳まれる");
        assert_eq!(only_group.len(), 3, "Group の中の clip 3本が動く");
    }

    /// trim の preview は 1 フレーム未満に潰さない・composition の外に出ない。
    #[test]
    fn trim_preview_is_clamped_to_a_frame_and_the_composition() {
        let fps = Fps::try_new(30, 1).expect("30fps");
        let span = (2.0, 5.0);
        let one = 1.0 / 30.0;
        let left = clamped_trim(TrimEdge::In, span, 16.0, fps, 10.0);
        assert!((left - (5.0 - one)).abs() < 1e-4, "入り点は出し点の1コマ手前まで: {left}");
        assert_eq!(clamped_trim(TrimEdge::In, span, 16.0, fps, -3.0), 0.0);
        let right = clamped_trim(TrimEdge::Out, span, 16.0, fps, 1.0);
        assert!((right - (2.0 + one)).abs() < 1e-4, "出し点は入り点の1コマ後ろまで: {right}");
        assert_eq!(clamped_trim(TrimEdge::Out, span, 16.0, fps, 99.0), 16.0);
    }
}
