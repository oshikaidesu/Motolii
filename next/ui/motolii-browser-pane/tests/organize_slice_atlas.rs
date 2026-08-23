//! B08 第4切片(素材の整理)ORACLE — 発注書「Browser 第4切片 — 素材の整理
//! (B08 残り+タグ/コレクション)」の view 配線を、`rail_filter_atlas.rs`/
//! `library_tabs_atlas.rs` と同じ手口(`motolii-shell` を経由せず
//! `browser_pane::pane_view` を `iced_test::simulator` で直叩き)で固定する。
//!
//! **未実行**(発注書の検収線は `cargo check --tests -p motolii-browser-pane`
//! まで — このファイルの全テストはコンパイルのみ確認済みで、実行はしていない)。
//!
//! 対象:
//! 1. 並べ替えチップ(`model::SORT_KEYS`)は media タブにのみ現れ、click が
//!    `Message::SelectSortKey` を publish する。
//! 2. 並べ替えチップは preview タブ(effects/create/panels)には漏れない。
//! 3. grid/list 表示形式トグルは、実際にカードの行内配置を変える
//!    (grid = 同じ行に並ぶ/list = 別々の行に積まれる)。

use iced_test::selector::{Candidate, Target};

use motolii_browser_pane::model::AssetListItem;
use motolii_browser_pane::{pane_view, LibraryTab, Message, PaneState, SortKey, ViewMode};
use motolii_store::AssetId;
use motolii_tokens_rs::{Colors, Dimensions};

/// 他の atlas テストと同じ「`find` を尽きるまで繰り返す」手口。
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

fn text_bounds(targets: &[Target], content: &str) -> iced::Rectangle {
    targets
        .iter()
        .filter_map(|t| match t {
            Target::Text { content: c, .. } if c == content => Some(t.bounds()),
            _ => None,
        })
        .next()
        .unwrap_or_else(|| panic!("Target::Text の content={content:?} が見つからない"))
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

// ---------------------------------------------------------------------------
// 1. 並べ替えチップ(media のみ、click で SelectSortKey)。
// ---------------------------------------------------------------------------

/// **本命**: media タブの filter shelf に3つの並べ替えチップ(Name/Date
/// added/Type)が atlas に現れる。
#[test]
fn media_tab_shows_all_sort_chips() {
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
    for expected in ["Name", "Date added", "Type"] {
        assert!(
            texts.iter().any(|content| content == expected),
            "media タブに sort チップ {expected:?} が無い: {texts:?}"
        );
    }
}

/// 並べ替えチップの click が `Message::SelectSortKey` を publish する(Q0)。
#[test]
fn clicking_a_sort_chip_publishes_select_sort_key() {
    let items = fixture_items();
    let state = PaneState::new();
    let build = || pane_view(&state, &items, None, Dimensions::default(), Colors::default());

    let targets = collect_targets(build());
    let bounds = targets
        .iter()
        .find_map(|target| match target {
            Target::Text { content, .. } if content == "Type" => Some(target.bounds()),
            _ => None,
        })
        .expect("Type sort チップの Target::Text が見つからない");
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
        messages.contains(&Message::SelectSortKey(SortKey::Kind)),
        "Type チップの click が SelectSortKey(Kind) を publish しない: {messages:?}"
    );
}

// ---------------------------------------------------------------------------
// 2. 並べ替えチップは preview タブに漏れない(sort は実属性台帳専用)。
// ---------------------------------------------------------------------------

#[test]
fn sort_chips_do_not_leak_into_preview_tabs() {
    let items = fixture_items();
    for tab in [LibraryTab::Effects, LibraryTab::Create, LibraryTab::Panels] {
        let state = state_on(tab);
        let targets = collect_targets(pane_view(
            &state,
            &items,
            None,
            Dimensions::default(),
            Colors::default(),
        ));
        let texts = text_contents(&targets);
        assert!(
            !texts.iter().any(|content| content == "Date added"),
            "{tab:?} タブに media 専用の sort チップが漏れている: {texts:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// 3. grid/list 表示形式トグルが実際にカード配置を変える。
// ---------------------------------------------------------------------------

/// **本命**: Grid モードでは先頭2枚が同じ行(同じ y)に並び、List モードでは
/// 別々の行(異なる y)に積まれる — `columns_for`(lib.rs)の列数切替の
/// 描画結果を、アイコンボタンを click せず `Message::SelectViewMode` を
/// 直接 `PaneState::update` へ送って確かめる(icon-only ボタンは
/// `Target::Text` を持たないため、他の atlas テストと同じ「テキストの中心を
/// click する」手口が使えない — state 層からの遷移を経由する)。
#[test]
fn view_mode_toggle_changes_card_layout_from_row_to_stack() {
    let items = vec![
        item(0, "intro-clip", "video/mp4"),
        item(1, "logo-mark", "image/png"),
    ];

    let grid_state = PaneState::new();
    assert_eq!(grid_state.view_mode(), ViewMode::Grid, "既定は Grid のはず");
    let grid_targets = collect_targets(pane_view(
        &grid_state,
        &items,
        None,
        Dimensions::default(),
        Colors::default(),
    ));
    let grid_a = text_bounds(&grid_targets, "intro-clip");
    let grid_b = text_bounds(&grid_targets, "logo-mark");
    assert_eq!(
        grid_a.y, grid_b.y,
        "Grid モードで2枚のカードが同じ行(同じ y)に並んでいない"
    );
    assert_ne!(
        grid_a.x, grid_b.x,
        "Grid モードで2枚のカードが重なっている(x が同じ)"
    );

    let mut list_state = PaneState::new();
    list_state.update(Message::SelectViewMode(ViewMode::List));
    let list_targets = collect_targets(pane_view(
        &list_state,
        &items,
        None,
        Dimensions::default(),
        Colors::default(),
    ));
    let list_a = text_bounds(&list_targets, "intro-clip");
    let list_b = text_bounds(&list_targets, "logo-mark");
    assert_ne!(
        list_a.y, list_b.y,
        "List モードで2枚のカードが別行に積まれていない"
    );
}
