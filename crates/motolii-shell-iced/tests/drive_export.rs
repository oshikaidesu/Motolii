//! 書き出しの開始とキャンセル — red 先行。
//!
//! 実体は既存の `motolii_ui::export_seat`(別 thread・headless GpuCtx)をそのまま
//! 共用する。iced 殻が足すのは「押す口」と「帯の見え方」だけで、**2つ目の
//! 書き出し経路を作らない**。
//!
//! 帯の言い方も egui shell と同じ:
//! - 実行中は `Exporting… {経過}s` と `Cancel`、Export ボタンは**消える**(二重起動なし)
//! - 終わり方(完了 / キャンセル / 失敗)は同じ帯の一言になる
//!
//! 進捗の刻みは host で違う。egui は `request_repaint_after(200ms)` で自分を起こし、
//! iced は書き出し中だけ `window::frames()` を購読して `Message::ExportPolled` を
//! 受ける。**購読は `Simulator` の外**なので、ここでは同じ Message を直に流す
//! (窓での刻みは手動確認事項)。

mod common;

use common::{drain, press};
use motolii_shell_iced::{view, Message, ScriptedPrompts, Shell};
use motolii_ui::blitz_shell::UiIntent;

/// 座席1つ・**中身が1本**・書き出し先の台本を持った殻。
///
/// 空の project は `export failed: document has no video source clip` で即座に
/// 終わってしまい、「走っている書き出し」を観測できない。ここで実 clip を1本
/// 置いておくと、開始も実行中の帯もキャンセルも**本物の走行**の上で見られる。
fn seated_with_a_clip(name: &str) -> Option<(Shell, std::path::PathBuf)> {
    if !motolii_testkit::ffmpeg_or_skip() {
        return None;
    }
    // 書き出し worker は headless GpuCtx を自分で立てる。無い環境では
    // 「始まったが失敗した」しか観測できないので、gpu_or_skip と同じ扱いで降りる。
    // 借りた ctx はここで返す(worker は自分のを立てる)。
    drop(motolii_testkit::gpu_or_skip()?);
    let dir = motolii_testkit::tmp_dir(name);
    let project = dir.join("fresh.json");
    let output = dir.join("out.mp4");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project),
        export_path: Some(output.clone()),
        ..ScriptedPrompts::default()
    });
    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);
    assert!(shell.is_seated());

    let clip = common::starter_media_dir().join("starter-clip.mp4");
    let dropped = common::feed(
        iced_test::simulator(view(&shell)),
        [common::file_dropped(&clip), common::redraw()],
    );
    drain(&mut shell, dropped);
    assert_eq!(shell.track_item_count(), 1, "書き出す中身が立っていない");
    Some((shell, output))
}

