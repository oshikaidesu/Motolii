//! 運転席 — 窓を開けずに shell を動かす。
//!
//! 見るのは背骨1(書き口が1箇所)と M13(拒否が必ず出る)と、
//! **描画キャッシュが `revision()` で正しく落ちること**。

use motolii_shell::{Message, Shell};

fn shell() -> Shell {
    Shell::new().0
}

#[test]
fn adding_a_layer_shows_up_and_undo_takes_it_back() {
    let mut shell = shell();
    assert_eq!(shell.layer_count(), 0);

    shell.update(Message::AddLayer);
    assert_eq!(shell.layer_count(), 1, "layer を足しても増えない");

    // **1操作 = 1 Undo**。`AddLayer` は内部では AddLayer + SetMeta の2 intent だが、
    // 利用者から見れば1操作なので Undo 1回で消えなければならない(M10)。
    shell.update(Message::Undo);
    assert_eq!(
        shell.layer_count(),
        0,
        "Undo 1回で消えない = 1操作が複数 undo になっている"
    );
    assert!(!shell.can_undo(), "底より前へ戻れてはいけない");
}

#[test]
fn rejection_reaches_the_status_band() {
    let mut shell = shell();
    shell.update(Message::Undo);
    assert_eq!(
        shell.status(),
        Some("これ以上戻せない"),
        "戻せない時に何も言わないのは M13 違反(無反応ゼロ)"
    );

    // 次の操作で理由が消えること(古い理由が居座らない)。
    shell.update(Message::AddLayer);
    assert_eq!(shell.status(), None);
}

#[test]
fn frame_cache_follows_revision_and_playhead() {
    let mut shell = shell();
    shell.update(Message::AddLayer);
    let first = shell.frame_token().expect("frame");

    // 同じ入力なら描き直さない。
    shell.update(Message::Select(motolii_store::LayerId(1)));
    assert_eq!(shell.frame_token(), Some(first.clone()), "選択だけで描き直している");

    // 再生位置が動いたら描き直す。
    shell.update(Message::ScrubTo(10));
    assert_ne!(shell.frame_token(), Some(first.clone()), "scrub で描き直していない");

    // undo で Document が戻ったら描き直す(store 世代は変わらないので
    // `revision()` が edit 位置も見ていないとここが落ちる)。
    let scrubbed = shell.frame_token().expect("frame");
    shell.update(Message::Undo);
    assert_ne!(shell.frame_token(), Some(scrubbed), "undo で描き直していない");
}

/// **核の一周の前半** — 落とす → 素材が立つ → Stage に絵が出る。
#[test]
fn dropping_a_video_puts_a_layer_on_the_stage() {
    use motolii_testkit::{ffmpeg_or_skip, tmp_dir};
    use std::process::Command;

    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("shell-drop");
    let video = dir.join("drop.mp4");
    let out = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=orange:s=128x128:d=1:r=30",
            "-pix_fmt",
            "yuv420p",
            "-c:v",
            "libx264",
        ])
        .arg(&video)
        .output()
        .expect("ffmpeg");
    assert!(out.status.success());

    let mut shell = shell();
    let before = shell.frame_token();

    shell.update(Message::AdmitPaths(vec![video]));

    assert_eq!(shell.layer_count(), 1, "落とした素材が layer にならない");
    assert_eq!(shell.status(), None, "受理できたのに拒否理由が出ている");
    assert_ne!(shell.frame_token(), before, "Stage が描き直されていない");

    // 1操作 = 1 undo
    shell.update(Message::Undo);
    assert_eq!(shell.layer_count(), 0, "落とした分が Undo 1回で消えない");
}

/// **開けない物は理由つきで飛ばす**(M2)。黙って消さない。
#[test]
fn unopenable_files_are_rejected_with_a_reason() {
    use motolii_testkit::tmp_dir;

    let dir = tmp_dir("shell-reject");
    let junk = dir.join("not-a-video.txt");
    std::fs::write(&junk, b"hello").unwrap();

    let mut shell = shell();
    shell.update(Message::AdmitPaths(vec![junk]));

    assert_eq!(shell.layer_count(), 0, "開けない物を layer にしてしまった");
    let status = shell.status().expect("拒否理由が出ていない = M2 違反");
    assert!(
        status.contains("not-a-video.txt"),
        "どのファイルが駄目だったか分からない: {status}"
    );
}

/// **3本まとめて落として1操作**。winit は1ファイル1事象で送ってくるので、
/// そのまま処理すると 3 undo になる(旧 workspace が `drive_drop.rs` で守っていた性質)。
#[test]
fn three_drops_become_one_operation() {
    use motolii_testkit::{ffmpeg_or_skip, tmp_dir};
    use std::process::Command;

    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("shell-batch");
    let mut paths = Vec::new();
    for i in 0..3 {
        let path = dir.join(format!("clip{i}.mp4"));
        let out = Command::new("ffmpeg")
            .args([
                "-y",
                "-f",
                "lavfi",
                "-i",
                "color=c=green:s=64x64:d=1:r=30",
                "-pix_fmt",
                "yuv420p",
                "-c:v",
                "libx264",
            ])
            .arg(&path)
            .output()
            .expect("ffmpeg");
        assert!(out.status.success());
        paths.push(path);
    }

    let mut shell = shell();
    for path in paths {
        shell.update(Message::DropReceived(path));
    }
    assert_eq!(shell.layer_count(), 0, "区切りが来る前に取り込んでいる");

    shell.update(Message::FlushDrops);
    assert_eq!(shell.layer_count(), 3, "3本とも入っていない");

    shell.update(Message::Undo);
    assert_eq!(shell.layer_count(), 0, "3本の取り込みが Undo 1回で消えない");
}
