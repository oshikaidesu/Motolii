//! 構造の対称化 ORACLE(利用者実窓指摘 2026-08-22 「Browser の構造(ディレク
//! トリ、タブ)が media タブにしか適用されていない」への直接対応)。
//!
//! media タブの構造文法 — 左 rail=スコープ選択 / 上=検索+フィルタチップ /
//! 右=カタログ — が effects/create/panels の3タブにも通っていることを、
//! `rail_filter_atlas.rs`/`library_tabs_atlas.rs` と同じ atlas 手口
//! (`iced_test::simulator` 直叩き、`motolii-shell` 非経由)で固定する:
//!
//! 1. **rail とフィルタ行の存在**(S6 常設可視): 各タブで rail 行
//!    (mock `.tabScoped-*` の `.locationRow`)とフィルタチップ(mock
//!    `.filterGroup[data-filter-group=*]`)が両方 atlas に現れる — カテゴリ
//!    ラベルは rail とチップの2箇所に同時に出る(mock と同配置)。
//! 2. **rail 選択でカタログが絞られる**: scope を選んだ状態の `pane_view` で
//!    結果件数(`Results N`)とカードの顔ぶれが変わる。
//! 3. **rail/チップの click が Message を publish する**(Q0: 触れそうな物は
//!    実際に効く)。
//! 4. **カード click が従来 Message 経路(`Message::SelectCard` → root
//!    `Message::Browser` 畳み)を通る** — タブ4つ全てで証明する。
//!
//! rail の並びは S0(慣習順)= mock `.tabScoped-*` の掲載順そのまま。

use iced_test::selector::{Candidate, Target};

use motolii_browser_pane::model::AssetListItem;
use motolii_browser_pane::{
    pane_view, CardKey, LibraryTab, Message, PaneState, PreviewScope, PreviewTag,
};
use motolii_store::AssetId;
use motolii_tokens_rs::{Colors, Dimensions};

/// `rail_filter_atlas.rs` と同じ「`find` を尽きるまで繰り返す」手口
/// (`Simulator` に `find_all` が無い、iced_test 0.14.0 実測)。
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

fn targets_on(state: &PaneState, items: &[AssetListItem]) -> Vec<Target> {
    collect_targets(pane_view(
        state,
        items,
        Dimensions::default(),
        Colors::default(),
    ))
}

/// content が一致する `Target::Text` の個数(rail 行とフィルタチップが同じ
/// ラベルで**2箇所**に出ることを数えるため)。
fn count_text(targets: &[Target], content: &str) -> usize {
    text_contents(targets)
        .iter()
        .filter(|candidate| candidate.as_str() == content)
        .count()
}

