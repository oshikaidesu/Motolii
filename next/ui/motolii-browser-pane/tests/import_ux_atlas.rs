//! 取り込み UX 続編(B08)の空状態 ORACLE — Results の空状態は**2値を区別**
//! する(裁定185 の精神: 説明文は最小の1句):
//! - 台帳自体が空(まだ何も取り込んでいない)= 「Drop files here」 —
//!   取り込みの入口(drop)を言う。
//! - 台帳はあるが絞り込みで0件 = 「No matches」。
//! - preview タブ(静的カタログ — 「台帳が空」の面が無い)= 「No matches」。
//!
//! `library_tabs_atlas.rs` と同じ手口(`pane_view` を `iced_test::simulator`
//! で直叩き)。**テストは書くが実行しない**(裁定189 追いつきターンの規律)。

use iced_test::selector::{Candidate, Target};

use motolii_browser_pane::model::AssetListItem;
use motolii_browser_pane::{pane_view, LibraryTab, Message, PaneState};
use motolii_store::AssetId;
use motolii_tokens_rs::{Colors, Dimensions};

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

fn texts_on(state: &PaneState, items: &[AssetListItem]) -> Vec<String> {
    text_contents(&collect_targets(pane_view(
        state,
        items,
        Dimensions::default(),
        Colors::default(),
    )))
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

/// **本命**: 台帳が空の media タブは「Drop files here」— 取り込みの入口を
/// 1句で言う(「No matches」ではない — まだ探して外した訳ではない)。
#[test]
fn an_empty_ledger_invites_a_drop() {
    let state = PaneState::new();
    let texts = texts_on(&state, &[]);
    assert!(
        texts.iter().any(|content| content == "Drop files here"),
        "空台帳の media タブに Drop files here が無い: {texts:?}"
    );
    assert!(
        !texts.iter().any(|content| content == "No matches"),
        "空台帳で No matches が出ている(2値の混線): {texts:?}"
    );
}

/// 台帳はあるが絞り込みで0件 = 「No matches」(drop の誘いは出ない —
/// 素材はもうある)。
#[test]
fn a_filtered_out_ledger_says_no_matches() {
    let items = vec![item(0, "intro-clip", "video/mp4")];
    let mut state = PaneState::new();
    state.update(Message::QueryChanged("zzz-no-such-asset".to_owned()));
    let texts = texts_on(&state, &items);
    assert!(
        texts.iter().any(|content| content == "No matches"),
        "絞り込み0件で No matches が出ない: {texts:?}"
    );
    assert!(
        !texts.iter().any(|content| content == "Drop files here"),
        "台帳があるのに Drop files here が出ている: {texts:?}"
    );
}

/// preview タブの絞り込み0件も同じ「No matches」(旧「No matching items」の
/// 文言二重化を畳む — 同じ意味に別の句を使わない)。
#[test]
fn preview_tabs_share_the_no_matches_copy() {
    let mut state = PaneState::new();
    state.update(Message::SelectTab(LibraryTab::Effects));
    state.update(Message::QueryChanged("zzz-no-such-effect".to_owned()));
    let texts = texts_on(&state, &[]);
    assert!(
        texts.iter().any(|content| content == "No matches"),
        "preview タブの0件で No matches が出ない: {texts:?}"
    );
}
