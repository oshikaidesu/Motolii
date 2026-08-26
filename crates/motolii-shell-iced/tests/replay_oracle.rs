//! replay oracle(常設)— red 先行。
//!
//! egui 版 `blitz_shell/drive_tests.rs` の
//! `a_recorded_session_replays_into_the_same_shell_state` の対応物である。
//! 駆動器だけが違い(kittest → `iced_test::Simulator`)、**審判は同じ型の同じ問い**:
//! 記録した intent 列を窓の無い新しい `ShellGateway` へ再実行すると、座席・
//! revision・track item 数・言われたことが一致するか。
//!
//! 前提は「初期状態が同じ」ことなので、駆動が作った project ファイルは消してから
//! replay する(`NewProject` は既にあるファイルを踏まないのが決定済みの意味であり、
//! その意味ごと再現するために world を初期状態へ戻す)。

mod common;

use common::{
    command_key, double_click, drain, feed, file_dropped, press, redraw, starter_media_dir,
};
use motolii_shell_iced::inspector_pane::InspectorEvent;
use motolii_shell_iced::widgets::ScrubEvent;
use motolii_shell_iced::{view, Message, ScriptedPrompts, Shell};
use motolii_ui::blitz_shell::{ShellGateway, UiEditParam, UiIntent, UiItemFlag};

#[test]
fn a_recorded_session_replays_into_the_same_shell_state() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("iced_intent_replay");
    let project = dir.join("fresh.json");
    let clip = starter_media_dir().join("starter-clip.mp4");
    let still = starter_media_dir().join("starter-still.png");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project.clone()),
        ..ScriptedPrompts::default()
    });

    // 運転席で1セッション回す: 作る → 落とす → もう1本落とす →
    // Browser のカードをダブルクリックで置く(M-4a) → 保存する。
    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);
    let dropped = feed(
        iced_test::simulator(view(&shell)),
        [file_dropped(&clip), redraw()],
    );
    drain(&mut shell, dropped);
    let dropped = feed(
        iced_test::simulator(view(&shell)),
        [file_dropped(&still), redraw()],
    );
    drain(&mut shell, dropped);
    // Browser 経由の配置も**同じ AdmitPaths** として台本に載る(合流点が1本の証拠)。
    let placed = double_click(iced_test::simulator(view(&shell)), "starter-clip.mp4");
    drain(&mut shell, placed);
    let typed = feed(iced_test::simulator(view(&shell)), command_key('s'));
    drain(&mut shell, typed);

    // M-4b: Inspector の編集も同じ台本に載る。選ぶ → M → ◇(Position)→
    // Opacity の確定、を1列で。
    let layer = shell
        .document()
        .expect("seated")
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .map(|item| match item {
            motolii_doc::TrackItem::Clip(clip) => clip.envelope.layer_id.get(),
            motolii_doc::TrackItem::Group(group) => group.envelope.layer_id.get(),
        })
        .next()
        .expect("落とした素材の clip");
    drain(&mut shell, vec![Message::LayerSelected(layer)]);
    let pressed = press(iced_test::simulator(view(&shell)), "M");
    drain(&mut shell, pressed);
    let pressed = press(iced_test::simulator(view(&shell)), "\u{25c7}");
    drain(&mut shell, pressed);
    drain(
        &mut shell,
        vec![Message::Inspector(InspectorEvent::Scrub {
            param: UiEditParam::Opacity,
            component: 0,
            event: ScrubEvent::Committed(0.25),
        })],
    );
    let keyed_components = match shell.inspector() {
        motolii_shell_iced::InspectorSeat::Ready(model) => model
            .transform_row(UiEditParam::Position)
            .expect("Position 行")
            .components
            .clone(),
        other => panic!("Inspector が座っていない: {other:?}"),
    };

    let intents: Vec<UiIntent> = shell
        .intents()
        .into_iter()
        .map(|event| event.intent)
        .collect();
    let driven_path = shell.project_path();
    let driven_revision = shell.revision();
    let driven_items = shell.track_item_count();
    let driven_reports = shell.reports();
    assert_eq!(
        intents,
        vec![
            UiIntent::NewProject {
                path: project.clone()
            },
            UiIntent::AdmitPaths {
                paths: vec![clip.clone()]
            },
            UiIntent::AdmitPaths {
                paths: vec![still.clone()]
            },
            // Browser のダブルクリック。**intent は OS ドロップと同じ1種類**なので、
            // 台本を読む側に「Browser 用の別レール」は要らない。
            UiIntent::AdmitPaths {
                paths: vec![clip.clone()]
            },
            UiIntent::SaveProject,
            UiIntent::SelectLayer {
                layer,
                additive: false
            },
            UiIntent::ToggleItemFlag {
                layer,
                flag: UiItemFlag::Mute,
            },
            UiIntent::KeyParamAtPlayhead {
                layer,
                param: UiEditParam::Position,
                components: keyed_components,
            },
            UiIntent::SetParamComponent {
                layer,
                param: UiEditParam::Opacity,
                component: 0,
                value: 0.25,
            },
            UiIntent::EndParamEdit,
        ],
        "運転席の1セッションが intent 列として素直に読めない"
    );
    assert!(driven_items > 0, "replay で比べる中身がまず要る");

    // 座席は project ファイルの OS lock を握っている。replay が同じ project を
    // 開けるよう、先に殻ごと落とす(lock を返す)。
    drop(shell);
    // world を初期状態へ戻す: NewProject が作ったファイルだけを消す。
    std::fs::remove_file(&project).expect("駆動が作った project を消して初期状態へ戻す");

    let replayed = ShellGateway::replay(&intents);

    assert_eq!(
        replayed.project().map(|seat| seat.path().to_path_buf()),
        driven_path,
        "replay は同じ project へ座り直す"
    );
    assert_eq!(
        replayed.revision(),
        driven_revision,
        "writer 世代が一致する(同じ command 列が通った)"
    );
    assert_eq!(
        replayed.track_item_count(),
        driven_items,
        "帯だけ揃って Document が違うのは不合格"
    );

    // 編集の中身も一致する: M / Position キー / Opacity が replay 側の Document に立つ。
    let replayed_doc = replayed
        .project()
        .expect("replay は同じ座席に着く")
        .snapshot();
    let envelope = replayed_doc
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .find_map(|item| match item {
            motolii_doc::TrackItem::Clip(clip) if clip.envelope.layer_id.get() == layer => {
                Some(&clip.envelope)
            }
            _ => None,
        })
        .expect("同じ clip が居る");
    assert!(!envelope.visible, "M(mute)が replay で再現されない");
    assert!(
        matches!(
            envelope.transform.position,
            motolii_doc::DocParam::Keyframes(_)
        ),
        "Position のキーが replay で再現されない"
    );

    // 結果のログの要点: replay が言ったことは、駆動が言ったことの中に
    // **同じ順で**現れる。
    let replayed_reports: Vec<String> = replayed
        .transcript()
        .entries()
        .into_iter()
        .map(|event| event.text)
        .collect();
    assert!(!replayed_reports.is_empty(), "replay も言うべきことを言う");
    let mut driven = driven_reports.iter();
    for line in &replayed_reports {
        assert!(
            driven.any(|seen| seen == line),
            "replay の一言 {line:?} が駆動セッションの台帳に同じ順で無い。\n\
             駆動: {driven_reports:#?}\nreplay: {replayed_reports:#?}"
        );
    }
}

