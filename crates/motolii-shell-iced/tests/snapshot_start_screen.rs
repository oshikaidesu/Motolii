//! スタート画面の snapshot oracle — 見た目の後退を窓を開かずに落とす。
//!
//! `iced_test` は公式に snapshot 口を持つ(`Simulator::snapshot(&Theme)` →
//! `Snapshot::matches_image`)。golden PNG が無ければ初回に作られ、以後は
//! byte 一致で照合される。ファイル名には renderer 名が付く
//! (`start_screen-<renderer>.png`)ので、renderer が替わった環境では別 golden に
//! なる — 別 renderer の絵を同じ byte 列と主張しない、という `iced_test` 側の
//! 正直さをそのまま借りる。
//!
//! egui 側の対応物は `egui_kittest` の `snapshot`。ここでも判定の粒度は同じ
//! 「1 pixel もずれない」である。

use motolii_shell_iced::{theme, view, ScriptedPrompts, Shell};

/// スタート画面(座席なし)が golden と一致する。
///
/// 窓と同じ 980x650 で描く(`main.rs` の `window_size`)。theme は製品の
/// [`theme::product`] — iced 既定 palette で描いた絵を golden にしない。
#[test]
fn the_start_screen_matches_its_golden_image() {
    let shell = Shell::new(ScriptedPrompts::default());
    let mut ui = iced_test::Simulator::with_size(
        iced_test::core::Settings::default(),
        iced::Size::new(980.0, 650.0),
        view(&shell),
    );

    let snapshot = ui.snapshot(&theme::product()).expect("draw the start screen");
    assert!(
        snapshot
            .matches_image(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/snapshots/start_screen"
            ))
            .expect("read or write the golden image"),
        "スタート画面が tests/snapshots/start_screen-<renderer>.png と一致しない。\
         意図した変更なら golden を消して作り直す"
    );
}
