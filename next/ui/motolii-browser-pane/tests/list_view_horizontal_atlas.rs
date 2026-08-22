//! Browser 第5切片(List 表示の水平カード)ORACLE — mock
//! `browser-library.css:304-307`
//! `.libraryBrowser[data-view="list"]` の3宣言(サムネ小46px+テキスト右)を
//! `motolii-taffy::TaffyBox` 経由で組んだ結果を、`organize_slice_atlas.rs`/
//! `rail_filter_atlas.rs` と同じ手口(`motolii-shell` を経由せず
//! `browser_pane::pane_view` を `iced_test::simulator` で直叩き)で固定する。
//!
//! **未実行**(発注書の検収線は `cargo check --tests -p motolii-browser-pane`
//! まで — このファイルの全テストはコンパイルのみ確認済みで、実行はしていない)。
//!
//! 対象:
//! 1. List モードでは thumb(種別グリフの `Target::Text`)が名前の**左**に来る
//!    (mock の水平カード — グリフは button 経由描画された `Target::Text` として
//!    atlas に現れる、[`organize_slice_atlas.rs`] 冒頭 doc と同じ手口)。
//! 2. List モードでは同一カードの名前/caption が縦に積まれたまま
//!    (thumb+テキストの水平化は「外側」だけ — テキスト内部の積み方は不変)。
//! 3. List モードでは名前欄が Grid モードよりずっと広い(`Length::Fill` で
//!    行幅いっぱいへ伸びる — mock `grid-template-columns:1fr` の意味どおり)。
//! 4. create タブ(preview カード経由、`mouse_area` 経路)でも同じ水平化が効く
//!    (media/preview の両カードが [`card_body`] を共有する契約の実測)。
//! 5. `Dimensions::browser_list_thumb_width` が mock 実測(46px、
//!    `browser-library.css:306`)と一致する(JSON 正本の値そのものを固定する
//!    単純な回帰 oracle)。

use iced_test::selector::{Candidate, Target};

use motolii_browser_pane::model::AssetListItem;
use motolii_browser_pane::{pane_view, LibraryTab, Message, PaneState, ViewMode};
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

fn list_state() -> PaneState {
    let mut state = PaneState::new();
    state.update(Message::SelectViewMode(ViewMode::List));
    state
}

// ---------------------------------------------------------------------------
// 1+2. media タブ: thumb が名前の左に来る/名前・caption は縦積みのまま。
// ---------------------------------------------------------------------------

/// **本命**: List モードで video 種別カードの thumb グリフ(`▣`、
/// `model::Category::glyph`)が名前(`intro-clip`)の**左**に描かれる —
/// mock の水平カード(`.libraryThumb{width:46px}` 左+`.cardCopy{flex:1}` 右)
/// の転写そのもの。
#[test]
fn list_mode_places_the_thumb_glyph_to_the_left_of_the_name() {
    let items = vec![item(0, "intro-clip", "video/mp4")];
    let state = list_state();
    let targets = collect_targets(pane_view(
        &state,
        &items,
        None,
        Dimensions::default(),
        Colors::default(),
    ));

    let thumb = text_bounds(&targets, "▣");
    let name = text_bounds(&targets, "intro-clip");

    assert!(
        thumb.x + thumb.width <= name.x,
        "List モードで thumb({thumb:?})が名前({name:?})の左に来ていない"
    );
}

/// caption(`Video · —`、[`model::format_duration`] の空丸め)は名前の**下**
/// (同じ x 近辺・大きい y)に残る — 水平化は thumb/テキスト間だけで、
/// テキスト内部(名前→caption)の縦積みは Grid モードと不変。
#[test]
fn list_mode_keeps_the_name_and_caption_stacked_vertically() {
    let items = vec![item(0, "intro-clip", "video/mp4")];
    let state = list_state();
    let targets = collect_targets(pane_view(
        &state,
        &items,
        None,
        Dimensions::default(),
        Colors::default(),
    ));

    let name = text_bounds(&targets, "intro-clip");
    let caption = text_bounds(&targets, "Video · —");

    assert!(
        caption.y > name.y,
        "List モードで caption({caption:?})が名前({name:?})の下に来ていない"
    );
    // 縦積み = 同じ列(x はほぼ同じ)。TaffyBox の測定近似(widget.rs 冒頭 doc
    // の既知の近似)を見込んで許容誤差を持たせる。
    assert!(
        (name.x - caption.x).abs() < 1.0,
        "名前/caption が同じ x 列に積まれていない: name={name:?} caption={caption:?}"
    );
}

// ---------------------------------------------------------------------------
// 3. List モードは名前欄が Grid モードよりずっと広い(Length::Fill)。
// ---------------------------------------------------------------------------

/// **本命**: List モードの名前欄幅(`Length::Fill` — 行いっぱいから thumb 幅
/// を引いた残り)は、Grid モードの固定カード幅(`CARD_WIDTH_ROW_HEIGHT_RATIO`
/// × row_height = 120px)よりずっと広い(mock `grid-template-columns:1fr` の
/// 意味そのもの — list は1カラムが行幅いっぱい)。
#[test]
fn list_mode_name_field_is_much_wider_than_the_fixed_grid_card() {
    let items = vec![item(0, "intro-clip", "video/mp4")];

    let grid_targets = collect_targets(pane_view(
        &PaneState::new(),
        &items,
        None,
        Dimensions::default(),
        Colors::default(),
    ));
    let grid_name = text_bounds(&grid_targets, "intro-clip");

    let list_targets = collect_targets(pane_view(
        &list_state(),
        &items,
        None,
        Dimensions::default(),
        Colors::default(),
    ));
    let list_name = text_bounds(&list_targets, "intro-clip");

    assert!(
        list_name.width > grid_name.width,
        "List モードの名前欄({list_name:?})が Grid モードの固定カード幅\
         ({grid_name:?})より広くなっていない"
    );
}

// ---------------------------------------------------------------------------
// 4. create タブ(mouse_area 経路)でも同じ水平化が効く。
// ---------------------------------------------------------------------------

/// **本命**: create タブの preview カード(`mouse_area` 経路、B36)も
/// [`card_body`] を共有するため、List モードで同じ水平化(thumb 左+名前右)
/// になる — Rectangle カードの thumb グリフ(`□`)で確認する。
#[test]
fn list_mode_horizontal_layout_also_applies_to_create_tab_cards() {
    let mut state = PaneState::new();
    state.update(Message::SelectTab(LibraryTab::Create));
    state.update(Message::SelectViewMode(ViewMode::List));

    let targets = collect_targets(pane_view(
        &state,
        &[],
        None,
        Dimensions::default(),
        Colors::default(),
    ));

    let thumb = text_bounds(&targets, "□");
    let name = text_bounds(&targets, "Rectangle");

    assert!(
        thumb.x + thumb.width <= name.x,
        "create タブの List モードで thumb({thumb:?})が名前({name:?})の左に\
         来ていない"
    );
}

// ---------------------------------------------------------------------------
// 5. JSON 正本の値そのものを固定する単純な回帰 oracle。
// ---------------------------------------------------------------------------

/// **ORACLE**: `Dimensions::browser_list_thumb_width` は mock 実測
/// (`browser-library.css:306` `.libraryThumb{width:46px}`)と一致する。
#[test]
fn browser_list_thumb_width_matches_the_mock_measurement() {
    assert_eq!(Dimensions::default().browser_list_thumb_width, 46.0);
}
