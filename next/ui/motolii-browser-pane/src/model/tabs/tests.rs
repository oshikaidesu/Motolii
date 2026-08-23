use super::*;
use crate::model::test_support::mixed_ledger;

// -----------------------------------------------------------------
// タブ4種(mock `.libraryTabs`)+ preview-local カタログ(B3 転写の
// 取り残し回収、2026-08-22 実窓不合格対応)。
// -----------------------------------------------------------------

/// **ORACLE**: mock html:412-415 のタブ並び(`data-tab="media"/"effects"/
/// "create"/"panels"`)とラベルをそのまま転写している。
#[test]
fn library_tabs_follow_the_mock_order_and_labels() {
    assert_eq!(
        LIBRARY_TABS,
        [
            LibraryTab::Media,
            LibraryTab::Effects,
            LibraryTab::Create,
            LibraryTab::Panels
        ]
    );
    let labels: Vec<&str> = LIBRARY_TABS.into_iter().map(LibraryTab::label).collect();
    assert_eq!(labels, ["Media", "Effects", "Create", "Panels"]);
}

/// media タブは Document 台帳投影の経路であって静的カタログを持たない
/// (発注: 「media タブは従来どおり Document 台帳投影で、静的データを
/// 混ぜない」)。
#[test]
fn media_tab_has_no_preview_catalog() {
    assert!(preview_catalog(LibraryTab::Media).is_empty());
}

/// effects の preview カタログは mock html:522-530 の3枚+実在 plugin の
/// Glow(発注: 「effects は実在 plugin 名 Glow を含めてよい」)+ Mask
/// (裁定205 施工第2号 §A — 新規レイヤーを作らないので Create タブではなく
/// ここに置く判断、`EFFECTS_PREVIEW` doc 参照)。
#[test]
fn effects_preview_catalog_contains_the_mock_cards_and_glow() {
    let names: Vec<&str> = preview_catalog(LibraryTab::Effects)
        .iter()
        .map(|card| card.name)
        .collect();
    for expected in ["Echo Bloom", "Opacity", "Sine", "Glow", "Mask"] {
        assert!(
            names.contains(&expected),
            "effects カタログに {expected:?} が無い: {names:?}"
        );
    }
}

/// create/panels の preview カタログ: create は mock html:532-537 の2枚を
/// 先頭に保ち、B36 消化分(Solid/Null — `CreateKind` doc の消化台帳)を
/// 末尾へ。panels は mock html:539-547 のカードそのまま。
#[test]
fn create_and_panels_preview_catalogs_match_the_mock_cards() {
    let create: Vec<&str> = preview_catalog(LibraryTab::Create)
        .iter()
        .map(|card| card.name)
        .collect();
    assert_eq!(create, ["Rectangle", "Ellipse", "Solid", "Null", "Text"]);

    let panels: Vec<&str> = preview_catalog(LibraryTab::Panels)
        .iter()
        .map(|card| card.name)
        .collect();
    assert_eq!(panels, ["Asset tagging", "Notes", "Export notes"]);
}

/// **ORACLE**: media タブの catalog は Document 台帳投影([`visible`] と
/// 同じ絞り込み)だけで組まれ、静的 preview データは1枚も混ざらない。
#[test]
fn media_catalog_projects_the_document_ledger_only() {
    let items = mixed_ledger();
    let cards = catalog(
        LibraryTab::Media,
        &items,
        RailScope::AllMedia,
        PreviewScope::All,
        "",
    );
    assert_eq!(cards.len(), items.len());
    assert!(
        cards
            .iter()
            .all(|card| matches!(card, CatalogCard::Media(_))),
        "media タブに Preview カードが混ざっている: {cards:?}"
    );
}

/// media タブの catalog は scope/query の絞り込み([`visible`])をそのまま通す。
#[test]
fn media_catalog_still_narrows_by_scope_and_query() {
    let items = mixed_ledger();
    let cards = catalog(
        LibraryTab::Media,
        &items,
        RailScope::Video,
        PreviewScope::All,
        "",
    );
    assert_eq!(cards.len(), 2, "video 2件へ絞れていない: {cards:?}");
}

/// **ORACLE**: effects タブの catalog は静的 preview カタログだけで組まれ、
/// Document 台帳の素材は1枚も混ざらない(タブ別のカタログ投影)。
#[test]
fn effects_catalog_ignores_the_media_ledger() {
    let items = mixed_ledger();
    let cards = catalog(
        LibraryTab::Effects,
        &items,
        RailScope::AllMedia,
        PreviewScope::All,
        "",
    );
    assert_eq!(cards.len(), preview_catalog(LibraryTab::Effects).len());
    assert!(
        cards
            .iter()
            .all(|card| matches!(card, CatalogCard::Preview(_))),
        "effects タブに台帳素材が混ざっている: {cards:?}"
    );
}

