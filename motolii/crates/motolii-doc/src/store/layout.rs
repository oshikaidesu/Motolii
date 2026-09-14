//! 箱と流し込み — Group の Display が Flex / Grid なら、直下の子を CSS の規則で並べる。計算は taffy
//! ([箱と流し込みの法](../../../../../docs/reviews/2026-09-14-layout-law.md))。
//! 並べた結果は書類に書かない: その時刻の子の位置・大きさ・輪郭の伸びを解くだけ。
//! 子の Position は並べた位置からのずれ(`position: relative`)、Scale は `zoom`(箱ごと大きくなり隣を押す)。

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use taffy::prelude::*;

use crate::doc::core::RationalTime;
use crate::doc::eval::Value;
use crate::doc::store::{property, LayerId, LayerSource, PropertyId, StoreError, StoreView};

pub const DISPLAY: &str = "layout.display";
pub const FLEX_DIRECTION: &str = "layout.flex_direction";
pub const FLEX_WRAP: &str = "layout.flex_wrap";
pub const JUSTIFY_CONTENT: &str = "layout.justify_content";
pub const ALIGN_ITEMS: &str = "layout.align_items";
pub const GAP: &str = "layout.gap";
pub const PADDING: &str = "layout.padding";
pub const GRID_COLUMNS: &str = "layout.grid_columns";
pub const GRID_ROWS: &str = "layout.grid_rows";
/// 格子の線ごとの太さ(fr)。`layout.column.<n>` / `layout.row.<n>`、n は 1 から。鍵が打てる。
pub const COLUMN_PREFIX: &str = "layout.column.";
pub const ROW_PREFIX: &str = "layout.row.";
pub const BACKGROUND: &str = "layout.background";
pub const BORDER_RADIUS: &str = "layout.border_radius";
pub const OVERFLOW: &str = "layout.overflow";
pub const HORIZONTAL_SIZING: &str = "layout.horizontal_sizing";
pub const VERTICAL_SIZING: &str = "layout.vertical_sizing";
pub const WIDTH: &str = "layout.width";
pub const HEIGHT: &str = "layout.height";
pub const POSITION_TYPE: &str = "layout.position_type";
pub const ALIGN_SELF: &str = "layout.align_self";
pub const COLUMN_START: &str = "layout.column_start";
pub const ROW_START: &str = "layout.row_start";
pub const COLUMN_SPAN: &str = "layout.column_span";
pub const ROW_SPAN: &str = "layout.row_span";
pub const OBJECT_FIT: &str = "layout.object_fit";
pub const DEPTH_ALIGNMENT: &str = "layout.depth_alignment";
/// 映像の中身が格子の枠を占める(CSS Exclusions の写し): Blob Track を持つ層を指すと、その塊が重なる枠は空けて、
/// 子は残りの枠へ流れる。塊は host が解いた箱(解析の橋)。
pub const EXCLUSIONS: &str = "layout.exclusions";
/// 間合いの法(2026-09-14): 物が外との距離を宣言する。容器の子なら CSS の margin、容器の外の兄弟同士は押し合う。
pub const MARGIN: &str = "layout.margin";
/// 詰まった時に誰がどれだけ譲るかの比(CSS の flex-shrink)。0 は譲らない。
pub const FLEX_SHRINK: &str = "layout.flex_shrink";
/// 解いた行き先が変わった時の移り方(CSS の transition)。秒と、区間の形。
pub const TRANSITION_DURATION: &str = "layout.transition_duration";
pub const TRANSITION_EASING: &str = "layout.transition_easing";
/// 移り方を始めるまでの遅れ(CSS の transition-delay)。
pub const TRANSITION_DELAY: &str = "layout.transition_delay";
/// 並べる容器が子の移り方の遅れを配る(GSAP の stagger の amount と from)。遅れ = Stagger × 起点からの距離 / 容器の最大の距離。
pub const STAGGER: &str = "layout.stagger";
pub const STAGGER_FROM: &str = "layout.stagger_from";
/// 文字が避けて流れる物の形(CSS `shape-outside`、宣言するのは避けられる物の側)。同じ親の、折り返す文字が避ける。
/// Margin Box = 箱 + Margin、Content = 形の輪郭(Blob Track の層は塊の箱)。間は `Shape Margin`。
pub const SHAPE_OUTSIDE: &str = "layout.shape_outside";
pub const SHAPE_MARGIN: &str = "layout.shape_margin";
/// 並びに効く回転(visionOS の rotation3DLayout): 回した物の軸に沿った箱で並べる。層の Rotation / Tilt は見た目だけ。
pub const LAYOUT_ROTATION: &str = "layout.rotation";
pub const LAYOUT_TILT_X: &str = "layout.tilt_x";
pub const LAYOUT_TILT_Y: &str = "layout.tilt_y";
/// Flex Direction の Depth(奥へ積む): 子を奥行き + Gap ずつ奥へ、面の上の位置は Align Items で揃える。
pub const DIRECTION_DEPTH: i64 = 4;

/// 欄: (property, 窓の名前, 既定値, 範囲, 選択肢)。名前は CSS の語、大きさの決め方だけ Figma(裁定 2026-09-14)。
pub type Row = (&'static str, &'static str, Value, Option<(f64, f64)>, &'static [&'static str]);

const SIZING: &[&str] = &["Hug", "Fill", "Fixed"];

/// Group の欄。Display が None の間は Display だけが意味を持つ。
pub const GROUP_ROWS: &[Row] = &[
    (DISPLAY, "Display", Value::Enum(0), None, &["None", "Flex", "Grid"]),
    (FLEX_DIRECTION, "Flex Direction", Value::Enum(0), None, &["Row", "Column", "Row Reverse", "Column Reverse", "Depth"]),
    (FLEX_WRAP, "Flex Wrap", Value::Enum(0), None, &["No Wrap", "Wrap", "Wrap Reverse"]),
    (JUSTIFY_CONTENT, "Justify Content", Value::Enum(0), None, &["Start", "End", "Center", "Space Between", "Space Around", "Space Evenly"]),
    (ALIGN_ITEMS, "Align Items", Value::Enum(0), None, &["Stretch", "Start", "End", "Center"]),
    (DEPTH_ALIGNMENT, "Depth Alignment", Value::Enum(0), None, &["Back", "Center", "Front"]),
    (GRID_COLUMNS, "Grid Columns", Value::F64(2.0), Some((1.0, 64.0)), &[]),
    (GRID_ROWS, "Grid Rows", Value::F64(0.0), Some((0.0, 64.0)), &[]),
    (EXCLUSIONS, "Exclusions", Value::LayerId(0), None, &[]),
    (GAP, "Gap", Value::F64(0.0), Some((0.0, 100000.0)), &[]),
    (PADDING, "Padding", Value::Vec2([0.0, 0.0]), None, &[]),
    (HORIZONTAL_SIZING, "Horizontal Sizing", Value::Enum(0), None, SIZING),
    (VERTICAL_SIZING, "Vertical Sizing", Value::Enum(0), None, SIZING),
    (WIDTH, "Width", Value::F64(400.0), Some((0.0, 100000.0)), &[]),
    (HEIGHT, "Height", Value::F64(300.0), Some((0.0, 100000.0)), &[]),
    (BACKGROUND, "Background", Value::Color([1.0, 1.0, 1.0, 0.0]), None, &[]),
    (BORDER_RADIUS, "Border Radius", Value::F64(0.0), Some((0.0, 100000.0)), &[]),
    (OVERFLOW, "Overflow", Value::Enum(0), None, &["Visible", "Clip"]),
    (STAGGER, "Stagger", Value::F64(0.0), Some((0.0, 60.0)), &[]),
    (STAGGER_FROM, "Stagger From", Value::Enum(0), None, &["Start", "Center", "End", "Edges"]),
];

/// 並ぶ子の欄(親の Display が Flex / Grid の時)。
pub const ITEM_ROWS: &[Row] = &[
    (POSITION_TYPE, "Position Type", Value::Enum(0), None, &["Relative", "Absolute"]),
    (HORIZONTAL_SIZING, "Horizontal Sizing", Value::Enum(0), None, SIZING),
    (VERTICAL_SIZING, "Vertical Sizing", Value::Enum(0), None, SIZING),
    (WIDTH, "Width", Value::F64(100.0), Some((0.0, 100000.0)), &[]),
    (HEIGHT, "Height", Value::F64(100.0), Some((0.0, 100000.0)), &[]),
    (ALIGN_SELF, "Align Self", Value::Enum(0), None, &["Auto", "Stretch", "Start", "End", "Center"]),
    (COLUMN_START, "Column Start", Value::F64(0.0), Some((0.0, 64.0)), &[]),
    (ROW_START, "Row Start", Value::F64(0.0), Some((0.0, 64.0)), &[]),
    (COLUMN_SPAN, "Column Span", Value::F64(1.0), Some((1.0, 64.0)), &[]),
    (ROW_SPAN, "Row Span", Value::F64(1.0), Some((1.0, 64.0)), &[]),
    (OBJECT_FIT, "Object Fit", Value::Enum(0), None, &["Fill", "Contain", "Cover", "None"]),
    (LAYOUT_ROTATION, "Layout Rotation", Value::F64(0.0), None, &[]),
    (LAYOUT_TILT_X, "Layout Tilt X", Value::F64(0.0), None, &[]),
    (LAYOUT_TILT_Y, "Layout Tilt Y", Value::F64(0.0), None, &[]),
];

/// 形の層の素材座標は、輪郭の canvas の左上(反アリアスの 1 画素の外)が原点。
/// Display の Group の箱もその座標で [1, 1]..[1 + 幅, 1 + 高さ] に置き、背景の形と子の枠が同じ所に来る。
const CANVAS_MARGIN: f32 = 1.0;

/// 間合いの欄(容器の中でも外でも、すべての物)。
pub const SPACE_ROWS: &[Row] = &[
    (MARGIN, "Margin", Value::F64(0.0), Some((0.0, 100000.0)), &[]),
    (FLEX_SHRINK, "Flex Shrink", Value::F64(1.0), Some((0.0, 1000.0)), &[]),
    (TRANSITION_DURATION, "Transition Duration", Value::F64(0.0), Some((0.0, 60.0)), &[]),
    (TRANSITION_EASING, "Transition Easing", Value::Enum(0), None, &["Ease", "Linear", "Ease In", "Ease Out", "Ease In Out"]),
    (TRANSITION_DELAY, "Transition Delay", Value::F64(0.0), Some((0.0, 60.0)), &[]),
    (SHAPE_OUTSIDE, "Shape Outside", Value::Enum(0), None, &["None", "Margin Box", "Content"]),
    (SHAPE_MARGIN, "Shape Margin", Value::F64(0.0), Some((0.0, 100000.0)), &[]),
];

/// 格子の線の太さの既定(fr)。
pub const TRACK_DEFAULT: f64 = 1.0;

/// 番号付きの線の名前(`layout.column.3` → `Column 3`)。
pub fn track_label(property: &str) -> Option<String> {
    let number = |prefix: &str| property.strip_prefix(prefix).and_then(|n| n.parse::<u32>().ok()).filter(|n| *n >= 1);
    number(COLUMN_PREFIX).map(|n| format!("Column {n}")).or_else(|| number(ROW_PREFIX).map(|n| format!("Row {n}")))
}

pub fn row(property: &str) -> Option<&'static Row> {
    GROUP_ROWS.iter().chain(ITEM_ROWS).chain(SPACE_ROWS).find(|row| row.0 == property)
}

pub fn choices(property: &str) -> &'static [&'static str] {
    row(property).map_or(&[], |row| row.4)
}

/// その時刻に並べた結果。`slots` は並ぶ子、`sizes` は Display の Group の箱の大きさ(素材座標で [0, 0]..size)。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Frame {
    pub slots: HashMap<LayerId, Slot>,
    pub sizes: HashMap<LayerId, [f32; 2]>,
    /// 容器の外で押し合った物の、親の空間でのずれ。
    pub nudges: HashMap<LayerId, [f32; 2]>,
    /// Display の Group の奥行きの範囲 [手前, 奥]。揃えなら面が奥で [-奥行き, 0]、奥へ積むなら [0, 積んだ厚み]。
    pub depths: HashMap<LayerId, [f32; 2]>,
}

/// 並ぶ子の変換の差し替え: 層の Position と Scale の代わりに使う値と、形の輪郭の伸び。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    pub position: [f32; 2],
    pub scale: [f32; 2],
    pub stretch: [f32; 2],
    /// 横が Fill の文字: その幅で折り返す(素材座標の幅、Scale で割った値)。
    pub wrap: Option<f32>,
    /// 奥行きの揃えで足す z(Position Z に足す)。面が奥(z = 0)、物は camera 側(負)へ出る。
    pub z: f32,
    /// 奥行きに足す倍率(Scale Z に掛ける)。網・点群の奥行きは描く側が xy の拡縮から伸ばすので、今は 1。
    pub scale_z: f32,
    /// 並びに効く回転(Tilt X・Tilt Y・Rotation、度)。層の Rotation / Tilt に足す。
    pub rotation: [f32; 3],
    /// 拡縮・回転の中心(素材座標)。Anchor を書いていなければ箱の中心(CSS の transform-origin: 50% 50%)。
    pub anchor: [f32; 2],
}

/// view の寿命の間、時刻ごとに 1 回だけ解く(view は値を変えない)。
pub(crate) type Memo = Rc<RefCell<HashMap<RationalTime, Rc<Frame>>>>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Sizing { Hug, Fill, Fixed }

