use super::*;
use motolii_edit::{Animate, Document, Intent};
use crate::doc::eval::Keyframe;
use crate::doc::store::{layout, property, Composition, EffectId, EffectInstance, Fps, Interp, KeyframeTrack, LayerAttrsPatch, LayerId, LayerMeta, LayerProjection, LayerSource, LayerTiming, PathSource, PropertyId, RationalTime, Shape, ShapeNode, Value};
use crate::doc::vector::{Brush, Fill, Point, Rgb};
use crate::render::engine::Engine;

const W: u32 = 160;
const H: u32 = 100;

/// 灰色でない住む箱(Display の Group、角丸 0)の中を、白い四角が右下へまっすぐ漂う。`gpu` なら子に Bounce のブロック、
/// そうでなければ親の Overflow = Bounce(書類の CPU の法)。
fn scene(gpu: bool) -> Document {
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [0.0; 4] })).unwrap();
    let (group, child) = (LayerId(1), LayerId(2));
    let two_d = LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() };
    doc.apply_all([
        Intent::AddLayer(group),
        Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 90) } },
        Intent::SetAttrs { layer: group, patch: two_d.clone() },
        Intent::AddLayer(child),
        Intent::SetMeta { layer: child, meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(0, None, 90) } },
        Intent::SetAttrs { layer: child, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d } },
        Intent::SetShapes { layer: child, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 16.0, y: 16.0 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }) })] },
    ]).unwrap();
    let put = |doc: &mut Document, layer, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
    put(&mut doc, group, property::POSITION, Value::Vec2([10.0, 10.0]));
    put(&mut doc, group, layout::DISPLAY, Value::Enum(1));
    put(&mut doc, group, layout::HORIZONTAL_SIZING, Value::Enum(2));
    put(&mut doc, group, layout::VERTICAL_SIZING, Value::Enum(2));
    put(&mut doc, group, layout::WIDTH, Value::F64(120.0));
    put(&mut doc, group, layout::HEIGHT, Value::F64(70.0));
    put(&mut doc, child, layout::POSITION_TYPE, Value::Enum(1));
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe { t: RationalTime::ZERO, value: Value::Vec2([30.0, 20.0]), interp: Interp::Linear, spatial: None });
    track.insert(Keyframe { t: RationalTime::try_new(3, 1).unwrap(), value: Value::Vec2([30.0 + 390.0, 20.0 + 240.0]), interp: Interp::Linear, spatial: None });
    doc.apply(Intent::SetTrack { layer: child, property: PropertyId::new(property::POSITION).unwrap(), track }).unwrap();
    if gpu {
        doc.apply(Intent::SetEffects { layer: child, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.bounce_block".into() }] }).unwrap();
    } else {
        put(&mut doc, group, layout::OVERFLOW, Value::Enum(2));
    }
    doc
}

/// 住む箱(Display の Group、(10,10) から 120×70、`clip` なら Overflow Clip)の底の白い四角(16 px、箱の (30, 50))が、
/// `start` コマ目から Arrive で下から来る(From 90・Distance 80・Arrive 0.7・Bounce 0・Stagger 0・Spin 0)。
fn arrive_scene(start: i64, clip: bool) -> Document {
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [0.0; 4] })).unwrap();
    let (group, child) = (LayerId(1), LayerId(2));
    let two_d = LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() };
    doc.apply_all([
        Intent::AddLayer(group),
        Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 90) } },
        Intent::SetAttrs { layer: group, patch: two_d.clone() },
        Intent::AddLayer(child),
        Intent::SetMeta { layer: child, meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(start, None, 90) } },
        Intent::SetAttrs { layer: child, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d } },
        Intent::SetShapes { layer: child, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 16.0, y: 16.0 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }) })] },
        Intent::SetEffects { layer: child, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.arrive".into() }] },
    ]).unwrap();
    let put = |doc: &mut Document, layer, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
    put(&mut doc, group, property::POSITION, Value::Vec2([10.0, 10.0]));
    put(&mut doc, group, layout::DISPLAY, Value::Enum(1));
    put(&mut doc, group, layout::HORIZONTAL_SIZING, Value::Enum(2));
    put(&mut doc, group, layout::VERTICAL_SIZING, Value::Enum(2));
    put(&mut doc, group, layout::WIDTH, Value::F64(120.0));
    put(&mut doc, group, layout::HEIGHT, Value::F64(70.0));
    if clip {
        put(&mut doc, group, layout::OVERFLOW, Value::Enum(1));
    }
    put(&mut doc, child, layout::POSITION_TYPE, Value::Enum(1));
    put(&mut doc, child, property::POSITION, Value::Vec2([30.0, 50.0]));
    for (name, value) in [("side", 90.0), ("distance", 80.0), ("arrive", 0.7), ("bounce", 0.0), ("stagger", 0.0), ("spin", 0.0)] {
        put(&mut doc, child, &format!("{}0.param.{name}", property::EFFECT_PREFIX), Value::F64(value));
    }
    doc
}

/// 描かれた行(alpha > 128 の画素の y、重複なし・昇順)。
fn lit_rows(pixels: &[u8]) -> Vec<u32> {
    let mut rows: Vec<u32> = pixels.chunks_exact(4).enumerate().filter(|(_, c)| c[3] > 128).map(|(i, _)| (i as u32) / W).collect();
    rows.sort_unstable();
    rows.dedup();
    rows
}

