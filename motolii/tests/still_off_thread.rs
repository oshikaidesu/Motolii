use motolii::render::engine::Engine;
use motolii::doc::export::export_still;
use motolii::doc::store::{Composition, Document, Fps, Intent};
use std::sync::mpsc;
use std::time::Duration;

const W: u32 = 64;
const H: u32 = 64;

fn doc_with_comp() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: W,
        height: H,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 30,
        background: [0.2, 0.4, 0.6, 1.0],
    }))
    .unwrap();
    doc
}

#[test]
fn export_still_returns_within_timeout_and_writes_a_real_png() {
    let doc = doc_with_comp();
    let out_path = std::env::temp_dir().join(format!(
        "motolii-export-still-test-{}.png",
        std::process::id()
    ));
    let out_path_for_thread = out_path.clone();

    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut engine = Engine::new().expect("headless Engine を用意できない");
        let view = doc.view();
        let result = export_still(&mut engine, &view, 0, &out_path_for_thread);
        let _ = tx.send(result);
    });

    let result = rx
        .recv_timeout(Duration::from_secs(60))
        .expect("export_still が60秒以内に戻らなかった(device.poll の無限待ちを疑う)");
    let report = result.expect("export_still が失敗した");
    assert_eq!(report.out_path, out_path);

    let img = image::open(&out_path).expect("書き出された PNG を読めない");
    assert_eq!(img.width(), W);
    assert_eq!(img.height(), H);

    let _ = std::fs::remove_file(&out_path);
}
