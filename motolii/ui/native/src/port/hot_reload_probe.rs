use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct Probe(std::path::PathBuf);
impl Drop for Probe { fn drop(&mut self) { let _ = std::fs::remove_file(&self.0); } }

/// FSEvents は同じ file の続けざまの変更を 1 つに潰す。人の保存の間隔を真似る。
fn a_moment() { std::thread::sleep(std::time::Duration::from_millis(1500)); }

fn await_wake(flag: &AtomicBool) -> bool {
    let started = std::time::Instant::now();
    while started.elapsed() < std::time::Duration::from_secs(5) {
        if flag.swap(false, Ordering::AcqRel) { return true; }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    false
}

#[test]
#[ignore]
fn saving_a_vism_file_reaches_the_window() {
    assert!(crate::render::engine::catalog_reads_disk(), "焼き込み build: .cargo/config.toml の IS_IN_RERUN_WORKSPACE が無い");
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../crates/motolii-render/vism");
    let file = Probe(dir.join("zz_hot_probe.fs"));
    let _ = std::fs::remove_file(&file.0);
    let mut rt = crate::EditorRuntime::open("").unwrap();
    rt.request(serde_json::json!({"op":"reloadEffects"})).unwrap();
    let before = crate::render::engine::catalog_generation();
    let woke = Arc::new(AtomicBool::new(false));
    let flag = woke.clone();
    let _watch = crate::render::engine::watch_effect_catalog(move || flag.store(true, Ordering::Release)).unwrap();
    let ids = |rt: &mut crate::EditorRuntime| -> Vec<String> {
        rt.status_response(None, None).unwrap()["catalog"].as_array().unwrap().iter().map(|r| r["id"].as_str().unwrap().to_owned()).collect()
    };
    let errors = |rt: &mut crate::EditorRuntime| -> Vec<String> {
        rt.status_response(None, None).unwrap()["catalogErrors"].as_array().unwrap().iter().map(|e| e.as_str().unwrap().to_owned()).collect()
    };
    assert!(!ids(&mut rt).contains(&"probe.zz_hot".to_owned()));

    // 1. 新しい効果を保存 → 見張りが起きる → 読み直せば棚に載り、絵の鍵(世代)が動く。
    std::fs::write(&file.0, "/*{ \"ID\": \"probe.zz_hot\", \"LABEL\": \"Hot\", \"STAGE\": \"pass\", \"INPUTS\": [ { \"NAME\": \"inputImage\", \"TYPE\": \"image\" } ] }*/\nvoid main() { gl_FragColor = IMG_THIS_PIXEL(inputImage).bgra; }\n").unwrap();
    assert!(await_wake(&woke), "見張りが 5 秒待っても起きない");
    let key = rt.image_key();
    rt.request(serde_json::json!({"op":"reloadEffects"})).unwrap();
    assert!(crate::render::engine::catalog_generation() > before, "世代が進まない");
    assert_ne!(rt.image_key(), key, "絵の鍵が世代を含んでいない");
    assert!(ids(&mut rt).contains(&"probe.zz_hot".to_owned()), "{:?}", ids(&mut rt));
    assert!(errors(&mut rt).iter().all(|e| !e.contains("zz_hot_probe")), "{:?}", errors(&mut rt));

    // 2. 壊して保存 → 前の物が残り、理由が名前付きで出る。
    a_moment();
    std::fs::write(&file.0, "/*{ \"ID\": \"probe.zz_hot\", \"STAGE\": \"pass\", \"INPUTS\": [ { \"NAME\": \"inputImage\", \"TYPE\": \"image\" } ] }*/\nvoid main() { gl_FragColor = ; }\n").unwrap();
    assert!(await_wake(&woke));
    rt.request(serde_json::json!({"op":"reloadEffects"})).unwrap();
    assert!(ids(&mut rt).contains(&"probe.zz_hot".to_owned()), "壊した瞬間に棚から消えた");
    assert!(errors(&mut rt).iter().any(|e| e.starts_with("zz_hot_probe:")), "{:?}", errors(&mut rt));

    // 3. 消す → 理由が「source removed」に変わる。
    a_moment();
    std::fs::remove_file(&file.0).unwrap();
    assert!(await_wake(&woke));
    rt.request(serde_json::json!({"op":"reloadEffects"})).unwrap();
    assert!(errors(&mut rt).iter().any(|e| e.contains("zz_hot_probe") && e.contains("source removed")), "{:?}", errors(&mut rt));
}

/// A saved Vism reaches a layer that did not change: its program, and the Views it asks for, follow
/// the file (no build, no restart). A broken manifest keeps the last good one until it is fixed.
#[test]
#[ignore]
fn saving_a_vism_reaches_a_still_layer_and_its_views() {
    assert!(crate::render::engine::catalog_reads_disk(), "焼き込み build: .cargo/config.toml の IS_IN_RERUN_WORKSPACE が無い");
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../crates/motolii-render/vism");
    let file = Probe(dir.join("zz_view_probe.wgsl"));
    let vism = |views: &str, rgb: &str| format!(
        "/*{{ \"ID\": \"probe.zz_view\", \"LABEL\": \"View Probe\", \"STAGE\": \"surface\"{views} }}*/\nfn surface(in: SurfaceIn, p: SurfaceParams) -> vec3f {{ return vec3f({rgb}) + view_sample(0u, vec2f(0.5), 0.0).rgb * 0.0; }}\n");
    let one_view = r#", "VIEWS": [ { "FROM": "layer", "LOOK": [0, 0, -1] } ], "VIEW_SIZE": 32"#;
    let _ = std::fs::remove_file(&file.0);
    let mut rt = crate::EditorRuntime::open("").unwrap();
    let woke = Arc::new(AtomicBool::new(false));
    let flag = woke.clone();
    let _watch = crate::render::engine::watch_effect_catalog(move || flag.store(true, Ordering::Release)).unwrap();
    // A save, as the window sees it: the watcher wakes, the window asks the shelf again.
    let save = |rt: &mut crate::EditorRuntime, text: String| {
        a_moment();
        std::fs::write(&file.0, text).unwrap();
        assert!(await_wake(&woke), "the watcher did not wake");
        rt.request(serde_json::json!({"op":"reloadEffects"})).unwrap();
    };
    save(&mut rt, vism(one_view, "1.0, 0.0, 0.0"));
    rt.request(serde_json::json!({"op":"create","kind":"rectangle"})).unwrap();
    rt.request(serde_json::json!({"op":"applyEffect","pluginId":"probe.zz_view"})).unwrap();
    let mut draw = |rt: &mut crate::EditorRuntime| {
        let before = rt.engine.surface_work().layer_views;
        let pixels = rt.engine.render_frame(&rt.doc.view(), crate::render::doc::core::RationalTime::ZERO).unwrap();
        assert!(rt.engine.layer_failures().is_empty(), "{:?}", rt.engine.layer_failures());
        (pixels, rt.engine.surface_work().layer_views - before)
    };
    let (red, views) = draw(&mut rt);
    assert_eq!(views, 1, "the Vism asked for one View");
    assert_eq!(draw(&mut rt).0, red, "the same frame again");

    // The WGSL changes, the layer does not: the picture follows the file.
    save(&mut rt, vism(one_view, "0.0, 0.0, 1.0"));
    let (blue, views) = draw(&mut rt);
    assert_ne!(blue, red, "a saved WGSL reaches a still layer");
    assert_eq!(views, 1);

    // The manifest drops its View: the host draws none.
    save(&mut rt, vism("", "0.0, 0.0, 1.0"));
    assert_eq!(draw(&mut rt).1, 0, "no View asked, none drawn");

    // A View without FROM is refused: the last good Vism stays, the reason is named; fixing it recovers.
    save(&mut rt, vism(r#", "VIEWS": [ { "LOOK": [0, 0, -1] } ]"#, "0.0, 1.0, 0.0"));
    let errors: Vec<String> = rt.status_response(None, None).unwrap()["catalogErrors"].as_array().unwrap().iter().map(|e| e.as_str().unwrap().to_owned()).collect();
    assert!(errors.iter().any(|e| e.starts_with("zz_view_probe:") && e.contains("FROM")), "{errors:?}");
    let (kept, views) = draw(&mut rt);
    assert_eq!((kept, views), (blue.clone(), 0), "the last good Vism is kept");
    save(&mut rt, vism(one_view, "0.0, 1.0, 0.0"));
    let (green, views) = draw(&mut rt);
    assert_ne!(green, blue, "fixed, it recovers");
    assert_eq!(views, 1);
}

/// Puts a file back as it was when dropped (a shelf file the probe edits in place).
struct Restore(std::path::PathBuf, String);
impl Drop for Restore { fn drop(&mut self) { let _ = std::fs::write(&self.0, &self.1); } }

/// The standard material is a module on the shelf: saving it reaches every surface (Standard
/// Glass included) with no build, as a Vism does.
#[test]
#[ignore]
fn saving_the_standard_material_reaches_its_surfaces() {
    assert!(crate::render::engine::catalog_reads_disk(), "焼き込み build: .cargo/config.toml の IS_IN_RERUN_WORKSPACE が無い");
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../crates/motolii-render/vism/material.wgsl");
    let original = std::fs::read_to_string(&path).unwrap();
    let _restore = Restore(path.clone(), original.clone());
    // Every path through shade_surface starts here (with or without an environment).
    let returned = "    let roughness = clamp(surface.x, 0.0, 1.0);";
    assert!(original.contains(returned), "shade_surface's first line moved; update the probe");
    let mut rt = crate::EditorRuntime::open("").unwrap();
    let woke = Arc::new(AtomicBool::new(false));
    let flag = woke.clone();
    let _watch = crate::render::engine::watch_effect_catalog(move || flag.store(true, Ordering::Release)).unwrap();
    rt.request(serde_json::json!({"op":"create","kind":"rectangle"})).unwrap();
    rt.request(serde_json::json!({"op":"applyEffect","pluginId":"motolii.glass"})).unwrap();
    let draw = |rt: &mut crate::EditorRuntime| rt.engine.render_frame(&rt.doc.view(), crate::render::doc::core::RationalTime::ZERO).unwrap();
    let before = draw(&mut rt);
    assert!(before.chunks_exact(4).any(|p| p[0] > 0 || p[1] > 0 || p[2] > 0), "the glass is drawn");
    a_moment();
    std::fs::write(&path, original.replace(returned, &format!("    if surface.x > -1.0 {{ return vec3f(1.0, 0.0, 1.0); }}\n{returned}"))).unwrap();
    assert!(await_wake(&woke), "the watcher did not wake");
    rt.request(serde_json::json!({"op":"reloadEffects"})).unwrap();
    let magenta = draw(&mut rt);
    assert!(rt.engine.layer_failures().is_empty(), "{:?}", rt.engine.layer_failures());
    assert_ne!(magenta, before, "a saved material reaches the glass");
    a_moment();
    std::fs::write(&path, &original).unwrap();
    assert!(await_wake(&woke));
    rt.request(serde_json::json!({"op":"reloadEffects"})).unwrap();
    assert_eq!(draw(&mut rt), before, "and back");
}