/// 箱の切りは箱の枠で(CSS の overflow: clip は要素の箱に掛かり、中で transform した子は箱で切れる): Overflow Clip の箱の
/// 子が Arrive で下から来る途中、箱の外(下)の画素は透明。着いた後は箱の中に丸ごと見える。
#[test]
fn the_boxs_clip_cuts_what_arrive_moves_in_the_boxs_frame() {
    let doc = arrive_scene(0, true);
    let mut engine = Engine::new().unwrap();
    let fps = Fps::try_new(30, 1).unwrap();
    let at = |engine: &mut Engine, frame: i64| lit_rows(&engine.render_frame(&doc.view(), RationalTime::try_from_frame(frame, fps).unwrap()).unwrap());
    let landed = at(&mut engine, 30);
    let bottom = 10 + 70 + 1;
    assert!(landed.len() >= 15 && *landed.last().unwrap() < bottom, "landed: the whole square sits inside the box: {landed:?}");
    // frame 9 = 0.3 s of 0.7: left = (1 − 3/7)^3 ≈ 0.19 → 15 px below its slot → the square straddles the box's bottom.
    let mid = at(&mut engine, 9);
    assert!(!mid.is_empty() && mid[0] > landed[0], "mid-arrive: the square is on its way up (rows {mid:?} vs landed {landed:?})");
    assert!(*mid.last().unwrap() < bottom && mid.len() < landed.len(), "mid-arrive: the box cuts it at its bottom edge, the offset moved only the content: rows {mid:?}");
}

/// つなぐ線は、ブロックが動かした相手に付いて行く(線の端が GPU で相手の motion を読む、利用者 2026-09-18
/// 「位置は毎コマ変わるのに GPU じゃないの変すぎ」): 静かな白い四角と、Arrive で下から来る白い四角を赤い線で結ぶ。
/// 来る途中(frame 9)の線の下端は、着いた後(frame 30)より下にある。
#[test]
fn a_connector_follows_what_a_block_moved() {
    let mut doc = arrive_scene(0, false);
    let (still, line) = (LayerId(3), LayerId(4));
    let two_d = LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() };
    let white = Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() });
    doc.apply_all([
        Intent::AddLayer(still),
        Intent::SetMeta { layer: still, meta: LayerMeta { source: LayerSource::Shape, order: 2, timing: LayerTiming::place(0, None, 90) } },
        Intent::SetAttrs { layer: still, patch: LayerAttrsPatch { parent: Some(Some(LayerId(1))), ..two_d.clone() } },
        Intent::SetShapes { layer: still, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 16.0, y: 16.0 } }, ops: Vec::new(), stroke: None, fill: white })] },
        Intent::AddLayer(line),
        Intent::SetMeta { layer: line, meta: LayerMeta { source: LayerSource::Shape, order: 3, timing: LayerTiming::place(0, None, 90) } },
        Intent::SetAttrs { layer: line, patch: LayerAttrsPatch { parent: Some(Some(LayerId(1))), ..two_d } },
        Intent::SetShapes { layer: line, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 4.0, y: 4.0 } }, ops: Vec::new(), fill: None,
            stroke: Some(crate::doc::vector::Stroke { brush: Brush::Solid(Rgb { r: 1.0, g: 0.0, b: 0.0 }), width: 3.0, ..Default::default() }) })] },
    ]).unwrap();
    let put = |doc: &mut Document, layer, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
    put(&mut doc, still, layout::POSITION_TYPE, Value::Enum(1));
    put(&mut doc, still, property::POSITION, Value::Vec2([90.0, 8.0]));
    put(&mut doc, line, layout::CONNECT_FROM, Value::LayerId(still.0));
    put(&mut doc, line, layout::CONNECT_TO, Value::LayerId(LayerId(2).0));
    let mut engine = Engine::new().unwrap();
    let fps = Fps::try_new(30, 1).unwrap();
    let red_bottom = |engine: &mut Engine, frame: i64| -> Option<u32> {
        let pixels = engine.render_frame(&doc.view(), RationalTime::try_from_frame(frame, fps).unwrap()).unwrap();
        pixels.chunks_exact(4).enumerate().filter(|(_, c)| c[0] > 150 && c[1] < 90 && c[2] < 90 && c[3] > 128).map(|(i, _)| (i as u32) / W).max()
    };
    let landed = red_bottom(&mut engine, 30).expect("the line is drawn once the square has landed");
    let mid = red_bottom(&mut engine, 9).expect("the line is drawn while the square is on its way");
    assert!(mid > landed + 6, "mid-arrive the line's lower end is with the square below its slot: mid {mid} vs landed {landed}");
}

/// GPU のブロックが描く位置は、書類の CPU の Bounce と同じ(読み戻さずに描く側の頂点で動く)。
#[test]
fn the_bounce_block_draws_where_the_cpu_bounce_does() {
    let (cpu, gpu) = (scene(false), scene(true));
    let mut engine = Engine::new().unwrap();
    for frame in [0, 20, 45, 70] {
        let t = RationalTime::try_from_frame(frame, Fps::try_new(30, 1).unwrap()).unwrap();
        let expected = engine.render_frame(&cpu.view(), t).unwrap();
        let actual = engine.render_frame(&gpu.view(), t).unwrap();
        let lit = |p: &[u8]| p.chunks_exact(4).enumerate().filter(|(_, c)| c[3] > 128).map(|(i, _)| ((i as u32) % W, (i as u32) / W)).collect::<Vec<_>>();
        let (e, a) = (lit(&expected), lit(&actual));
        let centre = |v: &[(u32, u32)]| { let n = v.len().max(1) as f32; (v.iter().map(|p| p.0 as f32).sum::<f32>() / n, v.iter().map(|p| p.1 as f32).sum::<f32>() / n) };
        assert!(!e.is_empty() && (e.len() as i64 - a.len() as i64).abs() < 40, "frame {frame}: the square is drawn once in both ({} vs {} px)", e.len(), a.len());
        let (ce, ca) = (centre(&e), centre(&a));
        assert!((ce.0 - ca.0).abs() < 1.0 && (ce.1 - ca.1).abs() < 1.0, "frame {frame}: cpu centre {ce:?}, gpu centre {ca:?}");
    }
}