/// Stage 島つきの台本(M-2)。Stage が帯へ喋っても **intent 列は汚れず**、
/// replay は同じ Document へ着く。
///
/// M-2 の Stage 島は新しい `UiIntent` を持たない(ギズモの意味論は M-2 後)。
/// つまり「Stage 系 intent の台本」とは、Stage pane が立っている座席で
/// 従来 intent(NewProject / AdmitPaths)が回り、Stage の報告
/// (`Message::StageReported` — intent では**ない**)が混ざっても replay が
/// 崩れないことの審判である。ここが緑である限り、`--intent-log` は Stage の
/// 故障や報告に影響されない。
#[test]
fn a_session_with_stage_reports_replays_from_intents_alone() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("iced_stage_replay");
    let project = dir.join("fresh.json");
    let clip = starter_media_dir().join("starter-clip.mp4");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project.clone()),
        ..ScriptedPrompts::default()
    });

    // 作る → Stage が一言喋る → 落とす → また喋る。
    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);
    drain(
        &mut shell,
        vec![motolii_shell_iced::Message::StageReported(vec![
            "stage: 初期化しました(テスト台本)".to_owned(),
        ])],
    );
    let dropped = feed(
        iced_test::simulator(view(&shell)),
        [file_dropped(&clip), redraw()],
    );
    drain(&mut shell, dropped);
    drain(
        &mut shell,
        vec![motolii_shell_iced::Message::StageReported(vec![
            "stage: 合成フレームを載せられない(テスト台本)".to_owned(),
        ])],
    );

    let intents: Vec<UiIntent> = shell
        .intents()
        .into_iter()
        .map(|event| event.intent)
        .collect();
    assert_eq!(
        intents,
        vec![
            UiIntent::NewProject {
                path: project.clone()
            },
            UiIntent::AdmitPaths {
                paths: vec![clip.clone()]
            },
        ],
        "Stage の報告が intent 列へ混ざってはならない"
    );
    assert!(
        shell
            .reports()
            .iter()
            .any(|line| line.contains("合成フレーム")),
        "Stage の報告は transcript には残る"
    );

    let driven_revision = shell.revision();
    let driven_items = shell.track_item_count();
    drop(shell);
    std::fs::remove_file(&project).expect("駆動が作った project を消して初期状態へ戻す");

    let replayed = ShellGateway::replay(&intents);
    assert_eq!(
        replayed.revision(),
        driven_revision,
        "writer 世代が一致する"
    );
    assert_eq!(
        replayed.track_item_count(),
        driven_items,
        "Stage の報告抜きの intent 列だけで同じ Document へ着く"
    );
}