/// 幅で高さが決まる子(横が Fill の文字)。taffy が幅を決めてから問う。
#[derive(Clone, Copy)]
struct Measure {
    layer: LayerId,
    scale: f32,
}

struct Leaf {
    node: NodeId,
    /// 横が Fill の文字(折り返し幅を枠から決める)。
    text_fill: bool,
    layer: LayerId,
    /// 素材座標の箱(伸ばす前)。
    bounds: [f32; 4],
    sizing: [Sizing; 2],
    fit: i64,
}

impl StoreView<'_> {
    /// 移り方(Transition)が遡るコマ数の最大。host は解析の入力(Blob の塊)をこのコマ数だけ前まで置く。
    /// 移り方は重なる(折り返しの移り方が前の時刻で組み、その時刻の避ける物がさらに前の形を混ぜる)ので、長い順に 2 つの和と遅れ 2 つ分。
    pub fn transition_reach(&self, t: RationalTime) -> Result<i64, StoreError> {
        let Some(comp) = self.composition()? else { return Ok(0) };
        let mut longest = [0.0f64; 2];
        let mut waits = 0.0f64;
        for layer in self.layers() {
            waits = waits.max(self.number(layer, TRANSITION_DELAY, 0.0, t)? + self.number(layer, STAGGER, 0.0, t)?);
            let d = self.number(layer, TRANSITION_DURATION, 0.0, t)?;
            if d > longest[0] {
                longest = [d, longest[0]];
            } else if d > longest[1] {
                longest[1] = d;
            }
        }
        Ok(((longest[0] + longest[1] + 2.0 * waits) * comp.fps.as_f64()).round() as i64 + 1)
    }

    /// その時刻に並べた結果。Display の Group が無ければ空。
    pub fn layout_frame(&self, t: RationalTime) -> Result<Rc<Frame>, StoreError> {
        if let Some(hit) = self.layout_memo().borrow().get(&t) {
            return Ok(hit.clone());
        }
        // 解いている間に同じ時刻を問われたら(面の Group の親を辿る時など)、空の結果で答えて巡らない。
        self.layout_memo().borrow_mut().insert(t, Rc::new(Frame::default()));
        let frame = Rc::new(self.compute_layout(t)?);
        self.layout_memo().borrow_mut().insert(t, frame.clone());
        Ok(frame)
    }

    /// 並ぶ子なら、層の Position と Scale の代わりに使う値。
    pub(crate) fn laid_out(&self, layer: LayerId, t: RationalTime) -> Result<Option<Slot>, StoreError> {
        let Some(parent) = self.attrs(layer)?.unwrap_or_default().parent else { return Ok(None) };
        if self.display(parent, t)? == 0 {
            return Ok(None);
        }
        let Some(mut slot) = self.layout_frame(t)?.slots.get(&layer).copied() else { return Ok(None) };
        // 移り方: 少し前の行き先を、区間の重みで混ぜる(位置と大きさ)。
        let mut acc = [[0.0f32; 2]; 4];
        let mut total = 0.0f32;
        for (at, weight) in self.transition_samples(layer, t)? {
            if let Some(past) = self.layout_frame(at)?.slots.get(&layer) {
                for (sum, value) in acc.iter_mut().zip([past.position, past.scale, past.stretch, past.anchor]) {
                    for axis in 0..2 {
                        sum[axis] += value[axis] * weight;
                    }
                }
                total += weight;
            }
        }
        if total > 1e-6 {
            [slot.position, slot.scale, slot.stretch, slot.anchor] = acc.map(|sum| sum.map(|v| v / total));
        }
        Ok(Some(slot))
    }

    /// 並べる Group の箱の大きさ(移り方を混ぜた後)。背景・切り抜き・層の箱が読む。
    pub(crate) fn group_size(&self, group: LayerId, t: RationalTime) -> Result<Option<[f32; 2]>, StoreError> {
        let Some(now) = self.layout_frame(t)?.sizes.get(&group).copied() else { return Ok(None) };
        let mut acc = [0.0f32; 2];
        let mut total = 0.0f32;
        for (at, weight) in self.transition_samples(group, t)? {
            if let Some(past) = self.layout_frame(at)?.sizes.get(&group) {
                acc[0] += past[0] * weight;
                acc[1] += past[1] * weight;
                total += weight;
            }
        }
        Ok(Some(if total > 1e-6 { acc.map(|v| v / total) } else { now }))
    }

    /// 容器の外で押し合ったずれ(移り方を混ぜた後)。
    pub(crate) fn nudge(&self, layer: LayerId, t: RationalTime) -> Result<[f32; 2], StoreError> {
        let now = self.layout_frame(t)?.nudges.get(&layer).copied().unwrap_or([0.0; 2]);
        let samples = self.transition_samples(layer, t)?;
        if samples.is_empty() {
            return Ok(now);
        }
        let mut acc = [0.0f32; 2];
        let mut total = 0.0f32;
        for (at, weight) in samples {
            let past = self.layout_frame(at)?.nudges.get(&layer).copied().unwrap_or([0.0; 2]);
            acc[0] += past[0] * weight;
            acc[1] += past[1] * weight;
            total += weight;
        }
        Ok(if total > 1e-6 { acc.map(|v| v / total) } else { now })
    }

    /// 文字の折り返しの移り方: Transition を持つ文字は、少し前のコマの組の字の位置との差を区間の重みで混ぜる。
    /// 字は元の文字の byte で対にする。前のコマに無い字は動かさない。差が無ければ None。
    pub(crate) fn glyph_offsets(&self, layer: LayerId, t: RationalTime) -> Result<Option<std::sync::Arc<Vec<[f32; 2]>>>, StoreError> {
        if !self.meta(layer)?.is_some_and(|m| m.source == LayerSource::Text) {
            return Ok(None);
        }
        let samples = self.transition_samples(layer, t)?;
        if samples.is_empty() {
            return Ok(None);
        }
        let Some(comp) = self.composition()? else { return Ok(None) };
        let canvas = crate::doc::vector::Canvas { width: comp.width, height: comp.height, origin_x: 0, origin_y: 0 };
        // 字は元の文字の byte で対にする(組み直しで行頭の空白が落ちても、隣の字と取り違えない)。
        let glyphs = |at: RationalTime| -> Result<Vec<(usize, [f32; 2])>, StoreError> {
            let Some(document) = self.resolved_text_document(layer, at)? else { return Ok(Vec::new()) };
            let around = self.flow_around(layer, at)?;
            let Ok(Some(shaped)) = crate::doc::store::text_frame::shape_document_around(&document, at, &canvas, around.as_deref().map_or(&[], Vec::as_slice)) else { return Ok(Vec::new()) };
            Ok(shaped.lines.iter().flat_map(|line| line.glyph_bytes.iter().zip(&line.glyph_xs).map(move |(b, x)| (*b, [*x, line.baseline_y]))).collect())
        };
        let now = glyphs(t)?;
        let mut sum = vec![[0.0f32; 2]; now.len()];
        let mut weight = vec![0.0f32; now.len()];
        for (at, w) in samples {
            let past: HashMap<usize, [f32; 2]> = glyphs(at)?.into_iter().collect();
            for (n, (byte, here)) in now.iter().enumerate() {
                if let Some(p) = past.get(byte) {
                    sum[n][0] += (p[0] - here[0]) * w;
                    sum[n][1] += (p[1] - here[1]) * w;
                    weight[n] += w;
                }
            }
        }
        let offsets: Vec<[f32; 2]> = sum.iter().zip(&weight).map(|(s, w)| if *w > 1e-6 { [s[0] / w, s[1] / w] } else { [0.0, 0.0] }).collect();
        Ok(offsets.iter().any(|o| o[0].abs() > 1e-3 || o[1].abs() > 1e-3).then(|| std::sync::Arc::new(offsets)))
    }

    /// Transition の標本: (時刻, 重み)。位置(t) = Σ (E(uₖ₊₁) − E(uₖ)) · 行き先(t − 遅れ − D·uₖ)。時刻はコマに丸める。
    /// 遅れがコマの途中なら、前後のコマへ重みを分ける(遅れが時刻で変わっても位置が跳ばない)。
    /// Duration も遅れも 0 なら空(今の行き先そのまま)。
    fn transition_samples(&self, layer: LayerId, t: RationalTime) -> Result<Vec<(RationalTime, f32)>, StoreError> {
        let Some(comp) = self.composition()? else { return Ok(Vec::new()) };
        let fps = comp.fps;
        let base = self.base_samples(layer, t)?;
        // 自分の移り方を持たない並ぶ子は、容器の移り方を借りる: 容器の箱が移る途中は、その見えている箱の中で並ぶ
        // (CSS で幅が移る間、中身は毎コマその幅で並び直すのと同じ。揃え・伸びは箱の大きさに線形なので、同じ重みで混ぜれば一致する)。
        if base.is_empty() && self.number(layer, TRANSITION_DELAY, 0.0, t)? <= 0.0 {
            if let Some(parent) = self.attrs(layer)?.unwrap_or_default().parent.filter(|p| self.display(*p, t).is_ok_and(|d| d != 0)) {
                return self.transition_samples(parent, t);
            }
        }
        let delay = self.transition_delay(layer, t, &base)? * fps.as_f64();
        if delay <= 1e-6 {
            return base.into_iter().map(|(back, w)| Ok((self.frame_time(back, t)?, w))).collect();
        }
        let base = if base.is_empty() { vec![(0.0, 1.0)] } else { base };
        let mut out = Vec::with_capacity(base.len() * 2);
        for (back, w) in base {
            let at = back + delay;
            let lo = at.floor();
            let frac = (at - lo) as f32;
            out.push((self.frame_time(lo, t)?, w * (1.0 - frac)));
            if frac > 1e-4 {
                out.push((self.frame_time(lo + 1.0, t)?, w * frac));
            }
        }
        Ok(out)
    }

    /// 遅れの無い標本: (遡るコマ数, 重み)。
    fn base_samples(&self, layer: LayerId, t: RationalTime) -> Result<Vec<(f64, f32)>, StoreError> {
        let duration = self.number(layer, TRANSITION_DURATION, 0.0, t)?;
        let Some(comp) = self.composition()? else { return Ok(Vec::new()) };
        let frames = (duration * comp.fps.as_f64()).round();
        if frames < 1.0 {
            return Ok(Vec::new());
        }
        let easing = self.choice(layer, TRANSITION_EASING, t)?;
        // コマごとに 1 つ(時刻をずらしても重みの形が変わらない、畳み込みとして滑らか)。長い移り方だけ間引く。
        let n = frames.min(120.0) as usize;
        Ok((0..n).map(|k| {
            let (u0, u1) = (k as f64 / n as f64, (k + 1) as f64 / n as f64);
            ((frames * u0).round(), (ease(easing, u1) - ease(easing, u0)) as f32)
        }).collect())
    }

    /// 今のコマから `back` コマ前の時刻(0 より前は 0)。
    fn frame_time(&self, back: f64, t: RationalTime) -> Result<RationalTime, StoreError> {
        let Some(comp) = self.composition()? else { return Ok(t) };
        let now = t.try_to_frame_round(comp.fps).map_err(|e| StoreError::Property(e.to_string()))?;
        RationalTime::try_from_frame((now - back as i64).max(0), comp.fps).map_err(|e| StoreError::Property(e.to_string()))
    }

    /// 移り方の遅れ(秒): 自分の Transition Delay + 並べる親の Stagger が配る分。
    /// 配る分は、遅れと長さの窓より前(t − Stagger − Duration)に並んでいた場所の、起点からの距離で決める
    /// (GSAP の stagger が今居る場所で測るのと同じ)。動いている途中の位置で測ると、動くほど遅れが変わって戻る。
    fn transition_delay(&self, layer: LayerId, t: RationalTime, base: &[(f64, f32)]) -> Result<f64, StoreError> {
        let own = self.number(layer, TRANSITION_DELAY, 0.0, t)?.max(0.0);
        let Some(parent) = self.attrs(layer)?.unwrap_or_default().parent else { return Ok(own) };
        let stagger = self.number(parent, STAGGER, 0.0, t)?.max(0.0);
        if stagger <= 0.0 || self.display(parent, t)? == 0 {
            return Ok(own);
        }
        let Some(comp) = self.composition()? else { return Ok(own) };
        let window = base.iter().map(|(back, _)| *back).fold(0.0, f64::max) + 1.0 + ((stagger + own) * comp.fps.as_f64()).ceil();
        let before = self.layout_frame(self.frame_time(window, t)?)?;
        let (Some(size), Some(slot)) = (before.sizes.get(&parent).copied(), before.slots.get(&layer).copied()) else { return Ok(own) };
        let p = glam::Vec2::from(slot.position);
        let (w, h) = (size[0].max(1e-3), size[1].max(1e-3));
        let corner = glam::vec2(CANVAS_MARGIN, CANVAS_MARGIN);
        let diagonal = glam::vec2(w, h).length();
        let from_centre = (p - corner - glam::vec2(w, h) * 0.5).length() / (diagonal * 0.5);
        let reach = match self.choice(parent, STAGGER_FROM, t)? {
            1 => from_centre,
            2 => (p - corner - glam::vec2(w, h)).length() / diagonal,
            3 => 1.0 - from_centre,
            _ => (p - corner).length() / diagonal,
        };
        Ok(own + stagger * f64::from(reach.clamp(0.0, 1.0)))
    }

    fn number(&self, layer: LayerId, name: &str, default: f64, t: RationalTime) -> Result<f64, StoreError> {
        Ok(match self.value_at(layer, &PropertyId::new(name)?, t)? {
            Some(Value::F64(v)) => v,
            Some(Value::Enum(v)) => v as f64,
            _ => default,
        })
    }

    fn choice(&self, layer: LayerId, name: &str, t: RationalTime) -> Result<i64, StoreError> {
        Ok(match self.value_at(layer, &PropertyId::new(name)?, t)? {
            Some(Value::Enum(v)) => v,
            Some(Value::F64(v)) => v.round() as i64,
            _ => 0,
        })
    }

    fn pair(&self, layer: LayerId, name: &str, default: [f32; 2], t: RationalTime) -> Result<[f32; 2], StoreError> {
        Ok(match self.value_at(layer, &PropertyId::new(name)?, t)? {
            Some(Value::Vec2(v)) => [v[0] as f32, v[1] as f32],
            _ => default,
        })
    }

    pub(crate) fn layout_display(&self, layer: LayerId, t: RationalTime) -> Result<i64, StoreError> {
        self.display(layer, t)
    }

    /// 0 = None、1 = Flex、2 = Grid。Group 以外は None。
    fn display(&self, layer: LayerId, t: RationalTime) -> Result<i64, StoreError> {
        if !self.meta(layer)?.is_some_and(|m| m.source == LayerSource::Group) {
            return Ok(0);
        }
        self.choice(layer, DISPLAY, t)
    }

    /// その時刻に居る層か(居ない層は並びに参加しない、`display: none`)。
    fn here(&self, layer: LayerId, t: RationalTime) -> Result<bool, StoreError> {
        let (Some(meta), Some(comp)) = (self.meta(layer)?, self.composition()?) else { return Ok(false) };
        let frame = t.try_to_frame_floor(comp.fps).map_err(|e| StoreError::Property(e.to_string()))?;
        Ok(meta.timing.source_frame(frame).is_some())
    }

    /// 層の箱(素材座標)。形は輪郭の canvas、文字は行の箱、並べない Group は子の箱を合わせた物。
    /// 効果の広がりは含めない。箱を持たない層(Null・Camera・画・動画はまだ)は `None`。
    pub fn layer_box(&self, layer: LayerId, t: RationalTime) -> Result<Option<[f32; 4]>, StoreError> {
        let Some(meta) = self.meta(layer)? else { return Ok(None) };
        Ok(match meta.source {
            LayerSource::Shape => shape_box(&self.shapes_at(layer, t)?),
            LayerSource::Text => self.text_box(layer, t, None)?,
            LayerSource::File { path, .. } => self.analysis().and_then(|a| a.extent(&path)).filter(|e| e[0] > 0.0 && e[1] > 0.0).map(|e| [0.0, 0.0, e[0], e[1]]),
            LayerSource::Group => {
                if self.display(layer, t)? != 0 {
                    return Ok(self.group_size(layer, t)?.map(|s| [CANVAS_MARGIN, CANVAS_MARGIN, CANVAS_MARGIN + s[0], CANVAS_MARGIN + s[1]]));
                }
                let mut acc: Option<[f32; 4]> = None;
                for child in self.layers() {
                    // 中の Display の Group は数えない(並べる途中でここへ来るので、解き直すと巡る)。
                    if self.attrs(child)?.unwrap_or_default().parent != Some(layer) || !self.here(child, t)? || self.display(child, t)? != 0 {
                        continue;
                    }
                    let Some(b) = self.layer_box(child, t)? else { continue };
                    let local = self.local_transform(child, t)?;
                    for corner in [[b[0], b[1]], [b[2], b[1]], [b[0], b[3]], [b[2], b[3]]] {
                        let p = local.transform_point2(glam::Vec2::from(corner));
                        acc = Some(match acc {
                            None => [p.x, p.y, p.x, p.y],
                            Some(a) => [a[0].min(p.x), a[1].min(p.y), a[2].max(p.x), a[3].max(p.y)],
                        });
                    }
                }
                acc
            }
            _ => None,
        })
    }

    fn compute_layout(&self, t: RationalTime) -> Result<Frame, StoreError> {
        let mut frame = Frame::default();
        let layers = self.layers();
        let mut children: HashMap<LayerId, Vec<(i16, LayerId)>> = HashMap::new();
        let mut displayed = Vec::new();
        for &layer in &layers {
            let Some(meta) = self.meta(layer)? else { continue };
            if meta.source == LayerSource::Group && self.display(layer, t)? != 0 {
                displayed.push(layer);
            }
            if let Some(parent) = self.attrs(layer)?.unwrap_or_default().parent {
                if self.here(layer, t)? {
                    children.entry(parent).or_default().push((meta.order, layer));
                }
            }
        }
        if displayed.is_empty() {
            self.push_apart(t, &displayed, &mut frame)?;
            return Ok(frame);
        }
        for list in children.values_mut() {
            list.sort();
        }
        for &root in &displayed {
            let parent = self.attrs(root)?.unwrap_or_default().parent;
            if parent.is_some_and(|p| displayed.contains(&p)) {
                continue;
            }
            let mut tree: TaffyTree<Measure> = TaffyTree::new();
            tree.disable_rounding();
            let (mut leaves, mut groups) = (Vec::new(), Vec::new());
            let node = self.container(&mut tree, root, t, &children, &displayed, true, &mut leaves, &mut groups)?;
            let sizing = self.sizing(root, t)?;
            let available = |axis: usize, name: &str| -> Result<AvailableSpace, StoreError> {
                Ok(if sizing[axis] == Sizing::Fixed { AvailableSpace::Definite(self.number(root, name, 0.0, t)? as f32) } else { AvailableSpace::MaxContent })
            };
            let space = Size { width: available(0, WIDTH)?, height: available(1, HEIGHT)? };
            tree.compute_layout_with_measure(node, space, |known, available, _, measure, _| match measure {
                Some(m) => self.measure_text(*m, known, available, t),
                None => Size::ZERO,
            })
            .map_err(|e| StoreError::Property(format!("layout: {e}")))?;
            if self.exclude_blobs(&mut tree, &groups, t)? {
                tree.compute_layout_with_measure(node, space, |known, available, _, measure, _| match measure {
                    Some(m) => self.measure_text(*m, known, available, t),
                    None => Size::ZERO,
                })
                .map_err(|e| StoreError::Property(format!("layout: {e}")))?;
            }
            let taffy = |e: taffy::TaffyError| StoreError::Property(format!("layout: {e}"));
            let shifted = |mut placed: taffy::Layout| { placed.location.x += CANVAS_MARGIN; placed.location.y += CANVAS_MARGIN; placed };
            let mut groups_order = groups.clone();
            for (node, layer, is_root) in groups {
                let placed = shifted(*tree.layout(node).map_err(taffy)?);
                frame.sizes.insert(layer, [placed.size.width, placed.size.height]);
                if !is_root {
                    let bounds = [CANVAS_MARGIN, CANVAS_MARGIN, CANVAS_MARGIN + placed.size.width, CANVAS_MARGIN + placed.size.height];
                    let slot = self.slot(layer, t, bounds, [Sizing::Hug; 2], 3, placed)?;
                    frame.slots.insert(layer, slot);
                }
            }
            for leaf in leaves {
                let placed = shifted(*tree.layout(leaf.node).map_err(taffy)?);
                let slot = if leaf.text_fill {
                    let scale = self.pair(leaf.layer, property::SCALE, [1.0, 1.0], t)?[0].abs().max(1e-3);
                    let wrap = placed.size.width / scale;
                    let bounds = self.text_box(leaf.layer, t, Some(wrap))?.unwrap_or(leaf.bounds);
                    Slot { wrap: Some(wrap), ..self.slot(leaf.layer, t, bounds, [Sizing::Hug; 2], 3, placed)? }
                } else {
                    self.slot(leaf.layer, t, leaf.bounds, leaf.sizing, leaf.fit, placed)?
                };
                frame.slots.insert(leaf.layer, slot);
            }
            // 奥行きの揃えは内の Group から(外の Group は内の奥行きを子の奥行きとして読む)。
            for (_, group, _) in groups_order {
                self.align_depth(group, t, &children, &mut frame)?;
            }
        }
        self.push_apart(t, &displayed, &mut frame)?;
        Ok(frame)
    }

    /// 容器の外の兄弟同士の押し合い(間合いの法 2・3): Margin を宣言した物の箱(親の空間、間合いで広げる)の重なりを、
    /// 決まった回数だけ押し戻す。中心から中心への向きへ、Flex Shrink の比で分ける。その瞬間の宣言だけから解く。
    fn push_apart(&self, t: RationalTime, displayed: &[LayerId], frame: &mut Frame) -> Result<(), StoreError> {
        const ROUNDS: usize = 32;
        let mut families: HashMap<Option<LayerId>, Vec<(LayerId, [f32; 4], f32)>> = HashMap::new();
        for layer in self.layers() {
            let margin = self.number(layer, MARGIN, 0.0, t)? as f32;
            if margin <= 0.0 || !self.here(layer, t)? {
                continue;
            }
            let parent = self.attrs(layer)?.unwrap_or_default().parent;
            if parent.is_some_and(|p| displayed.contains(&p)) {
                continue;
            }
            // 並べる Group の箱は今解いた大きさ(覚えにはまだ入っていない)。
            let b = match frame.sizes.get(&layer) {
                Some(size) => [CANVAS_MARGIN, CANVAS_MARGIN, CANVAS_MARGIN + size[0], CANVAS_MARGIN + size[1]],
                None => match self.layer_box(layer, t)? { Some(b) => b, None => continue },
            };
            // 押し合いの出発点は書いた位置(鍵・親)。ずれを含めた変換を読むと、前の時刻のずれを辿って巡る。
            let local = self.authored_local(layer, t)?;
            let corners = [[b[0], b[1]], [b[2], b[1]], [b[0], b[3]], [b[2], b[3]]].map(|c| local.transform_point2(glam::Vec2::from(c)));
            let lo = corners.iter().fold(glam::Vec2::MAX, |a, p| a.min(*p)) - glam::Vec2::splat(margin);
            let hi = corners.iter().fold(glam::Vec2::MIN, |a, p| a.max(*p)) + glam::Vec2::splat(margin);
            let shrink = self.number(layer, FLEX_SHRINK, 1.0, t)?.max(0.0) as f32;
            families.entry(parent).or_default().push((layer, [lo.x, lo.y, hi.x, hi.y], shrink));
        }
        for (_, mut items) in families {
            if items.len() < 2 {
                continue;
            }
            items.sort_by_key(|item| item.0);
            let mut moved = vec![[0.0f32; 2]; items.len()];
            // 各回で全部の対を今の箱から同時に測って、まとめて動かす(順に動かすと、対の順番と止まる回で結果が跳ぶ)。
            for _ in 0..ROUNDS {
                let mut step = vec![glam::Vec2::ZERO; items.len()];
                for i in 0..items.len() {
                    for j in i + 1..items.len() {
                        let (a, b) = (items[i].1, items[j].1);
                        let (si, sj) = (items[i].2, items[j].2);
                        if si + sj <= 0.0 {
                            continue;
                        }
                        let (wi, wj) = (si / (si + sj), sj / (si + sj));
                        // 中心から中心への向きに、離れるのに要るだけ押す(浅い軸で押すと、軸が入れ替わる瞬間に向きが 90° 跳ぶ)。
                        let centre = |r: [f32; 4]| glam::vec2((r[0] + r[2]) * 0.5, (r[1] + r[3]) * 0.5);
                        let gap = centre(b) - centre(a);
                        let dir = if gap.length() > 1e-4 { gap.normalize() } else { glam::Vec2::X };
                        let half = glam::vec2((a[2] - a[0] + b[2] - b[0]) * 0.5, (a[3] - a[1] + b[3] - b[1]) * 0.5);
                        let need = |axis: usize| {
                            let (g, u, h) = (gap[axis], dir[axis], half[axis]);
                            if u.abs() < 1e-6 { f32::INFINITY } else { ((h - g.abs()) / u.abs()).max(0.0) }
                        };
                        let depth = need(0).min(need(1));
                        if !depth.is_finite() || depth <= 0.0 {
                            continue;
                        }
                        step[i] -= dir * depth * wi * 0.5;
                        step[j] += dir * depth * wj * 0.5;
                    }
                }
                for (k, d) in step.into_iter().enumerate() {
                    items[k].1[0] += d.x;
                    items[k].1[2] += d.x;
                    items[k].1[1] += d.y;
                    items[k].1[3] += d.y;
                    moved[k][0] += d.x;
                    moved[k][1] += d.y;
                }
            }
            for (item, d) in items.iter().zip(moved) {
                if d != [0.0, 0.0] {
                    frame.nudges.insert(item.0, d);
                }
            }
        }
        Ok(())
    }

    /// 書いた値だけの層の変換(並べた結果・押し合いのずれを含まない)。
    fn authored_local(&self, layer: LayerId, t: RationalTime) -> Result<glam::Affine2, StoreError> {
        Ok(crate::doc::core::LayerPlacement::from_transform(
            self.pair(layer, property::ANCHOR, [0.0, 0.0], t)?,
            self.resolve_position(layer, t)?,
            self.pair(layer, property::SCALE, [1.0, 1.0], t)?,
            self.number(layer, property::ROTATION, 0.0, t)? as f32,
            self.number(layer, property::SKEW, 0.0, t)? as f32,
            self.number(layer, property::SKEW_AXIS, 0.0, t)? as f32,
        ))
    }

    /// 折り返す文字が避ける物(同じ親で Shape Outside を持つ兄弟)を、文字の枠の座標の多角形で。無ければ None。
    /// 物が Transition を持てば、少し前の時刻の形も重みつきで渡す(文字の組みは重みの過半が覆う所を避ける)。
    pub(crate) fn flow_around(&self, text: LayerId, t: RationalTime) -> Result<Option<std::sync::Arc<Vec<crate::doc::store::text_frame::Obstacle>>>, StoreError> {
        if !self.meta(text)?.is_some_and(|m| m.source == LayerSource::Text) {
            return Ok(None);
        }
        let parent = self.attrs(text)?.unwrap_or_default().parent;
        let mut to_text: Option<glam::Affine2> = None;
        let mut out = Vec::new();
        for layer in self.layers() {
            if layer == text || self.attrs(layer)?.unwrap_or_default().parent != parent {
                continue;
            }
            let mode = self.choice(layer, SHAPE_OUTSIDE, t)?;
            if mode == 0 || !self.here(layer, t)? {
                continue;
            }
            if to_text.is_none() {
                if self.resolved_text_document(text, t)?.and_then(|d| d.wrap_size).is_none() {
                    return Ok(None);
                }
                to_text = Some(self.world_2d(text, t)?.inverse());
            }
            let to_text = to_text.unwrap_or(glam::Affine2::IDENTITY);
            let margin = self.number(layer, SHAPE_MARGIN, 0.0, t)?.max(0.0) as f32;
            let mut samples = self.transition_samples(layer, t)?;
            if samples.is_empty() {
                samples.push((t, 1.0));
            }
            for (at, weight) in samples {
                for poly in self.declared_shape(layer, mode, at)? {
                    if poly.len() >= 3 {
                        out.push(crate::doc::store::text_frame::Obstacle { margin, weight, points: poly.into_iter().map(|p| to_text.transform_point2(p).to_array()).collect() });
                    }
                }
            }
        }
        Ok((!out.is_empty()).then(|| std::sync::Arc::new(out)))
    }

    /// Shape Outside が宣言する形(comp の多角形)。Margin Box = 箱 + Margin、Content = 輪郭か Blob の塊の箱。
    fn declared_shape(&self, layer: LayerId, mode: i64, t: RationalTime) -> Result<Vec<Vec<glam::Vec2>>, StoreError> {
        let rect = |b: [f32; 4]| vec![glam::vec2(b[0], b[1]), glam::vec2(b[2], b[1]), glam::vec2(b[2], b[3]), glam::vec2(b[0], b[3])];
        if mode == 2 {
            if let Some(marks) = self.analysis().and_then(|a| a.blobs(layer, crate::doc::store::EffectId(0), t)) {
                return Ok(marks.iter().map(|mark| {
                    let (c, h) = (glam::Vec2::from(mark.center), glam::Vec2::from(mark.size) * 0.5);
                    rect([c.x - h.x, c.y - h.y, c.x + h.x, c.y + h.y])
                }).collect());
            }
        }
        let world = self.world_2d(layer, t)?;
        let local = self.outline(layer, t)?;
        if mode == 1 {
            let grow = self.number(layer, MARGIN, 0.0, t)? as f32;
            let (lo, hi) = local.iter().flatten().fold((glam::Vec2::MAX, glam::Vec2::MIN), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
            if lo.x > hi.x {
                return Ok(Vec::new());
            }
            return Ok(vec![rect([lo.x - grow, lo.y - grow, hi.x + grow, hi.y + grow]).into_iter().map(|p| world.transform_point2(p)).collect()]);
        }
        Ok(local.into_iter().map(|poly| poly.into_iter().map(|p| world.transform_point2(p)).collect()).collect())
    }

    /// 層の輪郭(素材座標の多角形)。形は曲線を刻んだ輪郭(並べた伸びを込み)、他は層の箱。
    fn outline(&self, layer: LayerId, t: RationalTime) -> Result<Vec<Vec<glam::Vec2>>, StoreError> {
        let Some(meta) = self.meta(layer)? else { return Ok(Vec::new()) };
        if meta.source == LayerSource::Shape {
            let stretch = self.laid_out(layer, t)?.map_or([1.0, 1.0], |slot| slot.stretch);
            let shapes = crate::doc::vector::stretch_outline(&self.shapes_at(layer, t)?, stretch);
            let Ok(Some(canvas)) = crate::doc::vector::content_canvas(&shapes) else { return Ok(Vec::new()) };
            let origin = glam::vec2(canvas.origin_x as f32, canvas.origin_y as f32);
            let mut out = Vec::new();
            for shape in crate::doc::vector::flatten(&shapes).unwrap_or_default() {
                for instance in crate::doc::vector::resolve(&shape).unwrap_or_default() {
                    for contour in &instance.path {
                        let n = contour.vertices.len();
                        let mut poly = Vec::new();
                        let p = |v: crate::doc::vector::Point| glam::vec2(v.x as f32, v.y as f32);
                        for i in 0..if contour.closed { n } else { n.saturating_sub(1) } {
                            let (a, b) = (&contour.vertices[i], &contour.vertices[(i + 1) % n]);
                            let (p0, p3) = (p(a.point), p(b.point));
                            let (p1, p2) = (p0 + p(a.out_tangent), p3 + p(b.in_tangent));
                            for k in 0..8 {
                                let u = k as f32 / 8.0;
                                let w = 1.0 - u;
                                poly.push(p0 * w * w * w + p1 * 3.0 * w * w * u + p2 * 3.0 * w * u * u + p3 * u * u * u + origin);
                            }
                        }
                        out.push(poly);
                    }
                }
            }
            return Ok(out);
        }
        Ok(self.layer_box(layer, t)?.map(|b| vec![vec![glam::vec2(b[0], b[1]), glam::vec2(b[2], b[1]), glam::vec2(b[2], b[3]), glam::vec2(b[0], b[3])]]).unwrap_or_default())
    }

    /// Display の Group の背景(Background の色が透明でなければ)。角は Border Radius。描くのは形の層と同じ道。
    pub fn background_shapes(&self, group: LayerId, t: RationalTime) -> Result<Option<Vec<crate::doc::vector::ShapeNode>>, StoreError> {
        use crate::doc::vector::{Brush, Fill, FillRule, OpKind, PathSource, Point, RepeaterTransform, Rgb, Shape, ShapeGroup, ShapeNode, ShapeOp};
        if self.display(group, t)? == 0 {
            return Ok(None);
        }
        let color = match self.value_at(group, &PropertyId::new(BACKGROUND)?, t)? {
            Some(Value::Color(c)) => c,
            _ => return Ok(None),
        };
        let Some(size) = self.group_size(group, t)? else { return Ok(None) };
        if color[3] <= 0.0 || size[0] <= 0.0 || size[1] <= 0.0 {
            return Ok(None);
        }
        let radius = self.number(group, BORDER_RADIUS, 0.0, t)?.max(0.0);
        let (w, h) = (f64::from(size[0]), f64::from(size[1]));
        let leaf = ShapeNode::Leaf(Shape {
            source: PathSource::Rectangle { size: Point { x: w, y: h } },
            ops: if radius > 0.0 { vec![ShapeOp::new(OpKind::RoundedCorners { radius: radius.min(w.min(h) * 0.5) })] } else { Vec::new() },
            fill: Some(Fill { brush: Brush::Solid(Rgb { r: color[0], g: color[1], b: color[2] }), rule: FillRule::NonZero, opacity: color[3], hidden: false }),
            stroke: None,
        });
        Ok(Some(vec![ShapeNode::Group(ShapeGroup {
            transform: RepeaterTransform { position: Point { x: w * 0.5, y: h * 0.5 }, ..RepeaterTransform::IDENTITY },
            children: vec![leaf],
        })]))
    }

    /// Exclusions: 格子の Group が指す Blob Track の塊が重なる枠へ、見えない子を明示の位置で置く。自動の子は残りの枠へ流れる。
    /// 塊は comp の px なので、Group の素材座標へ戻してから枠と比べる。置いたら true(並べ直す)。
    fn exclude_blobs(&self, tree: &mut TaffyTree<Measure>, groups: &[(NodeId, LayerId, bool)], t: RationalTime) -> Result<bool, StoreError> {
        let taffy = |e: taffy::TaffyError| StoreError::Property(format!("layout: {e}"));
        let mut changed = false;
        for &(node, group, _) in groups {
            if self.display(group, t)? != 2 {
                continue;
            }
            let source = match self.value_at(group, &PropertyId::new(EXCLUSIONS)?, t)? {
                // 層を指す欄は窓から数として届くこともある(Blob Track の Track Layer と同じ読み方)。
                Some(Value::LayerId(id)) if id != 0 => LayerId(id),
                Some(Value::F64(v)) if v >= 1.0 => LayerId(v.round() as u64),
                _ => continue,
            };
            let Some(marks) = self.analysis().and_then(|a| a.blobs(source, crate::doc::store::EffectId(0), t)) else { continue };
            let taffy::tree::DetailedLayoutInfo::Grid(info) = tree.detailed_layout_info(node) else { continue };
            let lines = |tracks: &taffy::compute::detailed_info::DetailedGridTracksInfo, start: f32| {
                let mut at = start;
                let mut out = Vec::new();
                for (i, size) in tracks.sizes.iter().enumerate() {
                    at += tracks.gutters.get(i).copied().unwrap_or(0.0);
                    out.push((at, at + size));
                    at += size;
                }
                out
            };
            let padding = tree.layout(node).map_err(taffy)?.padding;
            let columns = lines(&info.columns, padding.left + CANVAS_MARGIN);
            let rows = lines(&info.rows, padding.top + CANVAS_MARGIN);
            let (skip_c, skip_r) = (info.columns.negative_implicit_tracks as usize, info.rows.negative_implicit_tracks as usize);
            let (n_c, n_r) = (info.columns.explicit_tracks as usize, info.rows.explicit_tracks as usize);
            let to_local = self.world_2d(group, t)?.inverse();
            let mut taken = std::collections::BTreeSet::new();
            for mark in marks {
                let half = glam::Vec2::from(mark.size) * 0.5;
                let (lo, hi) = (glam::Vec2::from(mark.center) - half, glam::Vec2::from(mark.center) + half);
                let corners = [lo, glam::vec2(hi.x, lo.y), glam::vec2(lo.x, hi.y), hi].map(|p| to_local.transform_point2(p));
                let min = corners.iter().fold(glam::Vec2::MAX, |a, p| a.min(*p));
                let max = corners.iter().fold(glam::Vec2::MIN, |a, p| a.max(*p));
                for (c, &(c0, c1)) in columns.iter().enumerate().skip(skip_c).take(n_c) {
                    for (r, &(r0, r1)) in rows.iter().enumerate().skip(skip_r).take(n_r) {
                        if min.x < c1 && max.x > c0 && min.y < r1 && max.y > r0 {
                            taken.insert((c - skip_c, r - skip_r));
                        }
                    }
                }
            }
            for (c, r) in taken {
                let blocker = Style {
                    grid_column: Line { start: GridPlacement::from_line_index(c as i16 + 1), end: GridPlacement::from_span(1) },
                    grid_row: Line { start: GridPlacement::from_line_index(r as i16 + 1), end: GridPlacement::from_span(1) },
                    ..Style::default()
                };
                let leaf = tree.new_leaf(blocker).map_err(taffy)?;
                tree.add_child(node, leaf).map_err(taffy)?;
                changed = true;
            }
        }
        Ok(changed)
    }

    /// 層の 2D の world(親を辿る)。塊を Group の素材座標へ戻す時。
    fn world_2d(&self, layer: LayerId, t: RationalTime) -> Result<glam::Affine2, StoreError> {
        let mut world = self.local_transform(layer, t)?;
        let mut seen = std::collections::HashSet::from([layer]);
        let mut next = self.attrs(layer)?.unwrap_or_default().parent;
        while let Some(parent) = next.filter(|p| seen.insert(*p)) {
            world = self.local_transform(parent, t)? * world;
            next = self.attrs(parent)?.unwrap_or_default().parent;
        }
        Ok(world)
    }

    /// 文字の行の箱。`wrap` があればその幅で折り返して組み、横は [0, wrap](CSS の block の幅、揃えはその中)。
    fn text_box(&self, layer: LayerId, t: RationalTime, wrap: Option<f32>) -> Result<Option<[f32; 4]>, StoreError> {
        let (Some(mut document), Some(comp)) = (self.authored_text_document(layer, t)?, self.composition()?) else { return Ok(None) };
        let canvas = crate::doc::vector::Canvas { width: comp.width, height: comp.height, origin_x: 0, origin_y: 0 };
        if let Some(width) = wrap {
            document.wrap_size = Some([width.max(1.0), comp.height as f32]);
        }
        let shaped = crate::doc::store::text_frame::shape_document(&document, t, &canvas).ok().flatten();
        Ok(shaped.and_then(|shaped| crate::doc::store::text_frame::line_box(&document, &shaped, &canvas)).map(|b| match wrap {
            Some(width) => [0.0, b[1], width.max(1.0), b[3]],
            None => b,
        }))
    }

    /// 横が Fill の文字: 決まった幅で折り返した高さ(Scale の zoom 込み)。
    fn measure_text(&self, measure: Measure, known: Size<Option<f32>>, available: Size<AvailableSpace>, t: RationalTime) -> Size<f32> {
        let width = known.width.or(match available.width {
            AvailableSpace::Definite(w) => Some(w),
            _ => None,
        });
        let wrap = width.map(|w| w / measure.scale);
        match self.text_box(measure.layer, t, wrap).ok().flatten() {
            Some(b) => Size { width: width.unwrap_or((b[2] - b[0]) * measure.scale), height: known.height.unwrap_or((b[3] - b[1]) * measure.scale) },
            None => Size::ZERO,
        }
    }

    /// Overflow が Clip の並べる Group なら、その箱(素材座標)と角の丸み。
    pub(crate) fn clip_box(&self, group: LayerId, t: RationalTime) -> Result<Option<([f32; 4], f32)>, StoreError> {
        if self.display(group, t)? == 0 || self.choice(group, OVERFLOW, t)? != 1 {
            return Ok(None);
        }
        let Some(size) = self.group_size(group, t)? else { return Ok(None) };
        let b = [CANVAS_MARGIN, CANVAS_MARGIN, CANVAS_MARGIN + size[0], CANVAS_MARGIN + size[1]];
        Ok(Some((b, self.number(group, BORDER_RADIUS, 0.0, t)?.max(0.0) as f32)))
    }

    /// 層の奥行きの範囲(素材座標の z、[手前, 奥])。平らな物は [0, 0]、押し出しは [0, Depth]、網・点群は bounds の
    /// 奥行きを中心に、並べる Group は [-奥行き, 0]。Scale Z(と 3D の Object Fit)を掛ける。
    fn depth_range(&self, layer: LayerId, t: RationalTime, frame: &Frame) -> Result<[f32; 2], StoreError> {
        let Some(meta) = self.meta(layer)? else { return Ok([0.0; 2]) };
        let scale_z = self.number(layer, property::SCALE_Z, 1.0, t)? as f32 * frame.slots.get(&layer).map_or(1.0, |s| s.scale_z);
        let range = match &meta.source {
            LayerSource::Shape | LayerSource::Text => [0.0, self.number(layer, property::DEPTH, 0.0, t)?.max(0.0) as f32],
            LayerSource::File { path, .. } => {
                // 描く側と同じ: 奥行きは xy の拡縮の平均で伸びる(depth_scaled)。
                let scale = frame.slots.get(&layer).map_or(self.pair(layer, property::SCALE, [1.0, 1.0], t)?, |s| s.scale);
                let d = self.analysis().and_then(|a| a.extent(path)).map_or(0.0, |e| e[2]) * (scale[0].abs() + scale[1].abs()) * 0.5;
                [-d * 0.5, d * 0.5]
            }
            LayerSource::Group => frame.depths.get(&layer).copied().unwrap_or([0.0, 0.0]),
            _ => [0.0, 0.0],
        };
        let range = [range[0] * scale_z, range[1] * scale_z];
        match frame.slots.get(&layer).map(|s| s.rotation).filter(|r| *r != [0.0; 3]) {
            Some(rotation) => {
                let bounds = match meta.source {
                    LayerSource::Group => frame.sizes.get(&layer).map_or([0.0; 4], |s| [0.0, 0.0, s[0], s[1]]),
                    _ => self.layer_box(layer, t)?.unwrap_or([0.0; 4]),
                };
                let scale = self.pair(layer, property::SCALE, [1.0, 1.0], t)?;
                let anchor = self.item_anchor(layer, t, bounds)?;
                let (lo, hi) = footprint(bounds, [range[0], range[1]], anchor, [scale[0], scale[1], 1.0], rotation);
                Ok([lo[2], hi[2]])
            }
            None => Ok(range),
        }
    }

    /// 並ぶ子の拡縮・回転の中心。Anchor を書いた層はその値、書いていなければ箱の中心(CSS の transform-origin)。
    fn item_anchor(&self, layer: LayerId, t: RationalTime, bounds: [f32; 4]) -> Result<[f32; 2], StoreError> {
        Ok(match self.value_at(layer, &PropertyId::new(property::ANCHOR)?, t)? {
            Some(Value::Vec2(v)) => [v[0] as f32, v[1] as f32],
            _ => [(bounds[0] + bounds[2]) * 0.5, (bounds[1] + bounds[3]) * 0.5],
        })
    }

    /// 並べる前の奥行きの範囲(Scale Z 込み、Object Fit と回転は無し)。葉の箱を測る時。
    fn raw_depth(&self, layer: LayerId, t: RationalTime) -> Result<[f32; 2], StoreError> {
        self.depth_range(layer, t, &Frame::default())
    }

    /// 回転の欄(Tilt X・Tilt Y・Rotation)。
    fn layout_rotation(&self, layer: LayerId, t: RationalTime) -> Result<[f32; 3], StoreError> {
        Ok([self.number(layer, LAYOUT_TILT_X, 0.0, t)? as f32, self.number(layer, LAYOUT_TILT_Y, 0.0, t)? as f32, self.number(layer, LAYOUT_ROTATION, 0.0, t)? as f32])
    }

    /// Depth Alignment: 子の奥行きの最大が Group の奥行き。Back = 子の奥を面(z = 0)に、Front = 子の手前を
    /// Group の手前(-奥行き)に、Center = 中心を揃える(visionOS の depthAlignment)。
    fn align_depth(&self, group: LayerId, t: RationalTime, children: &HashMap<LayerId, Vec<(i16, LayerId)>>, frame: &mut Frame) -> Result<(), StoreError> {
        let alignment = self.choice(group, DEPTH_ALIGNMENT, t)?;
        let mut ranges = Vec::new();
        for &(_, child) in children.get(&group).map(Vec::as_slice).unwrap_or(&[]) {
            if frame.slots.contains_key(&child) {
                ranges.push((child, self.depth_range(child, t, frame)?));
            }
        }
        if self.display(group, t)? == 1 && self.choice(group, FLEX_DIRECTION, t)? == DIRECTION_DEPTH {
            // 奥へ積む: 重ね順の上(番号の大きい物)が一番手前、面(z = 0)に手前を合わせ、奥行き + Gap ずつ奥へ。
            let gap = self.number(group, GAP, 0.0, t)? as f32;
            let mut cursor = 0.0f32;
            for (child, [front, back]) in ranges.into_iter().rev() {
                if let Some(slot) = frame.slots.get_mut(&child) {
                    slot.z = cursor - front;
                }
                cursor += back - front + gap;
            }
            frame.depths.insert(group, [0.0, (cursor - gap).max(0.0)]);
            return Ok(());
        }
        let depth = ranges.iter().map(|(_, r)| r[1] - r[0]).fold(0.0f32, f32::max);
        for (child, [front, back]) in ranges {
            let z = match alignment {
                1 => -depth * 0.5 - (front + back) * 0.5,
                2 => -depth - front,
                _ => -back,
            };
            if let Some(slot) = frame.slots.get_mut(&child) {
                slot.z = z;
            }
        }
        frame.depths.insert(group, [-depth, 0.0]);
        Ok(())
    }

    fn sizing(&self, layer: LayerId, t: RationalTime) -> Result<[Sizing; 2], StoreError> {
        let of = |v: i64| match v { 1 => Sizing::Fill, 2 => Sizing::Fixed, _ => Sizing::Hug };
        Ok([of(self.choice(layer, HORIZONTAL_SIZING, t)?), of(self.choice(layer, VERTICAL_SIZING, t)?)])
    }

    /// 子としての style(並ぶ側の欄)。`natural` は Hug の時の大きさ。
    fn item_style(&self, layer: LayerId, t: RationalTime, natural: [f32; 2], style: &mut Style) -> Result<[Sizing; 2], StoreError> {
        let sizing = self.sizing(layer, t)?;
        let fixed = [self.number(layer, WIDTH, 100.0, t)? as f32, self.number(layer, HEIGHT, 100.0, t)? as f32];
        let dimension = |axis: usize| match sizing[axis] {
            Sizing::Hug => Dimension::length(natural[axis]),
            Sizing::Fixed => Dimension::length(fixed[axis]),
            Sizing::Fill => Dimension::auto(),
        };
        style.size = Size { width: dimension(0), height: dimension(1) };
        style.min_size = Size { width: Dimension::length(0.0), height: Dimension::length(0.0) };
        // CSS の既定は 1 だが、書いていない子は縮めない(Hug の箱を潰さない)。書いた比だけ譲る。
        style.flex_shrink = match self.value_at(layer, &PropertyId::new(FLEX_SHRINK)?, t)? {
            Some(Value::F64(v)) => v.max(0.0) as f32,
            _ => 0.0,
        };
        let margin = self.number(layer, MARGIN, 0.0, t)?.max(0.0) as f32;
        if margin > 0.0 {
            let m = LengthPercentageAuto::length(margin);
            style.margin = Rect { left: m, right: m, top: m, bottom: m };
        }
        if sizing.contains(&Sizing::Fill) {
            let row = matches!(self.choice_of_parent(layer, FLEX_DIRECTION, t)?, 0 | 2);
            let main = if row { 0 } else { 1 };
            if sizing[main] == Sizing::Fill {
                style.flex_grow = 1.0;
                style.flex_basis = Dimension::length(0.0);
            }
            if sizing[1 - main] == Sizing::Fill {
                style.align_self = Some(AlignSelf::STRETCH);
            }
        }
        match self.choice(layer, ALIGN_SELF, t)? {
            1 => style.align_self = Some(AlignSelf::STRETCH),
            2 => style.align_self = Some(AlignSelf::FLEX_START),
            3 => style.align_self = Some(AlignSelf::FLEX_END),
            4 => style.align_self = Some(AlignSelf::CENTER),
            _ => {}
        }
        let line = |start: f64, span: f64| -> Line<GridPlacement> {
            let span = span.round().max(1.0) as u16;
            if start >= 1.0 {
                Line { start: GridPlacement::from_line_index(start.round() as i16), end: GridPlacement::from_span(span) }
            } else {
                Line { start: GridPlacement::Auto, end: GridPlacement::from_span(span) }
            }
        };
        style.grid_column = line(self.number(layer, COLUMN_START, 0.0, t)?, self.number(layer, COLUMN_SPAN, 1.0, t)?);
        style.grid_row = line(self.number(layer, ROW_START, 0.0, t)?, self.number(layer, ROW_SPAN, 1.0, t)?);
        if self.choice_of_parent(layer, FLEX_DIRECTION, t)? == DIRECTION_DEPTH && self.attrs(layer)?.unwrap_or_default().parent.map(|p| self.display(p, t)).transpose()? == Some(1) {
            style.grid_column = line(1.0, 1.0);
            style.grid_row = line(1.0, 1.0);
        }
        Ok(sizing)
    }

    fn choice_of_parent(&self, layer: LayerId, name: &str, t: RationalTime) -> Result<i64, StoreError> {
        match self.attrs(layer)?.unwrap_or_default().parent {
            Some(parent) => self.choice(parent, name, t),
            None => Ok(0),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn container(
        &self,
        tree: &mut TaffyTree<Measure>,
        group: LayerId,
        t: RationalTime,
        children: &HashMap<LayerId, Vec<(i16, LayerId)>>,
        displayed: &[LayerId],
        is_root: bool,
        leaves: &mut Vec<Leaf>,
        groups: &mut Vec<(NodeId, LayerId, bool)>,
    ) -> Result<NodeId, StoreError> {
        let display = self.display(group, t)?;
        let padding = self.pair(group, PADDING, [0.0, 0.0], t)?;
        let gap = self.number(group, GAP, 0.0, t)? as f32;
        let mut style = Style {
            display: if display == 2 { Display::Grid } else { Display::Flex },
            flex_direction: match self.choice(group, FLEX_DIRECTION, t)? { 1 => FlexDirection::Column, 2 => FlexDirection::RowReverse, 3 => FlexDirection::ColumnReverse, _ => FlexDirection::Row },
            flex_wrap: match self.choice(group, FLEX_WRAP, t)? { 1 => FlexWrap::Wrap, 2 => FlexWrap::WrapReverse, _ => FlexWrap::NoWrap },
            justify_content: Some(match self.choice(group, JUSTIFY_CONTENT, t)? { 1 => JustifyContent::FLEX_END, 2 => JustifyContent::CENTER, 3 => JustifyContent::SPACE_BETWEEN, 4 => JustifyContent::SPACE_AROUND, 5 => JustifyContent::SPACE_EVENLY, _ => JustifyContent::FLEX_START }),
            align_items: Some(match self.choice(group, ALIGN_ITEMS, t)? { 1 => AlignItems::FLEX_START, 2 => AlignItems::FLEX_END, 3 => AlignItems::CENTER, _ => AlignItems::STRETCH }),
            gap: Size { width: LengthPercentage::length(gap), height: LengthPercentage::length(gap) },
            padding: Rect {
                left: LengthPercentage::length(padding[0]),
                right: LengthPercentage::length(padding[0]),
                top: LengthPercentage::length(padding[1]),
                bottom: LengthPercentage::length(padding[1]),
            },
            ..Style::default()
        };
        if display == 1 && self.choice(group, FLEX_DIRECTION, t)? == DIRECTION_DEPTH {
            // 面の上では全員が 1 つの枠に重なる(奥行きは後で積む)。揃えは Align Items を縦横に。
            style.display = Display::Grid;
            style.grid_template_columns = vec![GridTemplateComponent::Single(TrackSizingFunction::AUTO)];
            style.grid_template_rows = vec![GridTemplateComponent::Single(TrackSizingFunction::AUTO)];
            style.justify_items = style.align_items;
        }
        if display == 2 {
            let tracks = |count: &str, prefix: &str, default: f64| -> Result<Vec<GridTemplateComponent<String>>, StoreError> {
                let n = self.number(group, count, default, t)?.round().clamp(0.0, 64.0) as u32;
                (1..=n).map(|i| Ok(fr(self.number(group, &format!("{prefix}{i}"), TRACK_DEFAULT, t)?.max(0.0) as f32))).collect()
            };
            style.grid_template_columns = tracks(GRID_COLUMNS, COLUMN_PREFIX, 2.0)?;
            style.grid_template_rows = tracks(GRID_ROWS, ROW_PREFIX, 0.0)?;
        }
        if is_root {
            let sizing = self.sizing(group, t)?;
            let fixed = |axis: usize, name: &str| -> Result<Dimension, StoreError> {
                Ok(if sizing[axis] == Sizing::Fixed { Dimension::length(self.number(group, name, 0.0, t)? as f32) } else { Dimension::auto() })
            };
            style.size = Size { width: fixed(0, WIDTH)?, height: fixed(1, HEIGHT)? };
        } else {
            let sizing = self.item_style(group, t, [0.0, 0.0], &mut style)?;
            let auto = |axis: usize, d: Dimension| if sizing[axis] == Sizing::Hug { Dimension::auto() } else { d };
            style.size = Size { width: auto(0, style.size.width), height: auto(1, style.size.height) };
        }
        let mut nodes = Vec::new();
        for &(_, child) in children.get(&group).map(Vec::as_slice).unwrap_or(&[]) {
            if self.choice(child, POSITION_TYPE, t)? == 1 {
                continue;
            }
            if displayed.contains(&child) {
                nodes.push(self.container(tree, child, t, children, displayed, false, leaves, groups)?);
                continue;
            }
            let bounds = self.layer_box(child, t)?.unwrap_or([0.0; 4]);
            let scale = self.pair(child, property::SCALE, [1.0, 1.0], t)?;
            let (lo, hi) = footprint(bounds, self.raw_depth(child, t)?, [0.0, 0.0], [scale[0], scale[1], 1.0], self.layout_rotation(child, t)?);
            let natural = [hi[0] - lo[0], hi[1] - lo[1]];
            let mut item = Style::default();
            let sizing = self.item_style(child, t, natural, &mut item)?;
            let text_fill = sizing[0] == Sizing::Fill && self.meta(child)?.is_some_and(|m| m.source == LayerSource::Text);
            let node = if text_fill {
                if sizing[1] == Sizing::Hug {
                    item.size.height = Dimension::auto();
                }
                tree.new_leaf_with_context(item, Measure { layer: child, scale: scale[0].abs().max(1e-3) })
            } else {
                tree.new_leaf(item)
            }
            .map_err(|e| StoreError::Property(format!("layout: {e}")))?;
            leaves.push(Leaf { node, text_fill, layer: child, bounds, sizing, fit: self.choice(child, OBJECT_FIT, t)? });
            nodes.push(node);
        }
        let node = tree.new_with_children(style, &nodes).map_err(|e| StoreError::Property(format!("layout: {e}")))?;
        groups.push((node, group, is_root));
        Ok(node)
    }

    /// 置かれた枠へ、層の箱を合わせる Position と Scale(と形の輪郭の伸び)。Position の値はずれとして足す。
    fn slot(&self, layer: LayerId, t: RationalTime, bounds: [f32; 4], sizing: [Sizing; 2], fit: i64, placed: taffy::Layout) -> Result<Slot, StoreError> {
        let scale = self.pair(layer, property::SCALE, [1.0, 1.0], t)?;
        let offset = self.resolve_position(layer, t)?;
        let cell = [placed.size.width, placed.size.height];
        let natural = [(bounds[2] - bounds[0]) * scale[0].abs(), (bounds[3] - bounds[1]) * scale[1].abs()];
        let mut factor = [1.0f32; 2];
        for axis in 0..2 {
            if sizing[axis] != Sizing::Hug && natural[axis] > 1e-6 {
                factor[axis] = cell[axis] / natural[axis];
            }
        }
        let stretched: Vec<f32> = (0..2).filter(|a| sizing[*a] != Sizing::Hug).map(|a| factor[a]).collect();
        factor = match fit {
            1 if !stretched.is_empty() => { let k = stretched.iter().copied().fold(f32::INFINITY, f32::min); [k, k] }
            2 if !stretched.is_empty() => { let k = stretched.iter().copied().fold(0.0, f32::max); [k, k] }
            3 => [1.0, 1.0],
            _ => factor,
        };
        let is_shape = self.meta(layer)?.is_some_and(|m| m.source == LayerSource::Shape);
        let (stretch, bounds, scale) = if is_shape && factor != [1.0, 1.0] {
            let shapes = crate::doc::vector::stretch_outline(&self.shapes_at(layer, t)?, factor);
            (factor, shape_box(&shapes).unwrap_or(bounds), scale)
        } else {
            ([1.0, 1.0], bounds, [scale[0] * factor[0], scale[1] * factor[1]])
        };
        let rotation = self.layout_rotation(layer, t)?;
        let anchor = self.item_anchor(layer, t, bounds)?;
        let (lo, hi) = footprint(bounds, self.raw_depth(layer, t)?, anchor, [scale[0], scale[1], 1.0], rotation);
        let shown = [hi[0] - lo[0], hi[1] - lo[1]];
        let mut position = [0.0; 2];
        for axis in 0..2 {
            let target = [placed.location.x, placed.location.y][axis] + (cell[axis] - shown[axis]) * 0.5;
            position[axis] = target - lo[axis] + offset[axis];
        }
        // 奥行きのある素材は、描く側が xy の拡縮の平均を奥行きに掛ける(球は球のまま)。ここで Scale Z に掛けると二重になる。
        let scale_z = 1.0;
        Ok(Slot { position, scale, stretch, wrap: None, z: 0.0, scale_z, rotation, anchor })
    }
}

fn shape_box(shapes: &[crate::doc::vector::ShapeNode]) -> Option<[f32; 4]> {
    let canvas = crate::doc::vector::content_canvas(shapes).ok().flatten()?;
    let b = crate::doc::vector::content_bounds(shapes).ok().flatten()?;
    let (ox, oy) = (canvas.origin_x as f64, canvas.origin_y as f64);
    Some([(b[0] + ox) as f32, (b[1] + oy) as f32, (b[2] + ox) as f32, (b[3] + oy) as f32])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{
        blank_project, rect_shape, ContentKeyframe, ContentTrack, Document, FontRef, Intent, LayerAttrsPatch, LayerMeta, LayerTiming, TextDocument, TextDocumentStyle,
        TextJustify, TextStyleId,
    };

    const T: RationalTime = RationalTime::ZERO;

    fn add(doc: &mut Document, id: u64, source: LayerSource, parent: Option<LayerId>) -> LayerId {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source, order: id as i16, timing: LayerTiming::place(0, None, 300) } },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { parent: Some(parent), ..Default::default() } },
        ])
        .unwrap();
        layer
    }

    fn put(doc: &mut Document, layer: LayerId, name: &str, value: Value) {
        doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
    }

    fn rect(doc: &mut Document, id: u64, parent: LayerId, size: [f32; 2]) -> LayerId {
        let layer = add(doc, id, LayerSource::Shape, Some(parent));
        doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], size)] }).unwrap();
        layer
    }

    /// 画面の上の箱(world): 解いた変換を、輪郭を伸ばした後の箱に掛ける。
    fn shown(doc: &Document, layer: LayerId, t: RationalTime) -> [f32; 4] {
        let view = doc.view();
        let resolved = view.resolved_layers(t).unwrap();
        let r = resolved.iter().find(|l| l.id == layer).unwrap();
        let b = if r.source == LayerSource::Shape {
            shape_box(&crate::doc::vector::stretch_outline(&view.shapes_at(layer, t).unwrap(), r.shape_stretch)).unwrap()
        } else {
            view.layer_box(layer, t).unwrap().unwrap()
        };
        let (lo, hi) = (r.placement.transform.transform_point2(glam::vec2(b[0], b[1])), r.placement.transform.transform_point2(glam::vec2(b[2], b[3])));
        // 群の箱の左上(素材座標の 1 画素の外)から測る。
        [lo.x.min(hi.x), lo.y.min(hi.y), lo.x.max(hi.x), lo.y.max(hi.y)].map(|v| ((v - CANVAS_MARGIN) * 100.0).round() / 100.0)
    }

    fn flex_row(doc: &mut Document) -> LayerId {
        let group = add(doc, 1, LayerSource::Group, None);
        put(doc, group, DISPLAY, Value::Enum(1));
        put(doc, group, GAP, Value::F64(10.0));
        put(doc, group, PADDING, Value::Vec2([20.0, 8.0]));
        group
    }

    #[test]
    fn a_flex_row_butts_boxes_with_gap_and_padding_and_hugs_them() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let a = rect(&mut doc, 2, group, [100.0, 50.0]);
        let b = rect(&mut doc, 3, group, [60.0, 50.0]);
        let c = rect(&mut doc, 4, group, [40.0, 50.0]);
        assert_eq!(shown(&doc, a, T), [20.0, 8.0, 120.0, 58.0]);
        assert_eq!(shown(&doc, b, T), [130.0, 8.0, 190.0, 58.0]);
        assert_eq!(shown(&doc, c, T), [200.0, 8.0, 240.0, 58.0]);
        assert_eq!(doc.view().layer_box(group, T).unwrap(), Some([1.0, 1.0, 261.0, 67.0]), "Hug: padding + boxes + gaps");
        let background = doc.view().background_shapes(group, T).unwrap();
        assert!(background.is_none(), "no Background colour, nothing to draw");
        put(&mut doc, group, BACKGROUND, Value::Color([0.2, 0.2, 0.6, 1.0]));
        let background = doc.view().background_shapes(group, T).unwrap().unwrap();
        assert_eq!(shape_box(&background), Some([1.0, 1.0, 261.0, 67.0]), "the background is drawn where the box is");
    }

    #[test]
    fn scale_is_zoom_and_position_is_a_relative_offset() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let a = rect(&mut doc, 2, group, [100.0, 50.0]);
        let b = rect(&mut doc, 3, group, [60.0, 50.0]);
        put(&mut doc, a, property::SCALE, Value::Vec2([2.0, 1.0]));
        assert_eq!(shown(&doc, a, T), [20.0, 8.0, 220.0, 58.0], "the box grows");
        assert_eq!(shown(&doc, b, T)[0], 230.0, "and pushes the neighbour");
        put(&mut doc, a, property::POSITION, Value::Vec2([0.0, 30.0]));
        assert_eq!(shown(&doc, a, T)[1], 38.0, "Position moves the layer from its place");
        assert_eq!(shown(&doc, b, T)[0], 230.0, "without moving anyone else");
    }

    #[test]
    fn an_absolute_child_and_a_layer_out_of_time_take_no_space() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let a = rect(&mut doc, 2, group, [100.0, 50.0]);
        let b = rect(&mut doc, 3, group, [60.0, 50.0]);
        let c = rect(&mut doc, 4, group, [40.0, 50.0]);
        put(&mut doc, a, POSITION_TYPE, Value::Enum(1));
        assert_eq!(shown(&doc, b, T)[0], 20.0);
        doc.apply(Intent::SetTiming { layer: b, timing: LayerTiming::place(50, None, 300) }).unwrap();
        assert_eq!(shown(&doc, c, T)[0], 20.0, "b is not there yet at frame 0");
    }

    #[test]
    fn grid_tracks_are_keyable_fr_and_fill_stretches_the_outline() {
        let mut doc = blank_project();
        let grid = add(&mut doc, 1, LayerSource::Group, None);
        for (name, value) in [
            (DISPLAY, Value::Enum(2)),
            (GRID_COLUMNS, Value::F64(2.0)),
            (GRID_ROWS, Value::F64(1.0)),
            (HORIZONTAL_SIZING, Value::Enum(2)),
            (VERTICAL_SIZING, Value::Enum(2)),
            (WIDTH, Value::F64(400.0)),
            (HEIGHT, Value::F64(100.0)),
            ("layout.column.1", Value::F64(3.0)),
        ] {
            put(&mut doc, grid, name, value);
        }
        let wide = rect(&mut doc, 2, grid, [10.0, 10.0]);
        let round = rect(&mut doc, 3, grid, [10.0, 10.0]);
        for layer in [wide, round] {
            put(&mut doc, layer, HORIZONTAL_SIZING, Value::Enum(1));
            put(&mut doc, layer, VERTICAL_SIZING, Value::Enum(1));
        }
        put(&mut doc, round, OBJECT_FIT, Value::Enum(1));
        assert_eq!(shown(&doc, wide, T), [0.0, 0.0, 300.0, 100.0], "3fr of 400, squashed and stretched");
        assert_eq!(shown(&doc, round, T), [300.0, 0.0, 400.0, 100.0], "Contain: 100 x 100, centred in the 100 wide cell");

        let mut track = crate::doc::eval::KeyframeTrack::new();
        track.insert(crate::doc::eval::Keyframe { t: T, value: Value::F64(3.0), interp: crate::doc::eval::Interp::Linear, spatial: Default::default() });
        track.insert(crate::doc::eval::Keyframe { t: RationalTime::try_from_frame(10, doc.view().composition().unwrap().unwrap().fps).unwrap(), value: Value::F64(1.0), interp: crate::doc::eval::Interp::Linear, spatial: Default::default() });
        doc.apply(Intent::SetTrack { layer: grid, property: PropertyId::new("layout.column.1").unwrap(), track }).unwrap();
        let later = RationalTime::try_from_frame(10, doc.view().composition().unwrap().unwrap().fps).unwrap();
        assert_eq!(shown(&doc, wide, later), [0.0, 0.0, 200.0, 100.0], "the key on Column 1 moves the line");
    }

    fn put_text(doc: &mut Document, layer: LayerId, content: &str) {
        let mut track = ContentTrack::new();
        track.insert(ContentKeyframe { t: T, content: content.to_owned() });
        let style = TextDocumentStyle {
            id: TextStyleId(0),
            font: FontRef { path: String::new(), fingerprint: None, family: "Helvetica".to_owned(), style: String::new() },
            size: 48.0, fill: [1.0; 4], line_height: None, tracking: 0.0, axes: vec![], features: vec![],
        };
        doc.apply(Intent::SetTextDocument { layer, document: TextDocument {
            content: track, justify: TextJustify::Left, wrap_size: None, styles: vec![style], slot_id: None, ranges: vec![], alignment: Default::default(), runs: vec![],
        } }).unwrap();
    }

    #[test]
    fn text_filling_its_cell_wraps_at_the_cell_and_grows_down() {
        let mut doc = blank_project();
        let column = add(&mut doc, 1, LayerSource::Group, None);
        for (name, value) in [(DISPLAY, Value::Enum(1)), (FLEX_DIRECTION, Value::Enum(1)), (HORIZONTAL_SIZING, Value::Enum(2)), (WIDTH, Value::F64(300.0))] {
            put(&mut doc, column, name, value);
        }
        let words = add(&mut doc, 2, LayerSource::Text, Some(column));
        put_text(&mut doc, words, "Taro Yamada is the creative director of this studio");
        let below = rect(&mut doc, 3, column, [40.0, 40.0]);
        let one_line = shown(&doc, below, T)[1];
        put(&mut doc, words, HORIZONTAL_SIZING, Value::Enum(1));
        let view = doc.view();
        let wrap = view.resolved_text_document(words, T).unwrap().unwrap().wrap_size.expect("wraps");
        assert_eq!(wrap[0], 300.0, "at the cell width");
        drop(view);
        assert!(shown(&doc, below, T)[1] > one_line + 40.0, "the wrapped lines push the next item down: {one_line} → {}", shown(&doc, below, T)[1]);

        put(&mut doc, words, property::SCALE, Value::Vec2([2.0, 2.0]));
        assert_eq!(doc.view().resolved_text_document(words, T).unwrap().unwrap().wrap_size.unwrap()[0], 150.0, "Scale is zoom: the words wrap at half the width, then double");
    }

    #[test]
    fn overflow_clip_cuts_every_descendant_at_the_groups_box() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let a = rect(&mut doc, 2, group, [100.0, 50.0]);
        assert!(doc.view().resolved_layers(T).unwrap().iter().find(|l| l.id == a).unwrap().masks.is_empty(), "Visible cuts nothing");
        put(&mut doc, group, OVERFLOW, Value::Enum(1));
        put(&mut doc, group, property::POSITION, Value::Vec2([300.0, 200.0]));
        put(&mut doc, a, property::POSITION, Value::Vec2([0.0, 30.0]));
        let resolved = doc.view().resolved_layers(T).unwrap();
        let layer = resolved.iter().find(|l| l.id == a).unwrap();
        assert_eq!(layer.masks.len(), 1);
        assert_eq!(layer.masks[0].mode, crate::doc::store::MaskMode::Intersect);
        let world: Vec<glam::Vec2> = layer.masks[0].shape.vertices.iter().map(|v| layer.placement.transform.transform_point2(glam::vec2(v.point[0] as f32, v.point[1] as f32))).collect();
        let (lo, hi) = world.iter().fold((glam::Vec2::MAX, glam::Vec2::MIN), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
        assert_eq!((lo.round(), hi.round()), (glam::vec2(301.0, 201.0), glam::vec2(441.0, 267.0)), "the group's box on screen, whatever the child's offset");
    }

    #[test]
    fn things_on_a_laid_out_face_share_its_point_even_when_the_face_is_tilted() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        put(&mut doc, group, property::POSITION, Value::Vec2([300.0, 200.0]));
        put(&mut doc, group, property::ROTATION_Y, Value::F64(30.0));
        let flat = rect(&mut doc, 2, group, [100.0, 50.0]);
        let lifted = rect(&mut doc, 3, group, [60.0, 50.0]);
        put(&mut doc, lifted, property::POSITION_Z, Value::F64(-40.0));
        let resolved = doc.view().resolved_layers(T).unwrap();
        let plane = |id| resolved.iter().find(|l| l.id == id).unwrap().placement.plane;
        let world = resolved.iter().find(|l| l.id == group).unwrap().placement.world_transform.unwrap();
        let b = doc.view().layer_box(group, T).unwrap().unwrap();
        let face = world.transform_point3(glam::vec3((b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5, 0.0)).to_array();
        assert_eq!(plane(group), Some(face), "the tilted group is the face");
        assert_eq!(plane(flat), Some(face), "a flat child lies on it and stacks by order");
        assert_eq!(plane(lifted), None, "a child lifted off the face sorts by distance");
    }

    #[test]
    fn depth_alignment_sets_each_childs_z_against_the_deepest_one() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let slab = rect(&mut doc, 2, group, [100.0, 50.0]);
        let card = rect(&mut doc, 3, group, [60.0, 50.0]);
        put(&mut doc, slab, property::DEPTH, Value::F64(100.0));
        let z = |doc: &Document, id| doc.view().layout_frame(T).unwrap().slots[&id].z;
        assert_eq!((z(&doc, slab), z(&doc, card)), (-100.0, 0.0), "Back: backs on the face, the slab stands out toward the camera");
        assert_eq!(doc.view().layout_frame(T).unwrap().depths[&group], [-100.0, 0.0]);
        put(&mut doc, group, DEPTH_ALIGNMENT, Value::Enum(1));
        assert_eq!((z(&doc, slab), z(&doc, card)), (-100.0, -50.0), "Center: the card floats at the slab's middle");
        put(&mut doc, group, DEPTH_ALIGNMENT, Value::Enum(2));
        assert_eq!((z(&doc, slab), z(&doc, card)), (-100.0, -100.0), "Front: fronts together");
        let resolved = doc.view().resolved_layers(T).unwrap();
        assert_eq!(resolved.iter().find(|l| l.id == card).unwrap().placement.z, -100.0, "the z reaches the resolved layer");
    }

    #[test]
    fn flex_direction_depth_stacks_children_back_by_thickness_and_gap() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        put(&mut doc, group, FLEX_DIRECTION, Value::Enum(DIRECTION_DEPTH));
        put(&mut doc, group, GAP, Value::F64(20.0));
        let back = rect(&mut doc, 2, group, [100.0, 50.0]);
        let slab = rect(&mut doc, 3, group, [100.0, 50.0]);
        let front = rect(&mut doc, 4, group, [100.0, 50.0]);
        put(&mut doc, slab, property::DEPTH, Value::F64(30.0));
        let frame = doc.view().layout_frame(T).unwrap();
        assert_eq!([frame.slots[&front].z, frame.slots[&slab].z, frame.slots[&back].z], [0.0, 20.0, 70.0], "top of the stack in front, then thickness + gap each");
        assert_eq!(frame.depths[&group], [0.0, 70.0]);
        assert_eq!(frame.slots[&front].position, frame.slots[&back].position, "one spot on the face");
    }

    #[test]
    fn layout_rotation_lays_out_the_turned_box_and_rotation_does_not() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let turned = rect(&mut doc, 2, group, [100.0, 50.0]);
        let next = rect(&mut doc, 3, group, [60.0, 50.0]);
        put(&mut doc, turned, property::ROTATION, Value::F64(90.0));
        assert_eq!(shown(&doc, next, T)[0], 130.0, "Rotation is only the look");
        put(&mut doc, turned, property::ROTATION, Value::F64(0.0));
        put(&mut doc, turned, LAYOUT_ROTATION, Value::F64(90.0));
        assert_eq!(shown(&doc, next, T)[0], 80.0, "Layout Rotation: the 100 x 50 box stands 50 wide");
        let turned_box = shown(&doc, turned, T);
        assert_eq!([turned_box[0], turned_box[2] - turned_box[0], turned_box[3] - turned_box[1]], [20.0, 50.0, 100.0], "and sits in its place turned");
    }

    #[test]
    fn exclusions_leave_the_cells_a_tracked_blob_covers_empty() {
        let mut doc = blank_project();
        let grid = add(&mut doc, 1, LayerSource::Group, None);
        for (name, value) in [
            (DISPLAY, Value::Enum(2)), (GRID_COLUMNS, Value::F64(3.0)), (GRID_ROWS, Value::F64(3.0)),
            (HORIZONTAL_SIZING, Value::Enum(2)), (VERTICAL_SIZING, Value::Enum(2)), (WIDTH, Value::F64(300.0)), (HEIGHT, Value::F64(300.0)),
            (EXCLUSIONS, Value::LayerId(20)),
        ] {
            put(&mut doc, grid, name, value);
        }
        let cards: Vec<LayerId> = (2..11).map(|id| rect(&mut doc, id, grid, [10.0, 10.0])).collect();
        for &card in &cards {
            put(&mut doc, card, HORIZONTAL_SIZING, Value::Enum(1));
            put(&mut doc, card, VERTICAL_SIZING, Value::Enum(1));
        }
        let mut inputs = crate::doc::store::analysis::AnalysisInputs::default();
        // 真ん中の枠(comp の 101..201)に人の群れが 1 つ。
        inputs.set_blobs(LayerId(20), crate::doc::store::EffectId(0), T, vec![crate::doc::store::analysis::BlobMark { id: 1, center: [151.0, 151.0], size: [30.0, 30.0], age: 0 }]);
        let view = doc.view().with_analysis(&inputs);
        let resolved = view.resolved_layers(T).unwrap();
        let centre = |id: LayerId| {
            let r = resolved.iter().find(|l| l.id == id).unwrap();
            let b = shape_box(&crate::doc::vector::stretch_outline(&view.shapes_at(id, T).unwrap(), r.shape_stretch)).unwrap();
            r.placement.transform.transform_point2(glam::vec2((b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5))
        };
        let in_middle = cards.iter().filter(|&&c| { let p = centre(c); p.x > 101.0 && p.x < 201.0 && p.y > 101.0 && p.y < 201.0 }).count();
        assert_eq!(in_middle, 0, "no card sits where the crowd is");
        assert!(cards.iter().any(|&c| centre(c).y >= 300.0), "the ninth card flows into an implicit row below the grid");
    }

    #[test]
    fn free_boxes_keep_their_declared_distance_and_the_one_that_does_not_yield_stays() {
        let mut doc = blank_project();
        let a = add(&mut doc, 1, LayerSource::Shape, None);
        let b = add(&mut doc, 2, LayerSource::Shape, None);
        for (layer, x) in [(a, 100.0), (b, 150.0)] {
            doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
            put(&mut doc, layer, property::POSITION, Value::Vec2([x, 100.0]));
            put(&mut doc, layer, MARGIN, Value::F64(10.0));
        }
        let left = |doc: &Document, id| shown(doc, id, T)[0];
        // 素のままなら 50 px 重なる。間合い 10 ずつ → 箱の間は 20 空く。譲りは半分ずつ。
        assert_eq!(left(&doc, b) - (left(&doc, a) + 100.0), 20.0, "they stand apart by both margins");
        assert_eq!((left(&doc, a), left(&doc, b)), (65.0, 185.0), "each yields half");
        put(&mut doc, a, FLEX_SHRINK, Value::F64(0.0));
        assert_eq!((left(&doc, a), left(&doc, b)), (100.0, 220.0), "a does not yield, b takes all of it");
    }

    #[test]
    fn a_transition_moves_to_the_new_place_over_its_duration_without_state() {
        let mut doc = blank_project();
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let at = |f: i64| RationalTime::try_from_frame(f, fps).unwrap();
        let a = add(&mut doc, 1, LayerSource::Shape, None);
        let b = add(&mut doc, 2, LayerSource::Shape, None);
        for layer in [a, b] {
            doc.apply(Intent::SetShapes { layer, shapes: vec![rect_shape([255; 4], [100.0, 100.0])] }).unwrap();
            put(&mut doc, layer, MARGIN, Value::F64(10.0));
        }
        put(&mut doc, a, property::POSITION, Value::Vec2([100.0, 100.0]));
        put(&mut doc, a, FLEX_SHRINK, Value::F64(0.0));
        // b は 30 コマ目に a の真上へ飛び込む(鍵は Hold)。押し戻されて右へ。
        let mut track = crate::doc::eval::KeyframeTrack::new();
        track.insert(crate::doc::eval::Keyframe { t: at(0), value: Value::Vec2([600.0, 100.0]), interp: crate::doc::eval::Interp::Hold, spatial: Default::default() });
        track.insert(crate::doc::eval::Keyframe { t: at(30), value: Value::Vec2([150.0, 100.0]), interp: crate::doc::eval::Interp::Hold, spatial: Default::default() });
        doc.apply(Intent::SetTrack { layer: b, property: PropertyId::new("position").unwrap(), track }).unwrap();
        put(&mut doc, b, TRANSITION_DURATION, Value::F64(0.5));
        put(&mut doc, b, TRANSITION_EASING, Value::Enum(1));
        // 移り方が掛かるのは解いた関係(押し戻し)だけ。b 自身の鍵の動きは区間イージングの係で、そのまま効く。
        let x = |f: i64| shown(&doc, b, at(f))[0];
        assert!(x(30) < 160.0, "b lands where its key says, on top of a, and only begins to be pushed: {}", x(30));
        assert!(x(37) > 170.0 && x(37) < 210.0, "halfway through the duration it is being pushed out: {}", x(37));
        assert_eq!(x(45), 220.0, "after the duration it rests at the solved distance");
        assert_eq!(x(37), shown(&doc, b, at(37))[0], "the same frame asked again gives the same picture");
    }

    #[test]
    fn a_container_staggers_its_childrens_transitions_by_distance_and_nothing_jumps() {
        let mut doc = blank_project();
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let at = |f: i64| RationalTime::try_from_frame(f, fps).unwrap();
        // 1 行に 4 つ並べ、30 コマ目に Flex Direction を Column へ(Hold)。子は下へ積み直す。
        let group = flex_row(&mut doc);
        let mut track = crate::doc::eval::KeyframeTrack::new();
        track.insert(crate::doc::eval::Keyframe { t: at(0), value: Value::Enum(0), interp: crate::doc::eval::Interp::Hold, spatial: Default::default() });
        track.insert(crate::doc::eval::Keyframe { t: at(30), value: Value::Enum(1), interp: crate::doc::eval::Interp::Hold, spatial: Default::default() });
        doc.apply(Intent::SetTrack { layer: group, property: PropertyId::new(FLEX_DIRECTION).unwrap(), track }).unwrap();
        let children: Vec<LayerId> = (2..6).map(|id| rect(&mut doc, id, group, [60.0, 60.0])).collect();
        for &child in &children {
            put(&mut doc, child, TRANSITION_DURATION, Value::F64(0.5));
            put(&mut doc, child, TRANSITION_EASING, Value::Enum(4));
        }
        put(&mut doc, group, STAGGER, Value::F64(1.0));
        let y = |doc: &Document, child: LayerId, f: i64| shown(doc, child, at(f))[1];
        // 最初の子は並びの上で位置が変わらない。2 番目から、起点から遠いほど遅れて動き出す。
        let started = |doc: &Document, child: LayerId| (30..120).find(|f| (y(doc, child, *f) - y(doc, child, 29)).abs() > 1.0);
        let (s1, s3) = (started(&doc, children[1]).expect("moves"), started(&doc, children[3]).expect("moves"));
        assert!(s3 > s1 + 5, "the far child starts later than the near one: {s1} vs {s3}");
        for &child in &children {
            let ys: Vec<f32> = (25..120).map(|f| y(&doc, child, f)).collect();
            // 速さの変わり方(2 階の差)が小さい = 跳ばない、尖らない。
            let biggest = ys.windows(3).map(|w| (w[2] - 2.0 * w[1] + w[0]).abs()).fold(0.0, f32::max);
            assert!(biggest < 12.0, "no frame jumps or kinks (largest change of speed {biggest})");
        }
        put(&mut doc, group, STAGGER_FROM, Value::Enum(2));
        let (e1, e3) = (started(&doc, children[1]).expect("moves"), started(&doc, children[3]).expect("moves"));
        assert!(e1 > e3, "from the end, the near-the-start child waits: {e1} vs {e3}");
    }

    #[test]
    fn a_picture_takes_the_size_the_host_measured() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let picture = add(&mut doc, 2, LayerSource::File { path: "/tmp/photo.png".to_owned(), fingerprint: None }, Some(group));
        let after = rect(&mut doc, 3, group, [40.0, 40.0]);
        let mut inputs = crate::doc::store::analysis::AnalysisInputs::default();
        inputs.set_extent("/tmp/photo.png", [320.0, 180.0, 0.0]);
        let view = doc.view().with_analysis(&inputs);
        let frame = view.layout_frame(T).unwrap();
        assert_eq!(view.layer_box(picture, T).unwrap(), Some([0.0, 0.0, 320.0, 180.0]));
        // 箱の左端 = 位置 - 中心 + 箱の左(形の素材座標は輪郭の canvas の 1 画素外が原点、画は 0)。
        let left = |id: LayerId, min: f32| frame.slots[&id].position[0] - frame.slots[&id].anchor[0] + min;
        assert_eq!(left(after, 1.0), left(picture, 0.0) + 320.0 + 10.0, "the next item starts after the picture and the gap");
    }

    #[test]
    fn wrapping_text_flows_around_a_sibling_that_declares_its_shape() {
        let mut doc = blank_project();
        let words = add(&mut doc, 2, LayerSource::Text, None);
        let mut track = ContentTrack::new();
        track.insert(ContentKeyframe { t: T, content: "Shibuya crossing at nine in the evening, the lights change and three thousand people walk at once across the white lines".to_owned() });
        let style = TextDocumentStyle {
            id: TextStyleId(0),
            font: FontRef { path: String::new(), fingerprint: None, family: "Helvetica".to_owned(), style: String::new() },
            size: 40.0, fill: [1.0; 4], line_height: None, tracking: 0.0, axes: vec![], features: vec![],
        };
        let comp = doc.view().composition().unwrap().unwrap();
        doc.apply(Intent::SetTextDocument { layer: words, document: TextDocument {
            content: track, justify: TextJustify::Left, wrap_size: Some([600.0, comp.height as f32]), styles: vec![style], slot_id: None, ranges: vec![], alignment: Default::default(), runs: vec![],
        } }).unwrap();
        let object = add(&mut doc, 3, LayerSource::Shape, None);
        doc.apply(Intent::SetShapes { layer: object, shapes: vec![rect_shape([255; 4], [160.0, 160.0])] }).unwrap();
        let middle = doc.view().world_2d(words, T).unwrap().transform_point2(glam::vec2(300.0, comp.height as f32 * 0.5));
        put(&mut doc, object, property::POSITION, Value::Vec2([middle.x as f64, middle.y as f64]));
        let canvas = crate::doc::vector::Canvas { width: comp.width, height: comp.height, origin_x: 0, origin_y: 0 };
        // 物の占める範囲(文字の枠の座標)と重なる字の数。
        let overlapping = |doc: &Document| {
            let view = doc.view();
            let resolved = view.resolved_layers(T).unwrap();
            let around = resolved.iter().find(|l| l.id == words).unwrap().flow_around.clone().expect("the object is declared");
            let (lo, hi) = around.iter().flat_map(|o| o.points.iter()).fold((glam::Vec2::MAX, glam::Vec2::MIN), |(lo, hi), p| (lo.min(glam::Vec2::from(*p)), hi.max(glam::Vec2::from(*p))));
            let document = view.resolved_text_document(words, T).unwrap().unwrap();
            let count = |shaped: &crate::doc::vector::text::ShapedText| shaped.contours.iter().filter(|c| {
                let (a, b) = c.vertices.iter().fold((glam::Vec2::MAX, glam::Vec2::MIN), |(a, b), v| (a.min(glam::vec2(v.point.x as f32, v.point.y as f32)), b.max(glam::vec2(v.point.x as f32, v.point.y as f32))));
                a.x < hi.x && b.x > lo.x && a.y < hi.y && b.y > lo.y
            }).count();
            let plain = crate::doc::store::text_frame::shape_document(&document, T, &canvas).unwrap().unwrap();
            let flowed = crate::doc::store::text_frame::shape_document_around(&document, T, &canvas, &around).unwrap().unwrap();
            let right_of = flowed.contours.iter().any(|c| c.vertices.iter().all(|v| v.point.x as f32 > hi.x && (v.point.y as f32) > lo.y && (v.point.y as f32) < hi.y));
            (count(&plain), count(&flowed), plain.contours.len() == flowed.contours.len(), right_of)
        };
        put(&mut doc, object, SHAPE_OUTSIDE, Value::Enum(2));
        assert!(doc.view().resolved_layers(T).unwrap().iter().find(|l| l.id == words).unwrap().flow_around.is_some());
        let (before, after, all_glyphs, both_sides) = overlapping(&doc);
        assert!(before > 0, "without flowing, the words run under the object");
        assert_eq!(after, 0, "the words flow around it");
        assert!(all_glyphs, "no word is lost");
        assert!(both_sides, "lines continue on the other side of the object (wrap-flow: both)");

        put(&mut doc, object, SHAPE_OUTSIDE, Value::Enum(0));
        assert!(doc.view().resolved_layers(T).unwrap().iter().find(|l| l.id == words).unwrap().flow_around.is_none(), "None declares nothing");
    }

    #[test]
    fn a_growing_line_of_text_pushes_its_neighbour() {
        let mut doc = blank_project();
        let group = flex_row(&mut doc);
        let name = add(&mut doc, 2, LayerSource::Text, Some(group));
        let icon = rect(&mut doc, 3, group, [40.0, 40.0]);
        let x = |doc: &mut Document, content: &str| {
            let mut track = ContentTrack::new();
            track.insert(ContentKeyframe { t: T, content: content.to_owned() });
            let style = TextDocumentStyle {
                id: TextStyleId(0),
                font: FontRef { path: String::new(), fingerprint: None, family: "Helvetica".to_owned(), style: String::new() },
                size: 48.0, fill: [1.0; 4], line_height: None, tracking: 0.0, axes: vec![], features: vec![],
            };
            doc.apply(Intent::SetTextDocument { layer: name, document: TextDocument {
                content: track, justify: TextJustify::Left, wrap_size: None, styles: vec![style], slot_id: None, ranges: vec![], alignment: Default::default(), runs: vec![],
            } }).unwrap();
            shown(doc, icon, T)[0]
        };
        let short = x(&mut doc, "Taro");
        let long = x(&mut doc, "Taro Yamada");
        assert!(long > short + 50.0, "the icon moves right as the name grows: {short} → {long}");
        let text = shown(&doc, name, T);
        assert_eq!((text[0], text[1]), (20.0, 8.0), "the line box sits at the padding");
    }
}