/// 場(`SCOPE: room`)は、掛かった層でなく**同じ住む箱に居る他の全員**を動かす。相手は効果を 1 枚も持たない。
#[test]
fn a_field_moves_everyone_in_its_room_who_carries_no_effect() {
    let fps = Fps::try_new(30, 1).unwrap();
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps, duration_frames: 90, background: [0.0; 4] })).unwrap();
    let (group, ball, field) = (LayerId(1), LayerId(2), LayerId(3));
    let two_d = LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() };
    let square = |size: f32, fill: Rgb| ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: size as f64, y: size as f64 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(fill), ..Default::default() }) });
    doc.apply_all([
        Intent::AddLayer(group),
        Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 90) } },
        Intent::SetAttrs { layer: group, patch: two_d.clone() },
        Intent::AddLayer(ball),
        Intent::SetMeta { layer: ball, meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(0, None, 90) } },
        Intent::SetAttrs { layer: ball, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d.clone() } },
        Intent::SetShapes { layer: ball, shapes: vec![square(16.0, Rgb { r: 1.0, g: 1.0, b: 1.0 })] },
        Intent::AddLayer(field),
        Intent::SetMeta { layer: field, meta: LayerMeta { source: LayerSource::Shape, order: 2, timing: LayerTiming::place(0, None, 90) } },
        Intent::SetAttrs { layer: field, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d } },
        Intent::SetShapes { layer: field, shapes: vec![square(4.0, Rgb { r: 1.0, g: 0.0, b: 0.0 })] },
    ]).unwrap();
    let put = |doc: &mut Document, layer, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
    put(&mut doc, group, property::POSITION, Value::Vec2([10.0, 10.0]));
    put(&mut doc, group, layout::DISPLAY, Value::Enum(1));
    put(&mut doc, group, layout::HORIZONTAL_SIZING, Value::Enum(2));
    put(&mut doc, group, layout::VERTICAL_SIZING, Value::Enum(2));
    put(&mut doc, group, layout::WIDTH, Value::F64(120.0));
    put(&mut doc, group, layout::HEIGHT, Value::F64(70.0));
    for layer in [ball, field] {
        put(&mut doc, layer, layout::POSITION_TYPE, Value::Enum(1));
    }
    put(&mut doc, ball, property::POSITION, Value::Vec2([30.0, 15.0]));
    put(&mut doc, field, property::POSITION, Value::Vec2([100.0, 15.0]));
    // 一様(Spread 1)の場を下(+90°)へ。落ちる速さは外の解き手(Rapier)が決めるので、
    // ここで見るのは「効果を持たない隣人が場だけで下へ動き、箱の中で止まる」という法の意味。
    doc.apply(Intent::SetEffects { layer: field, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.field".into() }] }).unwrap();
    for (name, value) in [("spread", 1.0), ("turn", 0.0), ("angle", 90.0), ("strength", 40.0), ("reach", 0.0)] {
        doc.apply(Intent::SetConstant { layer: field, property: PropertyId::effect_param(EffectId(0), name).unwrap(), value: Value::F64(value) }).unwrap();
    }
    let mut engine = Engine::new().unwrap();
    // 白い四角(場の元は赤)の真ん中。
    let centre = |pixels: &[u8]| {
        let hits: Vec<(f32, f32)> = pixels.chunks_exact(4).enumerate()
            .filter(|(_, c)| c[3] > 128 && c[2] > 128)
            .map(|(i, _)| ((i as u32 % W) as f32, (i as u32 / W) as f32)).collect();
        let n = hits.len().max(1) as f32;
        (hits.len(), hits.iter().map(|p| p.0).sum::<f32>() / n, hits.iter().map(|p| p.1).sum::<f32>() / n)
    };
    let (n0, x0, y0) = centre(&engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap());
    let (n1, x1, y1) = centre(&engine.render_frame(&doc.view(), RationalTime::try_from_frame(30, fps).unwrap()).unwrap());
    let (n2, _, y2) = centre(&engine.render_frame(&doc.view(), RationalTime::try_from_frame(60, fps).unwrap()).unwrap());
    assert!(n0 > 100 && n1 > 100 && n2 > 100, "白い四角がどのコマにも在る ({n0} / {n1} / {n2} px)");
    assert!((x1 - x0).abs() < 2.0, "横には動かない ({x0} → {x1})");
    assert!(y1 > y0 + 5.0 && y2 >= y1 - 1.0, "場だけで下へ動く ({y0} → {y1} → {y2})");
    // 箱の床(親の Group の下辺)より下へは行かない。
    assert!(y2 < 88.0, "箱の中で止まる (y {y2})");
}

/// 場の動きはコマからコマへ飛ばない(利用者 2026-09-16「動きは離散的にならないように」)。
/// 同じ画を 30 コマ描いて、真ん中の動いた量の差(加速度)が、動いた量そのものより小さいことを見る。
#[test]
fn a_field_moves_without_jumping_between_frames() {
    let fps = Fps::try_new(30, 1).unwrap();
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps, duration_frames: 40, background: [0.0; 4] })).unwrap();
    let (group, ball, field) = (LayerId(1), LayerId(2), LayerId(3));
    let two_d = LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() };
    let square = |size: f64, fill: Rgb| ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: size, y: size } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(fill), ..Default::default() }) });
    doc.apply_all([
        Intent::AddLayer(group),
        Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 40) } },
        Intent::SetAttrs { layer: group, patch: two_d.clone() },
        Intent::AddLayer(ball),
        Intent::SetMeta { layer: ball, meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(0, None, 40) } },
        Intent::SetAttrs { layer: ball, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d.clone() } },
        Intent::SetShapes { layer: ball, shapes: vec![square(14.0, Rgb { r: 1.0, g: 1.0, b: 1.0 })] },
        Intent::AddLayer(field),
        Intent::SetMeta { layer: field, meta: LayerMeta { source: LayerSource::Shape, order: 2, timing: LayerTiming::place(0, None, 40) } },
        Intent::SetAttrs { layer: field, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d } },
        Intent::SetShapes { layer: field, shapes: vec![square(3.0, Rgb { r: 1.0, g: 0.0, b: 0.0 })] },
    ]).unwrap();
    let put = |doc: &mut Document, layer, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
    put(&mut doc, group, property::POSITION, Value::Vec2([8.0, 8.0]));
    put(&mut doc, group, layout::DISPLAY, Value::Enum(1));
    put(&mut doc, group, layout::HORIZONTAL_SIZING, Value::Enum(2));
    put(&mut doc, group, layout::VERTICAL_SIZING, Value::Enum(2));
    put(&mut doc, group, layout::WIDTH, Value::F64(140.0));
    put(&mut doc, group, layout::HEIGHT, Value::F64(84.0));
    for layer in [ball, field] {
        put(&mut doc, layer, layout::POSITION_TYPE, Value::Enum(1));
    }
    put(&mut doc, ball, property::POSITION, Value::Vec2([20.0, 20.0]));
    put(&mut doc, field, property::POSITION, Value::Vec2([110.0, 60.0]));
    doc.apply(Intent::SetEffects { layer: field, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.field".into() }] }).unwrap();
    // 元へ寄る場(Spread 0、Turn 0)。一番動きが速い所を含む 30 コマを見る。
    for (name, value) in [("spread", 0.0), ("turn", 0.0), ("strength", 220.0), ("reach", 0.0), ("tumble", 0.0), ("hold", 0.0)] {
        doc.apply(Intent::SetConstant { layer: field, property: PropertyId::effect_param(EffectId(0), name).unwrap(), value: Value::F64(value) }).unwrap();
    }
    let mut engine = Engine::new().unwrap();
    let centre = |pixels: &[u8]| {
        let hits: Vec<(f32, f32)> = pixels.chunks_exact(4).enumerate()
            .filter(|(_, c)| c[3] > 128 && c[2] > 128)
            .map(|(i, _)| ((i as u32 % W) as f32, (i as u32 / W) as f32)).collect();
        let n = hits.len().max(1) as f32;
        (hits.iter().map(|p| p.0).sum::<f32>() / n, hits.iter().map(|p| p.1).sum::<f32>() / n)
    };
    let path: Vec<(f32, f32)> = (0..30).map(|f| centre(&engine.render_frame(&doc.view(), RationalTime::try_from_frame(f, fps).unwrap()).unwrap())).collect();
    let step: Vec<f32> = path.windows(2).map(|w| ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt()).collect();
    let moved: f32 = step.iter().sum();
    assert!(moved > 20.0, "場が動かしている ({moved}px)");
    let jump = step.windows(2).map(|w| (w[1] - w[0]).abs()).fold(0.0f32, f32::max);
    let fastest = step.iter().copied().fold(0.0f32, f32::max);
    assert!(jump <= fastest * 0.5, "コマ間の変わり方が跳ばない(最大の差 {jump}px、一番速いコマ {fastest}px)");
}

/// 落ちて積もった後は震えない(利用者 2026-09-16「まだまだガッタガタやぞ」)。
/// 画素の平均差では物ごとの震えが見えないので、物ごとのずれをコマ順に読んで測る。
#[test]
fn things_stop_moving_once_they_have_settled() {
    let fps = Fps::try_new(30, 1).unwrap();
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps, duration_frames: 90, background: [0.0; 4] })).unwrap();
    let (group, field) = (LayerId(1), LayerId(2));
    let two_d = LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() };
    let square = |size: f64| ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: size, y: size } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }) });
    doc.apply_all([
        Intent::AddLayer(group),
        Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 90) } },
        Intent::SetAttrs { layer: group, patch: two_d.clone() },
        Intent::AddLayer(field),
        Intent::SetMeta { layer: field, meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(0, None, 90) } },
        Intent::SetAttrs { layer: field, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d.clone() } },
        Intent::SetShapes { layer: field, shapes: vec![square(2.0)] },
    ]).unwrap();
    let put = |doc: &mut Document, layer, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
    put(&mut doc, group, property::POSITION, Value::Vec2([6.0, 6.0]));
    put(&mut doc, group, layout::DISPLAY, Value::Enum(1));
    put(&mut doc, group, layout::HORIZONTAL_SIZING, Value::Enum(2));
    put(&mut doc, group, layout::VERTICAL_SIZING, Value::Enum(2));
    put(&mut doc, group, layout::WIDTH, Value::F64(148.0));
    put(&mut doc, group, layout::HEIGHT, Value::F64(88.0));
    put(&mut doc, field, layout::POSITION_TYPE, Value::Enum(1));
    put(&mut doc, field, property::POSITION, Value::Vec2([74.0, 44.0]));
    // 12 個の四角を上からばらばらに落とす。
    for i in 0..12u64 {
        let layer = LayerId(10 + i);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: 2 + i as i16, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetAttrs { layer, patch: LayerAttrsPatch { parent: Some(Some(group)), ..two_d.clone() } },
            Intent::SetShapes { layer, shapes: vec![square(16.0)] },
        ]).unwrap();
        put(&mut doc, layer, layout::POSITION_TYPE, Value::Enum(1));
        put(&mut doc, layer, property::POSITION, Value::Vec2([8.0 + (i % 6) as f64 * 22.0, -20.0 - (i / 6) as f64 * 30.0]));
    }
    doc.apply(Intent::SetEffects { layer: field, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.field".into() }] }).unwrap();
    for (name, value) in [("spread", 1.0), ("turn", 0.0), ("angle", 90.0), ("strength", 400.0), ("reach", 0.0), ("tumble", 0.4), ("hold", 0.0)] {
        doc.apply(Intent::SetConstant { layer: field, property: PropertyId::effect_param(EffectId(0), name).unwrap(), value: Value::F64(value) }).unwrap();
    }
    let mut engine = Engine::new().unwrap();
    // 2.0 秒から 2.5 秒(落ちて積もった後)。
    let mut frames = Vec::new();
    for frame in 60..=75 {
        engine.render_frame(&doc.view(), RationalTime::try_from_frame(frame, fps).unwrap()).unwrap();
        frames.push(engine.block_states());
    }
    assert!(frames[0].len() >= 12, "物が並んでいる ({})", frames[0].len());
    let mut worst = 0.0f32;
    for pair in frames.windows(2) {
        for (a, b) in pair[0].iter().zip(pair[1].iter()) {
            worst = worst.max(((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt());
        }
    }
    assert!(worst < 1.0, "積もった後は震えない(1 コマの動きの最大 {worst}px)");
}

/// Push Apart を GPU のブロックにしても、書類の間合いの押し合い(CPU、32 回)と同じだけ押す。
#[test]
fn the_push_apart_block_pushes_like_the_margin_law() {
    use crate::render::compositor::effects::block_program::{program_for, read_state, BlockItem, BlockWorld};
    let place = |margin: bool| {
        let mut doc = Document::new().with_programs(crate::extensions::bundled());
        doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 1, background: [0.0; 4] })).unwrap();
        let spots = [[40.0, 40.0], [52.0, 44.0], [60.0, 30.0], [100.0, 60.0], [104.0, 64.0], [20.0, 80.0]];
        for (i, at) in spots.iter().enumerate() {
            let layer = LayerId(i as u64 + 1);
            doc.apply_all([
                Intent::AddLayer(layer),
                Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: i as i16, timing: LayerTiming::place(0, None, 1) } },
                Intent::SetAttrs { layer, patch: LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() } },
                Intent::SetShapes { layer, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 20.0, y: 14.0 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }) })] },
                Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2(*at) },
            ]).unwrap();
            if margin {
                doc.apply(Intent::SetConstant { layer, property: PropertyId::new(layout::MARGIN).unwrap(), value: Value::F64(5.0) }).unwrap();
            }
        }
        doc
    };
    let (plain, pushed) = (place(false), place(true));
    let t = RationalTime::ZERO;
    let frame = crate::picture::frame::layout_frame(&pushed.view(), t).unwrap();
    let view = plain.view();
    let items: Vec<BlockItem> = (1..=6).map(|i| {
        let id = LayerId(i);
        let b = crate::picture::boxes::layer_box(&view, id, t).unwrap().unwrap();
        let m = crate::picture::resolve::transform::local_transform(&view, id, t).unwrap();
        let (lo, hi) = (m.transform_point2(glam::vec2(b[0], b[1])), m.transform_point2(glam::vec2(b[2], b[3])));
        BlockItem { lo: lo.to_array(), hi: hi.to_array(), room_lo: [0.0; 2], room_size: [W as f32, H as f32], radius: 0.0, group: 0, margin: 0.0, weight: 1.0, ..Default::default() }
    }).collect();
    let engine = Engine::new().unwrap();
    let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
    let program = program_for(device, "push_apart");
    let mut world = BlockWorld::new(device);
    world.begin(device, queue, &items, 0.0);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
    program.record(device, queue, &mut encoder, &mut world, 0.0, &[0, 1, 2, 3, 4, 5], &[5.0]);
    let gpu = read_state(device, queue, &world, encoder);
    let mut moved = 0;
    for i in 0..6 {
        let cpu = frame.nudges.get(&LayerId(i as u64 + 1)).copied().unwrap_or([0.0, 0.0]);
        let g = gpu[i].translate;
        if cpu != [0.0, 0.0] { moved += 1; }
        assert!((g[0] - cpu[0]).abs() < 0.05 && (g[1] - cpu[1]).abs() < 0.05, "object {i}: gpu {g:?} cpu {cpu:?}");
    }
    assert!(moved >= 4, "the overlapping ones were pushed");
}

