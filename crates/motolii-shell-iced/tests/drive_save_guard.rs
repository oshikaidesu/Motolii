//! 保存と、未保存のまま座席を捨てさせない guard — red 先行。
//!
//! egui shell の意味をそのまま移す(移植であって再発明ではない):
//!
//! - `Cmd+S` は `UiIntent::SaveProject` になり、成否は帯が言う
//! - 帯は project 名を出し、未保存なら `● {name} — unsaved`
//! - 未保存のまま New / Open / 窓を閉じる をやると3択(Save / Discard / Cancel)。
//!   判断は `motolii_ui::blitz_shell::decide_unsaved` **そのもの**を通す —
//!   2つ目の判断を書かない
//!
//! 訊き手も egui shell と**同じ trait**(`blitz_shell::ShellPrompts`)で、
//! 台本(`ScriptedPrompts`)は native dialog を一切開かない。

mod common;

use std::path::PathBuf;

use common::{close_requested, command_key, drain, feed, file_dropped, press, redraw};
use motolii_shell_iced::{view, Outcome, ScriptedPrompts, Shell};
use motolii_ui::blitz_shell::{UiIntent, UnsavedChoice};

/// 未保存の編集を1つ載せた殻と、その周りの道具立て。
struct Dirty {
    shell: Shell,
    /// いま座っている project(New で作った物)。
    seated: PathBuf,
    /// guard の向こう側で開く project(既に在る物)。
    next: PathBuf,
    /// 汚すために置いた素材。journal の照合に使う。
    clip: PathBuf,
}

/// 台本つきで座席を1つ作り、**未保存の編集を1つ載せる**。
///
/// 汚し方は製品の道そのもの(OS ドロップ = `UiIntent::AdmitPaths`)。
/// テストだけの裏口で writer を触らない。
fn seated_and_dirty(name: &str, unsaved_choice: Option<UnsavedChoice>) -> Option<Dirty> {
    if !motolii_testkit::ffmpeg_or_skip() {
        return None;
    }
    let dir = motolii_testkit::tmp_dir(name);
    let seated = dir.join("fresh.json");
    let next = dir.join("second.json");
    motolii_ui::blitz_shell::create_project_file(&next).expect("guard の向こう側の project を作る");

    let mut shell = Shell::new(ScriptedPrompts {
        new_project_path: Some(seated.clone()),
        open_project_path: Some(next.clone()),
        unsaved_choice,
        ..ScriptedPrompts::default()
    });

    let pressed = press(iced_test::simulator(view(&shell)), view::NEW_PROJECT);
    drain(&mut shell, pressed);
    assert!(shell.is_seated(), "台本のパスへ座れていない");

    let clip = common::starter_media_dir().join("starter-clip.mp4");
    let dropped = feed(
        iced_test::simulator(view(&shell)),
        [file_dropped(&clip), redraw()],
    );
    drain(&mut shell, dropped);
    assert!(
        shell.is_dirty(),
        "素材を1本置いたのに未保存になっていない。帯: {:?}",
        shell.latest_report()
    );
    Some(Dirty {
        shell,
        seated,
        next,
        clip,
    })
}

/// `Cmd+S` が保存を通し、帯の ● が落ちる。
#[test]
fn cmd_s_saves_and_the_band_drops_the_unsaved_mark() {
    let Some(Dirty {
        mut shell, seated, ..
    }) = seated_and_dirty("iced_save_cmd_s", None)
    else {
        return;
    };
    let name = "fresh.json";

    // 未保存のあいだは ● 付きで名乗る。**絵まで見て**言う。
    {
        let mut before = iced_test::simulator(view(&shell));
        assert!(
            before.find(view::unsaved_label(name).as_str()).is_ok(),
            "未保存の印(● … — unsaved)が帯に出ていない"
        );
    }

    let typed = feed(iced_test::simulator(view(&shell)), command_key('s'));
    drain(&mut shell, typed);

    assert!(
        shell
            .intents()
            .iter()
            .any(|event| event.intent == UiIntent::SaveProject),
        "Cmd+S が SaveProject として journal に載っていない: {:?}",
        shell.intents()
    );
    assert!(!shell.is_dirty(), "保存したのに未保存のまま");
    assert!(
        shell
            .latest_report()
            .is_some_and(|report| report.contains("saved") && report.contains("fresh.json")),
        "保存の成立は帯が名指しで言う。実際: {:?}",
        shell.latest_report()
    );

    let mut after = iced_test::simulator(view(&shell));
    assert!(after.find(name).is_ok(), "保存後の帯が project 名を出さない");
    assert!(
        after.find(view::unsaved_label(name).as_str()).is_err(),
        "保存したのに ● が残っている"
    );
    assert!(seated.exists());
}

