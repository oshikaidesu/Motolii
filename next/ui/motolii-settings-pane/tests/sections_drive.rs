//! B12 第1切片の drive(発注書の受入条件を実 widget 木で見る):
//!
//! 1. **表示値と状態の一致** — comp の実値・ui_scale・キャッシュ実測値が
//!    そのまま欄に出る(下書きが有ればそちらが勝つ)。
//! 2. **Message 発火** — 数値欄への打鍵で `CompFieldInput`、Enter で
//!    `CompFieldSubmit`、旧項目(プリセット等)は `Legacy(..)` に包まれて
//!    出る。
//! 3. **飾り禁止** — 未結線(`preview_cache: None`)の間、PLAYBACK 節は
//!    見出しごと存在しない(「顔だけの設定」を1つも作らない)。
//!
//! 器具は inspector-pane `value_cell_legibility.rs` と同じ `iced_test::simulator`
//! (`&str` selector は text/text_input の**現在文字列**への完全一致 —
//! `iced_selector::Selector for &str` 実測)。

use motolii_settings_pane::sections::{
    view, CompField, CompFieldDraft, Message, PreviewCacheStats, ViewModel,
};
use motolii_settings_pane::{BackgroundPreset, Message as LegacyMessage};
use motolii_store::{Composition, Fps};
use motolii_tokens_rs::{Colors, Dimensions};

fn fixture_comp() -> Composition {
    Composition {
        width: 1920,
        height: 1080,
        fps: Fps::try_new(30, 1).expect("30fps"),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }
}

/// 全結線済み想定の [`ViewModel`](supervisor が `Shell` から組む形の再現)。
fn wired_model<'a>(
    composition: &'a Composition,
    comp_draft: Option<&'a CompFieldDraft>,
) -> ViewModel<'a> {
    ViewModel {
        composition: Some(composition),
        background_draft: None,
        comp_draft,
        ui_scale: 1.0,
        ui_scale_draft: None,
        preview_cache: Some(PreviewCacheStats {
            held_frames: 12,
            limit: 64,
        }),
    }
}

/// `view` は `Element<'static, _>` を返すので Simulator も `'static`。Renderer は
/// `iced::Renderer`(inspector-pane の柵と同じ — headless 実装持ち)。
fn simulator(
    model: ViewModel<'_>,
) -> iced_test::Simulator<'static, Message, iced::Theme, iced::Renderer> {
    iced_test::simulator(view(model, Dimensions::default(), Colors::default()))
}

/// 表示値と状態の一致(受入条件2)。section 見出し3枚と、各欄の現在値が
/// 実際の widget 木に居ることを1つずつ確かめる。
#[test]
fn every_section_and_value_renders_from_the_real_state() {
    let composition = fixture_comp();
    let mut ui = simulator(wired_model(&composition, None));
    for expected in [
        "SETTINGS",
        "COMPOSITION",
        "APPEARANCE",
        "PLAYBACK",
        "1920",    // Composition::width
        "1080",    // Composition::height
        "30",      // Composition::fps(format_fps)
        "300",     // Composition::duration_frames
        "100",     // Tokens::ui_scale(%表示)
        "12 / 64", // Engine::cached_frame_count() / FRAME_CACHE_LIMIT(注入値)
    ] {
        ui.find(expected)
            .unwrap_or_else(|error| panic!("{expected:?} が見えない: {error:?}"));
    }
}

/// 下書きが有る欄は下書きが勝つ(Enter まで Document を書かない文法の表示側)。
#[test]
fn a_comp_draft_wins_over_the_stored_value_in_its_cell() {
    let composition = fixture_comp();
    let draft = CompFieldDraft {
        field: CompField::Width,
        text: "20".to_owned(),
    };
    let mut ui = simulator(wired_model(&composition, Some(&draft)));
    ui.find("20").expect("下書き文字列が欄に出ていない");
    assert!(
        ui.find("1920").is_err(),
        "下書き中なのに保存値 1920 がまだ見えている"
    );
}

/// **Message 発火の本命(受入条件1)**: width セルへ click → 打鍵 → Enter で
/// `CompFieldInput(Width, ..)` の列と `CompFieldSubmit(Width)` が出る。
#[test]
fn typing_into_the_width_cell_emits_input_then_submit_messages() {
    let composition = fixture_comp();
    let mut ui = simulator(wired_model(&composition, None));
    ui.click("1920").expect("width セルを押せない");
    let _ = ui.typewrite("5");
    let _ = ui.tap_key(iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter));

    let messages: Vec<Message> = ui.into_messages().collect();
    assert!(
        messages
            .iter()
            .any(|message| matches!(message, Message::CompFieldInput(CompField::Width, text) if text.contains('5'))),
        "打鍵が CompFieldInput(Width, ..) にならない: {messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| matches!(message, Message::CompFieldSubmit(CompField::Width))),
        "Enter が CompFieldSubmit(Width) にならない: {messages:?}"
    );
}

/// 旧項目は `Legacy(..)` に包まれて出る(結線互換の縫い目が実際に通ること)。
#[test]
fn legacy_rows_emit_their_old_messages_wrapped_in_legacy() {
    let composition = fixture_comp();
    let mut ui = simulator(wired_model(&composition, None));
    ui.click("Black").expect("背景プリセット Black を押せない");
    let messages: Vec<Message> = ui.into_messages().collect();
    assert!(
        messages.contains(&Message::Legacy(LegacyMessage::BackgroundPreset(
            BackgroundPreset::Black
        ))),
        "Black プリセットが Legacy(BackgroundPreset(Black)) にならない: {messages:?}"
    );
}

/// **飾り禁止(受入条件3)**: キャッシュ実測が未結線(`None`)なら PLAYBACK 節は
/// 見出しごと出ない — 空箱・ダミー値の顔を作らない。
#[test]
fn the_playback_section_is_absent_until_cache_stats_are_wired() {
    let composition = fixture_comp();
    let mut model = wired_model(&composition, None);
    model.preview_cache = None;
    let mut ui = simulator(model);
    assert!(ui.find("PLAYBACK").is_err(), "未結線なのに PLAYBACK 見出しが出ている");
    assert!(ui.find("12 / 64").is_err(), "未結線なのにキャッシュ値が出ている");
}

/// comp が無い時は編集欄を1つも出さない(旧 view と同じ意味論を継承)。
#[test]
fn without_a_comp_no_editable_cell_is_offered() {
    let model = ViewModel {
        composition: None,
        background_draft: None,
        comp_draft: None,
        ui_scale: 1.0,
        ui_scale_draft: None,
        preview_cache: Some(PreviewCacheStats {
            held_frames: 0,
            limit: 64,
        }),
    };
    let mut ui = simulator(model);
    ui.find("comp が無い — 設定を編集できない")
        .expect("comp 不在の理由文が出ていない");
    assert!(ui.find("COMPOSITION").is_err(), "comp が無いのに COMPOSITION 節が出ている");
    assert!(
        ui.find("PLAYBACK").is_err(),
        "comp が無いのに PLAYBACK 節が出ている(body ごと畳むはず)"
    );
}