/// ブロックは効果の順につながる: 押し合って(Push Apart)から壁で折り返す(Bounce)と、全員が箱の中に収まる。
#[test]
fn blocks_chain_in_effect_order() {
    use crate::render::compositor::effects::block_program::{program_for, read_state, BlockItem, BlockWorld};
    let engine = Engine::new().unwrap();
    let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
    let push = program_for(device, "push_apart");
    let bounce = program_for(device, "bounce");
    let room = [200.0f32, 120.0];
    let mut items = Vec::new();
    for i in 0..40 {
        let x = 60.0 + (i as f32 * 37.0) % 180.0;
        let y = 20.0 + (i as f32 * 23.0) % 110.0;
        items.push(BlockItem { lo: [x, y], hi: [x + 12.0, y + 12.0], room_lo: [0.0; 2], room_size: room, radius: 0.0, group: 7, margin: 0.0, weight: 1.0, ..Default::default() });
    }
    let mut world = BlockWorld::new(device);
    world.begin(device, queue, &items, 0.0);
    let members: Vec<u32> = (0..40).collect();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
    push.record(device, queue, &mut encoder, &mut world, 0.0, &members, &[2.0]);
    bounce.record(device, queue, &mut encoder, &mut world, 0.0, &members, &[1.0]);
    let state = read_state(device, queue, &world, encoder);
    for (i, (item, o)) in items.iter().zip(&state).enumerate() {
        let (lo, hi) = ([item.lo[0] + o.translate[0], item.lo[1] + o.translate[1]], [item.hi[0] + o.translate[0], item.hi[1] + o.translate[1]]);
        assert!(lo[0] >= -0.01 && lo[1] >= -0.01 && hi[0] <= room[0] + 0.01 && hi[1] <= room[1] + 0.01, "object {i} ends inside the room: {lo:?} {hi:?}");
    }
    let spread = state.iter().filter(|o| o.translate != [0.0, 0.0]).count();
    assert!(spread > 20, "the crowd was pushed and folded: {spread} moved");
}

