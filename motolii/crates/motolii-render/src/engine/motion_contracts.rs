//! 時間と粒子の審判(実 GPU): 隣のコマを読むぼかし、層の動きのぼけ、粒子の層。
/// Pixel Motion Blur(光学フロー)の審判: 横に動く白い四角の縁の傾きが、1 コマの動き × シャッター角 / 360 の幅になる。
/// 隣のコマは `TIME_OFFSET_FRAMES` で読むので、fps を変えても「1 コマの動き」で測れる。飛んでも辿っても同じ絵。
mod pixel_motion_blur_follows_the_motion {
    use crate::doc::store::{property, Composition, Document, EffectId, EffectInstance, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value};
    use crate::render::engine::Engine;

    const W: u32 = 320;
    const H: u32 = 180;

    /// 1 コマに `step` px 右へ動く 30×60 の白い四角(黒地)。
    fn clip(dir: &std::path::Path, fps: i64, step: i64) -> Option<std::path::PathBuf> {
        let out = dir.join(format!("box-{fps}.mp4"));
        let status = std::process::Command::new("ffmpeg")
            .args(["-v", "error", "-y", "-f", "lavfi", "-i", &format!("color=c=black:s={W}x{H}:r={fps}:d=2"),
                   "-f", "lavfi", "-i", &format!("color=c=white:s=30x60:r={fps}:d=2"),
                   "-filter_complex", &format!("[0][1]overlay=x='n*{step}-30':y=60"),
                   "-pix_fmt", "yuv420p", "-c:v", "libx264", "-crf", "10"])
            .arg(&out).status().ok()?;
        status.success().then_some(out)
    }

    fn document(path: &std::path::Path, fps: i64, shutter: Option<f64>) -> Document {
        let fps = Fps::try_new(fps, 1).unwrap();
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps, duration_frames: 40, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None }, order: 0, timing: LayerTiming::place(0, None, 40) } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) },
        ]).unwrap();
        if let Some(shutter) = shutter {
            doc.apply_all([
                Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.pixel_motion_blur".into() }] },
                Intent::SetConstant { layer, property: PropertyId::effect_param(EffectId(0), "shutter").unwrap(), value: Value::F64(shutter) },
            ]).unwrap();
        }
        doc
    }

    /// 四角の真ん中の行で、黒でも白でもない画素の数(左右の縁の傾きの合計)。
    fn ramp(pixels: &[u8]) -> usize {
        (0..W as usize).map(|x| pixels[(90 * W as usize + x) * 4]).filter(|v| (12..243).contains(v)).count()
    }

    fn walked(doc: &Document, fps: i64, frame: i64) -> Vec<u8> {
        let mut engine = Engine::new().unwrap();
        let mut last = Vec::new();
        for f in 0..=frame { last = engine.render_frame(&doc.view(), RationalTime::try_from_frame(f, Fps::try_new(fps, 1).unwrap()).unwrap()).unwrap(); }
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        last
    }

    #[test]
    fn the_edges_smear_by_the_motion_of_one_frame_times_the_shutter() {
        let dir = tempfile::tempdir().unwrap();
        for (fps, step) in [(24, 20), (48, 10)] {
            let Some(path) = clip(dir.path(), fps, step) else { eprintln!("ffmpeg が無いので飛ばす"); return };
            let at = RationalTime::try_from_frame(10, Fps::try_new(fps, 1).unwrap()).unwrap();
            assert!(ramp(&Engine::new().unwrap().render_frame(&document(&path, fps, None).view(), at).unwrap()) <= 2, "素の縁は切り立っている");
            let doc = document(&path, fps, Some(180.0));
            let walked = walked(&doc, fps, 10);
            // 縁 2 本 × (1 コマの動き × 180/360) = 1 コマの動き。
            let (got, expected) = (ramp(&walked), step as usize);
            assert!(got.abs_diff(expected) <= 3, "{fps}fps: 縁の傾き {got} px(期待 {expected} px)");
            let jumped = Engine::new().unwrap().render_frame(&doc.view(), at).unwrap();
            assert_eq!(jumped, walked, "{fps}fps: 飛んで来た絵が辿った絵と違う");
            assert!(ramp(&Engine::new().unwrap().render_frame(&document(&path, fps, Some(0.0)).view(), at).unwrap()) <= 2, "シャッター 0 はぼけない");
        }
    }
}

