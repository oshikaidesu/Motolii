//! 広がりの法の審判(実 GPU): 周りを照らす効果の光は、宣言した余白の中で消える。
//! 余白が光より狭いと、余白の縁で光が段になって切れ、灰色の板が見える(2026-09-14 の総当たりで Glow・Radiance)。
use crate::doc::store::*;
use crate::doc::vector::{Brush, Fill, PathSource, Point, Rgb, Shape};
use crate::render::engine::{content_canvas, Engine};

const SIZE: u32 = 768;

/// 黒地の真ん中に白い円(直径 96)、効果は既定値のまま。
fn lit(plugin: &str) -> Document {
    let shapes = vec![ShapeNode::Leaf(Shape {
        source: PathSource::Ellipse { size: Point { x: 96.0, y: 96.0 } },
        ops: Vec::new(),
        stroke: None,
        fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }),
    })];
    let canvas = content_canvas(&shapes).unwrap().unwrap();
    let mut doc = blank_project();
    let mut comp = doc.view().composition().unwrap().unwrap();
    comp.width = SIZE;
    comp.height = SIZE;
    comp.background = [0.0, 0.0, 0.0, 1.0];
    let id = LayerId(1);
    let centre = f64::from(SIZE) / 2.0;
    doc.apply_all([
        Intent::SetComposition(comp),
        Intent::AddLayer(id),
        Intent::SetMeta { layer: id, meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, 60) } },
        Intent::SetShapes { layer: id, shapes },
        Intent::SetAttrs { layer: id, patch: LayerAttrsPatch { projection: Some(LayerProjection::TwoD), ..Default::default() } },
        Intent::SetConstant { layer: id, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([centre, centre]) },
        Intent::SetConstant { layer: id, property: PropertyId::new(property::ANCHOR).unwrap(), value: Value::Vec2([canvas.origin_x as f64, canvas.origin_y as f64]) },
        Intent::SetEffects { layer: id, effects: vec![EffectInstance { id: EffectId(0), plugin_id: plugin.into() }] },
    ])
    .unwrap();
    doc
}

#[test]
fn light_fades_out_inside_its_declared_extent() {
    let mut engine = Engine::new().unwrap();
    for plugin in ["motolii.glow", "motolii.radiance"] {
        let pixels = engine.render_frame(&lit(plugin).view(), RationalTime::ZERO).unwrap();
        assert!(engine.layer_failures().is_empty(), "{plugin}: {:?}", engine.layer_failures());
        let row = (SIZE / 2) as usize;
        let brightness = |x: usize| pixels[(row * SIZE as usize + x) * 4] as i32;
        // 円の縁(中心から 48 px)の少し外から、右の端まで。光は滑らかに落ち、途中で段にならない。
        let from = (SIZE / 2) as usize + 56;
        assert!(brightness(from) > 8, "{plugin}: no light around the circle ({})", brightness(from));
        let steepest = (from..SIZE as usize - 1).map(|x| brightness(x) - brightness(x + 1)).max().unwrap();
        let cut = (from..SIZE as usize - 1).max_by_key(|x| brightness(*x) - brightness(x + 1)).unwrap();
        assert!(steepest <= 8, "{plugin}: the light drops by {steepest} at x = {cut} — the extent cut it off");
    }
}