/// ブロックごとに自分の欄を読む(1 コマに送る前の書き込みで、後のブロックの欄が前のブロックに混ざらない)。
#[test]
fn each_block_reads_its_own_params() {
    use crate::render::compositor::effects::block_program::{program_for, read_state, BlockItem, BlockWorld};
    let engine = Engine::new().unwrap();
    let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
    let bounce = program_for(device, "bounce");
    let push = program_for(device, "push_apart");
    let items = [BlockItem { lo: [300.0, 20.0], hi: [310.0, 30.0], room_lo: [0.0; 2], room_size: [200.0, 100.0], radius: 0.0, group: 1, margin: 0.0, weight: 1.0, ..Default::default() }];
    let mut world = BlockWorld::new(device);
    world.begin(device, queue, &items, 0.0);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
    bounce.record(device, queue, &mut encoder, &mut world, 0.0, &[0], &[0.0]);
    push.record(device, queue, &mut encoder, &mut world, 0.0, &[0], &[9.0]);
    let state = read_state(device, queue, &world, encoder);
    assert_eq!(state[0].translate, [0.0, 0.0], "Bounce at strength 0 does not fold, whatever the next block's margin is");
}

/// 調べる口: `MOTOLII_BLOCK_DOC` の書類の `MOTOLII_BLOCK_FRAME` コマの物の箱とブロックの結果を出す。
#[test]
#[ignore]
fn dump_blocks() {
    use crate::render::compositor::effects::block_program::read_state;
    let doc = Document::load(std::env::var("MOTOLII_BLOCK_DOC").unwrap()).unwrap().with_programs(crate::extensions::bundled());
    let frame: i64 = std::env::var("MOTOLII_BLOCK_FRAME").unwrap().parse().unwrap();
    let view = doc.view();
    let fps = view.composition().unwrap().unwrap().fps;
    let t = RationalTime::try_from_frame(frame, fps).unwrap();
    let mut engine = Engine::new().unwrap();
    let scope = engine.gpu_device().push_error_scope(wgpu::ErrorFilter::Validation);
    engine.render_frame(&view, t).unwrap();
    eprintln!("gpu validation: {:?}", pollster::block_on(scope.pop()));
    eprintln!("layer failures: {:?}", engine.layer_failures());
    for (k, o) in engine.blocks.objects.iter().enumerate() { eprintln!("object {k}: {o:?}"); }
    for b in &engine.blocks.batches { eprintln!("batch stage {} {} {:?} {:?}", b.stage, b.plugin, b.params, b.members); }
    let Some(world) = engine.blocks.world.as_ref() else { return };
    let encoder = engine.compositor.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    for (k, o) in read_state(&engine.compositor.ctx.device, &engine.compositor.ctx.queue, world, encoder).iter().enumerate() { eprintln!("state {k}: {o:?}"); }
}