/// 角丸の矩形の閉じた道(時計回り、角は 3 次 Bézier の円弧近似)。`to` で各点を写す(接線は向きだけ写す)。
pub(crate) fn rounded_rect_path(b: [f32; 4], radius: f32, to: glam::Affine2) -> crate::doc::eval::Path {
    use crate::doc::eval::{Path, PathVertex};
    let r = radius.min((b[2] - b[0]) * 0.5).min((b[3] - b[1]) * 0.5).max(0.0);
    let k = r * 0.552_284_8;
    let mut vertices = Vec::new();
    let mut push = |p: [f32; 2], inn: [f32; 2], out: [f32; 2]| {
        let point = to.transform_point2(glam::Vec2::from(p));
        let (i, o) = (to.transform_vector2(glam::Vec2::from(inn)), to.transform_vector2(glam::Vec2::from(out)));
        vertices.push(PathVertex { point: [point.x as f64, point.y as f64], in_tangent: [i.x as f64, i.y as f64], out_tangent: [o.x as f64, o.y as f64] });
    };
    let (l, top, rt, bot) = (b[0], b[1], b[2], b[3]);
    push([l + r, top], [-k, 0.0], [0.0, 0.0]);
    push([rt - r, top], [0.0, 0.0], [k, 0.0]);
    push([rt, top + r], [0.0, -k], [0.0, 0.0]);
    push([rt, bot - r], [0.0, 0.0], [0.0, k]);
    push([rt - r, bot], [k, 0.0], [0.0, 0.0]);
    push([l + r, bot], [0.0, 0.0], [-k, 0.0]);
    push([l, bot - r], [0.0, k], [0.0, 0.0]);
    push([l, top + r], [0.0, 0.0], [0.0, -k]);
    Path { vertices, closed: true }
}

