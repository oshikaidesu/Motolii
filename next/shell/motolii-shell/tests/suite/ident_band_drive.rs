//! ident 帯(名前+種別+M/S glyph)の運転席検分。
//!
//! supervisor 訂正(2026-08-20)への応答: 「M グリフは出して結線してください —
//! 既に動いている `LayerAttrs.hidden` へ」。`Message::Inspector(inspector_pane::Message::ToggleHidden)`
//! 自体の効果は `inspector_drive.rs` が Message レベルで確かめ済み — このファイルは
//! **実際に描いた `view()` の木の上で** M glyph をクリックすると、期待した
//! `Message` が本当に出ることを見る(「配線した」と「動く」の間の見落としを
//! 埋める、`q0_fence.rs` は「何かは出る」までしか見ない)。

use iced_test::selector::{Candidate, Target};

use motolii_shell::inspector_pane;
use motolii_shell::tokens::Tokens;
use motolii_shell::{Message, Shell};

/// `q0_fence.rs::collect_targets` と同じ手口(`find_all` が無いので `find` を
/// 尽きるまで繰り返す)。ここでは1ファイル分の小さい検分にしか使わないので
/// 複製で足りる(共有ヘルパー化は発注範囲の外)。**`inspector_pane::view` を
/// 直叩きする**ので `Message` は pane ローカル(裁定160 切片8) — root
/// `motolii_shell::Message` ではない。
fn collect_targets(element: iced::Element<'_, inspector_pane::Message>) -> Vec<Target> {
    let mut ui = iced_test::simulator(element);
    let mut found: Vec<Target> = Vec::new();
    loop {
        let already = found.clone();
        let selector = move |candidate: Candidate<'_>| -> Option<Target> {
            let target = Target::from(candidate);
            if already.contains(&target) {
                None
            } else {
                Some(target)
            }
        };
        match ui.find(selector) {
            Ok(target) => found.push(target),
            Err(_) => break,
        }
        assert!(found.len() <= 5_000, "candidate 列挙が終わらない");
    }
    found
}

fn click_at(
    element: iced::Element<'_, inspector_pane::Message>,
    point: iced::Point,
) -> Vec<inspector_pane::Message> {
    let mut ui = iced_test::simulator(element);
    ui.point_at(point);
    let _ = ui.simulate([
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
            iced::mouse::Button::Left,
        )),
    ]);
    ui.into_messages().collect()
}

/// **本命**。M glyph は `button` として `Container` candidate を1つ持つ
/// (`q0_fence.rs` の doc どおり)。幅が `inspector_glyph_width` と一致する
/// `Container` は、値セル(`inspector_value_width` 幅、別値)とも `Space` の
/// S/Key 予約(`operate()` を実装しないので candidate に出ない)とも区別が付く
/// ので、これで一意に絞れる。
#[test]
fn clicking_the_mute_glyph_in_the_rendered_view_sends_the_toggle_message() {
    let mut shell = Shell::new().0;
    let _ = shell.update(Message::AddLayer);
    let selection = shell
        .inspector_selection()
        .expect("AddLayer 直後は選択があるはず");
    assert!(!selection.attrs.hidden, "既定で hidden になっている");

    let tokens = Tokens::default();
    let dims = tokens.dims.scaled(tokens.ui_scale);
    let colors = tokens.colors;
    let glyph_width = dims.inspector_glyph_width;

    let targets = collect_targets(inspector_pane::view(
        Some(&selection),
        None,
        None,
        dims,
        colors,
    ));
    let mute_target = targets
        .iter()
        .find(|target| {
            matches!(target, Target::Container { .. })
                && target
                    .visible_bounds()
                    .is_some_and(|bounds| (bounds.width - glyph_width).abs() < 0.5)
        })
        .unwrap_or_else(|| {
            panic!(
                "M glyph(幅 {glyph_width})らしき Container candidate が見つからない \
                 — ident 帯に M が出ていない、または結線が外れている疑い: {targets:?}"
            )
        });

    let bounds = mute_target
        .visible_bounds()
        .expect("M glyph の bounds が見えない");
    let point = iced::Point::new(
        bounds.x + bounds.width / 2.0,
        bounds.y + bounds.height / 2.0,
    );

    let messages = click_at(
        inspector_pane::view(Some(&selection), None, None, dims, colors),
        point,
    );
    assert_eq!(
        messages.len(),
        1,
        "M glyph click が期待どおり1件の Message を出さない: {messages:?}"
    );
    assert!(
        matches!(messages[0], inspector_pane::Message::ToggleHidden),
        "M glyph click が ToggleHidden 以外を出している: {messages:?}"
    );
}

/// M glyph を実際に `Shell::update` へ通すと、ident 帯の意図どおり on/off が
/// 見た目にも Container candidate として現れ続ける(消えたり隠れたりしない —
/// 押した後も押せる状態を保つ、という最小限の回帰)。
#[test]
fn the_mute_glyph_still_renders_after_toggling_hidden_on() {
    let mut shell = Shell::new().0;
    let _ = shell.update(Message::AddLayer);
    let _ = shell.update(Message::Inspector(inspector_pane::Message::ToggleHidden));
    let selection = shell.inspector_selection().expect("selection");
    assert!(selection.attrs.hidden, "toggle が効いていない");

    let tokens = Tokens::default();
    let dims = tokens.dims.scaled(tokens.ui_scale);
    let colors = tokens.colors;
    let glyph_width = dims.inspector_glyph_width;

    let targets = collect_targets(inspector_pane::view(
        Some(&selection),
        None,
        None,
        dims,
        colors,
    ));
    assert!(
        targets.iter().any(|target| {
            matches!(target, Target::Container { .. })
                && target
                    .visible_bounds()
                    .is_some_and(|bounds| (bounds.width - glyph_width).abs() < 0.5)
        }),
        "hidden=true の状態で M glyph が消えている(押した後に押せなくなっている)"
    );
}
