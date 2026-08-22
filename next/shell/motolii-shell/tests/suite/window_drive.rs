//! 窓台帳(S1 daemon 骨格、裁定182/188)の drive。
//!
//! probe(`docs/reviews/2026-08-22-multiwindow-probe.md`)の設計どおり、
//! `Shell` は daemon の State として窓台帳(`main_window`)を持ち、
//! `view_window`/`window_title` の窓別 dispatcher を晒す。この drive は
//! **runtime を開かずに**台帳の記帳と dispatcher の形を検分する — `iced::
//! window::open` は Id を同期で採番する(Task を走らせなくても台帳が読める、
//! `Shell::boot` doc 参照)ことがこの headless 検分の前提。

use motolii_shell::{Message, Shell};

/// boot(daemon の製品入口)は main 窓を台帳へ先行記帳する。`Shell::new`
/// (運転席・試験の入口)は従来どおり窓を開かない — 台帳も空のまま。
#[test]
fn boot_records_the_main_window_in_the_ledger_but_new_does_not() {
    let (booted, _task) = Shell::boot();
    assert!(
        booted.main_window().is_some(),
        "boot が main 窓を台帳へ記帳していない"
    );

    let (headless, _task) = Shell::new();
    assert!(
        headless.main_window().is_none(),
        "Shell::new(窓を開かない入口)が main 窓を記帳してしまっている"
    );
}

/// `view_window` は main 窓の Id にも、台帳に無い Id(開閉の境目の1フレームで
/// 来うる — probe の fallback)にも、main の絵を返して落ちない。
#[test]
fn view_window_serves_the_main_view_for_the_main_id_and_unknown_ids() {
    let (shell, _task) = Shell::boot();
    let main_id = shell.main_window().expect("boot 済みなら台帳にある");

    // main の絵であることの検分: main 窓にしか無い題帯ラベル(pane_grid の
    // "Stage")が見つかる(`pane_band_drive.rs` と同じ手口)。
    let mut ui = iced_test::simulator(shell.view_window(main_id));
    ui.find("Stage")
        .expect("view_window(main) に Stage 題帯が無い — main の絵が出ていない");

    // 台帳に無い Id でも panic しない(絵は main と同じ)。
    let (unknown_id, _open) = iced::window::open(iced::window::Settings::default());
    let mut ui = iced_test::simulator(shell.view_window(unknown_id));
    ui.find("Stage")
        .expect("view_window(未知 Id) が main の絵へ fallback していない");
}

/// 窓 title の dispatcher: main 窓(と未知 Id)は従来の `Shell::title`
/// ("Motolii")のまま。
#[test]
fn the_main_window_title_is_unchanged() {
    let (shell, _task) = Shell::boot();
    let main_id = shell.main_window().expect("boot 済みなら台帳にある");
    assert_eq!(shell.window_title(main_id), shell.title());
}

/// main 窓が閉じても台帳の記帳自体は残る(exit Task は runtime 側でしか
/// 走らないので、headless で観測できるのは「台帳が壊れない」ことまで —
/// exit の配線そのものは `Message::WindowClosed` 腕のソースが正本)。
#[test]
fn closing_the_main_window_does_not_corrupt_the_ledger() {
    let (mut shell, _task) = Shell::boot();
    let main_id = shell.main_window().expect("boot 済みなら台帳にある");
    let _exit_task = shell.update(Message::WindowClosed(main_id));
    assert_eq!(
        shell.main_window(),
        Some(main_id),
        "WindowClosed(main) が台帳を壊した"
    );
}
