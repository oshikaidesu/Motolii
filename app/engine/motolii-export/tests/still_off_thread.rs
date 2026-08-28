//! 回帰試験(`export_still_now` の UI スレッド固まり——`app/motolii/src/main.rs`
//! `start_still_export` の doc 参照)。この試験は makepad の event loop を持たないので
//! 実窓の相互待ち(Stage 描画と export の GPU 待ちの奪い合い)そのものは再現できない
//! ——それは `--hot --remote` の実窓でしか確かめられない(呼び出し元の doc 参照)。
//!
//! ここで機械的に縛れるのはこれだけ: **`export_still` を呼んだスレッドは有限時間で
//! 戻る**。`device.poll(wait_indefinitely())` の中身が壊れて本当に無限待ちへ戻ったら、
//! この試験は(壁時計 timeout で)固まる代わりに落ちる。合わせて、`report.out_path` に
//! 実物の PNG が書かれることも確認する——静止画書き出しが実測で一度も
//! 確認されていなかった穴(呼び出し元 commit 参照)を埋める。
use motolii_engine::Engine;
use motolii_export::export_still;
use motolii_store::{Composition, Document, Fps, Intent};
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

/// `start_still_export`(main.rs)が worker thread の中で組む形をそのまま写す:
/// **自分専用の `Engine::new()`**——UI スレッドの Stage 描画が使う Engine とは
/// 別インスタンス。ここでは呼び出し元スレッド自体が「UI スレッドに見立てた側」で、
/// 別スレッドで export を回して timeout つきで受け取る。
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
