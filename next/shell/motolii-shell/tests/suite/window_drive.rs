//! 窓台帳(S1 daemon 骨格、裁定182/188)の drive。
//!
//! probe(`docs/reviews/2026-08-22-multiwindow-probe.md`)の設計どおり、
//! `Shell` は daemon の State として窓台帳(`main_window`)を持ち、
//! `view_window`/`window_title` の窓別 dispatcher を晒す。この drive は
//! **runtime を開かずに**台帳の記帳と dispatcher の形を検分する — `iced::
//! window::open` は Id を同期で採番する(Task を走らせなくても台帳が読める、
//! `Shell::boot` doc 参照)ことがこの headless 検分の前提。

use motolii_shell::{Message, Shell};

use crate::file_drive::{drive, FakeDialogs};

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

/// OS の CloseRequested は main 窓を直ちに消さず、dirty なら確認結果を待つ。
/// fake dialog を使うことで、メニュー Quit だけでなく赤信号ボタンの経路も
/// 同じ `confirm_discard` 契約を通ることを headless に固定する。
#[test]
fn dirty_main_window_close_requests_discard_confirmation() {
    let fake = FakeDialogs::default();
    let (mut shell, _boot_task) = Shell::boot_with_dialogs(Box::new(fake.clone()));
    let main_id = shell.main_window().expect("boot 済みなら main 窓がある");
    drive(&mut shell, Message::AddLayer);
    assert!(shell.is_project_dirty(), "変更前から dirty になっている");

    // 既定応答(false)では、赤信号を押しても編集内容を捨てない。
    drive(&mut shell, Message::WindowCloseRequested(main_id));
    assert_eq!(fake.confirm_discard_calls(), 1, "OS close が確認を通っていない");
    assert!(shell.is_project_dirty(), "キャンセルしたのに dirty が消えた");

    // 許可(true)後の `iced::exit()` は runtime の終了Actionなので、ここでは
    // Task drainの対象にせず、`document_io.rs:545-548`のソース経路で確認する。
    // このテストは「拒否しても窓を維持する」という利用者保護の検収を担当する。
}

// ---------------------------------------------------------------------------
// S2: Settings の窓移住(浮かし第1号、裁定182/188)
// ---------------------------------------------------------------------------

fn toggle_settings(shell: &mut Shell) -> iced::Task<Message> {
    shell.update(Message::Settings(
        motolii_shell::settings_pane::sections::Message::Legacy(
            motolii_shell::settings_pane::Message::ToggleSettingsPanel,
        ),
    ))
}

/// **窓台帳の open/close/再open の状態遷移 oracle**(発注の落ちるテスト)。
/// `ToggleSettingsPanel` の意味は「レイアウト分岐」→「窓 open/close」へ
/// 変わった(probe §Q3)。台帳は同期で先行記帳/抹消される(`window::open` の
/// Id 同期採番 — headless でも遷移が読める)。再open は**新しい窓**
/// (別 Id)— 閉じた窓の Id は再利用されない。
#[test]
fn settings_toggle_walks_the_ledger_through_open_close_and_reopen() {
    let (mut shell, _task) = Shell::boot();
    assert!(
        shell.settings_window().is_none(),
        "起動直後から Settings 窓が記帳されている"
    );

    let _open = toggle_settings(&mut shell);
    let first = shell
        .settings_window()
        .expect("トグルで Settings 窓が台帳に載らない");

    let _close = toggle_settings(&mut shell);
    assert!(
        shell.settings_window().is_none(),
        "2度目のトグルで Settings 窓が台帳から消えない"
    );

    let _reopen = toggle_settings(&mut shell);
    let second = shell
        .settings_window()
        .expect("再オープンで Settings 窓が台帳に載らない");
    assert_ne!(first, second, "再オープンが古い窓 Id を使い回している");
}

/// Settings 窓の絵と title。中身は既存 `settings_pane::view` そのまま
/// (probe §Q4 — 投影のみ受ける純関数)なので、section header "SETTINGS" が
/// 窓の絵に実在する。main の絵からは Settings ストリップが**退去**した
/// (旧「header 直下の全幅ストリップ」— Q0: 閉じた道具は木に現れない、の
/// 窓版)。
#[test]
fn the_settings_window_serves_the_settings_view_and_the_main_view_drops_the_strip() {
    let (mut shell, _task) = Shell::boot();
    let _open = toggle_settings(&mut shell);
    let id = shell.settings_window().expect("台帳にある");

    assert_eq!(shell.window_title(id), "Settings", "Settings 窓の title が違う");

    let mut ui = iced_test::simulator(shell.view_window(id));
    ui.find("SETTINGS")
        .expect("Settings 窓に settings_pane::view の絵が出ていない");

    // main の絵に Settings の中身が残っていない(退去の直接検分 —
    // "UI Scale (%)" は settings_pane にしか無いラベル)。
    let mut ui = iced_test::simulator(shell.view());
    assert!(
        ui.find("UI Scale (%)").is_err(),
        "Settings 窓が開いているのに main の絵へも Settings が出ている(二重表示)"
    );
}

/// 閉→再開で状態が保持される(probe 実測 `state_persists=true` の製品版)。
/// 下書き(`background_draft`)は `Shell` に住む — 窓を閉じても失われない
/// (probe §Q3「窓を閉じても何も失われない」)。閉じて開き直した後に
/// Submit すると、閉じる前に打った下書きがそのまま確定する。
#[test]
fn a_background_draft_survives_closing_and_reopening_the_settings_window() {
    use motolii_shell::settings_pane::{self, BackgroundChannel};

    let (mut shell, _task) = Shell::boot();
    let _open = toggle_settings(&mut shell);
    let _ = shell.update(Message::Settings(settings_pane::sections::Message::Legacy(settings_pane::Message::BackgroundChannelInput(
        BackgroundChannel::A,
        "0".to_owned(),
    ))));

    let _close = toggle_settings(&mut shell);
    let _reopen = toggle_settings(&mut shell);

    let _ = shell.update(Message::Settings(settings_pane::sections::Message::Legacy(settings_pane::Message::BackgroundChannelSubmit(
        BackgroundChannel::A,
    ))));
    let background = shell.composition().expect("comp がある").background;
    assert_eq!(
        background,
        [0.0, 0.0, 0.0, 0.0],
        "閉→再開で background 下書きが失われた(窓を跨いで状態が保持されるはず)"
    );
}

/// OS の閉じるボタン経路(`Message::WindowClosed` — `close_events` 購読)でも
/// 台帳から抹消される。main の台帳は無傷(Settings 閉で main は生き続ける —
/// probe 実測 `main_alive_after_settings_close=true` の製品版)。
#[test]
fn an_os_close_of_the_settings_window_clears_only_the_settings_ledger() {
    let (mut shell, _task) = Shell::boot();
    let main_id = shell.main_window().expect("台帳にある");
    let _open = toggle_settings(&mut shell);
    let id = shell.settings_window().expect("台帳にある");

    let _ = shell.update(Message::WindowClosed(id));
    assert!(
        shell.settings_window().is_none(),
        "OS 閉経路で Settings 窓が台帳から消えない"
    );
    assert_eq!(shell.main_window(), Some(main_id), "main の台帳まで消えた");

    // 閉じた後の view_window(旧 Id)は main へ fallback(開閉境目の1フレーム
    // — S1 の fallback 検分と同じ形)。
    let mut ui = iced_test::simulator(shell.view_window(id));
    ui.find("Stage").expect("閉窓 Id が main の絵へ fallback していない");
}