/// preview タブでも検索文字列は効く(mock は `data-search` を全タブで
/// 照合する — 名前の部分一致・大小無視で転写)。
#[test]
fn preview_catalog_narrows_by_query_name_match() {
    let cards = catalog(
        LibraryTab::Effects,
        &[],
        RailScope::AllMedia,
        PreviewScope::All,
        "GLO",
    );
    assert_eq!(cards.len(), 1, "Glow 1件へ絞れていない: {cards:?}");
}

/// rail scope は media 種別(Video/Images/Audio)の語彙なので preview タブ
/// には効かない(mock でも `chooseTab` が非 media タブで `source='all'` へ
/// 戻す = scope は media 専用)。
#[test]
fn preview_catalog_ignores_the_media_rail_scope() {
    let cards = catalog(
        LibraryTab::Create,
        &[],
        RailScope::Video,
        PreviewScope::All,
        "",
    );
    assert_eq!(cards.len(), preview_catalog(LibraryTab::Create).len());
}

// -----------------------------------------------------------------
// 構造の対称化(2026-08-22): タブ別 rail カテゴリ + preview_visible。
// -----------------------------------------------------------------

/// **ORACLE**: タブ別 rail カテゴリは mock `.tabScoped-*` の掲載順そのまま
/// (html:444-446 / 454-455 / 463-465 — S0 慣習順)+ 末尾の `Masks`
/// (裁定205 施工第2号 §A で追加、mock に無い新規カテゴリ)。media は空
/// (`RailScope` の語彙が正)。
#[test]
fn preview_tags_follow_the_mock_declaration_per_tab() {
    assert!(preview_tags(LibraryTab::Media).is_empty());
    assert_eq!(
        preview_tags(LibraryTab::Effects),
        [
            PreviewTag::Color,
            PreviewTag::Utility,
            PreviewTag::Animation,
            PreviewTag::Masks,
        ]
    );
    assert_eq!(
        preview_tags(LibraryTab::Create),
        [PreviewTag::Shapes, PreviewTag::BuiltIn]
    );
    assert_eq!(
        preview_tags(LibraryTab::Panels),
        [PreviewTag::Tags, PreviewTag::Notes, PreviewTag::Export]
    );
}

/// rail 先頭の「全件」行ラベルは mock の `data-source="all"` 行そのまま。
#[test]
fn all_labels_follow_the_mock_rows() {
    assert_eq!(LibraryTab::Media.all_label(), "All media");
    assert_eq!(LibraryTab::Effects.all_label(), "All effects");
    assert_eq!(LibraryTab::Create.all_label(), "All create");
    assert_eq!(LibraryTab::Panels.all_label(), "All panels");
}

/// タグ語彙の整合: 各タブのカタログのカードが持つタグは、そのタブの rail
/// カテゴリに全て現れる(rail に無いタグで絞れないカードを作らない)。
/// 逆に各タグの `tab()` は自分が掲載されるタブと一致する。
#[test]
fn preview_card_tags_belong_to_their_tab_rail() {
    for tab in [LibraryTab::Effects, LibraryTab::Create, LibraryTab::Panels] {
        let rail = preview_tags(tab);
        for tag in rail {
            assert_eq!(tag.tab(), tab, "{tag:?} の tab() が掲載タブと不一致");
        }
        for card in preview_catalog(tab) {
            assert!(
                !card.tags.is_empty(),
                "{:?} がどのカテゴリにも属さない(rail から到達不能)",
                card.name
            );
            for tag in card.tags {
                assert!(
                    rail.contains(tag),
                    "{:?} のタグ {tag:?} が {tab:?} の rail に無い",
                    card.name
                );
            }
        }
    }
}

/// **ORACLE**: `PreviewScope::Tag` で preview カタログが絞れる(mock の
/// `data-tag-filter` 照合の転写 — effects の Color は Echo Bloom+Glow)。
#[test]
fn preview_visible_narrows_by_tag() {
    let color: Vec<&str> = preview_visible(
        LibraryTab::Effects,
        PreviewScope::Tag(PreviewTag::Color),
        "",
    )
    .iter()
    .map(|card| card.name)
    .collect();
    assert_eq!(color, ["Echo Bloom", "Glow"]);

    let notes: Vec<&str> =
        preview_visible(LibraryTab::Panels, PreviewScope::Tag(PreviewTag::Notes), "")
            .iter()
            .map(|card| card.name)
            .collect();
    assert_eq!(notes, ["Notes"]);
}

/// create のシェイプ2枚は Shapes/Built-in の両カテゴリに属する(mock
/// `data-tags="shape builtin"`)。B36 消化分(Solid/Null)は Built-in
/// のみ — `Shapes` scope は2枚のまま、`Built-in` scope は全4枚。
#[test]
fn create_cards_match_both_their_categories() {
    let shapes = preview_visible(LibraryTab::Create, PreviewScope::Tag(PreviewTag::Shapes), "");
    assert_eq!(shapes.len(), 2, "Shapes でシェイプ2枚が残らない");

    let builtin =
        preview_visible(LibraryTab::Create, PreviewScope::Tag(PreviewTag::BuiltIn), "");
    assert_eq!(builtin.len(), 5, "Built-in で create 全5枚が残らない");
}

