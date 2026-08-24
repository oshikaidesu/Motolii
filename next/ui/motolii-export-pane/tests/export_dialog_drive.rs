//! B09 第1切片の drive(発注の受入条件を実 widget 木で見る)。
//! **発注規律: テストは書くが実行しない** — 検収線は
//! `cargo check --tests -p motolii-export-pane` 緑(実行は supervisor 側の関門)。
//!
//! 1. **表示値と状態の一致** — comp の実値(解像度・fps・区間)・出力先・品質が
//!    そのまま出る。
//! 2. **Message 発火** — Export ボタン → `Message::Export`。toggler の発火
//!    (`QualitySelect`/`RangeSelect`)は `&str` selector が text 完全一致のみで
//!    ラベル無し toggler に届かないため、写像の単射性を lib.rs の unit tests で
//!    固定する(`every_*_choice_maps_onto_a_distinct_*`)。
//! 3. **飾り禁止** — 作業範囲が無い間「Work area only」は存在しない。comp が
//!    無ければ節ごと出ない。出力先未設定の間 Export は押せない。
//!
//! 器具は settings-pane `sections_drive.rs` と同じ `iced_test::simulator`
//! (`&str` selector は text の現在文字列への完全一致)。

use std::path::Path;

use motolii_export_pane::{
    view, AspectPreset, ExportProgress, ExportQuality, ExportRange, Message, ViewModel,
    WorkAreaFrames, CONTAINER_CODEC_LABEL,
};
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
fn wired_model<'a>(composition: &'a Composition, out_path: Option<&'a Path>) -> ViewModel<'a> {
    ViewModel {
        composition: Some(composition),
        out_path,
        quality: ExportQuality::Normal,
        range: ExportRange::Whole,
        work_area: Some(WorkAreaFrames { start: 100, end: 200 }),
        progress: None,
    }
}

/// `view` は `Element<'static, _>` を返すので Simulator も `'static`。
fn simulator(
    model: ViewModel<'_>,
) -> iced_test::Simulator<'static, Message, iced::Theme, iced::Renderer> {
    iced_test::simulator(view(model, Dimensions::default(), Colors::default()))
}

/// 表示値と状態の一致(受入条件1)。section 見出しと、comp 実値・実対応
/// コーデック・出力先が実際の widget 木に居ることを1つずつ確かめる。
#[test]
fn every_section_and_value_renders_from_the_real_state() {
    let composition = fixture_comp();
    let out = Path::new("/tmp/out.mp4");
    let mut ui = simulator(wired_model(&composition, Some(out)));
    for expected in [
        "EXPORT",
        "OUTPUT",
        "RANGE",
        "ASPECT",
        "RUN",
        "/tmp/out.mp4",         // ExportJob::out_path になる予定の表示
        CONTAINER_CODEC_LABEL,  // Encoder の実対応(1種)
        "1920 × 1080",          // Composition::{width, height}
        "30",                   // Composition::fps(format_fps)
        "0 – 299(300 frames)", // effective_range(Whole)
        "Export",
    ] {
        ui.find(expected)
            .unwrap_or_else(|error| panic!("{expected:?} が見えない: {error:?}"));
    }
}

/// プリセットは飾りの文字ではなく、比率・寸法を表示する実体のある
/// Message 発火面であることを widget 木で固定する。
#[test]
fn aspect_preset_buttons_show_label_and_dimensions() {
    let composition = fixture_comp();
    let out = Path::new("/tmp/out.mp4");
    let mut ui = simulator(wired_model(&composition, Some(out)));
    for expected in [
        "ASPECT",
        "16:9",
        "1920 × 1080",
        "9:16",
        "1080 × 1920",
        "1:1",
        "1080 × 1080",
    ] {
        ui.find(expected)
            .unwrap_or_else(|error| panic!("{expected:?} が見えない: {error:?}"));
    }

    ui.click("9:16").expect("9:16 プリセットを押せない");
    let messages: Vec<Message> = ui.into_messages().collect();
    assert!(
        messages.contains(&Message::AspectPresetSelect(AspectPreset::Portrait9x16)),
        "9:16 の要求が pane Message にならない: {messages:?}"
    );
}

