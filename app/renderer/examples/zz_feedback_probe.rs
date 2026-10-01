//! 使い捨ての確認(2026-09-20): Echo が効いているか。動く形を 12 コマ描き、Echo 有りと無しで
//! 「形が今いない所にどれだけ色が残っているか」を数える。Hold を変えて尾の長さも見る。
use motolii_edit::{Document, Intent};
use motolii_render::{doc::store::*, doc::vector::*, engine::Engine};

const SIZE: u32 = 64;
const FRAMES: i64 = 40;

fn fps() -> Fps { Fps::try_new(10, 1).unwrap() }
fn at(frame: i64) -> RationalTime { RationalTime::try_from_frame(frame, fps()).unwrap() }

fn document(echo: Option<(f64, f64)>) -> Document {
    let mut doc = Document::new().with_programs(motolii_render::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition { width: SIZE, height: SIZE, fps: fps(), duration_frames: FRAMES, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
    let layer = LayerId(1);
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe { t: RationalTime::ZERO, value: Value::Vec2([8.0, 32.0]), interp: Interp::Linear, spatial: None });
    track.insert(Keyframe { t: at(20), value: Value::Vec2([56.0, 32.0]), interp: Interp::Linear, spatial: None });
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, FRAMES) } },
        Intent::SetShapes { layer, shapes: vec![ShapeNode::Leaf(Shape {
            source: PathSource::Ellipse { size: Point { x: 8.0, y: 8.0 } },
            ops: Vec::new(), stroke: None,
            fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }),
        })] },
        Intent::SetTrack { layer, property: PropertyId::new(property::POSITION).unwrap(), track },
    ]).unwrap();
    if let Some((hold, strength)) = echo {
        doc.apply_all([
            Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.echo".into() }] },
            Intent::SetConstant { layer, property: PropertyId::effect_param(EffectId(0), "hold").unwrap(), value: Value::F64(hold) },
            Intent::SetConstant { layer, property: PropertyId::effect_param(EffectId(0), "strength").unwrap(), value: Value::F64(strength) },
        ]).unwrap();
    }
    doc
}

/// 12 コマ目まで辿った絵。
fn walked(doc: &Document, to: i64) -> Vec<u8> {
    let mut engine = Engine::new().unwrap();
    let mut last = Vec::new();
    for f in 0..=to { last = engine.render_frame(&doc.view(), at(f)).unwrap(); }
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    last
}

/// 明るい画素の数(形の跡の量)と、左から数えて一番遠い明るい列(尾の長さ)。
fn tail(px: &[u8]) -> (usize, i64) {
    let mut lit = 0usize;
    let mut leftmost = SIZE as i64;
    for y in 0..SIZE { for x in 0..SIZE {
        let p = ((y * SIZE + x) * 4) as usize;
        if px[p] > 24 { lit += 1; leftmost = leftmost.min(x as i64); }
    } }
    (lit, leftmost)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    re_log::setup_logging();
    for e in &motolii_render::compositor::refresh_effect_catalog().errors { eprintln!("catalog: {e}"); }
    let known = motolii_render::engine::known_effects();
    match known.iter().find(|d| d.plugin_id == "motolii.echo") {
        Some(d) => println!("card: {} stage={:?} persistent={} params={:?}", d.label, d.stage, d.persistent, d.params.iter().map(|p| (&p.label, p.default)).collect::<Vec<_>>()),
        None => { println!("card: motolii.echo が棚に無い"); return Ok(()) }
    }
    let (plain_lit, plain_left) = tail(&walked(&document(None), 12));
    println!("no echo     : lit={plain_lit} leftmost_x={plain_left}");
    for hold in [4.0, 12.0, 40.0] {
        let (lit, left) = tail(&walked(&document(Some((hold, 1.0))), 12));
        println!("echo hold={hold:<4}: lit={lit} leftmost_x={left}");
    }
    let (lit, left) = tail(&walked(&document(Some((12.0, 0.0))), 12));
    println!("echo strength=0: lit={lit} leftmost_x={left} (素の絵と同じはず)");
    // scrub-safe: 辿った絵と飛んで来た絵。
    let doc = document(Some((12.0, 1.0)));
    let a = walked(&doc, 12);
    let b = Engine::new()?.render_frame(&doc.view(), at(12))?;
    println!("scrub-safe  : differing={}/{}", a.iter().zip(&b).filter(|(x, y)| x != y).count(), a.len());
    Ok(())
}
