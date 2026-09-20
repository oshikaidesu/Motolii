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

/// 1 コマの時間を持ち主ごとに割る(Flutter 無し)。再生の Ticker と同じ順で
/// tick → renderInfo → view ごとに描く → status を回し、段ごとに時計を置く。
/// 拍を 1 つおきに二役へ振る: 偶数は本番の道(`motolii_probe_render` = IOSurface の
/// 紐付けも込み)、奇数は中の段を 1 つずつ。差が「橋と紐付けの取り分」。
/// `MOTOLII_PROBE_DOC` の書類、または `MOTOLII_PROBE_SCRIPT` の script、`MOTOLII_PROBE_SECONDS`(既定 12)秒、
/// `MOTOLII_PROBE_STAGE`(既定 `1000x700`)の Stage 窓。集計は終わりに 1 回。
#[test]
#[ignore]
fn frame_owners() {
    use crate::viewer::View;
    use objc2_core_foundation::{CFDictionary, CFNumber, CFString};
    let path = std::env::var("MOTOLII_PROBE_DOC").unwrap_or_default();
    let script = std::env::var("MOTOLII_PROBE_SCRIPT").unwrap_or_default();
    let seconds: f64 = std::env::var("MOTOLII_PROBE_SECONDS").ok().and_then(|v| v.parse().ok()).unwrap_or(12.0);
    let stage = std::env::var("MOTOLII_PROBE_STAGE").unwrap_or_else(|_| "1000x700".into());
    let (sw, sh) = stage.split_once('x').unwrap();
    let (sw, sh): (u32, u32) = (sw.parse().unwrap(), sh.parse().unwrap());
    // 書類を言われなければ、この file の見本(層 5 × 100 枚)で回す。
    // Script は実窓と同じ入口で書類へ落とすので、CSS の layout sample も
    // synthetic benchmark に取り替えず測れる。
    let mut rt = if path.is_empty() { crate::EditorRuntime::open("").unwrap() } else { crate::EditorRuntime::open(&path).unwrap() };
    if !script.is_empty() { rt.run_script_file(&script).unwrap(); }
    else if path.is_empty() { rt = runtime(5, 100.0); }
    let comp = rt.doc.view().composition().unwrap().unwrap().spec();
    // `stageWindow` は port の op ではなく lib.rs:275 の口。Stage tab が置く窓と同じ。
    rt.set_stage_window(&serde_json::json!({"width":sw,"height":sh,
        "roi":[0.0, 0.0, comp.width as f32, comp.height as f32]})).unwrap();

    let views = [View::Camera, View::User];
    // 本番の道の的。Flutter の host が作り置く物と同じ IOSurface。
    let surfaces: Vec<_> = views.iter().map(|v| {
        let w = rt.window(*v).unwrap();
        let keys: Vec<&CFString> = unsafe { vec![objc2_io_surface::kIOSurfaceWidth, objc2_io_surface::kIOSurfaceHeight, objc2_io_surface::kIOSurfaceBytesPerElement, objc2_io_surface::kIOSurfacePixelFormat] };
        let values = [CFNumber::new_i32(w.width as i32), CFNumber::new_i32(w.height as i32), CFNumber::new_i32(4), CFNumber::new_i32(u32::from_be_bytes(*b"BGRA") as i32)];
        let values: Vec<&CFNumber> = values.iter().map(|v| &**v).collect();
        unsafe { objc2_io_surface::IOSurfaceRef::new(CFDictionary::from_slices(&keys, &values).as_opaque()) }.expect("IOSurface")
    }).collect();
    // 中の段の的。紐付けを外して測るための素の texture。
    let device = rt.engine.gpu_device().clone();
    let targets: Vec<_> = views.iter().map(|v| {
        let w = rt.window(*v).unwrap();
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("frame-owners"), size: wgpu::Extent3d { width: w.width, height: w.height, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
            format: crate::render::compositor::PRESENTABLE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }).collect();

    let names = ["tick(FFI)", "renderInfo(FFI)", "status(FFI+serde)", "parse(serde)",
        "Camera whole(本番)", "Camera build+submit(CPU)", "Camera poll(GPU 待ち)", "Camera bounds", "Camera warm",
        "User whole(本番)", "User build+submit(CPU)", "User poll(GPU 待ち)", "User bounds", "User warm"];
    let mut owners: Vec<Vec<u64>> = names.iter().map(|_| Vec::new()).collect();
    let mut whole_tick = Vec::new();
    // 合否用: UI thread に居た時間を道ごとに。本番 = 出して返る、中の段 = その場で待つ(直す前の形)。
    let (mut budget, mut budget_waiting): (Vec<u64>, Vec<u64>) = (Vec::new(), Vec::new());
    let mut status_bytes = Vec::new();
    let (mut ticks, mut drawn, mut skipped) = (0u64, 0u64, 0u64);

    rt.request(serde_json::json!({"op":"play"})).unwrap();
    // 1 回目は shader と pipeline の compile。定常の値を測るので捨てる。
    for (view, target) in views.iter().zip(&targets) {
        let (t, cam, w) = (rt.time().unwrap(), rt.view_camera(*view).unwrap(), rt.window(*view).unwrap());
        rt.engine.render_frame_into_window(&rt.doc.view(), t, target, cam, true, &[], w).unwrap();
        rt.engine.gpu_device().poll(wgpu::PollType::wait_indefinitely()).unwrap();
    }

    let started = std::time::Instant::now();
    while started.elapsed().as_secs_f64() < seconds {
        let frame_start = std::time::Instant::now();
        ticks += 1;
        // 静かな tick の返信の `needsRender` と同じ審判(lib.rs:314 の image_key)。
        let before_image = rt.image_key();
        let t0 = std::time::Instant::now();
        rt.request(serde_json::json!({"op":"tick","quiet":true})).unwrap();
        owners[0].push(t0.elapsed().as_micros() as u64);
        if before_image == rt.image_key() {
            skipped += 1;
            whole_tick.push(frame_start.elapsed().as_micros() as u64);
            std::thread::sleep(std::time::Duration::from_micros(8_333));
            continue;
        }
        drawn += 1;
        let t0 = std::time::Instant::now();
        let _ = rt.request(serde_json::json!({"op":"renderInfo"})).unwrap();
        owners[1].push(t0.elapsed().as_micros() as u64);
        let production = drawn % 2 == 0;
        for (i, view) in views.iter().enumerate() {
            let base = 4 + i * 5;
            if production {
                let name = std::ffi::CString::new(view.name()).unwrap();
                let t0 = std::time::Instant::now();
                let code = unsafe { crate::motolii_probe_render(&mut rt, surfaces[i].id(), name.as_ptr()) };
                owners[base].push(t0.elapsed().as_micros() as u64);
                // 0 = 出した、1 = 番人が止めた(この harness は面が 1 枚なので有り得る)。
                assert!(code >= 0, "{:?}", rt.error);
                continue;
            }
            let target = &targets[i];
            let (t, cam, w) = (rt.time().unwrap(), rt.view_camera(*view).unwrap(), rt.window(*view).unwrap());
            rt.engine.set_realtime(true);
            let t0 = std::time::Instant::now();
            rt.engine.render_frame_into_window(&rt.doc.view(), t, target, cam, true, &[], w).unwrap();
            owners[base + 1].push(t0.elapsed().as_micros() as u64);
            let t0 = std::time::Instant::now();
            rt.engine.gpu_device().poll(wgpu::PollType::wait_indefinitely()).unwrap();
            owners[base + 2].push(t0.elapsed().as_micros() as u64);
            let t0 = std::time::Instant::now();
            rt.take_selection_bounds(*view, w);
            owners[base + 3].push(t0.elapsed().as_micros() as u64);
            let t0 = std::time::Instant::now();
            let _ = rt.engine.warm_upcoming(&rt.doc.view(), t);
            owners[base + 4].push(t0.elapsed().as_micros() as u64);
            rt.snapshot_cache.borrow_mut().invalidate_geometry();
        }
        let t0 = std::time::Instant::now();
        let status = rt.status().unwrap();
        let text = status.to_string();
        owners[2].push(t0.elapsed().as_micros() as u64);
        status_bytes.push(text.len() as u64);
        let t0 = std::time::Instant::now();
        let _: serde_json::Value = serde_json::from_str(&text).unwrap();
        owners[3].push(t0.elapsed().as_micros() as u64);
        whole_tick.push(frame_start.elapsed().as_micros() as u64);
        let spent = frame_start.elapsed().as_micros() as u64;
        if production { budget.push(spent) } else { budget_waiting.push(spent) }
        if spent < 8_333 { std::thread::sleep(std::time::Duration::from_micros(8_333 - spent)); }
    }
    rt.request(serde_json::json!({"op":"pause"})).unwrap();

    let stat = |v: &mut Vec<u64>| -> String {
        if v.is_empty() { return "       —        —        —  n=0".into() }
        v.sort_unstable();
        let p90 = v[(v.len() as f64 * 0.9).ceil() as usize - 1];
        format!("{:8.3} {:8.3} {:8.3}  n={}", v[v.len()/2] as f64/1000.0, p90 as f64/1000.0, v[v.len()-1] as f64/1000.0, v.len())
    };
    eprintln!("\nPROBE room=frame-owners doc={path} stage={sw}x{sh} comp={}x{} seconds={seconds}",
        comp.width, comp.height);
    eprintln!("ticks={ticks} drawn={drawn} skipped={skipped} ({:.0}%)", skipped as f64 * 100.0 / ticks.max(1) as f64);
    eprintln!("{:28} {:>8} {:>8} {:>8}", "owner (ms)", "median", "p90", "max");
    for (name, us) in names.iter().zip(owners.iter_mut()) {
        eprintln!("{:28} {}", name, stat(us));
    }
    eprintln!("{:28} {}", "1 tick (whole)", stat(&mut whole_tick));
    status_bytes.sort_unstable();
    eprintln!("status bytes median={} max={}", status_bytes.get(status_bytes.len()/2).copied().unwrap_or(0), status_bytes.last().copied().unwrap_or(0));
    // 合否は分布ではなく予算で。1 拍 8.333 ms に対して、UI thread に居た割合と超えた本数。
    // `waiting` はその場で GPU を待つ道(直す前の形)、`submit` は出して返る道(本番)。
    let verdict = |name: &str, ticks: &[u64]| {
        let over = |us: u64| ticks.iter().filter(|&&spent| spent > us).count();
        let spent: u64 = ticks.iter().sum();
        eprintln!("PROBE room=playback-budget path={name} ticks={} ui={:.0}% over16.7={} over33={}",
            ticks.len(), spent as f64 * 100.0 / (ticks.len().max(1) as f64 * 8_333.0), over(16_700), over(33_300));
    };
    verdict("waiting", &budget_waiting);
    verdict("submit", &budget);
    eprintln!("PROBE room=playback-budget skipped-by-guard={}", rt.frames.skipped());
}
