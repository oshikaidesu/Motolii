//! 仮タイムラインのデータモデル。**製品の意味には一切繋がない**。
//!
//! ここは意図的に「ただの struct」である。`motolii-doc` の Document も、intent も、
//! projection も持ち込まない。測りたいのは *データの形* ではなく、
//! *そのデータを触るジェスチャが iced の Elm 構造にどう収まるか* だけだから。

use std::collections::BTreeSet;

// ── 製品と揃えた定数 ───────────────────────────────────────────────────────
// 行高と端 hit 幅は「同じ手触りかどうか」を比べるために製品と同じ値にしてある。

/// 行(トラック)の高さ。製品の timeline と同じ 20px 固定。
pub const ROW_H: f32 = 20.0;

/// 行と行の隙間。0 だと矩形が繋がって見えるので 1px だけ空ける。
pub const ROW_GAP: f32 = 1.0;

/// ruler(playhead をスクラブする帯)の高さ。
pub const RULER_H: f32 = 24.0;

/// clip 端の trim 判定幅。
///
/// 発注書には「7px(製品の hit 幅と同じ)」とあったが、製品を読むと 7px は
/// **見た目の帯の幅**(モックの `.trimHandle{width:7px}`)で、掴める幅は
/// `timeline_editor/mod.rs:939` の `const TRIM_EDGE: f32 = 8.0;` である。
/// 手触りを揃えるのが目的なので、掴める方の 8.0 を採る。
pub const TRIM_HIT_W: f32 = 8.0;

/// これより細い clip には trim 端を作らない。製品の `TRIM_EDGE * 3.0`
/// (`mod.rs:951`)と同じ規則。端2つが潰し合って本体が掴めなくなるのを防ぐ。
pub const TRIM_MIN_BAR_W: f32 = TRIM_HIT_W * 3.0;

/// フレームレート。フレーム番号 → 秒 の換算にしか使わない。
pub const FPS: f64 = 30.0;

/// 横 zoom の下限・上限(1フレームあたりの px)。
pub const PX_PER_FRAME_MIN: f32 = 0.02;
pub const PX_PER_FRAME_MAX: f32 = 60.0;

pub type ClipId = u32;

/// 1つの clip。時間はすべて**フレーム**の整数で持つ。
///
/// 浮動小数の秒で持たないのは、スナップの正しさを型で守るため。
/// 「グリッドに吸い付く」は `i64` に落とす一点でだけ起きる(`Viewport::snap`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clip {
    pub id: ClipId,
    pub track: usize,
    pub start: i64,
    pub len: i64,
}

impl Clip {
    pub fn end(&self) -> i64 {
        self.start + self.len
    }
}

/// トラック × clip だけの、平たいタイムライン。
#[derive(Debug, Clone)]
pub struct Timeline {
    pub tracks: usize,
    pub clips: Vec<Clip>,
}

impl Timeline {
    /// `--clips N` 用のデモデータ。
    ///
    /// 既定 zoom で「画面にそこそこ載る」密度になるよう、clip を左詰めで敷き詰める。
    /// 500本を全部画面外に置いてしまうと frame 時間の実測が culling の測定になってしまう。
    pub fn demo(clip_count: usize) -> Self {
        let tracks = if clip_count <= 16 { 6 } else { 12 };
        let mut clips = Vec::with_capacity(clip_count);

        // トラックごとに「今どこまで埋まったか」を持って左から詰める。
        let mut cursor = vec![0i64; tracks];
        // 長さ・隙間を決定的に散らすための、種だけの線形合同法。再現性が要るので乱数は使わない。
        let mut seed: u64 = 0x5eed_1234;
        let mut next = |m: u64| -> u64 {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
            (seed >> 33) % m
        };

        for i in 0..clip_count {
            let track = i % tracks;
            let len = 18 + next(70) as i64;
            let gap = next(14) as i64;
            let start = cursor[track] + gap;
            cursor[track] = start + len;
            clips.push(Clip {
                id: i as ClipId,
                track,
                start,
                len,
            });
        }

        Self { tracks, clips }
    }

    /// 一番右の clip の終端。起動時の zoom を決めるのに使う。
    pub fn extent(&self) -> i64 {
        self.clips.iter().map(Clip::end).max().unwrap_or(0)
    }

    pub fn get(&self, id: ClipId) -> Option<&Clip> {
        self.clips.iter().find(|c| c.id == id)
    }

    pub fn get_mut(&mut self, id: ClipId) -> Option<&mut Clip> {
        self.clips.iter_mut().find(|c| c.id == id)
    }
}

/// 見え方の状態。ドキュメントではないので、undo にも保存にも乗らない側。
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    /// 1フレームあたりの px。これが横 zoom そのもの。
    pub px_per_frame: f32,
    /// 左端に来ているフレーム(小数のまま持つ。zoom のアンカー計算で丸めたくない)。
    pub scroll_x: f64,
    /// 縦スクロール量(px)。
    pub scroll_y: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            px_per_frame: 4.0,
            scroll_x: 0.0,
            scroll_y: 0.0,
        }
    }
}