/// 箱(素材座標の x, y と奥行き z)を、アンカーのまわりで拡縮・回した時の軸に沿った範囲。回し方は層の変換と同じ
/// (Tilt X・Tilt Y の後に Rotation と Scale)。回転が 0 なら拡縮した箱そのもの。
fn footprint(bounds: [f32; 4], depth: [f32; 2], anchor: [f32; 2], scale: [f32; 3], rotation: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let turn = glam::Quat::from_rotation_x(rotation[0].to_radians()) * glam::Quat::from_rotation_y(rotation[1].to_radians()) * glam::Quat::from_rotation_z(rotation[2].to_radians());
    let (mut lo, mut hi) = (glam::Vec3::splat(f32::INFINITY), glam::Vec3::splat(f32::NEG_INFINITY));
    for x in [bounds[0], bounds[2]] {
        for y in [bounds[1], bounds[3]] {
            for z in depth {
                let p = turn * (glam::vec3(x - anchor[0], y - anchor[1], z) * glam::Vec3::from(scale));
                lo = lo.min(p);
                hi = hi.max(p);
            }
        }
    }
    (lo.to_array(), hi.to_array())
}

/// CSS の timing function(区間の形)。Ease / Linear / Ease In / Ease Out / Ease In Out。
fn ease(kind: i64, u: f64) -> f64 {
    let (x1, y1, x2, y2) = match kind {
        1 => return u.clamp(0.0, 1.0),
        2 => (0.42, 0.0, 1.0, 1.0),
        3 => (0.0, 0.0, 0.58, 1.0),
        4 => (0.42, 0.0, 0.58, 1.0),
        _ => (0.25, 0.1, 0.25, 1.0),
    };
    let bezier = |a: f64, b: f64, s: f64| 3.0 * a * s * (1.0 - s).powi(2) + 3.0 * b * s * s * (1.0 - s) + s.powi(3);
    let u = u.clamp(0.0, 1.0);
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..40 {
        let mid = (lo + hi) * 0.5;
        if bezier(x1, x2, mid) < u { lo = mid } else { hi = mid }
    }
    bezier(y1, y2, (lo + hi) * 0.5)
}