/// 統合第2弾: Timeline の行選択(M-3)が Inspector(M-4b)へ座る、までを1本の
/// 台本にする。Browser のカード選択(pane-local, journal に載らない)とは違い、
/// Timeline のクリックは Stage クリックと同じ `UiIntent::SelectLayer` 1種類へ
/// 合流する — Inspector 側の呼び出し(`Message::LayerSelected`)と全く同じ
/// intent 形なので、replay も同じ Document 選択で着く。
#[test]
fn timeline_row_pick_selects_the_same_layer_the_inspector_reads() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("iced_timeline_select_replay");
    let project = dir.join("fresh.json");
    let still = starter_media_dir().join("starter-still.png");
    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(project.clone()),
        ..ScriptedPrompts::default()
    });

    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);
    let dropped = feed(
        iced_test::simulator(view(&shell)),
        [file_dropped(&still), redraw()],
    );
    drain(&mut shell, dropped);

    let layer = shell
        .document()
        .expect("seated")
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .map(|item| match item {
            motolii_doc::TrackItem::Clip(clip) => clip.envelope.layer_id.get(),
            motolii_doc::TrackItem::Group(group) => group.envelope.layer_id.get(),
        })
        .next()
        .expect("落とした still の clip");

    // Inspector はまだ誰も映していない(座っただけ)。
    assert!(
        matches!(
            shell.inspector(),
            motolii_shell_iced::InspectorSeat::NoSelection
        ),
        "選択前の Inspector は空状態のまま(幽霊選択を出さない)"
    );

    // Timeline の行を1つ picked(pane-local semantics の release 相当)する。
    // 実体は canvas のヒット判定を経由する本物の `TimelineMsg` — 座標計算では
    // なく、production の `Shell::update` → `TimelinePane::plan` の写像そのもの
    // を通す(drive_timeline.rs の生ポインタ駆動とは別の審判)。
    drain(
        &mut shell,
        vec![Message::Timeline(
            motolii_shell_iced::timeline::TimelineMsg::RowPicked {
                layer: motolii_doc::LayerId::from_raw(layer),
                additive: false,
            },
        )],
    );

    let selected = match shell.inspector() {
        motolii_shell_iced::InspectorSeat::Ready(model) => model.layer_id,
        other => panic!("Timeline の行選択が Inspector へ座らない: {other:?}"),
    };
    assert_eq!(
        selected, layer,
        "Timeline が選んだ layer と Inspector が映す layer が一致しない"
    );

    let intents: Vec<UiIntent> = shell
        .intents()
        .into_iter()
        .map(|event| event.intent)
        .collect();
    assert_eq!(
        intents.last(),
        Some(&UiIntent::SelectLayer {
            layer,
            additive: false
        }),
        "Timeline のクリックは Stage/Inspector と同じ SelectLayer 1種類に合流する"
    );

    let driven_revision = shell.revision();
    let driven_items = shell.track_item_count();
    drop(shell);
    std::fs::remove_file(&project).expect("駆動が作った project を消して初期状態へ戻す");

    let replayed = ShellGateway::replay(&intents);
    assert_eq!(
        replayed.revision(),
        driven_revision,
        "writer 世代が一致する"
    );
    assert_eq!(
        replayed.track_item_count(),
        driven_items,
        "同じ Document へ着く"
    );
}
