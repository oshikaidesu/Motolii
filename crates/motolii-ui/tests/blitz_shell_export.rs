//! shell 統合: 書き出し面(Export ボタンの後ろ)。dialog(`rfd`)はここに出てこない。
//!
//! テストが呼ぶのは dialog の**後ろ**にある関数だけで、製品も同じ関数を同じ順で
//! 呼ぶ(`blitz_shell_save.rs` / `blitz_shell_file_entry.rs` と同じ形):
//!
//! - 開始判断   … `can_start_export`(project が座っていて、実行中でない時だけ。
//!                 Export ボタンの enabled と二重起動防止が同じ関数を見る)
//! - 既定名     … `default_export_file_name`(save dialog の既定 `{project名}.mp4`)
//! - 開始列     … `ExportRun::start`(snapshot + path を渡すと**別thread**で
//!                 `export_document_video` が走る。UI は `try_finish` を poll する)
//! - キャンセル … `ExportRun::cancel` → export 側の cancel 口
//!                 (`export_document_video_cancellable`)が frame loop を早期終了し、
//!                 出力 file を残さない
//!
//! 実書き出しの fixture は `headless_mv_e2e.rs` の慣行(ffmpeg lavfi 生成、
//! `ffmpeg_or_skip` / `gpu_or_skip`)に合わせる。

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

use motolii_testkit::{ffmpeg_or_skip, gpu_or_skip, tmp_dir};
use motolii_ui::blitz_shell::{create_project_file, ProjectSeat};
use motolii_ui::export_seat::{
    can_start_export, default_export_file_name, run_export_worker, ExportFinish, ExportRequest,
    ExportRun,
};

/// `headless_mv_e2e.rs` と同じ ffmpeg fixture 生成。
fn run_ffmpeg(args: &[&str]) {
    let status = Command::new("ffmpeg")
        .args(["-v", "error", "-y"])
        .args(args)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success(), "ffmpeg failed: {args:?}");
}

fn make_video(path: &Path) {
    run_ffmpeg(&[
        "-f",
        "lavfi",
        "-i",
        "color=c=red:s=640x360:d=1.0:r=24",
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        path.to_str().unwrap(),
    ]);
}

/// 実 project を1つ作って開き、動画を1本 playhead へ取り込む(製品と同じ
/// `create_project_file` → `ProjectSeat::open` → `import_dropped_media` の列)。
fn seat_with_video(dir: &Path) -> (PathBuf, ProjectSeat) {
    let video = dir.join("clip.mp4");
    make_video(&video);
    let project = dir.join("mv.json");
    create_project_file(&project).expect("create project");
    let mut seat = ProjectSeat::open(&project).expect("open project");
    let outcome = seat.editor_mut().import_dropped_media(&[video]);
    assert_eq!(
        outcome.placed.len(),
        1,
        "fixture video must place: {outcome:?}"
    );
    (project, seat)
}

/// Export ボタンの enabled と二重起動防止は同じ判断関数を見る:
/// project が座っていて、実行中の書き出しが無い時だけ開始できる。
#[test]
fn export_can_start_only_when_seated_and_idle() {
    assert!(
        can_start_export(true, false),
        "座席あり・待機中は開始できる"
    );
    assert!(
        !can_start_export(true, true),
        "実行中は開始できない(二重起動防止 = ボタン disabled)"
    );
    assert!(
        !can_start_export(false, false),
        "project が居なければ書き出すものが無い"
    );
    assert!(!can_start_export(false, true));
}

/// save dialog の既定名は `{project名}.mp4`。名前が取れない path でも
/// 空文字にはしない(dialog に空の既定を出さない)。
#[test]
fn default_export_name_is_the_project_stem_with_mp4() {
    assert_eq!(
        default_export_file_name(Path::new("/tmp/mv.json")),
        "mv.mp4"
    );
    assert_eq!(
        default_export_file_name(Path::new("relative/untitled.json")),
        "untitled.mp4"
    );
    assert_eq!(default_export_file_name(Path::new("/")), "export.mp4");
}

