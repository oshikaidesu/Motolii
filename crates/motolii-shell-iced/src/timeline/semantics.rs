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
use motolii_ui::timeline_editor::{TimelineView, TrimEdge};
use motolii_ui::timeline_rows::TimelineRow;

/// 左レール(名前の列)の幅。egui 版 `RAIL_W` と同値。
pub const RAIL_W: f32 = 196.0;
/// ルーラの高さ。egui 版 `RULER_H` と同値。
pub const RULER_H: f32 = 36.0;
/// object 行の高さ。egui 版 `ROW_H`(小)と同値。
pub const ROW_H: f32 = 24.0;
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

    /// 行の面の上端(ルーラ + 波形帯の下)。
    pub fn rows_top(&self) -> f32 {
        RULER_H + self.wave_h
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
    /// ルーラの上(スクラブ)。`at_seconds` は指した時刻。
    Ruler { at_seconds: f32 },
    /// 左レールの行の上(クリック = 選択)。
    Rail { layer: LayerId },
    /// bar の上。`at_seconds` は掴んだ時刻(スナップ前)。
    Bar {
        layer: LayerId,
        zone: BarZone,
        at_seconds: f32,
    },
    /// 何も無い所。
    Empty,
}

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
    // ルーラはトラック面の上だけ(レールの頭は何でもない)。
    if y < RULER_H {
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
    if x < geometry.track_left() {
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

        // ルーラ。
        match hit_test(&document, &rows, view, &geometry, 0.0, 400.0, 10.0) {
            TimelineHit::Ruler { .. } => {}
            other => panic!("ルーラを外した: {other:?}"),
        }

        // レール。
        match hit_test(&document, &rows, view, &geometry, 0.0, 10.0, y) {
            TimelineHit::Rail { layer } => assert_eq!(layer, background),
            other => panic!("レールを外した: {other:?}"),
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
