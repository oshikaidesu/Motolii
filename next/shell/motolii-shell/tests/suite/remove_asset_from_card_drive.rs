//! 運転席 — MC-1(`A01-entry.tsv` `RemoveAsset` 行、2026-08-23結線)。
//!
//! `browser_pane::Message::RemoveAssetFromCard` は pane-local には状態を
//! 動かさない(`state.rs` の `Message::RemoveAssetFromCard(_) => {}` —
//! ORACLE、`create_from_card_drive.rs`/`mask_effect_from_card_drive.rs` と
//! 同じ形)。この試験は唯一の cargo test 対象(裁定220):
//! **「Browser の削除ボタンを押すと素材が台帳から消え、undo で戻る」**。
//! 畳んだ口(`create.rs::dispatch_browser_card_intent`)の最初の利用者になる。

use motolii_shell::{browser_pane, Message, Shell};

#[test]
fn removing_an_asset_from_the_browser_card_drops_it_from_the_ledger_and_undo_restores_it() {
    use motolii_testkit::{ffmpeg_or_skip, tmp_dir};

    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("browser-remove");
    let path = dir.join("clip.mp4");
    let out = std::process::Command::new("ffmpeg")
        .args(["-y", "-f", "lavfi", "-i", "color=c=red:s=64x64:d=1:r=30"])
        .args(["-pix_fmt", "yuv420p", "-c:v", "libx264"])
        .arg(&path)
        .output()
        .expect("ffmpeg");
    assert!(out.status.success());

    let mut shell = Shell::new().0;
    let _ = shell.update(Message::AdmitPaths(vec![path]));
    assert_eq!(shell.assets().len(), 1, "admit 直後に台帳へ載っていない");
    let asset_id = shell.assets()[0].id;

    // **押して消える**: RemoveAssetFromCard を発火すると台帳から消える
    // (`dispatch_browser_card_intent` → `remove_asset_from_card` →
    // `Intent::RemoveAsset` の経路)。
    let _ = shell.update(Message::Browser(browser_pane::Message::RemoveAssetFromCard(
        asset_id,
    )));
    assert_eq!(
        shell.assets().len(),
        0,
        "RemoveAssetFromCard を発火しても台帳から消えていない(Q0違反のまま)"
    );
    assert_eq!(shell.status(), None, "削除できたのに拒否理由が出ている");

    // **undo で戻る**: 1 Intent = 1 undo 段(他の card 系動詞と同じ規律)。
    let _ = shell.update(Message::Undo);
    assert_eq!(
        shell.assets().len(),
        1,
        "undo しても台帳に素材が戻っていない"
    );
    assert_eq!(shell.assets()[0].id, asset_id, "戻った素材の id が違う");
}