/// キャンセル済みの worker は何も作らない(thread の中身と同じ関数を同期で駆動)。
/// GPU も ffmpeg も要らない = どの環境でも決定的に通る。
#[test]
fn a_cancel_before_start_exports_nothing() {
    let dir = tmp_dir("blitz_shell_export_precancel");
    let out = dir.join("never.mp4");
    let request = ExportRequest {
        document: Arc::new(motolii_doc::Document::new_current()),
        project_root: None,
        output_path: out.clone(),
        frame_count: None,
        qp0: false,
    };
    let cancel = AtomicBool::new(true);
    let finish = run_export_worker(&request, &cancel);
    assert!(
        matches!(finish, ExportFinish::Cancelled),
        "pre-cancelled worker must report Cancelled, got {finish:?}"
    );
    assert!(!out.exists(), "cancelled export must not create the file");
}

/// dialog 抜きの開始列: snapshot + 明示 path で `ExportRun::start` すると
/// 別thread の書き出しが走り、`try_finish` の poll で完了が返る。
/// 未保存編集(dirty)のままでも**現 snapshot**から書き出せる。
#[test]
fn the_export_sequence_writes_the_snapshot_to_the_chosen_path() {
    if !ffmpeg_or_skip() {
        return;
    }
    let Some(_gpu_probe) = gpu_or_skip() else {
        return;
    };
    let dir = tmp_dir("blitz_shell_export_real");
    let (project, seat) = seat_with_video(&dir);
    assert!(
        seat.is_dirty(),
        "import は未保存編集 — dirty でも書き出せるのが前提"
    );

    let out = dir.join("mv-out.mp4");
    let mut run = ExportRun::start(ExportRequest {
        document: seat.snapshot(),
        project_root: project.parent().map(Path::to_path_buf),
        output_path: out.clone(),
        // v0 の製品は None(composition 全長)。テストは 2 frame で速く審判する。
        frame_count: Some(2),
        qp0: true,
    });
    assert!(!run.cancel_requested(), "開始直後は cancel されていない");

    let deadline = Instant::now() + Duration::from_secs(300);
    let finish = loop {
        if let Some(finish) = run.try_finish() {
            break finish;
        }
        assert!(Instant::now() < deadline, "export must finish in time");
        std::thread::sleep(Duration::from_millis(50));
    };
    match finish {
        ExportFinish::Done(report) => {
            assert_eq!(report.frames_written, 2, "asked frames get written");
        }
        other => panic!("export must succeed, got {other:?}"),
    }
    assert!(out.exists(), "the chosen path carries the exported file");
    assert!(
        run.try_finish().is_none(),
        "a finished run reports once — the UI clears it after the first answer"
    );
}

/// cancel 口の審判: cancel された書き出しは frame loop を早期終了し、
/// **出力 file を残さない**(部分 file が成果物に見えないこと)。
#[test]
fn cancel_stops_the_frame_loop_early_and_removes_the_file() {
    if !ffmpeg_or_skip() {
        return;
    }
    let Some(gpu) = gpu_or_skip() else {
        return;
    };
    let dir = tmp_dir("blitz_shell_export_cancel");
    let (project, seat) = seat_with_video(&dir);
    let snapshot = seat.snapshot();
    let runtime = motolii_plugins_firstparty::first_party_runtime().expect("first-party runtime");

    let out = dir.join("cancelled.mp4");
    // 立てた cancel は frame loop の毎周で見える — 最初の周で早期 return する。
    let cancel = AtomicBool::new(true);
    let err = motolii_export::export_document_video_cancellable(
        &gpu,
        &motolii_export::ExportJob {
            doc: &snapshot,
            runtime: &runtime,
            output_path: &out,
            project_root: project.parent(),
            frame_count: Some(5),
            qp0: true,
            data_tracks: motolii_eval::DataTracks::new(),
        },
        &cancel,
    )
    .expect_err("a cancelled export must not report success");
    assert!(
        matches!(err, motolii_export::ExportError::Cancelled),
        "the early return names the cancel, got {err:?}"
    );
    assert!(
        !out.exists(),
        "a cancelled export must remove the partial output file"
    );
}