/// 座席なしの `Cmd+S` は Document を作らず、**次の一手を名指しで**案内する。
#[test]
fn saving_without_a_seat_says_what_to_do_first() {
    let mut shell = Shell::new(ScriptedPrompts::default());

    let typed = feed(iced_test::simulator(view(&shell)), command_key('s'));
    drain(&mut shell, typed);

    assert!(!shell.is_seated());
    assert!(
        shell
            .latest_report()
            .is_some_and(|report| report.contains("Cmd+N") && report.contains("Cmd+O")),
        "作る/開く口を名指しで案内する。実際: {:?}",
        shell.latest_report()
    );
}

/// 未保存のまま Open を選び、**Cancel** と答えたら座席は動かない。
#[test]
fn an_unsaved_open_that_is_cancelled_keeps_the_seat() {
    let Some(Dirty {
        mut shell, seated, ..
    }) = seated_and_dirty("iced_save_guard_cancel", Some(UnsavedChoice::Cancel))
    else {
        return;
    };
    let before = shell.intent_count();

    let typed = feed(iced_test::simulator(view(&shell)), command_key('o'));
    drain(&mut shell, typed);

    assert_eq!(
        shell.project_path().as_deref(),
        Some(seated.as_path()),
        "Cancel と答えたのに座席が変わった"
    );
    assert_eq!(
        shell.intent_count(),
        before,
        "やめた操作は journal に載らない: {:?}",
        shell.intents()
    );
    assert!(shell.is_dirty(), "Cancel は編集を捨てない");
}

/// 未保存のまま Open を選び、**Save** と答えたら、Open の**前に**保存が記録される。
#[test]
fn an_unsaved_open_saves_first_and_the_journal_shows_the_order() {
    let Some(Dirty {
        mut shell,
        seated,
        next,
        clip,
    }) = seated_and_dirty("iced_save_guard_save", Some(UnsavedChoice::Save))
    else {
        return;
    };

    let typed = feed(iced_test::simulator(view(&shell)), command_key('o'));
    drain(&mut shell, typed);

    let intents: Vec<UiIntent> = shell
        .intents()
        .into_iter()
        .map(|event| event.intent)
        .collect();
    assert_eq!(
        intents,
        vec![
            UiIntent::NewProject {
                path: seated.clone()
            },
            UiIntent::AdmitPaths { paths: vec![clip] },
            UiIntent::SaveProject,
            UiIntent::OpenProject { path: next.clone() },
        ],
        "「保存してから続行」は保存も利用者の行動として、Open の前に残る"
    );
    assert_eq!(shell.project_path().as_deref(), Some(next.as_path()));
    assert!(!shell.is_dirty(), "開き直した座席は最初から clean");
}

/// **Discard** と答えたら保存せずに続行する(journal に SaveProject は出ない)。
#[test]
fn an_unsaved_open_that_discards_does_not_save() {
    let Some(Dirty {
        mut shell, next, ..
    }) = seated_and_dirty("iced_save_guard_discard", Some(UnsavedChoice::Discard))
    else {
        return;
    };

    let typed = feed(iced_test::simulator(view(&shell)), command_key('o'));
    drain(&mut shell, typed);

    assert!(
        !shell
            .intents()
            .iter()
            .any(|event| event.intent == UiIntent::SaveProject),
        "捨てると答えたのに保存している: {:?}",
        shell.intents()
    );
    assert_eq!(shell.project_path().as_deref(), Some(next.as_path()));
}

/// 窓を閉じる要求も「未保存のまま座席を捨てる」操作 — Cancel なら閉じない。
#[test]
fn closing_with_unsaved_changes_can_be_cancelled() {
    let Some(Dirty { mut shell, .. }) =
        seated_and_dirty("iced_close_guard_cancel", Some(UnsavedChoice::Cancel))
    else {
        return;
    };

    let asked = feed(iced_test::simulator(view(&shell)), [close_requested()]);
    assert_eq!(
        drain(&mut shell, asked),
        Outcome::Stay,
        "Cancel と答えたのに窓が閉じようとしている"
    );
    assert!(shell.is_seated(), "留まったのに座席を失っている");
}

/// 捨てると答えたら閉じる。捨てる物が無い殻は訊かずに閉じる。
#[test]
fn closing_after_discarding_or_when_clean_lets_the_window_go() {
    let Some(Dirty { mut shell, .. }) =
        seated_and_dirty("iced_close_guard_discard", Some(UnsavedChoice::Discard))
    else {
        return;
    };

    let asked = feed(iced_test::simulator(view(&shell)), [close_requested()]);
    assert_eq!(drain(&mut shell, asked), Outcome::Close);

    // 座席なし(= 捨てるものが無い)は、台本が Cancel を返す設定でも閉じる。
    let mut empty = Shell::new(ScriptedPrompts {
        unsaved_choice: Some(UnsavedChoice::Cancel),
        ..ScriptedPrompts::default()
    });
    let asked = feed(iced_test::simulator(view(&empty)), [close_requested()]);
    assert_eq!(
        drain(&mut empty, asked),
        Outcome::Close,
        "捨てるものが無いのに訊いている"
    );
}
