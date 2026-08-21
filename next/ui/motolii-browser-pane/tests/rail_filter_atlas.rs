//! B2 ORACLE: rail/filter の control が atlas(`iced_test::selector::Target`)
//! として見え、on_press が実際に Message を publish すること。
//!
//! `inspector_pane::tests::value_cell_legibility`(`motolii-inspector-pane/
//! tests/value_cell_legibility.rs`)と同じ手口 — `motolii-shell`(write-set
//! 外)を経由せず `browser_pane::view` を `iced_test::simulator` で直叩きする。
//! `motolii-shell` 側の `entrance_atlas_dump.rs`/`target_walk.rs` はまだこの
//! pane を歩かない(`Shell::view` へは B3 まで組み込まない、`lib.rs` 冒頭 doc
//! 参照)ので、この crate 自身が「atlas 器具と同じ手口で見える」ことを
//! 直接証明する。

use iced_test::selector::{Candidate, Target};

use motolii_browser_pane::model::AssetListItem;
use motolii_browser_pane::{view, Message, RailScope};
use motolii_store::AssetId;
use motolii_tokens_rs::{Colors, Dimensions};

/// `q0_fence.rs`/`value_cell_legibility.rs` と同じ「`find` を尽きるまで
/// 繰り返す」手口(`Simulator` に `find_all` が無い、iced_test 0.14.0 実測)。
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

/// **本命**: rail 4行(All media/Video/Images/Audio)+ filter shelf のチップ
/// (Video/Images/Audio)+ Clear ボタンが、すべて `Target::Text` の content
/// として atlas に現れる。検索欄も `Target::TextInput` として見える。
#[test]
fn rail_and_filter_shelf_controls_are_atlas_targets() {
    let items = fixture_items();
    let element = view(&items, RailScope::AllMedia, "", Dimensions::default(), Colors::default());
    let targets = collect_targets(element);

    let texts = text_contents(&targets);
    for expected in ["All media", "Video", "Images", "Audio", "Clear"] {
        assert!(
            texts.iter().any(|content| content == expected),
            "atlas に control の content={expected:?} が無い(見つかった text: {texts:?})"
        );
    }

    assert!(
        targets.iter().any(|target| matches!(target, Target::TextInput { .. })),
        "検索欄が Target::TextInput として atlas に現れない"
    );
}

/// rail scope ボタンを実際にクリックすると `Message::SelectScope` が publish
/// される(Q0: 「見えて押せそう」なだけでなく実際に効く)。
#[test]
fn clicking_a_scope_control_publishes_select_scope() {
    let items = fixture_items();
    let build = || view(&items, RailScope::AllMedia, "", Dimensions::default(), Colors::default());

    let targets = collect_targets(build());
    let bounds = targets
        .iter()
        .find_map(|target| match target {
            Target::Text { content, .. } if content == "Audio" => Some(target.bounds()),
            _ => None,
        })
        .expect("Audio ボタンの Target::Text が見つからない");
    let point = iced::Point::new(bounds.x + bounds.width / 2.0, bounds.y + bounds.height / 2.0);

    let mut ui = iced_test::simulator(build());
    ui.point_at(point);
    let _ = ui.simulate([
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)),
    ]);
    let messages: Vec<Message> = ui.into_messages().collect();

    assert!(
        messages.contains(&Message::SelectScope(RailScope::Audio)),
        "Audio ボタンのクリックが SelectScope(Audio) を publish しない: {messages:?}"
    );
}

/// Clear ボタンは `Message::ClearFilters` を publish する。
#[test]
fn clicking_clear_publishes_clear_filters() {
    let items = fixture_items();
    let build = || view(&items, RailScope::Video, "clip", Dimensions::default(), Colors::default());

    let targets = collect_targets(build());
    let bounds = targets
        .iter()
        .find_map(|target| match target {
            Target::Text { content, .. } if content == "Clear" => Some(target.bounds()),
            _ => None,
        })
        .expect("Clear ボタンの Target::Text が見つからない");
    let point = iced::Point::new(bounds.x + bounds.width / 2.0, bounds.y + bounds.height / 2.0);

    let mut ui = iced_test::simulator(build());
    ui.point_at(point);
    let _ = ui.simulate([
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)),
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)),
    ]);
    let messages: Vec<Message> = ui.into_messages().collect();

    assert!(
        messages.contains(&Message::ClearFilters),
        "Clear ボタンのクリックが ClearFilters を publish しない: {messages:?}"
    );
}

/// rail 選択で一覧の行数が実際に絞れる(結果件数の `Target::Text` を読む —
/// `catalog_view` の `Results N` 行)。
#[test]
fn selecting_a_scope_narrows_the_rendered_result_count() {
    let items = fixture_items();
    let all = collect_targets(view(
        &items,
        RailScope::AllMedia,
        "",
        Dimensions::default(),
        Colors::default(),
    ));
    let video_only = collect_targets(view(
        &items,
        RailScope::Video,
        "",
        Dimensions::default(),
        Colors::default(),
    ));

    let result_count = |targets: &[Target]| -> String {
        text_contents(targets)
            .into_iter()
            .find(|content| content.starts_with("Results "))
            .unwrap_or_else(|| panic!("Results 行が見つからない: {targets:?}"))
    };

    assert_eq!(result_count(&all), "Results 3");
    assert_eq!(result_count(&video_only), "Results 1", "Video scope で1件に絞れていない");
}
