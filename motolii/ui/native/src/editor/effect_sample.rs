//! 効果の札 = 作者の snapshot 画像(VST3 の Plug-in Snapshot と同じ置き方)。
use base64::Engine as _;
use serde_json::{json, Value as J};

/// `{"op":"visualSample","kind":"effect","id":plugin_id}` の返事。絵が無い効果は札を名前のままにする(error)。
pub(crate) fn reply(j: &J) -> Result<J, String> {
    let id = j["id"].as_str().ok_or("Missing effect id")?;
    let catalog = crate::render::engine::known_effects();
    let effect = catalog.iter().find(|d| d.plugin_id == id).ok_or("Unknown effect")?;
    let snapshot = effect.snapshot.as_ref().ok_or("No snapshot")?;
    Ok(json!({"image": base64::engine::general_purpose::STANDARD.encode(snapshot)}))
}

/// 同梱の札を吐く道具: `cargo test -p motolii-ui snapshots -- --ignored`。
/// 席ごとに 1 つの小さな書類(写真・星・球)を組み、Stage と同じ Engine で t=0.5s を描いて
/// `crates/motolii-render/vism/<id>_snapshot{,_2.0x}.png` に書く。作者の絵を置いた効果は触らない側の道具ではなく、
/// 同梱効果の絵の出所。
#[cfg(test)]
mod snapshots {
    use crate::doc::store::{
        Composition, Document, EffectId, EffectInstance, Fps, Intent, Interp, Keyframe, KeyframeTrack, LayerId,
        PropertyId, RationalTime, Value, property,
    };
    use crate::editor::create::{self, NewKind};
    use crate::render::compositor::EffectStage;
    use crate::render::engine::{EffectDescriptor, Engine};

    /// 1x の寸法(16:9)。3 倍で描いて縮める(px の既定値が並の大きさで効き、縁も滑らか)。
    const WIDTH: u32 = 160;
    const HEIGHT: u32 = 90;
    const SCALE: u32 = 3;
    const FPS: i64 = 30;
    const DURATION_FRAMES: i64 = 30;
    const TIME: f64 = 0.5;
    /// 写真は枠より広く切る: 1 秒の移動ぶん。
    const PHOTO_WIDTH: u32 = WIDTH * SCALE + 90;

    /// HERO の 75% では読めない効果の姿勢。
    const POSES: &[(&str, &[(&str, f64)])] = &[
        ("motolii.gain", &[("gain", 0.45)]),
        ("motolii.blur", &[("radius", 24.0)]),
        ("motolii.glow", &[("threshold", 0.7), ("intensity", 1.2), ("radius", 24.0), ("chromatic", 0.5)]),
        ("motolii.turbulent_warp", &[("amount", 24.0), ("size", 80.0)]),
        ("motolii.trim_paths", &[("end", 60.0)]),
        ("motolii.pucker_bloat", &[("amount", 60.0)]),
        ("motolii.twist", &[("angle", 120.0)]),
        ("motolii.extend_paths", &[("end", 30.0)]),
        ("motolii.bend", &[("angle", 120.0)]),
        ("motolii.extrude", &[("depth", 24.0)]),
        ("motolii.bevel", &[("radius", 10.0)]),
    ];

    /// 平らな席の見本は写真(同梱の photo studio の真ん中)。硬い縁・明部・階調・色が 1 枚で読める。
    fn photo() -> Result<String, String> {
        let hdr = image::load_from_memory(include_bytes!("../../assets/backgrounds/brown_photostudio_02.hdr")).map_err(|e| e.to_string())?.to_rgb32f();
        let (w, h) = (PHOTO_WIDTH, HEIGHT * SCALE);
        let crop = image::imageops::crop_imm(&hdr, 192, 102, 640, 309).to_image();
        let small = image::imageops::resize(&crop, w, h, image::imageops::FilterType::Triangle);
        let mut out = image::RgbaImage::new(w, h);
        for (x, y, p) in small.enumerate_pixels() {
            let tone = |v: f32| { let v = v * 1.6; ((v / (1.0 + v)).powf(1.0 / 2.2) * 255.0).round() as u8 };
            out.put_pixel(x, y, image::Rgba([tone(p[0]), tone(p[1]), tone(p[2]), 255]));
        }
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(out).write_to(&mut png, image::ImageFormat::Png).map_err(|e| e.to_string())?;
        create::builtin("effect-sample-photo-v2.png", &png.into_inner())
    }