/// 付いて置く札は、相手がブロックで動いた分だけ一緒に動く(GPU の中で、読み戻さずに)。
#[test]
fn an_anchored_label_follows_what_a_block_moved() {
    use crate::render::compositor::effects::block_program::read_state;
    let mut doc = scene(true);
    let label = LayerId(3);
    doc.apply_all([
        Intent::AddLayer(label),
        Intent::SetMeta { layer: label, meta: LayerMeta { source: LayerSource::Shape, order: 2, timing: LayerTiming::place(0, None, 90) } },
        Intent::SetAttrs { layer: label, patch: LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() } },
        Intent::SetShapes { layer: label, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 6.0, y: 4.0 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 0.0, b: 0.0 }), ..Default::default() }) })] },
        Intent::SetConstant { layer: label, property: PropertyId::new(layout::POSITION_ANCHOR).unwrap(), value: Value::LayerId(2) },
        Intent::SetConstant { layer: label, property: PropertyId::new(layout::POSITION_AREA).unwrap(), value: Value::Enum(2) },
    ]).unwrap();
    let mut engine = Engine::new().unwrap();
    let t = RationalTime::try_from_frame(45, Fps::try_new(30, 1).unwrap()).unwrap();
    engine.render_frame(&doc.view(), t).unwrap();
    let layers = engine.blocks.object_layers.clone();
    let world = engine.blocks.world.as_ref().unwrap();
    let encoder = engine.compositor.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    let state = read_state(&engine.compositor.ctx.device, &engine.compositor.ctx.queue, world, encoder);
    let at = |id: u64| state[layers.iter().position(|l| l.0 == id).expect("an object")].translate;
    assert!(at(2) != [0.0, 0.0], "the square was folded by Bounce");
    assert_eq!(at(3), at(2), "its label moved with it");
}

