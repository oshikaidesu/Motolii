//! 解析の入力の審判(実 GPU): Blob Track が元の層の絵から塊を拾い、素材を塊ごとに置く。

use crate::doc::store::{blob, property, Composition, Document, EffectId, EffectInstance, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value};
use crate::doc::vector::{Brush, Fill, PathSource, Point, Rgb, Shape, ShapeNode};
use crate::render::engine::Engine;

const W: u32 = 320;
const H: u32 = 180;

fn fps() -> Fps { Fps::try_new(25, 1).unwrap() }
fn at(frame: i64) -> RationalTime { RationalTime::try_from_frame(frame, fps()).unwrap() }

/// 黒地に白い四角 2 つ: 左の 30 × 30 は 1 コマ 4 px 右へ、右の 20 × 20 は止まっている。
fn clip(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let out = dir.join("squares.mp4");
    let status = std::process::Command::new("ffmpeg")
        .args(["-v", "error", "-y", "-f", "lavfi", "-i", &format!("color=c=black:s={W}x{H}:r=25:d=2"),
               "-f", "lavfi", "-i", "color=c=white:s=30x30:r=25:d=2", "-f", "lavfi", "-i", "color=c=white:s=20x20:r=25:d=2",
               "-filter_complex", "[0][1]overlay=x='20+n*4':y=40[a];[a][2]overlay=x=250:y=120",
               "-pix_fmt", "yuv420p", "-c:v", "libx264", "-crf", "12"])
        .arg(&out).status().ok()?;
    status.success().then_some(out)
}

fn document(path: &std::path::Path, persist: bool) -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps: fps(), duration_frames: 50, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
    let (source, material) = (LayerId(1), LayerId(2));
    doc.apply_all([
        Intent::AddLayer(source),
        Intent::SetMeta { layer: source, meta: LayerMeta { source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, None, 50) } },
        Intent::SetConstant { layer: source, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) },
        Intent::AddLayer(material),
        Intent::SetMeta { layer: material, meta: LayerMeta { source: LayerSource::Shape, order: 1, timing: LayerTiming::place(0, None, 50) } },
        Intent::SetShapes { layer: material, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 10.0, y: 10.0 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 0.0, b: 0.0 }), ..Default::default() }) })] },
        Intent::SetConstant { layer: material, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) },
        Intent::SetEffects { layer: material, effects: vec![EffectInstance { id: EffectId(0), plugin_id: blob::BLOB_TRACK.into() }] },
    ]).unwrap();
    for (name, value) in [("source", Value::LayerId(1)), ("material", Value::Vec2([10.0, 10.0])), ("min_area", Value::F64(50.0)), ("persist", Value::F64(if persist { 1.0 } else { 0.0 }))] {
        doc.apply(Intent::SetConstant { layer: material, property: PropertyId::effect_param(EffectId(0), name).unwrap(), value }).unwrap();
    }
    doc
}

fn px(frame: &[u8], x: u32, y: u32) -> [u8; 3] {
    let i = ((y * W + x) * 4) as usize;
    [frame[i], frame[i + 1], frame[i + 2]]
}

#[test]
fn blob_track_boxes_the_bright_squares_of_the_tracked_layer() {
    let dir = tempfile::tempdir().unwrap();
    let Some(path) = clip(dir.path()) else { eprintln!("ffmpeg が無いので飛ばす"); return };
    let doc = document(&path, false);
    let mut engine = Engine::new().unwrap();
    let frame = engine.render_frame(&doc.view(), at(10)).unwrap();
    assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    // 10 コマ目: 左の四角は x = 60..90、y = 40..70。右は 250..270、120..140。
    for (x, y) in [(75, 55), (260, 130)] {
        let [r, g, b] = px(&frame, x, y);
        assert!(r > 200 && g < 60 && b < 60, "塊の上に赤い素材: ({x}, {y}) = {:?}", [r, g, b]);
    }
    let [r, g, b] = px(&frame, 160, 100);
    assert!(r < 30 && g < 30 && b < 30, "何も無い所には置かない: {:?}", [r, g, b]);
}

#[test]
fn kept_ids_follow_the_moving_square_and_land_the_same_however_you_arrive() {
    let dir = tempfile::tempdir().unwrap();
    let Some(path) = clip(dir.path()) else { eprintln!("ffmpeg が無いので飛ばす"); return };
    let doc = document(&path, true);
    let mut walker = Engine::new().unwrap();
    let ids = |engine: &mut Engine, frame: i64| {
        let resolved = engine.resolved_with_analysis(&doc.view(), at(frame)).unwrap();
        let mut marks: Vec<(u32, f32)> = resolved.iter().filter(|l| l.id == LayerId(2)).map(|l| (l.copy, l.placement.transform.translation.x)).collect();
        marks.sort_by(|a, b| a.1.total_cmp(&b.1));
        marks.into_iter().map(|(id, _)| id).collect::<Vec<_>>()
    };
    let first = ids(&mut walker, 2);
    assert_eq!(first.len(), 2, "四角 2 つ: {first:?}");
    let mut walked = Vec::new();
    for f in 3..=12 {
        walked = walker.render_frame(&doc.view(), at(f)).unwrap();
        assert_eq!(ids(&mut walker, f), first, "{f} コマ目: 動いても左の四角は同じ ID");
    }
    let jumped = Engine::new().unwrap().render_frame(&doc.view(), at(12)).unwrap();
    assert_eq!(jumped, walked, "飛んで来た絵が辿った絵と違う");
}
