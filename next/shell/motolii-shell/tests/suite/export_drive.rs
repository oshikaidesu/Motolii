//! 運転席 — 第6波 shell 結線(B09: Export 窓、`export_pane` crate doc
//! 「shell 結線」節)。Settings 窓(S2)と同じ第2窓の型
//! (`toggle_export_window`/`view_window`/`window_title`)・`ToggleExportDialog`・
//! 品質/範囲の選択状態・Cmd+E を検分する。
//!
//! **未検分(RETURN 参照)**: `Message::Export(Export)` の実行は
//! `motolii_export::export_with_cancel` を同期に1回呼ぶだけ(フレーム単位の
//! 進捗コールバックが export 側に無い)ため、`export_out_path` を設定する
//! 公開 API がまだ無い(path 選択は rfd 経由で次波・見送り、crate doc の
//! 逸脱どおり)。よってこの試験は窓の開閉・品質/範囲選択・キーマップまでを
//! 見る — 実行そのもの(`start_export`)はここでは駆動できない。

use motolii_shell::{export_pane, Message, Shell};

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