/// 名指しの口を書類から: 鍵で動く Hub に Position Anchor で名指しした tile の Effector は、Hub の今の中心で判定する
/// (鍵は休みの箱を動かすので `anchor_centre` = 休みの箱の中心 + ずれ)。Hub は block を持たなくても物になる。
#[test]
fn an_effector_takes_its_centre_from_the_hub_the_tile_is_anchored_to() {
    use crate::render::compositor::effects::block_program::read_state;
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [0.0; 4] })).unwrap();
    let (hub, tile) = (LayerId(1), LayerId(2));
    let two_d = LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() };
    let square = |size: f64| vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: size, y: size } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }) })];
    doc.apply_all([
        Intent::AddLayer(hub),
        Intent::SetMeta { layer: hub, meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, 90) } },
        Intent::SetAttrs { layer: hub, patch: two_d.clone() },
        Intent::SetShapes { layer: hub, shapes: square(8.0) },
        Intent::AddLayer(tile),
        Intent::SetMeta { layer: tile, meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(0, None, 90) } },
        Intent::SetAttrs { layer: tile, patch: two_d },
        Intent::SetShapes { layer: tile, shapes: square(16.0) },
        Intent::SetEffects { layer: tile, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.effector".into() }] },
    ]).unwrap();
    let put = |doc: &mut Document, layer, name: &str, value: Value| doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
    put(&mut doc, tile, property::POSITION, Value::Vec2([80.0, 50.0]));
    put(&mut doc, tile, layout::POSITION_ANCHOR, Value::LayerId(hub.0));
    for (name, value) in [("shape", 0.0), ("size", 30.0), ("soft", 6.0), ("strength", 1.0), ("lift", -20.0)] {
        put(&mut doc, tile, &format!("{}0.param.{name}", property::EFFECT_PREFIX), Value::F64(value));
    }
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe { t: RationalTime::ZERO, value: Value::Vec2([20.0, 50.0]), interp: Interp::Linear, spatial: None });
    track.insert(Keyframe { t: RationalTime::try_new(3, 1).unwrap(), value: Value::Vec2([140.0, 50.0]), interp: Interp::Linear, spatial: None });
    doc.apply(Intent::SetTrack { layer: hub, property: PropertyId::new(property::POSITION).unwrap(), track }).unwrap();
    let mut engine = Engine::new().unwrap();
    let fps = Fps::try_new(30, 1).unwrap();
    let mut at = |frame: i64| {
        engine.render_frame(&doc.view(), RationalTime::try_from_frame(frame, fps).unwrap()).unwrap();
        let slot = |id: LayerId| engine.blocks.slots[&id] as usize;
        let (h, k) = (slot(hub), slot(tile));
        assert_eq!(engine.blocks.objects[k].anchor_slot, h as u32, "the tile names the hub");
        assert_eq!(engine.blocks.objects[h].anchor_slot, u32::MAX, "the hub names nobody");
        let world = engine.blocks.world.as_ref().unwrap();
        let encoder = engine.compositor.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let state = read_state(&engine.compositor.ctx.device, &engine.compositor.ctx.queue, world, encoder);
        (engine.blocks.objects[h].lo, state[k].translate)
    };
    let (hub_far, tile_far) = at(0);
    let (hub_over, tile_over) = at(45);
    assert!((hub_over[0] - hub_far[0] - 60.0).abs() < 0.5, "the keys move the hub's rest box: {hub_far:?} -> {hub_over:?}");
    assert_eq!(tile_far, [0.0, 0.0], "hub 60 px away: outside the sphere, the tile sits still");
    assert!((tile_over[1] + 20.0).abs() < 1e-3 && tile_over[0].abs() < 1e-3, "hub over the tile: the tile lifts by Lift: {tile_over:?}");
}

/// Wave: 時刻と物の順で縦の正弦波。Wavelength 個離れた物は同じ高さ、半分なら逆。
#[test]
fn the_wave_block_travels_along_things_in_order() {
    use crate::render::compositor::effects::block_program::{program_for, read_state, BlockItem, BlockWorld};
    let engine = Engine::new().unwrap();
    let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
    let wave = program_for(device, "wave");
    let items: Vec<BlockItem> = (0..8).map(|i| BlockItem { lo: [i as f32 * 20.0, 0.0], hi: [i as f32 * 20.0 + 10.0, 10.0], weight: 1.0, ..Default::default() }).collect();
    let mut world = BlockWorld::new(device);
    world.begin(device, queue, &items, 0.0);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
    wave.record(device, queue, &mut encoder, &mut world, 0.125, &(0..8).collect::<Vec<u32>>(), &[10.0, 2.0, 4.0]);
    let y: Vec<f32> = read_state(device, queue, &world, encoder).iter().map(|o| o.translate[1]).collect();
    let expect = |k: f32| 10.0 * (std::f32::consts::TAU * (2.0 * 0.125 - k / 4.0)).sin();
    for (k, v) in y.iter().enumerate() {
        assert!((v - expect(k as f32)).abs() < 1e-3, "object {k}: {v} vs {}", expect(k as f32));
    }
    assert!((y[0] - y[4]).abs() < 1e-3 && (y[0] + y[2]).abs() < 1e-3, "a wavelength apart: same; half: opposite {y:?}");
}

