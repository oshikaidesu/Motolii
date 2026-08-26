//! Stage の失敗報告 — **黙殺しない**(M-2 outcome 4)。
//!
//! 2つの層で審判する。
//!
//! 1. `Shell::update` の層: [`Message::StageReported`] が transcript(帯 /
//!    `--status-log`)へ写り、**journal には載らない**(intent ではない)
//! 2. 島の層(GPU): 壊れた評価済みフレームを差すと、島は黙って捨てずに
//!    mailbox → `RedrawRequested` → `Message::StageReported` の道で言う
//!
//! この file は他の GPU テストと **process を分けてある** — 評価済みフレームの席
//! (`install_probe_frame`)は process 全体で1つなので、壊れた絵を差すテストと
//! 正しい絵を差すテスト(`stage_island_pixels.rs`)は同居できない。

mod common;

use common::redraw;
use motolii_shell_iced::stage_island;
use motolii_shell_iced::{Message, ScriptedPrompts, Shell};

/// 帯に出る。journal には載らない。replay は Stage の故障まで再現しない。
#[test]
fn a_stage_report_lands_in_the_transcript_but_never_the_journal() {
    let mut shell = Shell::new(ScriptedPrompts::default());
    let outcome = shell.update(Message::StageReported(vec![
        "stage: だめでした".to_owned(),
        "stage: これもだめでした".to_owned(),
    ]));

    assert_eq!(outcome, motolii_shell_iced::Outcome::Stay);
    assert_eq!(
        shell.reports(),
        vec![
            "stage: だめでした".to_owned(),
            "stage: これもだめでした".to_owned()
        ],
        "Stage の報告は transcript に**全部**残る"
    );
    assert_eq!(
        shell.latest_report().as_deref(),
        Some("stage: これもだめでした"),
        "帯は最新の一言を映す"
    );
    assert_eq!(
        shell.intent_count(),
        0,
        "Stage の報告は intent ではない — journal に載せると replay が嘘になる"
    );
}

/// GPU 側: 壊れた評価済みフレーム(寸法と画素数が合わない)は黙って落ちず、
/// `Message::StageReported` になって出てくる。
#[test]
fn a_broken_probe_frame_is_reported_not_swallowed() {
    let Some(()) = common::gpu_or_skip() else {
        return;
    };
    stage_island::install_rerun_device_floor();

    // 4x4 のつもりで 3 画素ぶんしか無い — 席に着けない絵。
    stage_island::install_probe_frame(4, 4, vec![0xFF; 12]);

    let island = stage_island::StageIsland {
        composition_aspect: Some(1.0),
        document: None,
        playhead: 0.0,
        grab_probe: None,
    };
    let mut ui: iced_test::Simulator<'_, motolii_shell_iced::Message> =
        iced_test::Simulator::with_size(
            iced_test::core::Settings::default(),
            iced::Size::new(160.0, 160.0),
            iced::Element::from(
                iced::widget::shader(island)
                    .width(iced::Fill)
                    .height(iced::Fill),
            ),
        );

    // 1周目の snapshot で prepare が走って mailbox に溜まり、
    // 次の RedrawRequested が Message として吐き出す。
    let _ = ui
        .snapshot(&iced::Theme::Dark)
        .expect("headless snapshot が撮れる");
    let _ = ui.simulate([redraw()]);

    let reported: Vec<String> = ui
        .into_messages()
        .filter_map(|message| match message {
            Message::StageReported(lines) => Some(lines),
            _ => None,
        })
        .flatten()
        .collect();
    assert!(
        reported.iter().any(|line| line.contains("フレーム")),
        "壊れた評価済みフレームが報告になっていない。実際の報告: {reported:?}"
    );
}
