use crate::edit::{Animate, Document, Intent};
use crate::doc::store::*;

fn runtime(layers: u32, copies: f64) -> crate::EditorRuntime {
    let mut rt = crate::EditorRuntime::open("").unwrap();
    for i in 0..layers {
        let layer = LayerId(100 + u64::from(i));
        let repeat = EffectId(0);
        rt.doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: i as i16, timing: LayerTiming::place(0, None, 1800) } },
            Intent::SetShapes { layer, shapes: vec![rect_shape([200, 80, 40, 255], [60.0, 60.0])] },
            Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([100.0 + 40.0 * f64::from(i), 100.0 + 150.0 * f64::from(i)]) },
            Intent::SetEffects { layer, effects: vec![EffectInstance { id: repeat, plugin_id: crate::render::extensions::placement::REPEAT.to_owned() }] },
            Intent::SetConstant { layer, property: PropertyId::effect_param(repeat, "count").unwrap(), value: Value::F64(copies) },
            Intent::SetConstant { layer, property: PropertyId::effect_param(repeat, "position_each").unwrap(), value: Value::Vec2([90.0, 0.0]) },
        ]).unwrap();
    }
    rt
}

#[test]
fn playback_values_match_full_status_without_keyframe_metadata() {
    let mut rt = runtime(2, 1.0);
    let p = PropertyId::new(property::OPACITY).unwrap();
    let mut track = KeyframeTrack::new();
    for frame in [0, 10] {
        track.insert(Keyframe { t: RationalTime::try_from_frame(frame, rt.doc.view().composition().unwrap().unwrap().fps).unwrap(), value: Value::F64(frame as f64 / 10.0), interp: Interp::Linear, spatial: None });
    }
    rt.doc.apply(Intent::SetTrack { layer: LayerId(100), property: p, track }).unwrap();
    rt.request(serde_json::json!({"op":"select","ids":[100,101]})).unwrap();
    for frame in [0, 5, 10] {
        rt.viewer.frame = frame;
        let full = rt.status().unwrap();
        rt.viewer.clock.toggle();
        let live = rt.status().unwrap();
        rt.viewer.clock.toggle();
        for (expected, actual) in full["layers"].as_array().unwrap().iter().zip(live["liveLayers"].as_array().unwrap()) {
            for field in ["id", "x", "y", "bounds", "corners", "text"] { assert_eq!(expected[field], actual[field], "{field}"); }
            let compare = |a: &serde_json::Value, b: &serde_json::Value| {
                assert_eq!(a.as_array().unwrap().len(), b.as_array().unwrap().len());
                for (a,b) in a.as_array().unwrap().iter().zip(b.as_array().unwrap()) {
                    for field in ["id", "value", "keyedNow"] { assert_eq!(a[field],b[field], "{field}"); }
                    assert!(b.get("keys").is_none());
                }
            };
            compare(&expected["properties"], &actual["properties"]);
            for (a,b) in expected["effects"].as_array().unwrap().iter().zip(actual["effects"].as_array().unwrap()) { compare(&a["params"], &b["params"]); }
        }
    }
    // 選択を外した層は、軽い status では枠と位置だけ(値は Inspector が読まない)。
    rt.request(serde_json::json!({"op":"select","ids":[101]})).unwrap();
    let full = rt.status().unwrap();
    rt.viewer.clock.toggle();
    let light = rt.status().unwrap();
    rt.viewer.clock.toggle();
    let rows = light["liveLayers"].as_array().unwrap();
    assert_eq!(rows.len(), full["layers"].as_array().unwrap().len(), "行数は全層ぶん");
    for (expected, actual) in full["layers"].as_array().unwrap().iter().zip(rows) {
        for field in ["id", "x", "y", "bounds", "corners"] { assert_eq!(expected[field], actual[field], "{field}"); }
        if actual["id"] == 101 { assert!(actual["properties"].is_array()); } else { assert!(actual.get("properties").is_none(), "{actual}"); }
    }
    eprintln!("PROBE room=status light={} bytes, selected-only={} bytes", { rt.request(serde_json::json!({"op":"select","ids":[100,101]})).unwrap(); rt.viewer.clock.toggle(); let s = rt.status().unwrap().to_string().len(); rt.viewer.clock.toggle(); s }, light.to_string().len());
    let before = rt.full_status_revision.borrow().clone();
    let reply = unsafe { crate::motolii_probe_request(&mut rt, c"{\"op\":\"renderInfo\"}".as_ptr()) };
    let info: serde_json::Value = serde_json::from_str(unsafe { std::ffi::CStr::from_ptr(reply) }.to_str().unwrap()).unwrap();
    assert_eq!(info.as_object().unwrap().len(), 3, "width, height, views");
    assert_eq!(info["width"], rt.status().unwrap()["width"]);
    assert_eq!(*rt.full_status_revision.borrow(), before);
    rt.request(serde_json::json!({"op":"composition","width":640,"height":480})).unwrap();
    let reply = unsafe { crate::motolii_probe_request(&mut rt, c"{\"op\":\"renderInfo\"}".as_ptr()) };
    let info: serde_json::Value = serde_json::from_str(unsafe { std::ffi::CStr::from_ptr(reply) }.to_str().unwrap()).unwrap();
    assert_eq!(info, serde_json::json!({"width":640,"height":480,"views":[{"view":"Camera","width":640,"height":480}]}));
}

