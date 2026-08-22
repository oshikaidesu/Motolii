//! 利用者裁定(2026-08-22 朝、実窓不合格)による Browser pane 比率台帳の機械照合
//! (`docs/reviews/2026-08-22-browser-ratio-ledger.md` の実測本体はそちら —
//! この柵はその台帳の主要数値を「実装定数を `use` した両側チェック」として
//! 固定するだけの regression lock、`motolii-inspector-pane/tests/
//! inspector_ratio_ledger.rs`・`crates/motolii-shell-iced/tests/
//! css_metrics_oracle.rs` と同型の手口)。
//!
//! ## モック側はなぜ literal 定数か
//!
//! `next/` workspace は Blitz 機械抽出器具(`motolii-ui::css_metrics`)を
//! members に含まない(inspector_ratio_ledger.rs 冒頭 doc と同じ事情、
//! `next/Cargo.toml` 実測)。モック側は `next/reference/mocks/
//! browser-library.css` からの引用リテラル(行番号を doc comment に明記)
//! として pin し、実装側だけ `motolii_browser_pane`/`motolii_tokens_rs` の
//! 公開 API を `use` して読む。

use iced_test::selector::{Candidate, Target};

use motolii_browser_pane::model::AssetListItem;
use motolii_browser_pane::{view, Message, RailScope, FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO};
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
    }
}

fn fixture_items() -> Vec<AssetListItem> {
    vec![item(0, "intro-clip", "video/mp4")]
}

/// `next/reference/mocks/browser-library.css:206` `.filterShelf button,
/// .editorTags button{border-radius:8px}`(Clear ボタンも html:484
/// `class="clearFilter"` として同一セレクタに同居、css:229 参照)。
const MOCK_FILTER_CHIP_CORNER_RADIUS_PX: f32 = 8.0;

/// 台帳の主張(1): filter チップ/Clear の角丸比率定数は、既定 `row_height`
/// (20px)を掛けた時に mock の絶対px(8px)と厳密に一致する。
#[test]
fn filter_chip_corner_radius_ratio_matches_the_mock_pixel_value() {
    let dims = Dimensions::default();
    let radius_px = FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO * dims.row_height;
    assert_eq!(
        radius_px, MOCK_FILTER_CHIP_CORNER_RADIUS_PX,
        "FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO × row_height が \
         browser-library.css:206 の border-radius:8px と一致しない(実測 {radius_px})"
    );
}

/// 台帳の主張(2): rail 行文字・filter チップ文字・検索欄文字・結果件数文字は
/// 全て `dims.micro_text`(8) と同じ大きさで描かれる(`browser-library.css`
/// 実測 — `.locationRow`(css:150)/`.filterShelf button`(css:211)/
/// `#library-search`(css:83)/`.resultSummary strong,span`(css:225-226)が
/// 例外なく8pxであるのに対し、旧実装は `dims.caption_text`(9)を使っていた —
/// `card_view` の名前/caption(既に `micro_text` 確定、`lib.rs` の
/// `card_view` 参照)と同じ高さで描かれることをもって「micro_text を使って
/// いる」ことの間接証拠とする(このレーンは新しい pub API を増やさず、
/// `rail_filter_atlas.rs` と同じ atlas 手口だけで検証する方針のため)。
#[test]
fn rail_and_filter_and_search_and_summary_text_render_at_the_same_size_as_card_copy() {
    let items = fixture_items();
    let element = view(
        &items,
        RailScope::AllMedia,
        "",
        Dimensions::default(),
        Colors::default(),
    );
    let targets = collect_targets(element);

    // card copy(既に `micro_text` = 8 が確定している基準、`card_view` 参照)。
    let card_copy_height = text_bounds(&targets, "intro-clip").height;

    for content in ["Video", "Clear", "Results 1"] {
        let height = text_bounds(&targets, content).height;
        assert_eq!(
            height, card_copy_height,
            "{content:?} の描画高さ({height})が card copy の高さ({card_copy_height}, \
             micro_text=8基準)と一致しない — caption_text(9)のまま残っている疑い"
        );
    }
}
