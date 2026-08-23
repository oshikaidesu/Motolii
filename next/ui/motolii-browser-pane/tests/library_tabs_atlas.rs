//! タブ帯(mock `.libraryTabs` html:411-416)ORACLE — B3 転写の取り残し回収
//! (利用者実窓不合格 2026-08-22 「上のヘッダーが無い。ブラウザ/エフェクト/
//! クリエイト/パレットの4種類が無い。仮データも無い」への直接対応)。
//!
//! `rail_filter_atlas.rs` と同じ手口: `motolii-shell`(write-set 外)を経由せず
//! `browser_pane::pane_view` を `iced_test::simulator` で直叩きし、
//! - 4タブが**常設可視**(S6: メニューの奥に隠さない — どのタブが active でも
//!   4ラベル全部が atlas に現れる)
//! - クリックが実際に `Message::SelectTab` を publish する(Q0)
//! - タブごとに catalog の中身が切り替わる(media=Document 台帳投影 /
//!   effects・create・panels=preview-local 静的カタログ)
//! ことを確かめる。

use iced_test::selector::{Candidate, Target};

use motolii_browser_pane::model::AssetListItem;
use motolii_browser_pane::{pane_view, LibraryTab, Message, PaneState};
use motolii_store::AssetId;
use motolii_tokens_rs::{Colors, Dimensions};

/// `rail_filter_atlas.rs` と同じ「`find` を尽きるまで繰り返す」手口。
fn collect_targets(element: iced::Element<'_, Message>) -> Vec<Target> {
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

fn text_contents(targets: &[Target]) -> Vec<String> {
    targets
        .iter()
        .filter_map(|target| match target {
            Target::Text { content, .. } => Some(content.clone()),
            _ => None,
        })
        .collect()
}

fn item(id: u64, name: &str, kind: &str) -> AssetListItem {
    AssetListItem {
        id: AssetId::from_raw(id),
        name: name.to_owned(),
        kind: kind.to_owned(),
        path: None,
        fingerprint: format!("sha256:{name}"),
        duration: None,
        status: motolii_store::AssetStatus::Unchecked,
    }
}

fn fixture_items() -> Vec<AssetListItem> {
    vec![
        item(0, "intro-clip", "video/mp4"),
        item(1, "logo-mark", "image/png"),
        item(2, "room-tone", "audio/wav"),
    ]
}

fn state_on(tab: LibraryTab) -> PaneState {
    let mut state = PaneState::new();
    state.update(Message::SelectTab(tab));
    state
}

/// **本命(S6)**: どのタブが active でも、4タブのラベル全部が `Target::Text`
/// として atlas に現れる(常設可視 — 切替でタブ帯自体が消えない)。
#[test]
fn all_four_tabs_are_always_visible() {
    let items = fixture_items();
    for active in [
        LibraryTab::Media,
        LibraryTab::Effects,
        LibraryTab::Create,
        LibraryTab::Panels,
    ] {
        let state = state_on(active);
        let targets = collect_targets(pane_view(
            &state,
            &items,
            None,
            Dimensions::default(),
            Colors::default(),
        ));
        let texts = text_contents(&targets);
        for expected in ["Media", "Effects", "Create", "Panels"] {
            assert!(
                texts.iter().any(|content| content == expected),
                "active={active:?} でタブ {expected:?} が atlas に無い(text: {texts:?})"
            );
        }
    }
}

/// タブのクリックが `Message::SelectTab` を publish する(Q0: 見えるだけ
/// でなく実際に効く)。
#[test]
fn clicking_a_tab_publishes_select_tab() {
    let items = fixture_items();
    let state = PaneState::new();
    let build = || pane_view(&state, &items, None, Dimensions::default(), Colors::default());

    let targets = collect_targets(build());
    let bounds = targets
        .iter()
        .find_map(|target| match target {
            Target::Text { content, .. } if content == "Effects" => Some(target.bounds()),
            _ => None,
        })
        .expect("Effects タブの Target::Text が見つからない");
    let point = iced::Point::new(
        bounds.x + bounds.width / 2.0,
        bounds.y + bounds.height / 2.0,
    );

    let mut ui = iced_test::simulator(build());
    ui.point_at(point);
    let _ = ui.simulate([
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
            iced::mouse::Button::Left,
        )),
    ]);
    let messages: Vec<Message> = ui.into_messages().collect();

    assert!(
        messages.contains(&Message::SelectTab(LibraryTab::Effects)),
        "Effects タブのクリックが SelectTab(Effects) を publish しない: {messages:?}"
    );
}

/// media タブは Document 台帳投影のカードだけを描き、preview-local 静的
/// カタログのカード(Glow 等)は1枚も現れない。
#[test]
fn media_tab_renders_the_ledger_and_never_the_preview_catalog() {
    let items = fixture_items();
    let state = PaneState::new();
    let targets = collect_targets(pane_view(
        &state,
        &items,
        None,
        Dimensions::default(),
        Colors::default(),
    ));
    let texts = text_contents(&targets);

    assert!(
        texts.iter().any(|content| content == "intro-clip"),
        "台帳素材が描かれない: {texts:?}"
    );
    assert!(
        !texts.iter().any(|content| content == "Glow"),
        "media タブに preview-local の Glow が混ざっている: {texts:?}"
    );
}

/// effects タブは preview-local 静的カタログ(Glow を含む)を描き、Document
/// 台帳の素材は1枚も現れない。
#[test]
fn effects_tab_renders_the_preview_catalog_and_never_the_ledger() {
    let items = fixture_items();
    let state = state_on(LibraryTab::Effects);
    let targets = collect_targets(pane_view(
        &state,
        &items,
        None,
        Dimensions::default(),
        Colors::default(),
    ));
    let texts = text_contents(&targets);

    for expected in ["Echo Bloom", "Opacity", "Sine", "Glow"] {
        assert!(
            texts.iter().any(|content| content == expected),
            "effects タブに {expected:?} が描かれない: {texts:?}"
        );
    }
    assert!(
        !texts.iter().any(|content| content == "intro-clip"),
        "effects タブに台帳素材が混ざっている: {texts:?}"
    );
}

/// filter shelf のチップは**タブ別の語彙**(mock `.filterGroup[data-filter-
/// group=*]` — 構造の対称化 2026-08-22 で非 media タブにもタブ別チップが
/// 入ったが、media 語彙のチップが他タブへ漏れないことは不変)。検索欄
/// (mock では toolbar 領域、全タブ共有)は非 media タブでも残る。
/// タブ別チップ自体の存在は `tab_rail_symmetry_atlas.rs` が固定する。
#[test]
fn media_filter_chips_are_hidden_outside_the_media_tab() {
    let items = fixture_items();

    let media_targets = collect_targets(pane_view(
        &PaneState::new(),
        &items,
        None,
        Dimensions::default(),
        Colors::default(),
    ));
    assert!(
        text_contents(&media_targets)
            .iter()
            .any(|content| content == "Images"),
        "media タブに種別チップが無い"
    );

    let effects_state = state_on(LibraryTab::Effects);
    let effects_targets = collect_targets(pane_view(
        &effects_state,
        &items,
        None,
        Dimensions::default(),
        Colors::default(),
    ));
    let effects_texts = text_contents(&effects_targets);
    assert!(
        !effects_texts.iter().any(|content| content == "Images"),
        "effects タブに media 語彙のチップが残っている: {effects_texts:?}"
    );
    assert!(
        effects_targets
            .iter()
            .any(|target| matches!(target, Target::TextInput { .. })),
        "検索欄(全タブ共有)が effects タブに無い"
    );
}
