//! B12 第1切片+第2切片(AUTOSAVE)の drive(発注書の受入条件を実 widget 木で
//! 見る):
//!
//! 1. **表示値と状態の一致** — comp の実値・ui_scale・キャッシュ実測値・
//!    AutoSaveConfig の実値がそのまま欄に出る(下書きが有ればそちらが勝つ)。
//! 2. **Message 発火** — 数値欄への打鍵で `CompFieldInput`/`AutoSaveFieldInput`、
//!    Enter で `CompFieldSubmit`/`AutoSaveFieldSubmit`、旧項目(プリセット等)は
//!    `Legacy(..)` に包まれて出る。**`AutoSaveToggle` はここでは駆動しない**
//!    (`motolii-export-pane` の `export_dialog_drive.rs` 冒頭 doc と同じ既知
//!    制約 — `&str` selector は text の現在文字列への完全一致のみで、ラベル無し
//!    `toggler` 自体には届かない。行の label text をクリックしても toggler の
//!    `on_toggle` は発火しない。`AutoSaveToggle(bool)` は bool→bool の恒等
//!    写像で単射性の検算対象も無い — export-pane が `QualitySelect`/
//!    `RangeSelect` に対して行う「写像の単射性を unit test で固定する」代替すら
//!    不要、型があれば充分)。
//! 3. **飾り禁止** — 未結線(`preview_cache: None`)の間、PLAYBACK 節は
//!    見出しごと存在しない(「顔だけの設定」を1つも作らない)。AUTOSAVE は
//!    `AutoSaveConfig`(GPU 等の外部依存が無い生の値)なのでこの防波堤は
//!    要らない — 常に出る。
//!
//! 器具は inspector-pane `value_cell_legibility.rs` と同じ `iced_test::simulator`
//! (`&str` selector は text/text_input の**現在文字列**への完全一致 —
//! `iced_selector::Selector for &str` 実測)。

use motolii_settings_pane::sections::{
    view, AutoSaveField, AutoSaveFieldDraft, CompField, CompFieldDraft, Message,
    PreviewCacheStats, ViewModel,
};
use motolii_settings_pane::{BackgroundPreset, Message as LegacyMessage};
use motolii_store::{AutoSaveConfig, Composition, Fps};
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
        auto_save_enabled: true,
        // interval(5分)と generations(7)を意図的に異なる値にする — どちらも
        // 5 だと `ui.click("5")` 等の selector が2箇所にマッチして曖昧になる
        // (`AutoSaveConfig::default()` は偶然どちらも5、fixture では避ける)。
        auto_save_config: AutoSaveConfig {
            interval_secs: 300,
            generations: 7,
        },
        auto_save_draft: None,
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
        "AUTOSAVE",
        "PLAYBACK",
        "1920",    // Composition::width
        "1080",    // Composition::height
        "30",      // Composition::fps(format_fps)
        "300",     // Composition::duration_frames
        "100",     // Tokens::ui_scale(%表示)
        "12 / 64", // Engine::cached_frame_count() / FRAME_CACHE_LIMIT(注入値)
        "5",       // AutoSaveConfig::interval_secs=300(fixture 値、分表示)
        "7",       // AutoSaveConfig::generations(fixture 値)
        "Automatically Save Projects", // AUTOSAVE トグル行のラベル
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
        auto_save_enabled: true,
        auto_save_config: AutoSaveConfig::default(),
        auto_save_draft: None,
    };
    let mut ui = simulator(model);
    ui.find("comp が無い — 設定を編集できない")
        .expect("comp 不在の理由文が出ていない");
    assert!(ui.find("COMPOSITION").is_err(), "comp が無いのに COMPOSITION 節が出ている");
    assert!(
        ui.find("PLAYBACK").is_err(),
        "comp が無いのに PLAYBACK 節が出ている(body ごと畳むはず)"
    );
    assert!(
        ui.find("AUTOSAVE").is_err(),
        "comp が無いのに AUTOSAVE 節が出ている(現行 view の body 分岐は\
         comp 有無で丸ごと畳む文法 — APPEARANCE/AUTOSAVE も comp 非依存の\
         意味だが、この分岐自体は本切片のスコープ外なので現状の意味論を\
         継承するだけ)"
    );
}

// ---------------------------------------------------------------------------
// AUTOSAVE(第2切片): 表示値の一致・Message 発火・トグルの独立性を照合する。
// ---------------------------------------------------------------------------

/// AutoSaveConfig の実値がそのまま欄に出る(表示値と状態の一致)。
#[test]
fn auto_save_cells_render_the_real_config_values() {
    let composition = fixture_comp();
    let mut model = wired_model(&composition, None);
    model.auto_save_config = AutoSaveConfig {
        interval_secs: 600,
        generations: 8,
    };
    let mut ui = simulator(model);
    ui.find("10").expect("interval_secs=600(10分)が欄に出ていない");
    ui.find("8").expect("generations=8 が欄に出ていない");
}

/// 自動保存の下書きが有る欄は下書きが勝つ([`a_comp_draft_wins_over_the_stored_value_in_its_cell`]
/// と同じ形)。
#[test]
fn an_auto_save_draft_wins_over_the_stored_value_in_its_cell() {
    let composition = fixture_comp();
    let draft = AutoSaveFieldDraft {
        field: AutoSaveField::Generations,
        text: "3".to_owned(),
    };
    let mut model = wired_model(&composition, None);
    model.auto_save_draft = Some(&draft);
    let mut ui = simulator(model);
    ui.find("3").expect("下書き文字列(世代数)が欄に出ていない");
    assert!(
        ui.find("7").is_err(),
        "下書き中なのに保存値 7(fixture の世代数)がまだ見えている"
    );
}

/// **Message 発火**: Generations セルへ click → 打鍵 → Enter で
/// `AutoSaveFieldInput(Generations, ..)` と `AutoSaveFieldSubmit(Generations)`
/// が出る([`typing_into_the_width_cell_emits_input_then_submit_messages`] と
/// 同じ形)。
#[test]
fn typing_into_the_generations_cell_emits_input_then_submit_messages() {
    let composition = fixture_comp();
    let mut ui = simulator(wired_model(&composition, None));
    ui.click("7").expect("generations セル(fixture 値 7)を押せない");
    let _ = ui.typewrite("2");
    let _ = ui.tap_key(iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter));

    let messages: Vec<Message> = ui.into_messages().collect();
    assert!(
        messages.iter().any(|message| matches!(
            message,
            Message::AutoSaveFieldInput(AutoSaveField::Generations, text) if text.contains('2')
        )),
        "打鍵が AutoSaveFieldInput(Generations, ..) にならない: {messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| matches!(message, Message::AutoSaveFieldSubmit(AutoSaveField::Generations))),
        "Enter が AutoSaveFieldSubmit(Generations) にならない: {messages:?}"
    );
}

/// AUTOSAVE 節は `preview_cache` の結線状態と無関係に出る
/// ([`the_playback_section_is_absent_until_cache_stats_are_wired`] の裏返し —
/// `AutoSaveConfig` は GPU 等の外部依存が無い生の値なので飾り禁止の防波堤が
/// 要らないことの直接証拠)。
#[test]
fn the_autosave_section_survives_even_when_playback_is_unwired() {
    let composition = fixture_comp();
    let mut model = wired_model(&composition, None);
    model.preview_cache = None;
    let mut ui = simulator(model);
    assert!(ui.find("PLAYBACK").is_err(), "未結線なのに PLAYBACK 見出しが出ている");
    ui.find("AUTOSAVE")
        .expect("PLAYBACK 未結線につられて AUTOSAVE まで消えている");
}