/// Motion Blur(層の動き、Alight Motion の型)の審判: キーで 1 コマに 20 px 動く形の縁が、Tune 1 で 1 コマぶん(20 px)の傾きになる。
/// 真ん中は写しを足しても元の明るさのまま、止まった層と Position を切った層はぼけない、飛んでも辿っても同じ絵。
mod motion_blur_follows_the_keyframes {
    use crate::doc::store::{property, Composition, Document, EffectId, EffectInstance, Fps, Intent, Interp, Keyframe, KeyframeTrack, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value};
    use crate::doc::vector::{Brush, Fill, PathSource, Point, Rgb, Shape, ShapeNode};
    use crate::render::engine::Engine;

    const W: u32 = 320;
    const H: u32 = 180;
    const FRAMES: i64 = 24;

    fn fps() -> Fps { Fps::try_new(24, 1).unwrap() }
    fn at(frame: i64) -> RationalTime { RationalTime::try_from_frame(frame, fps()).unwrap() }

    fn document(moving: bool, blur: Option<&[(&str, f64)]>) -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps: fps(), duration_frames: FRAMES, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
        let layer = LayerId(1);
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe { t: RationalTime::ZERO, value: Value::Vec2([0.0, 60.0]), interp: Interp::Linear, spatial: None });
        let end = if moving { 20.0 * FRAMES as f64 } else { 0.0 };
        track.insert(Keyframe { t: at(FRAMES), value: Value::Vec2([end, 60.0]), interp: Interp::Linear, spatial: None });
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, FRAMES) } },
            Intent::SetShapes { layer, shapes: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: 40.0, y: 60.0 } }, ops: Vec::new(), stroke: None, fill: Some(Fill { brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }), ..Default::default() }) })] },
            Intent::SetTrack { layer, property: PropertyId::new(property::POSITION).unwrap(), track },
        ]).unwrap();
        if let Some(params) = blur {
            doc.apply(Intent::SetEffects { layer, effects: vec![EffectInstance { id: EffectId(0), plugin_id: crate::doc::store::motion::MOTION_BLUR.into() }] }).unwrap();
            for (name, value) in params {
                doc.apply(Intent::SetConstant { layer, property: PropertyId::effect_param(EffectId(0), name).unwrap(), value: Value::F64(*value) }).unwrap();
            }
        }
        doc
    }

    fn row(pixels: &[u8]) -> Vec<u8> { (0..W as usize).map(|x| pixels[(90 * W as usize + x) * 4]).collect() }
    fn ramp(pixels: &[u8]) -> usize { row(pixels).into_iter().filter(|v| (12..243).contains(v)).count() }

    fn render(doc: &Document, frame: i64) -> Vec<u8> {
        let mut engine = Engine::new().unwrap();
        let pixels = engine.render_frame(&doc.view(), at(frame)).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        pixels
    }

    #[test]
    fn the_edges_smear_by_one_frame_of_keyframed_motion() {
        assert!(ramp(&render(&document(true, None), 5)) <= 2, "素の縁は切り立っている");
        let doc = document(true, Some(&[]));
        let blurred = render(&doc, 5);
        let got = ramp(&blurred);
        // Tune 1 = 1 コマの動き(20 px)の幅で平均する: 左右の縁がそれぞれ 20 px の傾きになる。
        assert!(got.abs_diff(40) <= 6, "縁の傾き {got} px(期待 40 px): {:?}", row(&blurred));
        // 四角(40 px)より動きが小さいので、真ん中は元の白のまま(足して平均しても暗くならない)。
        assert!(row(&blurred).iter().filter(|v| **v >= 250).count() >= 15, "真ん中が暗い: {:?}", row(&blurred));
        let mut engine = Engine::new().unwrap();
        let mut walked = Vec::new();
        for f in 0..=5 { walked = engine.render_frame(&doc.view(), at(f)).unwrap(); }
        assert_eq!(walked, blurred, "飛んで来た絵が辿った絵と違う");
        assert!(ramp(&render(&document(false, Some(&[])), 5)) <= 2, "止まった層はぼけない");
        assert!(ramp(&render(&document(true, Some(&[("position", 0.0)])), 5)) <= 2, "Position を切ればぼけない");
        let double = ramp(&render(&document(true, Some(&[("tune", 2.0)])), 5));
        assert!(double > got + 10, "Tune 2 は Tune 1 より長い: {double} ≤ {got}");
    }
}