impl Viewport {
    /// 中身が全部入る zoom。`--clips 500` の frame 時間が
    /// 「culling の測定」になってしまわないよう、起動時にこれを当てる。
    /// 窓幅は起動前には分からないので、`main.rs` の `window_size` と同じ値を使う。
    pub fn fit(extent: i64, width_px: f32) -> Self {
        let px_per_frame = if extent > 0 {
            (width_px / extent as f32).clamp(PX_PER_FRAME_MIN, PX_PER_FRAME_MAX)
        } else {
            4.0
        };
        Self {
            px_per_frame,
            scroll_x: 0.0,
            scroll_y: 0.0,
        }
    }

    pub fn frame_to_x(&self, frame: f64) -> f32 {
        ((frame - self.scroll_x) * self.px_per_frame as f64) as f32
    }

    pub fn x_to_frame(&self, x: f32) -> f64 {
        self.scroll_x + x as f64 / self.px_per_frame as f64
    }

    /// フレームグリッドへのスナップ。**丸めはここ1箇所しかない**。
    pub fn snap(&self, frame: f64) -> i64 {
        frame.round() as i64
    }

    /// track 番号 → canvas 上の y(ruler の下から数える)。
    pub fn track_to_y(&self, track: usize) -> f32 {
        RULER_H + track as f32 * (ROW_H + ROW_GAP) - self.scroll_y
    }

    /// y → track 番号。ruler 帯の中や、行の隙間なら `None`。
    pub fn y_to_track(&self, y: f32, tracks: usize) -> Option<usize> {
        if y < RULER_H {
            return None;
        }
        let local = y - RULER_H + self.scroll_y;
        let idx = (local / (ROW_H + ROW_GAP)).floor();
        if idx < 0.0 {
            return None;
        }
        let idx = idx as usize;
        if idx >= tracks {
            return None;
        }
        // 隙間に落ちたら「行の上ではない」とする。製品の行間と同じ扱い。
        if local - idx as f32 * (ROW_H + ROW_GAP) > ROW_H {
            return None;
        }
        Some(idx)
    }

    /// カーソル下のフレームを固定したまま `px_per_frame` を倍率変更する。
    ///
    /// AE/Premiere と同じ「掴んだ時刻が動かない」zoom。これを外すと Cmd+ホイールが
    /// 一気に使い物にならなくなるので、spike でも省かない。
    pub fn zoom_at(&mut self, anchor_x: f32, factor: f32) {
        let anchor_frame = self.x_to_frame(anchor_x);
        self.px_per_frame = (self.px_per_frame * factor).clamp(PX_PER_FRAME_MIN, PX_PER_FRAME_MAX);
        self.scroll_x = anchor_frame - anchor_x as f64 / self.px_per_frame as f64;
    }
}

/// clip のどこを掴んだか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grab {
    /// 本体 = 移動。
    Body,
    /// 左端 = 入り点の trim。
    LeftEdge,
    /// 右端 = 出し点の trim。
    RightEdge,
}

/// hit test の答え。**純関数の戻り値**であって、状態ではない。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Hit {
    /// ruler の上(スクラブ)。
    Ruler,
    /// clip の上。
    Clip { id: ClipId, grab: Grab },
    /// 何も無い所。
    Empty,
}

/// canvas ローカル座標 → 何を掴んだか。
///
/// 副作用が無いので `update` からも `mouse_interaction` からも同じものを呼べる。
/// カーソル形状と実際に始まるジェスチャがズレない、という保証がこれ1本で付く。
pub fn hit_test(timeline: &Timeline, viewport: &Viewport, x: f32, y: f32) -> Hit {
    if y < RULER_H {
        return Hit::Ruler;
    }
    let Some(track) = viewport.y_to_track(y, timeline.tracks) else {
        return Hit::Empty;
    };

    // 手前(後に描かれた物)を優先する。描画順と hit 順を逆にしておかないと
    // 「見えている物が掴めない」が起きる。
    for clip in timeline.clips.iter().rev() {
        if clip.track != track {
            continue;
        }
        let x0 = viewport.frame_to_x(clip.start as f64);
        let x1 = viewport.frame_to_x(clip.end() as f64);
        if x < x0 || x > x1 {
            continue;
        }
        // 細い clip には端を作らない(製品と同じ規則)。
        let grab = if x1 - x0 < TRIM_MIN_BAR_W {
            Grab::Body
        } else if x - x0 <= TRIM_HIT_W {
            Grab::LeftEdge
        } else if x1 - x <= TRIM_HIT_W {
            Grab::RightEdge
        } else {
            Grab::Body
        };
        return Hit::Clip { id: clip.id, grab };
    }
    Hit::Empty
}

/// 選択集合。`BTreeSet` なのは描画順を安定させるためだけ。
pub type Selection = BTreeSet<ClipId>;

/// フレーム番号 → `mm:ss.ff` 表記。ruler の目盛にだけ使う。
pub fn frame_label(frame: i64) -> String {
    let total_secs = frame as f64 / FPS;
    let m = (total_secs / 60.0).floor() as i64;
    let s = (total_secs % 60.0).floor() as i64;
    let f = frame.rem_euclid(FPS as i64);
    format!("{m}:{s:02}.{f:02}")
}
