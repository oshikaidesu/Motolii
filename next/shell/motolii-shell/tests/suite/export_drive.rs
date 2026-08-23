//! 運転席 — 第6波 shell 結線(B09: Export 窓、`export_pane` crate doc
//! 「shell 結線」節)+第2波(2026-08-22、File 束の rfd 非同期化と同時発注:
//! 書き出し先の選択)。Settings 窓(S2)と同じ第2窓の型
//! (`toggle_export_window`/`view_window`/`window_title`)・`ToggleExportDialog`・
//! 品質/範囲の選択状態・書き出し先選択・Cmd+E を検分する。
//!
//! `Message::Export(PickOutputPath)`/`OutputPathChosen` は File 束
//! (`file_drive.rs`)と同じ非同期腕(`FileDialogs::pick_export_path` →
//! `iced::Task::perform`)なので、production の `Shell::new_fixture()`
//! (実 `RfdDialogs`)ではなく `crate::file_drive::shell_with_fake` +
//! `crate::file_drive::drive`(`Task<Message>` を同期に汲み取って後続の
//! `Message` を流し込む、`file_drive.rs` 冒頭 doc 参照)を使う ── この2本の
//! テスト binary は `suite_main.rs` で1本に束ねられている sibling module
//! なので、`crate::file_drive::` 経由でそのまま再利用できる。
//!
//! **未検分(RETURN 参照)**: `Message::Export(Export)` の実行(実際の
//! encode)は `motolii_export::export_with_cancel` を同期に1回呼ぶだけ
//! (フレーム単位の進捗コールバックが export 側に無い)。よってこの試験は
//! 窓の開閉・品質/範囲選択・書き出し先選択・キーマップまでを見る — 実行
//! そのもの(`start_export`)はここでは駆動できない。

use motolii_shell::{export_pane, Message, Shell};

use crate::file_drive::{drive, shell_with_fake};

#[test]
fn cmd_e_resolves_to_toggle_export_dialog() {
    use iced::keyboard::{Key, Modifiers};
    let key = Key::Character("e".into());
    let message = motolii_shell::resolve_navigation_key(&key, Modifiers::COMMAND, false);
    assert!(matches!(
        message,
        Some(Message::Export(export_pane::Message::ToggleExportDialog))
    ));
}

#[test]
fn cmd_e_is_not_stolen_while_typing() {
    use iced::keyboard::{Key, Modifiers};
    let key = Key::Character("e".into());
    assert!(
        motolii_shell::resolve_navigation_key(&key, Modifiers::COMMAND, true).is_none(),
        "captured=true(text 編集中)なのに Cmd+E が発火している"
    );
}

#[test]
fn toggle_export_dialog_opens_and_closes_the_window_ledger() {
    let mut shell = Shell::new_fixture().0;
    assert_eq!(shell.export_window(), None, "既定で Export 窓が開いている");

    let _ = shell.update(Message::Export(export_pane::Message::ToggleExportDialog));
    assert!(shell.export_window().is_some(), "ToggleExportDialog が窓を開いていない");

    let _ = shell.update(Message::Export(export_pane::Message::ToggleExportDialog));
    assert_eq!(shell.export_window(), None, "もう一度押しても閉じない(トグルになっていない)");
}

#[test]
fn window_closed_on_the_export_window_id_clears_the_ledger() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Export(export_pane::Message::ToggleExportDialog));
    let id = shell.export_window().expect("開いたはずの Export 窓が台帳に無い");

    let _ = shell.update(Message::WindowClosed(id));
    assert_eq!(shell.export_window(), None, "OS の閉じるボタン経路で台帳が抹消されていない");
}

#[test]
fn quality_and_range_select_update_the_held_state() {
    let mut shell = Shell::new_fixture().0;
    assert_eq!(shell.export_quality(), export_pane::ExportQuality::Normal, "既定品質が違う");
    assert_eq!(shell.export_range(), export_pane::ExportRange::Whole, "既定範囲が違う");

    let _ = shell.update(Message::Export(export_pane::Message::QualitySelect(
        export_pane::ExportQuality::Lossless,
    )));
    assert_eq!(shell.export_quality(), export_pane::ExportQuality::Lossless);

    let _ = shell.update(Message::Export(export_pane::Message::RangeSelect(
        export_pane::ExportRange::WorkArea,
    )));
    assert_eq!(shell.export_range(), export_pane::ExportRange::WorkArea);
}

#[test]
fn cancel_export_without_a_running_job_is_a_harmless_noop() {
    let mut shell = Shell::new_fixture().0;
    // 実行していない状態で Cancel が来ても panic しない(保持している
    // `Cancel` ハンドルが無い場合は何もしない、`update_export` の分岐)。
    let _ = shell.update(Message::Export(export_pane::Message::CancelExport));
    assert_eq!(shell.export_progress(), None);
}

#[test]
fn view_window_dispatches_to_the_export_window_when_the_ledger_has_it() {
    let mut shell = Shell::new_fixture().0;
    let _ = shell.update(Message::Export(export_pane::Message::ToggleExportDialog));
    let id = shell.export_window().expect("Export 窓が開いていない");
    // 型が組めることの確認(実際のピクセル比較は screenshot 器具の領分外 —
    // Settings 窓と同じ「単窓オフスクリーン合成の対象外」)。
    let _element = shell.view_window(id);
    assert_eq!(shell.window_title(id), "Export");
}