// -----------------------------------------------------------------
// B36: create タブの実体化 — `CreateKind` の語彙と `creates` の壁。
// -----------------------------------------------------------------

/// **ORACLE**: create タブのカードは全枚が `creates: Some` を宣言する
/// (「作る」を発火できないカードを create タブに置かない — Q0 触れそうで
/// 触れない物は不合格)。id → kind の対応も固定する。
#[test]
fn every_create_card_declares_its_create_kind() {
    let expected: [(&str, CreateKind); 5] = [
        ("rectangle", CreateKind::Rectangle),
        ("ellipse", CreateKind::Ellipse),
        ("solid", CreateKind::Solid),
        ("null", CreateKind::Null),
        ("text", CreateKind::Text),
    ];
    let cards = preview_catalog(LibraryTab::Create);
    assert_eq!(cards.len(), expected.len());
    for (card, (id, kind)) in cards.iter().zip(expected) {
        assert_eq!(card.id, id);
        assert_eq!(
            card.creates,
            Some(kind),
            "{:?} の creates が {kind:?} でない",
            card.name
        );
    }
}

/// effects/panels のカードは `creates: None`(「作る」は create タブ
/// だけの語彙 — 型の壁が2系統の混線を防ぐのと同じ形)。Glow/Mask は
/// `applies_to_selection` が `Some` になる代わりに `creates` は `None` の
/// まま(2つの意図の型が混ざらない、[`PreviewCard::applies_to_selection`]
/// doc 参照)。
#[test]
fn non_create_cards_never_declare_a_create_kind() {
    for tab in [LibraryTab::Effects, LibraryTab::Panels] {
        for card in preview_catalog(tab) {
            assert_eq!(
                card.creates, None,
                "{tab:?} の {:?} が creates を宣言している",
                card.name
            );
        }
    }
}

/// **ORACLE**(裁定205 施工第2号): Glow は `ApplyEffect("motolii.glow")`・
/// Mask は `AddMask` を宣言する。それ以外の effects カード(mock 転写の
/// 見せ札)は両方とも `None` のまま(`creates` も `applies_to_selection`
/// も無い = 何も発火できない、mock の「見せるだけ」の意図どおり)。
#[test]
fn effects_action_cards_declare_their_selection_action() {
    let cards = preview_catalog(LibraryTab::Effects);
    let by_id = |id: &str| cards.iter().find(|card| card.id == id).unwrap();
    assert_eq!(
        by_id("glow").applies_to_selection,
        Some(SelectionAction::ApplyEffect("motolii.glow"))
    );
    assert_eq!(
        by_id("mask").applies_to_selection,
        Some(SelectionAction::AddMask)
    );
    for id in ["echo-bloom", "opacity", "sine"] {
        assert_eq!(
            by_id(id).applies_to_selection,
            None,
            "{id:?} は見せ札のはずなのに applies_to_selection を持っている"
        );
    }
}

/// `All` は絞らない・並べ替えない(media の `AllMedia` と同じ意味)。
#[test]
fn preview_visible_all_keeps_the_declaration_order() {
    let cards = preview_visible(LibraryTab::Effects, PreviewScope::All, "");
    let names: Vec<&str> = cards.iter().map(|card| card.name).collect();
    assert_eq!(names, ["Echo Bloom", "Opacity", "Sine", "Glow", "Mask"]);
}

/// scope と query は同時に効く(AND — media の `visible` と同じ形)。
#[test]
fn preview_scope_and_query_combine_with_and_semantics() {
    let cards = preview_visible(
        LibraryTab::Effects,
        PreviewScope::Tag(PreviewTag::Color),
        "glo",
    );
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].name, "Glow");

    // Color scope だが Utility の名前で検索 → 0件。
    assert!(preview_visible(
        LibraryTab::Effects,
        PreviewScope::Tag(PreviewTag::Color),
        "opacity"
    )
    .is_empty());
}

/// カードの静的 id(mock `data-item`)は同一タブ内で一意 — [`CardKey::
/// Preview`] の同定子として衝突しない。
#[test]
fn preview_card_ids_are_unique_within_a_tab() {
    for tab in [LibraryTab::Effects, LibraryTab::Create, LibraryTab::Panels] {
        let ids: Vec<&str> = preview_catalog(tab).iter().map(|card| card.id).collect();
        let mut deduped = ids.clone();
        deduped.sort_unstable();
        deduped.dedup();
        assert_eq!(deduped.len(), ids.len(), "{tab:?} の id が重複: {ids:?}");
    }
}