/// 粒子の層(形の族、点の billboard)。閉じた式なので飛んでも辿っても同じ絵、跳ね返りは床の上に留まる(実 GPU)。
mod particles_are_a_closed_form {
    use crate::doc::store::{particles, property, Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value};
    use crate::render::engine::Engine;

    const W: u32 = 320;
    const H: u32 = 240;

    fn fps() -> Fps { Fps::try_new(24, 1).unwrap() }
    fn at(frame: i64) -> RationalTime { RationalTime::try_from_frame(frame, fps()).unwrap() }

    fn document(values: &[(&str, Value)]) -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: W, height: H, fps: fps(), duration_frames: 96, background: [0.0, 0.0, 0.0, 1.0] })).unwrap();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Particles, order: 0, timing: LayerTiming::place(0, None, 96) } },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([160.0, 160.0]) },
        ]).unwrap();
        for (name, value) in values {
            doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value: value.clone() }).unwrap();
        }
        doc
    }

    /// 明るい画素の数を、y の境で上と下に分けて。
    fn lit(frame: &[u8], split: u32) -> (usize, usize) {
        let (mut above, mut below) = (0, 0);
        for (i, px) in frame.chunks_exact(4).enumerate() {
            if px[0] > 60 {
                if (i as u32 / W) < split { above += 1 } else { below += 1 }
            }
        }
        (above, below)
    }

    fn render(doc: &Document, frame: i64) -> Vec<u8> {
        let mut engine = Engine::new().unwrap();
        let pixels = engine.render_frame(&doc.view(), at(frame)).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        pixels
    }

    #[test]
    fn particles_rise_from_the_emitter_and_land_the_same_however_you_arrive() {
        let doc = document(&[]);
        assert_eq!(lit(&render(&doc, 0), H), (0, 0), "入点ではまだ何も出ていない");
        let jumped = render(&doc, 24);
        let (above, below) = lit(&jumped, 150);
        assert!(above > 200 && above > below * 4, "既定は上へ飛ぶ: 上 {above} 下 {below}");
        let mut engine = Engine::new().unwrap();
        let mut walked = Vec::new();
        for f in 0..=24 { walked = engine.render_frame(&doc.view(), at(f)).unwrap(); }
        assert_eq!(walked, jumped, "飛んで来た絵が辿った絵と違う");
    }

    #[test]
    fn plexus_links_near_particles_with_lines() {
        let spread = [
            (particles::SPREAD, Value::F64(360.0)),
            (particles::SPEED, Value::F64(80.0)),
            (particles::RATE, Value::F64(60.0)),
            (particles::SIZE, Value::F64(2.0)),
            (particles::SIZE_END, Value::F64(2.0)),
            (particles::OPACITY_END, Value::F64(1.0)),
        ];
        let dots = document(&spread);
        let mut linked_values = spread.to_vec();
        linked_values.extend([(particles::CONNECT, Value::F64(50.0)), (particles::LINE_OPACITY, Value::F64(1.0)), (particles::LINE_WIDTH, Value::F64(1.5))]);
        let linked = document(&linked_values);
        let count = |frame: &[u8]| frame.chunks_exact(4).filter(|px| px[0] > 40).count();
        let (plain, lines) = (count(&render(&dots, 36)), count(&render(&linked, 36)));
        assert!(lines > plain * 2, "線が足される: 点だけ {plain} 線あり {lines}");
        let mut engine = Engine::new().unwrap();
        let mut walked = Vec::new();
        for f in 0..=36 { walked = engine.render_frame(&linked.view(), at(f)).unwrap(); }
        assert_eq!(walked, render(&linked, 36), "飛んで来た絵が辿った絵と違う");
    }

    #[test]
    fn bouncing_particles_stay_above_the_floor() {
        let doc = document(&[
            (particles::DIRECTION, Value::F64(90.0)),
            (particles::SPREAD, Value::F64(60.0)),
            (particles::GRAVITY, Value::F64(900.0)),
            (particles::BOUNCE, Value::F64(0.6)),
            (particles::FLOOR, Value::F64(40.0)),
            (particles::LIFE, Value::F64(3.0)),
            (particles::SIZE_END, Value::F64(8.0)),
            (particles::OPACITY_END, Value::F64(1.0)),
        ]);
        let frame = render(&doc, 48);
        // 床は出す元(y = 160)の 40 px 下 = 200。粒の直径 8 px ぶんの余裕。
        let (above, below) = lit(&frame, 206);
        assert!(above > 100, "粒が居る: {above}");
        assert_eq!(below, 0, "床より下に粒が居ない");
    }
}