// ---------------------------------------------------------------------------
// 書き出し先の選択(2026-08-22 第2波)
// ---------------------------------------------------------------------------

#[test]
fn picking_output_path_sets_the_destination() {
    let (mut shell, fake) = shell_with_fake();
    assert_eq!(shell.export_out_path(), None, "既定で書き出し先が設定されている");

    let dir = motolii_testkit::tmp_dir("export-drive-pick-output");
    let path = dir.join("render.mp4");
    fake.set_export_path(path.clone());

    drive(&mut shell, Message::Export(export_pane::Message::PickOutputPath));

    assert_eq!(
        shell.export_out_path(),
        Some(path.as_path()),
        "PickOutputPath が export_out_path を更新していない"
    );
}

#[test]
fn picking_output_path_does_nothing_when_the_dialog_is_cancelled() {
    let (mut shell, _fake) = shell_with_fake();
    // `_fake.export_path_response` は既定で `None`(キャンセル相当)。

    drive(&mut shell, Message::Export(export_pane::Message::PickOutputPath));

    assert_eq!(
        shell.export_out_path(),
        None,
        "キャンセルしたのに export_out_path が設定されている"
    );
}

#[test]
fn picking_output_path_again_replaces_the_previous_choice() {
    let (mut shell, fake) = shell_with_fake();
    let dir = motolii_testkit::tmp_dir("export-drive-pick-output-replace");
    let first = dir.join("first.mp4");
    let second = dir.join("second.mp4");

    fake.set_export_path(first.clone());
    drive(&mut shell, Message::Export(export_pane::Message::PickOutputPath));
    assert_eq!(shell.export_out_path(), Some(first.as_path()));

    fake.set_export_path(second.clone());
    drive(&mut shell, Message::Export(export_pane::Message::PickOutputPath));
    assert_eq!(
        shell.export_out_path(),
        Some(second.as_path()),
        "2回目の選択が1回目の path を上書きしていない"
    );
}

// ---------------------------------------------------------------------------
// 音声 mux(GOALS M9「音声mux込み」、C-3 波)
// ---------------------------------------------------------------------------

/// **機械で示す**: 音声を持つ media layer がある project を書き出すと、
/// 出来た mp4 に実際の音声トラックが乗っている(`motolii_media::probe_audio`
/// が音声ストリームを見つけられる)。`export_ops::run_export_job_for_test`
/// (production の `start_export` が背景スレッドで呼ぶのと同じ
/// `run_export_job` — `Task`/`iced::stream::channel` の非同期機構は経由しない、
/// `export_ops.rs` module doc 参照)を直接呼ぶ。
#[test]
fn exporting_a_project_with_a_media_layer_mux_es_its_audio_track() {
    use motolii_shell::export_ops;
    use motolii_store::{
        Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming,
    };
    use std::process::Command;

    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }

    let dir = motolii_testkit::tmp_dir("export-drive-audio-mux");
    let source = dir.join("source-with-audio.mp4");
    let out = dir.join("out.mp4");

    // 64x64・30fps・1秒の映像 + 440Hz の正弦波1本(video+audioが同じ file)。
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=orange:s=64x64:d=1:r=30",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=1",
            "-pix_fmt",
            "yuv420p",
            "-c:v",
            "libx264",
            "-c:a",
            "aac",
            "-shortest",
        ])
        .arg(&source)
        .status()
        .expect("ffmpeg 起動に失敗");
    assert!(status.success(), "音声付き fixture の生成に失敗");

    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 30,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Media {
                path: source.to_string_lossy().into_owned(),
                fingerprint: None,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 30),
        },
    })
    .unwrap();

    let outcome = export_ops::run_export_job_for_test(&doc, &out, false, 0, 30)
        .expect("音声つき project の書き出しが失敗した");
    assert!(outcome.audio_muxed, "音声ありの project なのに mux していない");
    assert!(out.exists(), "書き出し先に file が無い");

    let audio = motolii_media::probe_audio(&out)
        .expect("書き出した mp4 に音声トラックが無い(mux が実際には呼ばれていない)");
    assert_eq!(audio.codec_name, "aac", "muxed audio codec が想定と違う: {audio:?}");

    std::fs::remove_dir_all(&dir).ok();
}

/// 音声を持つ layer が無い project は mux を呼ばない(無駄な ffmpeg 起動を
/// しない、`export_ops.rs` module doc「音声 mux」節の分岐)。
#[test]
fn exporting_a_project_without_audio_does_not_mux() {
    use motolii_shell::export_ops;
    use motolii_store::{Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming};

    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }

    let dir = motolii_testkit::tmp_dir("export-drive-no-audio");
    let out = dir.join("silent.mp4");

    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 32,
        height: 32,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 10,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::Solid {
                rgba: [255, 255, 255, 255],
                width: 32,
                height: 32,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 10),
        },
    })
    .unwrap();

    let outcome = export_ops::run_export_job_for_test(&doc, &out, false, 0, 10)
        .expect("無音声 project の書き出しが失敗した");
    assert!(!outcome.audio_muxed, "音声の無い project なのに mux したことになっている");
    assert!(out.exists());

    std::fs::remove_dir_all(&dir).ok();
}