/// 近くの物だけ見る押し合い(升目)は、全組を見る同じ手順と同じ結果になる(散らばった 2000 個、4 組)。
#[test]
fn push_apart_over_neighbours_matches_all_pairs_at_scale() {
    use crate::render::compositor::effects::block_program::{program_for, read_state, BlockItem, BlockWorld};
    let mut rng = 3u32;
    let mut next = || { rng = rng.wrapping_mul(1664525).wrapping_add(1013904223); (rng >> 8) as f32 / (1u32 << 24) as f32 };
    let items: Vec<BlockItem> = (0..2000).map(|k| {
        let (x, y, w, h) = (next() * 3000.0, next() * 3000.0, 6.0 + next() * 14.0, 6.0 + next() * 14.0);
        BlockItem { lo: [x, y], hi: [x + w, y + h], room_size: [3000.0, 3000.0], group: k % 4, weight: 0.5 + next(), ..Default::default() }
    }).collect();
    let margin = 3.0f32;
    // CPU: 同じ手順を全組で(32 回、全員を同時に測って動かす)。
    let mut lo: Vec<[f32; 2]> = items.iter().map(|i| i.lo).collect();
    let mut hi: Vec<[f32; 2]> = items.iter().map(|i| i.hi).collect();
    for _ in 0..32 {
        let mut step = vec![[0.0f32; 2]; items.len()];
        for k in 0..items.len() {
            for j in 0..items.len() {
                if j == k || items[j].group != items[k].group { continue; }
                let (alo, ahi) = ([lo[k][0] - margin, lo[k][1] - margin], [hi[k][0] + margin, hi[k][1] + margin]);
                let (blo, bhi) = ([lo[j][0] - margin, lo[j][1] - margin], [hi[j][0] + margin, hi[j][1] + margin]);
                let gap = [(blo[0] + bhi[0]) * 0.5 - (alo[0] + ahi[0]) * 0.5, (blo[1] + bhi[1]) * 0.5 - (alo[1] + ahi[1]) * 0.5];
                let len = (gap[0] * gap[0] + gap[1] * gap[1]).sqrt();
                let dir = if len > 1e-4 { [gap[0] / len, gap[1] / len] } else if k < j { [1.0, 0.0] } else { [-1.0, 0.0] };
                let half = [((ahi[0] - alo[0]) + (bhi[0] - blo[0])) * 0.5, ((ahi[1] - alo[1]) + (bhi[1] - blo[1])) * 0.5];
                if half[0] - gap[0].abs() <= 0.0 || half[1] - gap[1].abs() <= 0.0 { continue; }
                let need = |a: usize| if dir[a].abs() >= 1e-6 { ((half[a] - gap[a].abs()) / dir[a].abs()).max(0.0) } else { 1e30 };
                let depth = need(0).min(need(1));
                if depth <= 0.0 || depth >= 1e29 { continue; }
                let w = items[k].weight / (items[k].weight + items[j].weight);
                step[k][0] -= dir[0] * depth * w * 0.5;
                step[k][1] -= dir[1] * depth * w * 0.5;
            }
        }
        for k in 0..items.len() {
            for a in 0..2 { lo[k][a] += step[k][a]; hi[k][a] += step[k][a]; }
        }
    }
    let engine = Engine::new().unwrap();
    let (device, queue) = (&engine.compositor.ctx.device, &engine.compositor.ctx.queue);
    let push = program_for(device, "push_apart");
    let mut world = BlockWorld::new(device);
    world.begin(device, queue, &items, 0.0);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("test") });
    push.record(device, queue, &mut encoder, &mut world, 0.0, &(0..items.len() as u32).collect::<Vec<u32>>(), &[margin]);
    let gpu = read_state(device, queue, &world, encoder);
    let mut moved = 0;
    assert_eq!(push_reach(), Some("margin".to_owned()), "Push Apart declares its margin as its reach");
    for k in 0..items.len() {
        let cpu = [lo[k][0] - items[k].lo[0], lo[k][1] - items[k].lo[1]];
        if cpu != [0.0, 0.0] { moved += 1; }
        let g = gpu[k].translate;
        assert!((g[0] - cpu[0]).abs() < 0.05 && (g[1] - cpu[1]).abs() < 0.05, "object {k}: gpu {g:?} cpu {cpu:?}");
    }
    assert!(moved > 20, "some of them overlapped and were pushed: {moved}");
}

fn push_reach() -> Option<String> {
    crate::render::compositor::effects::block_program::catalog_definition("push_apart").manifest.reach
}

/// Margin が箱よりずっと大きくても、届く距離の分だけ升目を広げるので相手を取りこぼさない。
#[test]
fn a_wide_margin_still_finds_its_neighbours() {
    use crate::render::compositor::effects::block_program::{neighbors, BlockItem};
    let items = [
        BlockItem { lo: [0.0, 0.0], hi: [4.0, 4.0], weight: 1.0, ..Default::default() },
        BlockItem { lo: [60.0, 0.0], hi: [64.0, 4.0], weight: 1.0, ..Default::default() },
    ];
    let (_, blind) = neighbors(&items, 0.0);
    assert!(blind.is_empty(), "without the reach, 60 px apart is out of a 8 px cell's 3×3");
    let (starts, list) = neighbors(&items, 40.0);
    assert_eq!((starts, list), (vec![0, 1, 2], vec![1, 0]), "with a 40 px margin they see each other");
}