/// 返事が来るまで `ExportPolled` を流す。窓では `window::frames()` の購読が
/// 同じ Message を運ぶ。
fn poll_until_finished(shell: &mut Shell) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while shell.export_running() {
        assert!(
            std::time::Instant::now() < deadline,
            "書き出しが 120s で返事をしない。帯: {:?}",
            shell.latest_report()
        );
        shell.update(Message::ExportPolled);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// スタート画面に Export は無い(座席が無ければ書き出す物が無い)。
/// **触れそうで触れない物を置かない**(Q0)。
#[test]
fn the_start_screen_offers_no_export() {
    let shell = Shell::new(ScriptedPrompts::default());
    let mut ui = iced_test::simulator(view(&shell));
    assert!(
        ui.find(view::EXPORT).is_err(),
        "座席が無いのに Export が立っている"
    );
}

/// Export を押すと書き出しが始まり、帯が実行中と Cancel を出す。
/// 実行中は Export が消える = 二重起動しようがない。
#[test]
fn pressing_export_starts_a_run_and_the_band_offers_cancel() {
    let Some((mut shell, output)) = seated_with_a_clip("iced_export_begin") else {
        return;
    };

    let pressed = press(iced_test::simulator(view(&shell)), view::EXPORT);
    drain(&mut shell, pressed);

    assert!(
        shell
            .intents()
            .iter()
            .any(|event| event.intent
                == UiIntent::BeginExport {
                    output: output.clone()
                }),
        "dialog の答え(保存先)ごと BeginExport が journal に載る: {:?}",
        shell.intents()
    );
    assert!(shell.export_running(), "書き出しが走っていない");
    assert!(
        shell
            .reports()
            .iter()
            .any(|line| line == &format!("exporting {}", output.display())),
        "開始は帯が名指しで言う。実際: {:?}",
        shell.reports()
    );

    {
        // `Simulator` は `view(&shell)` の Element を借りている。殻を可変で触る前に
        // 必ず消費し切らなければならない — 型が順番を強制する。
        let mut ui = iced_test::simulator(view(&shell));
        assert!(
            ui.find(view::exporting_label(0).as_str()).is_ok()
                || ui.find(view::exporting_label(1).as_str()).is_ok(),
            "実行中の帯に経過秒が出ていない"
        );
        assert!(
            ui.find(view::CANCEL_EXPORT).is_ok(),
            "実行中なのに Cancel が無い"
        );
        assert!(
            ui.find(view::EXPORT).is_err(),
            "実行中に Export が残っている(二重起動の口)"
        );
    }

    // thread を置き去りにしない。
    drain(&mut shell, vec![Message::CancelExportPressed]);
    poll_until_finished(&mut shell);
}

/// Cancel は殻から発行でき、返事が来たら帯が終わり方を1行で言い、
/// Export がまた押せるようになる。
#[test]
fn cancel_is_dispatched_and_the_band_closes_the_run() {
    let Some((mut shell, output)) = seated_with_a_clip("iced_export_cancel") else {
        return;
    };

    let pressed = press(iced_test::simulator(view(&shell)), view::EXPORT);
    drain(&mut shell, pressed);
    assert!(shell.export_running());

    let pressed = press(iced_test::simulator(view(&shell)), view::CANCEL_EXPORT);
    drain(&mut shell, pressed);
    assert!(
        shell
            .intents()
            .iter()
            .any(|event| event.intent == UiIntent::CancelExport),
        "Cancel が journal に載っていない: {:?}",
        shell.intents()
    );

    poll_until_finished(&mut shell);

    // 終わり方(`export cancelled — … was removed` / `Exported … (Nf)`)のどちらに
    // なるかは、キャンセルが frame loop に間に合うかの競争なので**固定しない**。
    // 固定するのは「1行で終わりを言い、書き出し先を名指しし、失敗ではない」こと。
    let report = shell.latest_report().unwrap_or_default();
    assert!(
        report.contains(&output.display().to_string()),
        "終わり方は書き出し先を名指しで言う。実際: {report:?}"
    );
    assert!(
        !report.contains("failed"),
        "止めただけの書き出しが失敗として言われている。実際: {report:?}"
    );

    let mut ui = iced_test::simulator(view(&shell));
    assert!(
        ui.find(view::EXPORT).is_ok(),
        "走り終わったのに Export が戻ってこない"
    );
    assert!(
        ui.find(view::CANCEL_EXPORT).is_err(),
        "走っていないのに Cancel が残っている"
    );
}

/// 保存先を訊いて断られたら**何も記録しない**(訊いただけの操作は intent ではない)。
#[test]
fn an_export_dialog_that_is_cancelled_leaves_no_trace() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("iced_export_no_answer");
    let project = dir.join("fresh.json");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project),
        // export_path は答えないまま(= dialog を閉じた)。
        ..ScriptedPrompts::default()
    });
    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);
    let before = shell.intent_count();

    let pressed = press(iced_test::simulator(view(&shell)), view::EXPORT);
    drain(&mut shell, pressed);

    assert_eq!(
        shell.intent_count(),
        before,
        "答えなかった dialog で intent が生えている: {:?}",
        shell.intents()
    );
    assert!(!shell.export_running());
}