    fn track(from: Value, to: Value) -> KeyframeTrack {
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe { t: RationalTime::ZERO, value: from, interp: Interp::Linear, spatial: None });
        track.insert(Keyframe { t: RationalTime::try_new(1, 1).expect("1s"), value: to, interp: Interp::Linear, spatial: None });
        track
    }

    fn document(stage: EffectStage) -> Result<(Document, LayerId), String> {
        let fps = Fps::try_new(FPS, 1).map_err(|e| e.to_string())?;
        let comp = ((WIDTH * SCALE) as f64, (HEIGHT * SCALE) as f64);
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition { width: WIDTH * SCALE, height: HEIGHT * SCALE, fps, duration_frames: DURATION_FRAMES, background: [0.0; 4] })).map_err(|e| e.to_string())?;
        let place = |layer: LayerId, order: i16, kind: NewKind| create::new_layer_intents(layer, order, 0, DURATION_FRAMES, fps, comp, kind, None);
        let subject = LayerId(1);
        let centered = |out: &Vec<Intent>| out.iter().find_map(|i| match i { Intent::SetConstant { property, value: Value::Vec2(p), .. } if property.name() == property::POSITION => Some(*p), _ => None }).unwrap_or([0.0, 0.0]);
        let intents = match stage {
            EffectStage::Pass | EffectStage::Warp => {
                let mut out = place(subject, 0, NewKind::Media { path: photo()?, name: "Sample".into() });
                let c = [comp.0 * 0.5, comp.1 * 0.5];
                out.push(Intent::SetConstant { layer: subject, property: PropertyId::new(property::ANCHOR).map_err(|e| e.to_string())?, value: Value::Vec2([PHOTO_WIDTH as f64 * 0.5, comp.1 * 0.5]) });
                // 1 秒動く: 時間で効く効果(残像・窓)が尾を引く。写真は動く分だけ広く切ってある。
                let travel = (PHOTO_WIDTH - WIDTH * SCALE) as f64 * 0.5;
                out.push(Intent::SetTrack { layer: subject, property: PropertyId::new(property::POSITION).map_err(|e| e.to_string())?, track: track(Value::Vec2([c[0] - travel, c[1]]), Value::Vec2([c[0] + travel, c[1]])) });
                out
            }
            EffectStage::Placement | EffectStage::Path | EffectStage::Solid => {
                let mut out = place(subject, 0, NewKind::Star);
                // 星は枠の高さの半分ほどに(既定は 16:9 の枠では小さい)。
                let grow = if stage == EffectStage::Placement { 1.3 } else { 1.8 };
                out.push(Intent::SetConstant { layer: subject, property: PropertyId::new(property::SCALE).map_err(|e| e.to_string())?, value: Value::Vec2([grow, grow]) });
                if stage == EffectStage::Placement {
                    // Repeater の既定は 100px 刻みで 3 つ。1 歩ぶん左へ寄せて、複製の群れが枠の中に並ぶ。
                    let c = centered(&out);
                    out.push(Intent::SetConstant { layer: subject, property: PropertyId::new(property::POSITION).map_err(|e| e.to_string())?, value: Value::Vec2([c[0] - 100.0 * grow, c[1]]) });
                }
                if stage == EffectStage::Solid {
                    // 立体は傾けて側面と縁を見せる(2.5D の板のまま回すと奥行きは出ない)。
                    out.push(Intent::SetAttrs { layer: subject, patch: crate::doc::store::LayerAttrsPatch { projection: Some(crate::doc::store::LayerProjection::ThreeD), ..Default::default() } });
                    out.push(Intent::SetTrack { layer: subject, property: PropertyId::new(property::ROTATION_Y).map_err(|e| e.to_string())?, track: track(Value::F64(-20.0), Value::F64(-50.0)) });
                }
                out.push(Intent::SetTrack { layer: subject, property: PropertyId::new(property::ROTATION).map_err(|e| e.to_string())?, track: track(Value::F64(0.0), Value::F64(45.0)) });
                out
            }
            EffectStage::Surface | EffectStage::Field | EffectStage::Clip => {
                let mut out = Vec::new();
                if stage == EffectStage::Surface {
                    out.extend(place(LayerId(2), 0, create::background("photo-studio")?));
                }
                out.extend(place(subject, 1, create::primitive("sphere")?));
                out.push(Intent::SetTrack { layer: subject, property: PropertyId::new(property::ROTATION_Y).map_err(|e| e.to_string())?, track: track(Value::F64(0.0), Value::F64(60.0)) });
                out
            }
        };
        doc.apply_all(intents).map_err(|e| e.to_string())?;
        Ok((doc, subject))
    }

    fn dress(doc: &mut Document, layer: LayerId, effect: &EffectDescriptor) -> Result<(), String> {
        let id = EffectId(0);
        doc.apply(Intent::SetEffects { layer, effects: vec![EffectInstance { id, plugin_id: effect.plugin_id.clone() }] }).map_err(|e| e.to_string())?;
        let hero = effect.params.iter().find(|p| p.hero).and_then(|p| p.range.map(|(min, max)| (p.name.as_str(), min + (max - min) * 0.75)));
        let pose: Vec<(&str, f64)> = match POSES.iter().find(|(id, _)| *id == effect.plugin_id) {
            Some((_, pose)) => pose.to_vec(),
            None => hero.into_iter().collect(),
        };
        for (name, value) in pose {
            doc.apply(Intent::SetConstant { layer, property: PropertyId::effect_param(id, name).map_err(|e| e.to_string())?, value: Value::F64(value) }).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn render(engine: &mut Engine, effect: &EffectDescriptor) -> Result<image::RgbaImage, String> {
        let (mut doc, layer) = document(effect.stage)?;
        dress(&mut doc, layer, effect)?;
        let at = RationalTime::try_new((TIME * FPS as f64).round() as i64, FPS).map_err(|e| e.to_string())?;
        let mut pixels = engine.render_frame(&doc.view(), at).map_err(|e| e.to_string())?;
        for pixel in pixels.chunks_exact_mut(4) {
            let a = pixel[3] as u32;
            if a > 0 { for c in &mut pixel[..3] { *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8; } }
        }
        image::RgbaImage::from_raw(WIDTH * SCALE, HEIGHT * SCALE, pixels).ok_or_else(|| "Sample frame has the wrong size".into())
    }

    #[test]
    #[ignore = "writes the bundled snapshots; run on purpose"]
    fn write_bundled_snapshots() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../crates/motolii-render/vism");
        let mut engine = Engine::new().unwrap();
        for effect in crate::render::engine::known_effects().iter() {
            let frame = image::DynamicImage::ImageRgba8(render(&mut engine, effect).unwrap());
            for (suffix, k) in [("_snapshot.png", 1), ("_snapshot_2.0x.png", 2)] {
                frame.thumbnail(WIDTH * k, HEIGHT * k).save(dir.join(format!("{}{suffix}", effect.plugin_id))).unwrap();
            }
        }
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 棚の全部の札に絵がある(shader の隣の `<id>_snapshot.png`)。
    #[test]
    fn every_effect_on_the_shelf_has_a_snapshot() {
        let missing = crate::render::engine::known_effects().iter().filter(|d| d.snapshot.is_none()).map(|d| d.plugin_id.clone()).collect::<Vec<_>>();
        assert!(missing.is_empty(), "no snapshot beside the shader: {missing:?}");
        let gain = reply(&json!({"id": "motolii.gain"})).unwrap();
        assert!(gain["image"].as_str().is_some_and(|s| s.starts_with("iVBOR")), "a PNG, base64");
        assert!(reply(&json!({"id": "motolii.nope"})).is_err());
    }
}
