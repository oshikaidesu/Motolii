//! Media Browser のダブルクリック下見入口の ORACLE。
//!
//! `motolii-shell`/Stage を経由せず Browser pane を直接動かし、media card の
//! 実面が「1 press=選択」「2 press=選択+PreviewMedia」を publish することだけ
//! を確認する。Source Monitor/再生 owner はまだ Shell WIRE の責任なので、この
//! テストはそこで止める。

use iced_test::selector::{Candidate, Target};

use motolii_browser_pane::model::AssetListItem;
use motolii_browser_pane::{pane_view, Message, PaneState, PreviewMedia};
use motolii_store::AssetId;
use motolii_tokens_rs::{Colors, Dimensions};

fn collect_targets(element: iced::Element<'_, Message>) -> Vec<Target> {
    let mut ui = iced_test::simulator(element);
    let mut found = Vec::new();
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
            Ok(target) if !found.contains(&target) => found.push(target),
            _ => break,
        }
        assert!(found.len() <= 5_000, "candidate 列挙が終わらない");
    }
    found
}

fn item() -> AssetListItem {
    AssetListItem {
        id: AssetId::from_raw(7),
        name: "intro-clip".to_owned(),
        kind: "video/mp4".to_owned(),
        path: Some("/tmp/intro-clip.mp4".to_owned()),
        fingerprint: "sha256:intro-clip".to_owned(),
        duration: None,
        status: motolii_store::AssetStatus::Unchecked,
    }
}

fn press_card(state: &PaneState, presses: usize) -> Vec<Message> {
    let items = vec![item()];
    let build = || {
        pane_view(
            state,
            &items,
            None,
            Dimensions::default(),
            Colors::default(),
        )
    };
    let target = collect_targets(build())
        .into_iter()
        .find_map(|target| match target {
            Target::Text { ref content, .. } if content == "intro-clip" => Some(target),
            _ => None,
        })
        .expect("media card の名前が atlas に無い");
    let bounds = target.bounds();
    let mut ui = iced_test::simulator(build());
    ui.point_at(iced::Point::new(
        bounds.x + bounds.width / 2.0,
        bounds.y + bounds.height / 2.0,
    ));
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

#[test]
fn media_card_double_click_publishes_preview() {
    let state = PaneState::new();
    let messages = press_card(&state, 2);
    assert!(
        messages.contains(&Message::PreviewMedia(PreviewMedia::new(
            AssetId::from_raw(7),
        ))),
        "media card のダブルクリックが PreviewMedia を publish しない: {messages:?}"
    );
}

#[test]
fn media_card_single_click_only_selects() {
    let state = PaneState::new();
    let messages = press_card(&state, 1);
    assert!(
        messages.iter().any(|message| matches!(
            message,
            Message::SelectCard(motolii_browser_pane::CardKey::Media(asset))
                if *asset == AssetId::from_raw(7)
        )),
        "media card の single click が選択を publish しない: {messages:?}"
    );
    assert!(
        !messages
            .iter()
            .any(|message| matches!(message, Message::PreviewMedia(_))),
        "single click で PreviewMedia が出ている: {messages:?}"
    );
}
