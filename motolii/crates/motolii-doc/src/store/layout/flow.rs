//! 並べる — 親の箱が子の場所を決める世界(CSS の flex と grid、解き手は taffy)。
//! 外から来るのは「どの層がどの親に、どんな大きさで」だけ。解いた枠は Slot にして返す。
//! ここが層の値を書き換えることはない。

use super::boxes::Extent;
use taffy::prelude::*;

use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Sizing { Hug, Fill, Fixed }

/// 幅で高さが決まる子(横が Fill の文字)。taffy が幅を決めてから問う。
#[derive(Clone, Copy)]
pub(super) struct Measure {
    pub(super) layer: LayerId,
    pub(super) scale: f32,
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

/// Grid の Group の明示の升目(素材座標、CANVAS_MARGIN と padding 込み)。Grid でなければ None。
fn grid_fields(tree: &TaffyTree<Measure>, node: NodeId) -> Option<(Vec<(f32, f32)>, Vec<(f32, f32)>)> {
    let taffy::tree::DetailedLayoutInfo::Grid(info) = tree.detailed_layout_info(node) else { return None };
    let padding = tree.layout(node).ok()?.padding;
    let lines = |tracks: &taffy::compute::detailed_info::DetailedGridTracksInfo, start: f32| {
        let mut at = start;
        let mut out = Vec::new();
        for (i, size) in tracks.sizes.iter().enumerate() {
            at += tracks.gutters.get(i).copied().unwrap_or(0.0);
            out.push((at, at + size));
            at += size;
        }
        let skip = tracks.negative_implicit_tracks as usize;
        out.into_iter().skip(skip).take(tracks.explicit_tracks as usize).collect::<Vec<_>>()
    };
    Some((lines(&info.columns, padding.left + CANVAS_MARGIN), lines(&info.rows, padding.top + CANVAS_MARGIN)))
}

pub(super) fn compute_layout(view: &StoreView<'_>, t: RationalTime) -> Result<Frame, StoreError> {
    let mut frame = Frame::default();
    let layers = view.layers();
    let mut children: HashMap<LayerId, Vec<(i16, LayerId)>> = HashMap::new();
    let mut displayed = Vec::new();
    for &layer in &layers {
        let Some(meta) = view.meta(layer)? else { continue };
        if meta.source == LayerSource::Group && view.display(layer, t)? != 0 {
            displayed.push(layer);
        }
        if let Some(parent) = view.attrs(layer)?.unwrap_or_default().parent {
            if view.here(layer, t)? {
                children.entry(parent).or_default().push((meta.order, layer));
            }
        }
    }
    if displayed.is_empty() {
        push_apart(view, t, &displayed, &mut frame)?;
        return Ok(frame);
    }
    for list in children.values_mut() {
        list.sort();
    }
    for &root in &displayed {
        let parent = view.attrs(root)?.unwrap_or_default().parent;
        if parent.is_some_and(|p| displayed.contains(&p)) {
            continue;
        }
        let mut tree: TaffyTree<Measure> = TaffyTree::new();
        tree.disable_rounding();
        let (mut leaves, mut groups) = (Vec::new(), Vec::new());
        let node = container(view, &mut tree, root, t, &children, &displayed, true, &mut leaves, &mut groups)?;
        let sizing = sizing(view, root, t)?;
        let available = |axis: usize, name: &str| -> Result<AvailableSpace, StoreError> {
            Ok(if sizing[axis] == Sizing::Fixed { AvailableSpace::Definite(view.number(root, name, 0.0, t)? as f32) } else { AvailableSpace::MaxContent })
        };
        let space = Size { width: available(0, WIDTH)?, height: available(1, HEIGHT)? };
        let measured = |known: Size<Option<f32>>, available: Size<AvailableSpace>, measure: Option<&mut Measure>| match measure {
            Some(m) => {
                // 折り返し幅は、決まった幅か、taffy が使ってよいと言った幅。どちらも無ければ折り返さない。
                let width = known.width.or(match available.width { AvailableSpace::Definite(w) => Some(w), _ => None });
                let [width, height] = super::text::measure_text(view, *m, width, known.height, t);
                Size { width, height }
            }
            None => Size::ZERO,
        };
        tree.compute_layout_with_measure(node, space, |known, available, _, measure, _| measured(known, available, measure))
        .map_err(|e| StoreError::Property(format!("layout: {e}")))?;
        if exclude_blobs(view, &mut tree, &groups, t)? {
            tree.compute_layout_with_measure(node, space, |known, available, _, measure, _| measured(known, available, measure))
            .map_err(|e| StoreError::Property(format!("layout: {e}")))?;
        }
        let taffy = |e: taffy::TaffyError| StoreError::Property(format!("layout: {e}"));
        let shifted = |mut placed: taffy::Layout| { placed.location.x += CANVAS_MARGIN; placed.location.y += CANVAS_MARGIN; placed };
        let mut groups_order = groups.clone();
        for (node, layer, is_root) in groups {
            let placed = shifted(*tree.layout(node).map_err(taffy)?);
            frame.sizes.insert(layer, [placed.size.width, placed.size.height]);
            if let Some(fields) = grid_fields(&tree, node) {
                frame.fields.insert(layer, fields);
            }
            if !is_root {
                let bounds = [CANVAS_MARGIN, CANVAS_MARGIN, CANVAS_MARGIN + placed.size.width, CANVAS_MARGIN + placed.size.height];
                let slot = slot(view, layer, t, bounds, [Sizing::Hug; 2], 3, placed)?;
                frame.slots.insert(layer, slot);
            }
            for &(_, child) in children.get(&layer).map(Vec::as_slice).unwrap_or(&[]) {
                if let Some(slot) = constrained(view, child, layer, t, [placed.size.width, placed.size.height])? {
                    frame.slots.insert(child, slot);
                }
            }
        }
        for leaf in leaves {
            let placed = shifted(*tree.layout(leaf.node).map_err(taffy)?);
            let slot = if leaf.text_fill {
                let scale = view.pair(leaf.layer, property::SCALE, [1.0, 1.0], t)?[0].abs().max(1e-3);
                let wrap = placed.size.width / scale;
                let bounds = super::text::text_box(view, leaf.layer, t, Some(wrap))?.unwrap_or(leaf.bounds);
                Slot { wrap: Some(wrap), ..slot(view, leaf.layer, t, bounds, [Sizing::Hug; 2], 3, placed)? }
            } else {
                slot(view, leaf.layer, t, leaf.bounds, leaf.sizing, leaf.fit, placed)?
            };
            frame.slots.insert(leaf.layer, slot);
        }
        // 奥行きの揃えは内の Group から(外の Group は内の奥行きを子の奥行きとして読む)。
        for (_, group, _) in groups_order {
            crate::doc::store::layout::boxes::align_depth(view, group, t, &children, &mut frame)?;
        }
    }
    push_apart(view, t, &displayed, &mut frame)?;
    Ok(frame)
}

/// 容器の外の兄弟同士の押し合い(間合いの法 2・3): Margin を宣言した物の箱(親の空間、間合いで広げる)の重なりを、
/// 決まった回数だけ押し戻す。中心から中心への向きへ、Flex Shrink の比で分ける。その瞬間の宣言だけから解く。
pub(crate) fn push_apart(view: &StoreView<'_>, t: RationalTime, displayed: &[LayerId], frame: &mut Frame) -> Result<(), StoreError> {
    const ROUNDS: usize = 32;
    // (層, 箱の最小, 箱の最大, 譲る比, 奥行きを持つか)。2D の物は奥行きの向きに押さない。
    let mut families: HashMap<Option<LayerId>, Vec<(LayerId, glam::Vec3, glam::Vec3, f32, bool)>> = HashMap::new();
    for layer in view.layers() {
        let margin = view.number(layer, MARGIN, 0.0, t)? as f32;
        if margin <= 0.0 || !view.here(layer, t)? {
            continue;
        }
        let attrs = view.attrs(layer)?.unwrap_or_default();
        let parent = attrs.parent;
        // 並ぶ子と、付いて置く物(流れの外)と、箱をつなぐ線・なぞる形(Margin は箱からの間合い)は押し合わない。
        if parent.is_some_and(|p| displayed.contains(&p)) || view.choice(layer, POSITION_AREA, t)? > 0
            || crate::doc::store::connect::connection(view, layer, t)?.is_some() || crate::doc::store::connect::tracing(view, layer, t)?.is_some() {
            continue;
        }
        // 並べる Group の箱は今解いた大きさ(覚えにはまだ入っていない)。
        let b = match frame.sizes.get(&layer) {
            Some(size) => [CANVAS_MARGIN, CANVAS_MARGIN, CANVAS_MARGIN + size[0], CANVAS_MARGIN + size[1]],
            None => match crate::doc::store::layout::boxes::layer_box(view, layer, t)? { Some(b) => b, None => continue },
        };
        // 押し合いの出発点は書いた位置(鍵・親)。ずれを含めた変換を読むと、前の時刻のずれを辿って巡る。
        let local = authored_local(view, layer, t)?;
        let corners = [[b[0], b[1]], [b[2], b[1]], [b[0], b[3]], [b[2], b[3]]].map(|c| local.transform_point2(glam::Vec2::from(c)));
        let lo = corners.iter().fold(glam::Vec2::MAX, |a, p| a.min(*p)) - glam::Vec2::splat(margin);
        let hi = corners.iter().fold(glam::Vec2::MIN, |a, p| a.max(*p)) + glam::Vec2::splat(margin);
        let spatial = attrs.projection != crate::doc::store::LayerProjection::TwoD;
        let (z0, z1) = if spatial {
            let z = view.number(layer, property::POSITION_Z, 0.0, t)? as f32;
            let range = crate::doc::store::layout::boxes::depth_range(view, layer, t, frame)?;
            ((z + range[0].min(range[1])) - margin, (z + range[0].max(range[1])) + margin)
        } else {
            (0.0, 0.0)
        };
        let shrink = view.number(layer, FLEX_SHRINK, 1.0, t)?.max(0.0) as f32;
        families.entry(parent).or_default().push((layer, glam::vec3(lo.x, lo.y, z0), glam::vec3(hi.x, hi.y, z1), shrink, spatial));
    }
    for (_, mut items) in families {
        if items.len() < 2 {
            continue;
        }
        items.sort_by_key(|item| item.0);
        let mut moved = vec![glam::Vec3::ZERO; items.len()];
        // 各回で全部の対を今の箱から同時に測って、まとめて動かす(順に動かすと、対の順番と止まる回で結果が跳ぶ)。
        for _ in 0..ROUNDS {
            let mut step = vec![glam::Vec3::ZERO; items.len()];
            for i in 0..items.len() {
                for j in i + 1..items.len() {
                    let (a, b) = (&items[i], &items[j]);
                    let (si, sj) = (a.3, b.3);
                    if si + sj <= 0.0 {
                        continue;
                    }
                    let (wi, wj) = (si / (si + sj), sj / (si + sj));
                    // 両方が奥行きを持つ時だけ、奥行きも測って押す(2D の物は面の上だけ)。
                    let axes = if a.4 && b.4 { 3 } else { 2 };
                    let mask = if axes == 3 { glam::Vec3::ONE } else { glam::vec3(1.0, 1.0, 0.0) };
                    // 中心から中心への向きに、離れるのに要るだけ押す(浅い軸で押すと、軸が入れ替わる瞬間に向きが 90° 跳ぶ)。
                    let gap = ((b.1 + b.2) * 0.5 - (a.1 + a.2) * 0.5) * mask;
                    let dir = if gap.length() > 1e-4 { gap.normalize() } else { glam::Vec3::X };
                    let half = ((a.2 - a.1) + (b.2 - b.1)) * 0.5;
                    let need = |axis: usize| {
                        let (g, u, h) = (gap[axis], dir[axis], half[axis]);
                        if u.abs() < 1e-6 { f32::INFINITY } else { ((h - g.abs()) / u.abs()).max(0.0) }
                    };
                    // 奥行きを測る対は、奥行きで重なっていなければ離れている。
                    let depth = (0..axes).map(need).fold(f32::INFINITY, f32::min);
                    if !depth.is_finite() || depth <= 0.0 || (0..axes).any(|axis| half[axis] - gap[axis].abs() <= 0.0) {
                        continue;
                    }
                    step[i] -= dir * depth * wi * 0.5;
                    step[j] += dir * depth * wj * 0.5;
                }
            }
            for (k, d) in step.into_iter().enumerate() {
                items[k].1 += d;
                items[k].2 += d;
                moved[k] += d;
            }
        }
        for (item, d) in items.iter().zip(moved) {
            if d.x != 0.0 || d.y != 0.0 {
                frame.nudges.insert(item.0, [d.x, d.y]);
            }
            if d.z != 0.0 {
                frame.nudges_z.insert(item.0, d.z);
            }
        }
    }
    Ok(())
}

/// 書いた値だけの層の変換(並べた結果・押し合いのずれを含まない)。
pub(super) fn authored_local(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<glam::Affine2, StoreError> {
    Ok(crate::doc::core::LayerPlacement::from_transform(
        crate::doc::store::layout::boxes::free_anchor(view, layer, t)?,
        crate::doc::store::view::resolve::transform::resolve_position(view, layer, t)?,
        view.pair(layer, property::SCALE, [1.0, 1.0], t)?,
        view.number(layer, property::ROTATION, 0.0, t)? as f32 + super::path::offset_rotation(view, layer, t)?,
        view.number(layer, property::SKEW, 0.0, t)? as f32,
        view.number(layer, property::SKEW_AXIS, 0.0, t)? as f32,
    ))
}

/// 流れの外の子の、親の箱への制約で付いていった置き場所(Figma の Constraints)。制約が Left / Top だけなら None(書いたまま)。
pub(crate) fn constrained(view: &StoreView<'_>, child: LayerId, parent: LayerId, t: RationalTime, size: [f32; 2]) -> Result<Option<Slot>, StoreError> {
    if view.choice(child, POSITION_TYPE, t)? != 1 {
        return Ok(None);
    }
    let modes = [view.choice(child, HORIZONTAL_CONSTRAINT, t)?, view.choice(child, VERTICAL_CONSTRAINT, t)?];
    let bounce = view.choice(parent, OVERFLOW, t)? == 2;
    if modes == [0, 0] && !bounce {
        return Ok(None);
    }
    // 基準は時刻 0 の親の箱。
    let design = if t == RationalTime::ZERO { size } else { view.layout_frame(RationalTime::ZERO)?.sizes.get(&parent).copied().unwrap_or(size) };
    let Some(b) = crate::doc::store::layout::boxes::layer_box(view, child, t)? else { return Ok(None) };
    let scale = view.pair(child, property::SCALE, [1.0, 1.0], t)?;
    let position = crate::doc::store::view::resolve::transform::resolve_position(view, child, t)?;
    let anchor = crate::doc::store::layout::boxes::free_anchor(view, child, t)?;
    let mut out_position = position;
    let mut out_scale = scale;
    for axis in 0..2 {
        let (lo, hi) = (position[axis] + (b[axis] - anchor[axis]) * scale[axis], position[axis] + (b[axis + 2] - anchor[axis]) * scale[axis]);
        let delta = size[axis] - design[axis];
        let ratio = if design[axis] > 1e-3 { size[axis] / design[axis] } else { 1.0 };
        let origin = CANVAS_MARGIN;
        let (new_lo, new_hi) = match modes[axis] {
            1 => (lo + delta, hi + delta),
            2 => (lo, hi + delta),
            3 => (lo + delta * 0.5, hi + delta * 0.5),
            4 => (origin + (lo - origin) * ratio, origin + (hi - origin) * ratio),
            _ => (lo, hi),
        };
        let extent = (b[axis + 2] - b[axis]).abs();
        if (hi - lo).abs() > 1e-6 && extent > 1e-6 {
            out_scale[axis] = scale[axis] * (new_hi - new_lo) / (hi - lo);
        }
        out_position[axis] = new_lo + (anchor[axis] - b[axis]) * out_scale[axis];
    }
    if bounce {
        let lo = [0, 1].map(|axis| out_position[axis] + (b[axis] - anchor[axis]).min(b[axis + 2] - anchor[axis]) * out_scale[axis]);
        let hi = [0, 1].map(|axis| out_position[axis] + (b[axis] - anchor[axis]).max(b[axis + 2] - anchor[axis]) * out_scale[axis]);
        let shift = bounced(lo, hi, size, view.number(parent, BORDER_RADIUS, 0.0, t)? as f32);
        out_position = [out_position[0] + shift[0], out_position[1] + shift[1]];
    }
    Ok(Some(Slot { position: out_position, scale: out_scale, stretch: [1.0, 1.0], wrap: None, z: 0.0, scale_z: 1.0, rotation: [0.0; 3], anchor }))
}

pub(crate) fn sizing(view: &StoreView<'_>, layer: LayerId, t: RationalTime) -> Result<[Sizing; 2], StoreError> {
    let of = |v: i64| match v { 1 => Sizing::Fill, 2 => Sizing::Fixed, _ => Sizing::Hug };
    Ok([of(view.choice(layer, HORIZONTAL_SIZING, t)?), of(view.choice(layer, VERTICAL_SIZING, t)?)])
}

/// 子としての style(並ぶ側の欄)。`natural` は Hug の時の大きさ。
pub(crate) fn item_style(view: &StoreView<'_>, layer: LayerId, t: RationalTime, natural: [f32; 2], style: &mut Style) -> Result<[Sizing; 2], StoreError> {
    let sizing = sizing(view, layer, t)?;
    let fixed = [view.number(layer, WIDTH, 100.0, t)? as f32, view.number(layer, HEIGHT, 100.0, t)? as f32];
    let dimension = |axis: usize| match sizing[axis] {
        Sizing::Hug => Dimension::length(natural[axis]),
        Sizing::Fixed => Dimension::length(fixed[axis]),
        Sizing::Fill => Dimension::auto(),
    };
    style.size = Size { width: dimension(0), height: dimension(1) };
    style.min_size = Size { width: Dimension::length(0.0), height: Dimension::length(0.0) };
    // CSS の既定は 1 だが、書いていない子は縮めない(Hug の箱を潰さない)。書いた比だけ譲る。
    style.flex_shrink = match view.value_at(layer, &PropertyId::new(FLEX_SHRINK)?, t)? {
        Some(Value::F64(v)) => v.max(0.0) as f32,
        _ => 0.0,
    };
    let margin = view.number(layer, MARGIN, 0.0, t)?.max(0.0) as f32;
    if margin > 0.0 {
        let m = LengthPercentageAuto::length(margin);
        style.margin = Rect { left: m, right: m, top: m, bottom: m };
    }
    if sizing.contains(&Sizing::Fill) {
        let row = matches!(choice_of_parent(view, layer, FLEX_DIRECTION, t)?, 0 | 2);
        let main = if row { 0 } else { 1 };
        if sizing[main] == Sizing::Fill {
            style.flex_grow = 1.0;
            style.flex_basis = Dimension::length(0.0);
        }
        if sizing[1 - main] == Sizing::Fill {
            style.align_self = Some(AlignSelf::STRETCH);
        }
    }
    match view.choice(layer, ALIGN_SELF, t)? {
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
    style.grid_column = line(view.number(layer, COLUMN_START, 0.0, t)?, view.number(layer, COLUMN_SPAN, 1.0, t)?);
    style.grid_row = line(view.number(layer, ROW_START, 0.0, t)?, view.number(layer, ROW_SPAN, 1.0, t)?);
    if choice_of_parent(view, layer, FLEX_DIRECTION, t)? == DIRECTION_DEPTH && view.attrs(layer)?.unwrap_or_default().parent.map(|p| view.display(p, t)).transpose()? == Some(1) {
        style.grid_column = line(1.0, 1.0);
        style.grid_row = line(1.0, 1.0);
    }
    Ok(sizing)
}

pub(crate) fn choice_of_parent(view: &StoreView<'_>, layer: LayerId, name: &str, t: RationalTime) -> Result<i64, StoreError> {
    match view.attrs(layer)?.unwrap_or_default().parent {
        Some(parent) => view.choice(parent, name, t),
        None => Ok(0),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn container(
    view: &StoreView<'_>,
    tree: &mut TaffyTree<Measure>,
    group: LayerId,
    t: RationalTime,
    children: &HashMap<LayerId, Vec<(i16, LayerId)>>,
    displayed: &[LayerId],
    is_root: bool,
    leaves: &mut Vec<Leaf>,
    groups: &mut Vec<(NodeId, LayerId, bool)>,
) -> Result<NodeId, StoreError> {
    let display = view.display(group, t)?;
    let padding = view.pair(group, PADDING, [0.0, 0.0], t)?;
    let gap = view.number(group, GAP, 0.0, t)? as f32;
    let mut style = Style {
        display: if display == 2 { Display::Grid } else { Display::Flex },
        flex_direction: match view.choice(group, FLEX_DIRECTION, t)? { 1 => FlexDirection::Column, 2 => FlexDirection::RowReverse, 3 => FlexDirection::ColumnReverse, _ => FlexDirection::Row },
        flex_wrap: match view.choice(group, FLEX_WRAP, t)? { 1 => FlexWrap::Wrap, 2 => FlexWrap::WrapReverse, _ => FlexWrap::NoWrap },
        justify_content: Some(match view.choice(group, JUSTIFY_CONTENT, t)? { 1 => JustifyContent::FLEX_END, 2 => JustifyContent::CENTER, 3 => JustifyContent::SPACE_BETWEEN, 4 => JustifyContent::SPACE_AROUND, 5 => JustifyContent::SPACE_EVENLY, _ => JustifyContent::FLEX_START }),
        align_items: Some(match view.choice(group, ALIGN_ITEMS, t)? { 1 => AlignItems::FLEX_START, 2 => AlignItems::FLEX_END, 3 => AlignItems::CENTER, _ => AlignItems::STRETCH }),
        gap: Size { width: LengthPercentage::length(gap), height: LengthPercentage::length(gap) },
        padding: Rect {
            left: LengthPercentage::length(padding[0]),
            right: LengthPercentage::length(padding[0]),
            top: LengthPercentage::length(padding[1]),
            bottom: LengthPercentage::length(padding[1]),
        },
        ..Style::default()
    };
    if display == 1 && view.choice(group, FLEX_DIRECTION, t)? == DIRECTION_DEPTH {
        // 面の上では全員が 1 つの枠に重なる(奥行きは後で積む)。揃えは Align Items を縦横に。
        style.display = Display::Grid;
        style.grid_template_columns = vec![GridTemplateComponent::Single(TrackSizingFunction::AUTO)];
        style.grid_template_rows = vec![GridTemplateComponent::Single(TrackSizingFunction::AUTO)];
        style.justify_items = style.align_items;
    }
    if display == 2 {
        let tracks = |count: &str, prefix: &str, default: f64| -> Result<Vec<GridTemplateComponent<String>>, StoreError> {
            let n = view.number(group, count, default, t)?.round().clamp(0.0, 64.0) as u32;
            (1..=n).map(|i| Ok(fr(view.number(group, &format!("{prefix}{i}"), TRACK_DEFAULT, t)?.max(0.0) as f32))).collect()
        };
        style.grid_template_columns = tracks(GRID_COLUMNS, COLUMN_PREFIX, 2.0)?;
        style.grid_template_rows = tracks(GRID_ROWS, ROW_PREFIX, 0.0)?;
    }
    if is_root {
        let sizing = sizing(view, group, t)?;
        let fixed = |axis: usize, name: &str| -> Result<Dimension, StoreError> {
            Ok(if sizing[axis] == Sizing::Fixed { Dimension::length(view.number(group, name, 0.0, t)? as f32) } else { Dimension::auto() })
        };
        style.size = Size { width: fixed(0, WIDTH)?, height: fixed(1, HEIGHT)? };
    } else {
        let sizing = item_style(view, group, t, [0.0, 0.0], &mut style)?;
        let auto = |axis: usize, d: Dimension| if sizing[axis] == Sizing::Hug { Dimension::auto() } else { d };
        style.size = Size { width: auto(0, style.size.width), height: auto(1, style.size.height) };
    }
    let mut nodes = Vec::new();
    for &(_, child) in children.get(&group).map(Vec::as_slice).unwrap_or(&[]) {
        if view.choice(child, POSITION_TYPE, t)? == 1 {
            continue;
        }
        if displayed.contains(&child) {
            nodes.push(container(view, tree, child, t, children, displayed, false, leaves, groups)?);
            continue;
        }
        let bounds = crate::doc::store::layout::boxes::layer_box(view, child, t)?.unwrap_or([0.0; 4]);
        let scale = view.pair(child, property::SCALE, [1.0, 1.0], t)?;
        let extent = crate::doc::store::layout::boxes::natural_extent(view, child, t, bounds, scale)?;
        let natural = [extent.hi[0] - extent.lo[0], extent.hi[1] - extent.lo[1]];
        let mut item = Style::default();
        let sizing = item_style(view, child, t, natural, &mut item)?;
        let text_fill = sizing[0] == Sizing::Fill && view.meta(child)?.is_some_and(|m| m.source == LayerSource::Text);
        let node = if text_fill {
            if sizing[1] == Sizing::Hug {
                item.size.height = Dimension::auto();
            }
            tree.new_leaf_with_context(item, Measure { layer: child, scale: scale[0].abs().max(1e-3) })
        } else {
            tree.new_leaf(item)
        }
        .map_err(|e| StoreError::Property(format!("layout: {e}")))?;
        leaves.push(Leaf { node, text_fill, layer: child, bounds, sizing, fit: view.choice(child, OBJECT_FIT, t)? });
        nodes.push(node);
    }
    // 子が全部流れの外でも、格子は升目を持つ(吸い付く子と格子の線が読む): 大きさ 0 の見えない子で格子の計算を走らせる
    // (taffy は子の無い箱を葉として解き、升目を出さない)。
    if nodes.is_empty() && view.display(group, t)? == 2 {
        nodes.push(tree.new_leaf(Style::default()).map_err(|e| StoreError::Property(format!("layout: {e}")))?);
    }
    let node = tree.new_with_children(style, &nodes).map_err(|e| StoreError::Property(format!("layout: {e}")))?;
    groups.push((node, group, is_root));
    Ok(node)
}

/// 置かれた枠へ、層の箱を合わせる Position と Scale(と形の輪郭の伸び)。Position の値はずれとして足す。
pub(crate) fn slot(view: &StoreView<'_>, layer: LayerId, t: RationalTime, bounds: [f32; 4], sizing: [Sizing; 2], fit: i64, placed: taffy::Layout) -> Result<Slot, StoreError> {
    let scale = view.pair(layer, property::SCALE, [1.0, 1.0], t)?;
    let offset = crate::doc::store::view::resolve::transform::resolve_position(view, layer, t)?;
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
    let is_shape = view.meta(layer)?.is_some_and(|m| m.source == LayerSource::Shape);
    let (stretch, bounds, scale) = if is_shape && factor != [1.0, 1.0] {
        (factor, stretched_shape_box(&view.shapes_at(layer, t)?, factor).unwrap_or(bounds), scale)
    } else {
        ([1.0, 1.0], bounds, [scale[0] * factor[0], scale[1] * factor[1]])
    };
    let Extent { lo, hi, anchor, rotation } = crate::doc::store::layout::boxes::placed_extent(view, layer, t, bounds, scale)?;
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

/// Exclusions: 格子の Group が指す Blob Track の塊が重なる枠へ、見えない子を明示の位置で置く。自動の子は残りの枠へ流れる。
/// 塊は comp の px なので、Group の素材座標へ戻してから枠と比べる。置いたら true(並べ直す)。
pub(crate) fn exclude_blobs(view: &StoreView<'_>, tree: &mut TaffyTree<Measure>, groups: &[(NodeId, LayerId, bool)], t: RationalTime) -> Result<bool, StoreError> {
    let taffy = |e: taffy::TaffyError| StoreError::Property(format!("layout: {e}"));
    let mut changed = false;
    for &(node, group, _) in groups {
        if view.display(group, t)? != 2 {
            continue;
        }
        let source = match view.value_at(group, &PropertyId::new(EXCLUSIONS)?, t)? {
            // 層を指す欄は窓から数として届くこともある(Blob Track の Track Layer と同じ読み方)。
            Some(Value::LayerId(id)) if id != 0 => LayerId(id),
            Some(Value::F64(v)) if v >= 1.0 => LayerId(v.round() as u64),
            _ => continue,
        };
        let Some(marks) = view.analysis().and_then(|a| a.blobs(source, crate::doc::store::EffectId(0), t)) else { continue };
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
        let to_local = crate::doc::store::layout::boxes::world_2d(view, group, t)?.inverse();
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
