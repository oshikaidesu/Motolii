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

/// 欄: (property, 窓の名前, 既定値, 範囲, 選択肢)。名前は CSS の語、大きさの決め方だけ Figma(裁定 2026-09-14)。
pub type Row = (&'static str, &'static str, Value, Option<(f64, f64)>, &'static [&'static str]);

const SIZING: &[&str] = &["Hug", "Fill", "Fixed"];

/// Group の欄。Display が None の間は Display だけが意味を持つ。
pub const GROUP_ROWS: &[Row] = &[
    (DISPLAY, "Display", Value::Enum(0), None, &["None", "Flex", "Grid"]),
    (FLEX_DIRECTION, "Flex Direction", Value::Enum(0), None, &["Row", "Column", "Row Reverse", "Column Reverse"]),
    (FLEX_WRAP, "Flex Wrap", Value::Enum(0), None, &["No Wrap", "Wrap", "Wrap Reverse"]),
    (JUSTIFY_CONTENT, "Justify Content", Value::Enum(0), None, &["Start", "End", "Center", "Space Between", "Space Around", "Space Evenly"]),
    (ALIGN_ITEMS, "Align Items", Value::Enum(0), None, &["Stretch", "Start", "End", "Center"]),
    (GRID_COLUMNS, "Grid Columns", Value::F64(2.0), Some((1.0, 64.0)), &[]),
    (GRID_ROWS, "Grid Rows", Value::F64(0.0), Some((0.0, 64.0)), &[]),
    (GAP, "Gap", Value::F64(0.0), Some((0.0, 100000.0)), &[]),
    (PADDING, "Padding", Value::Vec2([0.0, 0.0]), None, &[]),
    (HORIZONTAL_SIZING, "Horizontal Sizing", Value::Enum(0), None, SIZING),
    (VERTICAL_SIZING, "Vertical Sizing", Value::Enum(0), None, SIZING),
    (WIDTH, "Width", Value::F64(400.0), Some((0.0, 100000.0)), &[]),
    (HEIGHT, "Height", Value::F64(300.0), Some((0.0, 100000.0)), &[]),
    (BACKGROUND, "Background", Value::Color([1.0, 1.0, 1.0, 0.0]), None, &[]),
    (BORDER_RADIUS, "Border Radius", Value::F64(0.0), Some((0.0, 100000.0)), &[]),
    (OVERFLOW, "Overflow", Value::Enum(0), None, &["Visible", "Clip"]),
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
];

/// 形の層の素材座標は、輪郭の canvas の左上(反アリアスの 1 画素の外)が原点。
/// Display の Group の箱もその座標で [1, 1]..[1 + 幅, 1 + 高さ] に置き、背景の形と子の枠が同じ所に来る。
const CANVAS_MARGIN: f32 = 1.0;

/// 格子の線の太さの既定(fr)。
pub const TRACK_DEFAULT: f64 = 1.0;

/// 番号付きの線の名前(`layout.column.3` → `Column 3`)。
pub fn track_label(property: &str) -> Option<String> {
    let number = |prefix: &str| property.strip_prefix(prefix).and_then(|n| n.parse::<u32>().ok()).filter(|n| *n >= 1);
    number(COLUMN_PREFIX).map(|n| format!("Column {n}")).or_else(|| number(ROW_PREFIX).map(|n| format!("Row {n}")))
}

pub fn row(property: &str) -> Option<&'static Row> {
    GROUP_ROWS.iter().chain(ITEM_ROWS).find(|row| row.0 == property)
}

pub fn choices(property: &str) -> &'static [&'static str] {
    row(property).map_or(&[], |row| row.4)
}

/// その時刻に並べた結果。`slots` は並ぶ子、`sizes` は Display の Group の箱の大きさ(素材座標で [0, 0]..size)。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Frame {
    pub slots: HashMap<LayerId, Slot>,
    pub sizes: HashMap<LayerId, [f32; 2]>,
}

/// 並ぶ子の変換の差し替え: 層の Position と Scale の代わりに使う値と、形の輪郭の伸び。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    pub position: [f32; 2],
    pub scale: [f32; 2],
    pub stretch: [f32; 2],
    /// 横が Fill の文字: その幅で折り返す(素材座標の幅、Scale で割った値)。
    pub wrap: Option<f32>,
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
    /// その時刻に並べた結果。Display の Group が無ければ空。
    pub fn layout_frame(&self, t: RationalTime) -> Result<Rc<Frame>, StoreError> {
        if let Some(hit) = self.layout_memo().borrow().get(&t) {
            return Ok(hit.clone());
        }
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
        Ok(self.layout_frame(t)?.slots.get(&layer).copied())
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
            LayerSource::Group => {
                if self.display(layer, t)? != 0 {
                    return Ok(self.layout_frame(t)?.sizes.get(&layer).map(|s| [CANVAS_MARGIN, CANVAS_MARGIN, CANVAS_MARGIN + s[0], CANVAS_MARGIN + s[1]]));
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
            let taffy = |e: taffy::TaffyError| StoreError::Property(format!("layout: {e}"));
            let shifted = |mut placed: taffy::Layout| { placed.location.x += CANVAS_MARGIN; placed.location.y += CANVAS_MARGIN; placed };
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
        }
        Ok(frame)
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
        let Some(size) = self.layout_frame(t)?.sizes.get(&group).copied() else { return Ok(None) };
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
        let Some(size) = self.layout_frame(t)?.sizes.get(&group).copied() else { return Ok(None) };
        let b = [CANVAS_MARGIN, CANVAS_MARGIN, CANVAS_MARGIN + size[0], CANVAS_MARGIN + size[1]];
        Ok(Some((b, self.number(group, BORDER_RADIUS, 0.0, t)?.max(0.0) as f32)))
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
        style.flex_shrink = 0.0;
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
            let natural = [(bounds[2] - bounds[0]) * scale[0].abs(), (bounds[3] - bounds[1]) * scale[1].abs()];
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
        let anchor = self.pair(layer, property::ANCHOR, [0.0, 0.0], t)?;
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
        let shown = [(bounds[2] - bounds[0]) * scale[0].abs(), (bounds[3] - bounds[1]) * scale[1].abs()];
        let mut position = [0.0; 2];
        for axis in 0..2 {
            let low = (scale[axis] * (bounds[axis] - anchor[axis])).min(scale[axis] * (bounds[axis + 2] - anchor[axis]));
            let target = [placed.location.x, placed.location.y][axis] + (cell[axis] - shown[axis]) * 0.5;
            position[axis] = target - low + offset[axis];
        }
        Ok(Slot { position, scale, stretch, wrap: None })
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