#[test]
#[ignore]
fn probe() {
    for (layers, copies) in [(5, 1.0), (5, 30.0), (5, 100.0)] {
        let mut rt = runtime(layers, copies);
        let t = RationalTime::ZERO;
        let time = |f: &mut dyn FnMut()| { let s = std::time::Instant::now(); for _ in 0..5 { f(); } s.elapsed() / 5 };
        let resolve = time(&mut || { crate::render::picture::resolve::resolved_layers(&rt.doc.view(), t).unwrap(); });
        let status = time(&mut || { rt.status().unwrap(); });
        let status_bytes = rt.status().unwrap().to_string().len();
        *rt.full_status_revision.borrow_mut() = Some(format!("{:?}", rt.doc.revision()));
        rt.viewer.clock.toggle();
        let live = time(&mut || { rt.status().unwrap(); });
        let live_bytes = rt.status().unwrap().to_string().len();
        rt.viewer.clock.toggle();
        eprintln!("  live status={live:?} ({live_bytes} bytes)");
        let sig = time(&mut || { crate::snapshot::authored_signature(&rt.doc).unwrap(); });
        let depth = time(&mut || { let r = crate::render::picture::resolve::resolved_layers(&rt.doc.view(), t).unwrap(); rt.depth_layout(&r).unwrap(); });
        let catalog = crate::render::engine::known_effects();
        let inspector = time(&mut || { for id in rt.doc.view().layers() { crate::editor::functions::read::inspector_data_from_doc(&rt.doc.view(), id, t, &catalog); } });
        eprintln!("  signature={sig:?} depth={depth:?} inspector={inspector:?}");
        if copies == 1.0 {
            let status = rt.status().unwrap();
            let mut top: Vec<(usize, String)> = status.as_object().unwrap().iter().map(|(k, v)| (v.to_string().len(), k.clone())).collect();
            top.sort(); top.reverse();
            eprintln!("  top-level: {:?}", &top[..top.len().min(8)]);
            if let Some(layer) = status["layers"].as_array().and_then(|l| l.first()) {
                let mut fields: Vec<(usize, String)> = layer.as_object().unwrap().iter().map(|(k, v)| (v.to_string().len(), k.clone())).collect();
                fields.sort(); fields.reverse();
                eprintln!("  one layer: {:?}", &fields[..fields.len().min(8)]);
            }
        }
        let render = time(&mut || { rt.engine.render_frame_without_background(&rt.doc.view(), t).unwrap(); });
        eprintln!("layers={layers} copies={copies:4}: resolve={resolve:?} status={status:?} ({status_bytes} bytes) render={render:?}");
    }
}