/// クリック手口(`rail_filter_atlas.rs` と同じ): content の Text 中心を
/// 押して publish された Message 列を返す。
fn click_text(state: &PaneState, items: &[AssetListItem], content: &str) -> Vec<Message> {
    let build = || pane_view(state, items, Dimensions::default(), Colors::default());
    let targets = collect_targets(build());
    let bounds = targets
        .iter()
        .find_map(|target| match target {
            Target::Text { content: c, .. } if c == content => Some(target.bounds()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("Target::Text の content={content:?} が見つからない"));
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
    ui.into_messages().collect()
}

fn result_count(targets: &[Target]) -> String {
    text_contents(targets)
        .into_iter()
        .find(|content| content.starts_with("Results "))
        .expect("Results 行が見つからない")
}

// ---------------------------------------------------------------------------
// 1. rail とフィルタ行の存在(S6 常設可視・S0 慣習順=mock 掲載順)。
// ---------------------------------------------------------------------------

/// **本命(S6)**: 4タブ全てで rail 先頭の「All …」行が atlas に現れる
/// (mock `.tabScoped-*` の `data-source="all"` 行の転写)。
#[test]
fn every_tab_has_an_all_scope_rail_row() {
    let items = fixture_items();
    for (tab, all_label) in [
        (LibraryTab::Media, "All media"),
        (LibraryTab::Effects, "All effects"),
        (LibraryTab::Create, "All create"),
        (LibraryTab::Panels, "All panels"),
    ] {
        let state = state_on(tab);
        let targets = targets_on(&state, &items);
        assert!(
            count_text(&targets, all_label) >= 1,
            "{tab:?} タブに rail の {all_label:?} 行が無い: {:?}",
            text_contents(&targets)
        );
    }
}

/// **本命(S6)**: 各タブのカテゴリラベルが rail 行とフィルタチップの
/// **2箇所**に同時に現れる(mock: `.tabScoped-*` の `.locationRow` と
/// `.filterGroup[data-filter-group=*]` のチップが同ラベル)。
#[test]
fn category_labels_appear_in_both_the_rail_and_the_filter_shelf() {
    let items = fixture_items();
    let expectations: [(LibraryTab, &[&str]); 4] = [
        (LibraryTab::Media, &["Video", "Images", "Audio"]),
        (LibraryTab::Effects, &["Color", "Utility", "Animation"]),
        (LibraryTab::Create, &["Shapes", "Built-in"]),
        (LibraryTab::Panels, &["Tags", "Notes", "Export"]),
    ];
    for (tab, labels) in expectations {
        let state = state_on(tab);
        let targets = targets_on(&state, &items);
        for label in labels {
            assert!(
                count_text(&targets, label) >= 2,
                "{tab:?} タブで {label:?} が rail とチップの2箇所に出ていない\
                 (count={}): {:?}",
                count_text(&targets, label),
                text_contents(&targets)
            );
        }
        // Clear ボタン(mock: 各 filterGroup に同居)も全タブで見える。
        assert!(
            count_text(&targets, "Clear") >= 1,
            "{tab:?} タブに Clear が無い"
        );
        // 検索欄(mock toolbar 領域、全タブ共有)。
        assert!(
            targets
                .iter()
                .any(|target| matches!(target, Target::TextInput { .. })),
            "{tab:?} タブに検索欄が無い"
        );
    }
}

// ---------------------------------------------------------------------------
// 2. rail 選択でカタログが絞られる。
// ---------------------------------------------------------------------------

/// effects タブ: `Color` scope で Echo Bloom+Glow の2枚に絞れる
/// (Opacity/Sine は消える)。
#[test]
fn effects_rail_scope_color_narrows_the_catalog() {
    let items = fixture_items();
    let mut state = state_on(LibraryTab::Effects);
    let all = targets_on(&state, &items);
    assert_eq!(result_count(&all), "Results 4");

    state.update(Message::SelectPreviewScope(PreviewScope::Tag(
        PreviewTag::Color,
    )));
    let narrowed = targets_on(&state, &items);
    assert_eq!(result_count(&narrowed), "Results 2");
    let texts = text_contents(&narrowed);
    assert!(texts.iter().any(|content| content == "Echo Bloom"));
    assert!(texts.iter().any(|content| content == "Glow"));
    assert!(
        !texts.iter().any(|content| content == "Opacity"),
        "Color scope で Opacity が残っている: {texts:?}"
    );
}

/// create タブ: `Shapes` scope は Rectangle/Ellipse の2枚(両方 shape タグ)。
#[test]
fn create_rail_scope_shapes_keeps_both_shape_cards() {
    let items = fixture_items();
    let mut state = state_on(LibraryTab::Create);
    state.update(Message::SelectPreviewScope(PreviewScope::Tag(
        PreviewTag::Shapes,
    )));
    let targets = targets_on(&state, &items);
    assert_eq!(result_count(&targets), "Results 2");
}

/// panels タブ: `Notes` scope で1枚へ絞れる。
#[test]
fn panels_rail_scope_notes_narrows_to_one_card() {
    let items = fixture_items();
    let mut state = state_on(LibraryTab::Panels);
    state.update(Message::SelectPreviewScope(PreviewScope::Tag(
        PreviewTag::Notes,
    )));
    let targets = targets_on(&state, &items);
    assert_eq!(result_count(&targets), "Results 1");
    let texts = text_contents(&targets);
    assert!(
        !texts.iter().any(|content| content == "Asset tagging"),
        "Notes scope で Asset tagging が残っている: {texts:?}"
    );
}

// ---------------------------------------------------------------------------
// 3. rail/チップの click が Message を publish する(Q0)。
// ---------------------------------------------------------------------------

/// 非 media タブの rail 行 click は `Message::SelectPreviewScope` を publish
/// する(rail とチップは同ラベル・同 Message — 2つの入口が同じ状態を書く、
/// Ableton可視性原理どおり media と同型)。
#[test]
fn clicking_a_preview_rail_row_publishes_select_preview_scope() {
    let items = fixture_items();
    for (tab, label, expected) in [
        (LibraryTab::Effects, "All effects", PreviewScope::All),
        (
            LibraryTab::Create,
            "Shapes",
            PreviewScope::Tag(PreviewTag::Shapes),
        ),
        (
            LibraryTab::Panels,
            "Export",
            PreviewScope::Tag(PreviewTag::Export),
        ),
    ] {
        let state = state_on(tab);
        let messages = click_text(&state, &items, label);
        assert!(
            messages.contains(&Message::SelectPreviewScope(expected)),
            "{tab:?} タブの {label:?} click が SelectPreviewScope を publish しない: \
             {messages:?}"
        );
    }
}

/// 非 media タブの Clear click は `Message::ClearFilters` を publish する
/// (media と同じ従来 Message)。
#[test]
fn clicking_clear_on_a_preview_tab_publishes_clear_filters() {
    let items = fixture_items();
    let state = state_on(LibraryTab::Effects);
    let messages = click_text(&state, &items, "Clear");
    assert!(
        messages.contains(&Message::ClearFilters),
        "effects タブの Clear click が ClearFilters を publish しない: {messages:?}"
    );
}

// ---------------------------------------------------------------------------
// 4. カード click → 従来 Message 経路(タブ4つ全て)。
// ---------------------------------------------------------------------------

/// **本命(受入条件)**: タブ4つ全てで「rail 選択→カタログ絞り込み→カード
/// click で従来 Message」の経路が通る。rail scope で絞った状態のまま、
/// 残ったカードを click すると `Message::SelectCard` が publish される
/// (root 側は既存の `Message::Browser` 1本で畳む — shell 側配線は不変)。
#[test]
fn clicking_a_card_publishes_select_card_on_every_tab() {
    let items = fixture_items();

    // media: rail=Video で intro-clip 1枚に絞ってから click。
    let mut media = PaneState::new();
    media.update(Message::SelectScope(motolii_browser_pane::RailScope::Video));
    let messages = click_text(&media, &items, "intro-clip");
    assert!(
        messages.contains(&Message::SelectCard(CardKey::Media(AssetId::from_raw(0)))),
        "media カード click が SelectCard を publish しない: {messages:?}"
    );

    // effects: rail=Color で絞ってから Glow を click。
    let mut effects = state_on(LibraryTab::Effects);
    effects.update(Message::SelectPreviewScope(PreviewScope::Tag(
        PreviewTag::Color,
    )));
    let messages = click_text(&effects, &items, "Glow");
    assert!(
        messages.contains(&Message::SelectCard(CardKey::Preview("glow"))),
        "effects カード click が SelectCard を publish しない: {messages:?}"
    );

    // create: rail=Shapes のまま Rectangle を click。
    let mut create = state_on(LibraryTab::Create);
    create.update(Message::SelectPreviewScope(PreviewScope::Tag(
        PreviewTag::Shapes,
    )));
    let messages = click_text(&create, &items, "Rectangle");
    assert!(
        messages.contains(&Message::SelectCard(CardKey::Preview("rectangle"))),
        "create カード click が SelectCard を publish しない: {messages:?}"
    );

    // panels: 絞らず Asset tagging を click(ラベルがカード固有で一意)。
    let panels = state_on(LibraryTab::Panels);
    let messages = click_text(&panels, &items, "Asset tagging");
    assert!(
        messages.contains(&Message::SelectCard(CardKey::Preview("asset-tags"))),
        "panels カード click が SelectCard を publish しない: {messages:?}"
    );
}
