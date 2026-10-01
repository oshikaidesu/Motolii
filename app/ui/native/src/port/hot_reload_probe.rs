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
