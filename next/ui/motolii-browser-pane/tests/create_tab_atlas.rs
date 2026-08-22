//! create タブの実体化(B36)ORACLE — Browser 第3切片。
//!
//! `library_tabs_atlas.rs` と同じ手口(`motolii-shell` を経由せず
//! `browser_pane::pane_view` を `iced_test::simulator` で直叩き)で、
//! - create タブに B36 消化分込みの4カード(Rectangle/Ellipse/Solid/Null)が
//!   **常設可視**で並ぶ(S6)
//! - **ダブルクリック=作成**: カードの2連続 press が
//!   `Message::CreateFromCard { kind }` を publish する(AE/Figma 慣習 S0 —
//!   実際のレイヤー生成は shell 結線・次波なので、ここは Message 境界まで)
//! - **シングルクリック=選択**: 1回の press では `SelectCard` だけが出て
//!   `CreateFromCard` は出ない(2つの動詞が混線しない)
//! ことを確かめる。
//!
//! **テストは書くが実行しない**(裁定189 追いつきターンの規律 — supervisor が
//! 波末に一括実行する)。

use iced_test::selector::{Candidate, Target};

use motolii_browser_pane::model::AssetListItem;
use motolii_browser_pane::{pane_view, CardKey, CreateKind, LibraryTab, Message, PaneState};
use motolii_store::AssetId;
use motolii_tokens_rs::{Colors, Dimensions};

/// `library_tabs_atlas.rs` と同じ「`find` を尽きるまで繰り返す」手口。
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
    vec![item(0, "intro-clip", "video/mp4")]
}

fn create_state() -> PaneState {
    let mut state = PaneState::new();
    state.update(Message::SelectTab(LibraryTab::Create));
    state
}

/// content の Text 中心へ `presses` 回 press(+release)して publish された
/// Message 列を返す(`tab_rail_symmetry_atlas.rs::click_text` の連打版 —
/// ダブルクリックは press 2連で `mouse::click::Kind::Double` になる、fork
/// `core/src/mouse/click.rs` の時間閾値は実時間で simulate 内の連打は必ず
/// 収まる)。
fn press_text_times(
    state: &PaneState,
    items: &[AssetListItem],
    content: &str,
    presses: usize,
) -> Vec<Message> {
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
    let events: Vec<iced::Event> = std::iter::repeat_n(
        [
            iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(
                iced::mouse::Button::Left,
            )),
        ],
        presses,
    )
    .flatten()
    .collect();
    let _ = ui.simulate(events);
    ui.into_messages().collect()
}

/// **本命(S6)**: create タブに mock 転写2枚+B36 消化2枚の計4カードが並ぶ。
#[test]
fn create_tab_shows_all_four_create_cards() {
    let items = fixture_items();
    let state = create_state();
    let targets = collect_targets(pane_view(
        &state,
        &items,
        Dimensions::default(),
        Colors::default(),
    ));
    let texts = text_contents(&targets);
    for expected in ["Rectangle", "Ellipse", "Solid", "Null"] {
        assert!(
            texts.iter().any(|content| content == expected),
            "create タブに {expected:?} カードが無い: {texts:?}"
        );
    }
    assert!(
        texts.iter().any(|content| content == "Results 4"),
        "Results 4 が出ない: {texts:?}"
    );
}

/// **本命(B36)**: カードのダブルクリック(press 2連)が
/// `Message::CreateFromCard` を publish する。カード4枚全て。
#[test]
fn double_clicking_a_create_card_publishes_create_from_card() {
    let items = fixture_items();
    let state = create_state();
    for (label, kind) in [
        ("Rectangle", CreateKind::Rectangle),
        ("Ellipse", CreateKind::Ellipse),
        ("Solid", CreateKind::Solid),
        ("Null", CreateKind::Null),
    ] {
        let messages = press_text_times(&state, &items, label, 2);
        assert!(
            messages.contains(&Message::CreateFromCard { kind }),
            "{label:?} のダブルクリックが CreateFromCard({kind:?}) を publish \
             しない: {messages:?}"
        );
    }
}

/// シングルクリックは選択だけ — `SelectCard` は出るが `CreateFromCard` は
/// **出ない**(2つの動詞の分離、AE/Figma 慣習 S0)。
#[test]
fn a_single_click_selects_but_never_creates() {
    let items = fixture_items();
    let state = create_state();
    let messages = press_text_times(&state, &items, "Solid", 1);
    assert!(
        messages.contains(&Message::SelectCard(CardKey::Preview("solid"))),
        "シングルクリックが SelectCard を publish しない: {messages:?}"
    );
    assert!(
        !messages
            .iter()
            .any(|message| matches!(message, Message::CreateFromCard { .. })),
        "シングルクリックで CreateFromCard が出ている: {messages:?}"
    );
}

/// ダブルクリックでも選択は失われない — `SelectCard` も一緒に publish される
/// (ダブル=選択+作成。作成だけ起きて選択が浮くことはない)。
#[test]
fn a_double_click_also_selects_the_card() {
    let items = fixture_items();
    let state = create_state();
    let messages = press_text_times(&state, &items, "Null", 2);
    assert!(
        messages.contains(&Message::SelectCard(CardKey::Preview("null"))),
        "ダブルクリックで SelectCard が出ない: {messages:?}"
    );
    assert!(
        messages.contains(&Message::CreateFromCard {
            kind: CreateKind::Null
        }),
        "ダブルクリックで CreateFromCard が出ない: {messages:?}"
    );
}

/// effects/panels のカード(`creates: None` — button 経路)は何度 press
/// しても `CreateFromCard` を publish しない(「作る」は create タブだけの
/// 語彙)。
#[test]
fn non_create_cards_never_publish_create_from_card() {
    let items = fixture_items();
    let mut state = PaneState::new();
    state.update(Message::SelectTab(LibraryTab::Effects));
    let messages = press_text_times(&state, &items, "Glow", 2);
    assert!(
        !messages
            .iter()
            .any(|message| matches!(message, Message::CreateFromCard { .. })),
        "effects カードの連打で CreateFromCard が出ている: {messages:?}"
    );
}
