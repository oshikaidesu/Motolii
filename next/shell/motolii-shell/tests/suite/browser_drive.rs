//! 運転席 — Browser の一覧(裁定162 B1)。`AdmitPaths` が台帳へも記帳することを
//! 見る(`drive.rs::dropping_a_video_puts_a_layer_on_the_stage` と対の試験 —
//! あちらは layer 配置、こちらは台帳)。

use motolii_shell::{Message, Shell};

fn shell() -> Shell {
    Shell::new().0
}

fn make_clip(dir: &std::path::Path, name: &str, color: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    let out = std::process::Command::new("ffmpeg")
        .args([
            "-y", "-f", "lavfi", "-i",
        ])
        .arg(format!("color=c={color}:s=64x64:d=1:r=30"))
        .args(["-pix_fmt", "yuv420p", "-c:v", "libx264"])
        .arg(&path)
        .output()
        .expect("ffmpeg");
    assert!(out.status.success());
    path
}

/// **ORACLE (b)**: `AdmitPaths`(実 temp ファイル2本)→ `StoreView::assets()` に
/// 2件・layer も従来どおり配置される。
#[test]
fn admitting_two_files_registers_both_in_the_ledger_and_still_places_layers() {
    use motolii_testkit::{ffmpeg_or_skip, tmp_dir};

    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("browser-admit");
    let a = make_clip(&dir, "a.mp4", "orange");
    let b = make_clip(&dir, "b.mp4", "green");

    let mut shell = shell();
    let _ = shell.update(Message::AdmitPaths(vec![a, b]));

    assert_eq!(
        shell.assets().len(),
        2,
        "2本 admit したのに台帳に2件現れない"
    );
    assert_eq!(
        shell.layer_count(),
        2,
        "台帳への記帳と一緒に layer 配置(従来挙動)が壊れている"
    );
    assert_eq!(shell.status(), None, "受理できたのに拒否理由が出ている");
}

/// **ORACLE (b の続き)**: 同一ファイルを2回 drop しても台帳は1件のまま
/// (`AssetTable::admit` の content_hash 重複統合、shell 側で先回りしない)。
#[test]
fn dropping_the_same_file_twice_keeps_a_single_ledger_entry() {
    use motolii_testkit::{ffmpeg_or_skip, tmp_dir};

    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("browser-admit-dup");
    let clip = make_clip(&dir, "dup.mp4", "blue");

    let mut shell = shell();
    let _ = shell.update(Message::AdmitPaths(vec![clip.clone()]));
    assert_eq!(shell.assets().len(), 1, "1本目の admit で台帳に載らない");

    let _ = shell.update(Message::AdmitPaths(vec![clip]));
    assert_eq!(
        shell.assets().len(),
        1,
        "同一ファイルの再 drop で台帳が2件に増えた(重複統合が効いていない)"
    );
    // layer 配置は独立(bin-first) — 2回落とせば2 layer 置かれる(重複統合は
    // 台帳の関心事であって layer 配置の関心事ではない)。
    assert_eq!(shell.layer_count(), 2);
}

/// **順序**: 一覧は admit 順(id 昇順)で決定論。
#[test]
fn ledger_projection_preserves_admission_order() {
    use motolii_testkit::{ffmpeg_or_skip, tmp_dir};

    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("browser-admit-order");
    let first = make_clip(&dir, "first.mp4", "red");
    let second = make_clip(&dir, "second.mp4", "yellow");

    let mut shell = shell();
    let _ = shell.update(Message::AdmitPaths(vec![first]));
    let _ = shell.update(Message::AdmitPaths(vec![second]));

    let assets = shell.assets();
    assert_eq!(assets.len(), 2);
    assert!(
        assets[0].id < assets[1].id,
        "admit 順(id 昇順)を保っていない: {:?}",
        assets.iter().map(|a| a.id).collect::<Vec<_>>()
    );
}

/// **開けない物(probe 失敗)でも fingerprint が読めれば台帳には載る**
/// (bin-first: 記帳と配置は別の判断、`Shell::admit` の doc 参照)。layer は
/// 置かれない(従来どおり拒否理由が status 帯に出る、M2/M13)。
#[test]
fn an_unopenable_file_still_gets_a_ledger_entry_without_being_placed() {
    use motolii_testkit::tmp_dir;

    let dir = tmp_dir("browser-admit-junk");
    let junk = dir.join("not-a-video.txt");
    std::fs::write(&junk, b"hello").unwrap();

    let mut shell = shell();
    let _ = shell.update(Message::AdmitPaths(vec![junk]));

    assert_eq!(
        shell.assets().len(),
        1,
        "fingerprint は読めるはずのファイルが台帳に載らない"
    );
    assert_eq!(shell.layer_count(), 0, "開けない物を layer にしてしまった");
    assert!(
        shell.status().is_some(),
        "拒否理由が出ていない(M13: 無反応ゼロ違反)"
    );
}