/// **Message 発火の本命(受入条件2)**: Export ボタン押下で `Message::Export`。
#[test]
fn pressing_export_emits_the_export_message() {
    let composition = fixture_comp();
    let out = Path::new("/tmp/out.mp4");
    let mut ui = simulator(wired_model(&composition, Some(out)));
    ui.click("Export").expect("Export ボタンを押せない");
    let messages: Vec<Message> = ui.into_messages().collect();
    assert!(
        messages.contains(&Message::Export),
        "Export 押下が Message::Export にならない: {messages:?}"
    );
}

/// 範囲選択が effective_range の表示へ写る(WorkArea 選択時は交差区間)。
#[test]
fn selecting_the_work_area_narrows_the_displayed_range() {
    let composition = fixture_comp();
    let out = Path::new("/tmp/out.mp4");
    let mut model = wired_model(&composition, Some(out));
    model.range = ExportRange::WorkArea;
    let mut ui = simulator(model);
    ui.find("100 – 199(100 frames)")
        .expect("作業範囲選択が区間表示に写っていない");
}

/// **飾り禁止(受入条件3)**: 作業範囲が無い間、「Work area only」の選択肢は
/// 存在しない — 代わりに全体固定である事実だけ言う。
#[test]
fn the_work_area_toggle_is_absent_without_a_work_area() {
    let composition = fixture_comp();
    let out = Path::new("/tmp/out.mp4");
    let mut model = wired_model(&composition, Some(out));
    model.work_area = None;
    let mut ui = simulator(model);
    assert!(
        ui.find("Work area only").is_err(),
        "作業範囲が無いのに選択肢が出ている"
    );
    ui.find("作業範囲が無い — 全体を書き出す")
        .expect("全体固定の事実文が出ていない");
}

/// 出力先未設定の間、Export は押しても Message が出ない(disabled)+理由文。
#[test]
fn export_is_inert_and_explained_while_the_destination_is_unset() {
    let composition = fixture_comp();
    let mut ui = simulator(wired_model(&composition, None));
    ui.find("未設定").expect("出力先未設定の表示が無い");
    ui.find("出力先が未設定 — Export は押せない")
        .expect("押せない理由文が出ていない");
    let _ = ui.click("Export"); // disabled — 当たっても Message は出ない
    let messages: Vec<Message> = ui.into_messages().collect();
    assert!(
        !messages.contains(&Message::Export),
        "出力先未設定なのに Export が発火した: {messages:?}"
    );
}

/// 実行中は Export ボタンが消え、進捗と中断だけが出る(進捗投影の器)。
#[test]
fn a_running_export_shows_progress_and_cancel_instead_of_the_export_button() {
    let composition = fixture_comp();
    let out = Path::new("/tmp/out.mp4");
    let mut model = wired_model(&composition, Some(out));
    model.progress = Some(ExportProgress {
        frames_done: 123,
        frames_total: 300,
    });
    let mut ui = simulator(model);
    ui.find("123 / 300(41%)").expect("進捗表示が出ていない");
    assert!(
        ui.find("Export").is_err(),
        "実行中なのに Export ボタンがまだ出ている"
    );
}

/// comp が無い時は節を1つも出さない(settings と同じ意味論)。
#[test]
fn without_a_comp_no_section_is_offered() {
    let model = ViewModel {
        composition: None,
        out_path: None,
        quality: ExportQuality::Normal,
        range: ExportRange::Whole,
        work_area: None,
        progress: None,
    };
    let mut ui = simulator(model);
    ui.find("comp が無い — 書き出す対象が無い")
        .expect("comp 不在の理由文が出ていない");
    for absent in ["OUTPUT", "RANGE", "RUN", "Export"] {
        assert!(
            ui.find(absent).is_err(),
            "comp が無いのに {absent:?} が出ている"
        );
    }
}
