//! タブ(SP-6 分割: 元 `model.rs` から移送 — タブ4種+preview-local カタログ、
//! `catalog`/`preview_visible` によるタブ別カタログ投影)。

use super::projection::{visible, AssetListItem};
use super::rail::RailScope;
use motolii_store::AssetId;

/* motolii-component
id = "browser.shape_operator_catalog"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["ShapeOpKind", "EFFECTS_PREVIEW"]
meaning = ["ApplyOp", "ShapeOps"]
evaluation = ["preview_tags", "effects_action_cards_declare_their_selection_action"]
render = ["ShapeOps", "preview_catalog"]
observable = ["effects_action_cards_declare_their_selection_action"]
*/

// ---------------------------------------------------------------------------
// タブ4種+preview-local カタログ(mock `.libraryTabs`/`data-tab`、B3 転写の
// 取り残し回収 — 利用者実窓不合格 2026-08-22 への対応)。
// ---------------------------------------------------------------------------

/// Browser のタブ(mock html:412-415 `data-tab="media"/"effects"/"create"/
/// "panels"` の4値そのまま)。既定は media(mock `state = {tab: 'media'}`)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibraryTab {
    #[default]
    Media,
    Effects,
    Create,
    Panels,
}

/// タブ帯の並び順(mock の掲載順どおり — Media → Effects → Create → Panels)。
/// view 側・試験側の両方がこの1本の並びを共有する(`RAIL_SCOPES` と同じ形)。
pub const LIBRARY_TABS: [LibraryTab; 4] = [
    LibraryTab::Media,
    LibraryTab::Effects,
    LibraryTab::Create,
    LibraryTab::Panels,
];

impl LibraryTab {
    /// mock のタブラベルそのまま(html:412-415)。
    pub fn label(self) -> &'static str {
        match self {
            Self::Media => "Media",
            Self::Effects => "Effects",
            Self::Create => "Create",
            Self::Panels => "Panels",
        }
    }

    /// rail 先頭の「全件」行のラベル(mock `.tabScoped-*` の
    /// `data-source="all"` 行 — html:427 `All media`/443 `All effects`/
    /// 453 `All create`/462 `All panels` そのまま)。media タブの rail は
    /// 従来どおり [`RailScope::AllMedia`] の label が正だが、語彙は一致させる。
    pub fn all_label(self) -> &'static str {
        match self {
            Self::Media => "All media",
            Self::Effects => "All effects",
            Self::Create => "All create",
            Self::Panels => "All panels",
        }
    }
}

// ---------------------------------------------------------------------------
// 非 media タブの rail/filter(構造の対称化 — 利用者実窓指摘 2026-08-22
// 「Browser の構造が media タブにしか適用されていない」への対応)。
// ---------------------------------------------------------------------------

/// 非 media タブの rail 行/フィルタチップのカテゴリ(mock が**タブ別に宣言
/// している物**の転写 — `.tabScoped-effects/-create/-panels` の
/// `data-tag-filter` 行(html:444-446/454-455/463-465)と
/// `.filterGroup[data-filter-group="effects"/"create"/"panels"]` のチップ
/// (html:486-494)は同じタグ語彙を共有する。media の `RailScope` が rail と
/// filter shelf の両方を1つの語彙で賄うのと同型)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewTag {
    /// effects の Color 系カード(実装済み provider のみを掲載)。
    Color,
    /// create(mock `data-tag-filter="shape"`)。
    Shapes,
    /// create(mock `data-tag-filter="builtin"`)。
    BuiltIn,
    /// panels(mock `data-tag-filter="tags"`)。
    Tags,
    /// panels(mock `data-tag-filter="notes"`)。
    Notes,
    /// panels(mock `data-tag-filter="export"`)。
    Export,
    /// effects(mock に無い新規カテゴリ — 裁定205 施工第2号 §A。マスクは
    /// mock 転写ではなく Motolii 独自の追加なので、Color/Utility/Animation の
    /// どれにも属さない専用タグを1つ起こす、`EFFECTS_PREVIEW` の Mask カード
    /// doc 参照)。
    Masks,
    /// effects(mock に無い新規カテゴリ — 2026-08-24「ブラウザに8枚の札」発注。
    /// `motolii_vector::OpKind` 7種(選択中シェイプへ積む演算子)専用タグ。
    /// Masks と同じ理由で Color/Utility/Animation のどれにも属さない —
    /// マスクと違い「シェイプの中身を演算子で加工する」語彙なので Masks とも
    /// 別カテゴリにする(`SHAPE_OP_PREVIEW` doc 参照)。
    ShapeOps,
}

impl PreviewTag {
    /// mock の表示文言そのまま(rail 行とチップで共通 — html:444-446/454-455/
    /// 463-465 と html:487/490/493 は同ラベル)。`Masks` は mock に無いので
    /// 自然な英語の慣用句を採る(`SortKey::label` 等の新規 UI と同じ立場)。
    pub fn label(self) -> &'static str {
        match self {
            Self::Color => "Color",
            Self::Shapes => "Shapes",
            Self::BuiltIn => "Built-in",
            Self::Tags => "Tags",
            Self::Notes => "Notes",
            Self::Export => "Export",
            Self::Masks => "Masks",
            Self::ShapeOps => "Shape ops",
        }
    }

    /// このタグが属するタブ(タグ語彙はタブ別 — 試験がカタログとの整合を
    /// 照合する口)。
    pub fn tab(self) -> LibraryTab {
        match self {
            Self::Color | Self::Masks | Self::ShapeOps => {
                LibraryTab::Effects
            }
            Self::Shapes | Self::BuiltIn => LibraryTab::Create,
            Self::Tags | Self::Notes | Self::Export => LibraryTab::Panels,
        }
    }
}

/// effects タブの rail カテゴリ並び。実装済み provider の `Color`、
/// マスク、シェイプ演算子だけを掲載し、未実装の mock-only effect へ入口を作らない。
pub const EFFECTS_TAGS: [PreviewTag; 3] = [
    PreviewTag::Color,
    PreviewTag::Masks,
    PreviewTag::ShapeOps,
];

/// create タブの rail カテゴリ並び(mock html:454-455 の掲載順)。
pub const CREATE_TAGS: [PreviewTag; 2] = [PreviewTag::Shapes, PreviewTag::BuiltIn];

/// panels タブの rail カテゴリ並び(mock html:463-465 の掲載順)。
pub const PANELS_TAGS: [PreviewTag; 3] = [PreviewTag::Tags, PreviewTag::Notes, PreviewTag::Export];

/// タブごとの rail カテゴリ(=フィルタチップ)並び。**media は空**(media は
/// 従来どおり [`RailScope`] の語彙 — この関数は非 media タブ専用)。
pub fn preview_tags(tab: LibraryTab) -> &'static [PreviewTag] {
    match tab {
        LibraryTab::Media => &[],
        LibraryTab::Effects => &EFFECTS_TAGS,
        LibraryTab::Create => &CREATE_TAGS,
        LibraryTab::Panels => &PANELS_TAGS,
    }
}

/// 非 media タブの rail scope(media の [`RailScope`] と同格の
/// 「rail= スコープ選択」状態。mock `state.tag`(`''` = 全件)の転写 —
/// `All` が mock の `data-source="all"` 行、`Tag` が `data-tag-filter` 行)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreviewScope {
    /// 全件(mock rail の `All effects`/`All create`/`All panels` 行、既定)。
    #[default]
    All,
    Tag(PreviewTag),
}

impl PreviewScope {
    /// この scope が `card` を含むか。`All` は無条件で真(media の
    /// `RailScope::AllMedia` と同じ「全件に取りこぼしを作らない」)。
    fn matches(self, card: &PreviewCard) -> bool {
        match self {
            Self::All => true,
            Self::Tag(tag) => card.tags.contains(&tag),
        }
    }
}

/// カタログのカード1枚の同定(カード click の Message が運ぶ面)。media 由来
/// (台帳の `AssetId`)か preview-local 由来(mock `data-item` の静的 id)かを
/// 型で分ける — [`CatalogCard`] と同じ2系統の壁。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardKey {
    /// Document 台帳の素材(media タブ)。
    Media(AssetId),
    /// preview-local 静的カード([`PreviewCard::id`]、mock `data-item`)。
    Preview(&'static str),
}

/// create タブのカードが「作る」もの(B36 新規コンテンツ作成束の消化、
/// bundle canonical `create(kind: layer|comp|solid|null|shape)` の
/// **pane-local に表現できる部分集合**)。実際のレイヤー生成は shell 結線
/// (次波)— この enum は [`crate::state::Message::CreateFromCard`] が運ぶ
/// 型付きの意図語彙で、store の `LayerSource` 語彙(`Solid`/`Null`/`Shape`)へ
/// 1:1 で落ちる kind だけを持つ。
///
/// ## map 行の消化と見送り(B36、freq 降順=全行 freq 1)
/// - **消化**: 952(Shape Layer)→ [`Self::Rectangle`]/[`Self::Ellipse`]
///   (`LayerSource::Shape` — 矩形/楕円は `motolii-vector` の `ShapeNode` 語彙
///   既存)・900/959/313(New solid layer / Solid…)→ [`Self::Solid`]
///   (`LayerSource::Solid`)・898/903(New null layer / Null Object)→
///   [`Self::Null`](`LayerSource::Null`)。
/// - **見送り(store 拡張が要る)**: 175/176/243/244(Adjustment Layer —
///   `LayerSource` に adjustment 語彙が無い)・684/896/658 等(Composition 系 —
///   Document は単一 comp 構造で comp 台帳が無い)・645(直近コンポへ追加 —
///   同前)。
/// - **見送り(pane 外の領分)**: 654/664/665/666(Fit/Center — Stage の
///   view 操作)・656/657(Flowchart)・672/683/807/808/882/883(外部アプリ
///   連携)・691/713/716/943(保存/ポスター/AI/アスペクト比 — export・comp
///   設定系)。
/* motolii-component
id = "browser.shape_creator_catalog"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["CreateKind", "CREATE_PREVIEW"]
meaning = ["CreateFromCard", "PolyStar"]
evaluation = ["every_create_card_declares_its_create_kind", "create_tab_shows_all_six_create_cards"]
render = ["preview_catalog", "create_tab_shows_all_six_create_cards"]
observable = ["double_clicking_a_create_card_publishes_create_from_card"]
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateKind {
    /// 矩形シェイプレイヤー(map 952、`LayerSource::Shape`)。
    Rectangle,
    /// 楕円シェイプレイヤー(map 952、`LayerSource::Shape`)。
    Ellipse,
    /// 単色レイヤー(map 900/959/313、`LayerSource::Solid`)。
    Solid,
    /// ヌルレイヤー(map 898/903、`LayerSource::Null`)。
    Null,
    /// テキストレイヤー(`LayerSource::Text` + 既定 `TextDocument`、
    /// 2026-08-22 利用者裁定「追加するものは Browser の中に全部入れる」—
    /// 歌詞動画/MV ペルソナの致命的欠落〈テキストレイヤーを作る入口が
    /// リポ全体に存在しない〉への対処、`docs/reviews/2026-08-22-persona-lyric-mv.md`
    /// 参照。Solid/Null と同じく normal-map 出典を持たない Motolii 側の
    /// 判断で追加した1枚 — 既存4枚が全て「B36 map 行の消化」だったのと
    /// 出自は異なるが、`CreateKind` 自体が「store の `LayerSource` 語彙へ
    /// 1:1 で落ちる kind」という型の役目は変わらない。
    Text,
    /// 星/正多角形シェイプレイヤー(`motolii_vector::PathSource::PolyStar`、
    /// `LayerSource::Shape` — Rectangle/Ellipse と全く同じ形。2026-08-24
    /// 「ブラウザに8枚の札を足す」発注: `scripts/check_browser_entries.py` が
    /// 「型・描画・書き出しは在るのに札が無い」と検出した `PathSource` の
    /// 最後の1バリアント)。
    PolyStar,
}

/// カードが「新規レイヤーを作る」のではなく**選択中の単一レイヤーへ何かを
/// 足す**時に運ぶ意図(裁定205 施工第2号 §A/§B — マスク追加・エフェクト適用)。
///
/// [`CreateKind`] とは別 enum にした理由: `CreateKind` は「store の
/// `LayerSource` 語彙へ 1:1 で落ちる」という契約を doc で明示している
/// (直上の doc 参照)。マスク/エフェクトはどちらも新しい layer を作らない
/// (既存の選択レイヤーの component 列へ1件足すだけ)ので、その契約に混ぜると
/// 「`CreateKind` は必ず新規レイヤーを作る」という `every_create_card_
/// declares_its_create_kind` 系オラクルの前提が崩れる。**新しい grammar では
/// ない** — [`PreviewCard::creates`] と全く同じ「ダブルクリックで Message を
/// publish する」経路(`crate::preview_card_view`)を共有し、運ぶ payload の
/// 型が違うだけ。単一選択でない時にどう振る舞うか(no-op か拒否か)は pane の
/// 外(shell 側 supervisor)の責務 — この enum 自体は「何を足すか」しか運ばない
/// (`model::can_replace_source` が置き先の是非を pane の外に置くのと同型)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionAction {
    /// 選択中レイヤーへマスクを1枚追加する(`Intent::AddMask` — 「一覧への
    /// 追加」と「shape の初期値」を同じ `write()` で束ねる原子操作、
    /// `motolii_store::document` の `Intent::AddMask` doc 参照。壁7の恒久修正
    /// なので、これ以外の口(`SetMasks` 直呼び)を新たに作らない)。
    AddMask,
    /// 選択中レイヤーへ effect を1つ追加する。plugin id 文字列(裁定70)を
    /// そのまま運ぶ — 2026-08-22 時点で実在する pass は `"motolii.glow"` の
    /// 1つのみ(`motolii-compositor::effects::EffectPass::Glow`)。
    ApplyEffect(&'static str),
    /// 選択中レイヤーの shape へ演算子を1段積む([`ShapeOpKind`] が運ぶのは
    /// **どの演算子か**のタグだけ — 具体的な既定パラメータは shell 側
    /// (`motolii_vector::OpKind` の対応する既定値を組む)の仕事。この crate は
    /// `motolii-vector` を依存に引かない(背骨2「pane は engine 語彙の型を
    /// 直接持たない」と同じ壁)ので、`OpKind` 自身ではなく軽い tag を運ぶ —
    /// `ApplyEffect` が plugin id 文字列だけを運ぶのと同じ「型でなく識別子」
    /// の形。
    ApplyOp(ShapeOpKind),
}

/// [`SelectionAction::ApplyOp`] が運ぶ演算子タグ。`motolii_vector::OpKind` の
/// 7バリアントと1:1 対応するが、**この crate はその型を知らない** — 名前だけ
/// 揃えた独立の tag(`ShapeOpKind::TrimPath` → shell 側で
/// `motolii_vector::OpKind::TrimPath { .. }` の既定値を組む)。`OpKind` 自体は
/// `f64` フィールドを持つため `Eq`/`Copy` を導出できず(浮動小数は `Eq` 非実装)、
/// カードの静的カタログ(`Copy`+`Eq` が要る、`PreviewCard` の derive 参照)へ
/// そのまま埋め込めない——`ApplyEffect(&'static str)` が id 文字列を運ぶのと
/// 同じ理由で、値そのものではなく識別子を運ぶ形にした。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeOpKind {
    /// `motolii_vector::OpKind::TrimPath`。
    TrimPath,
    /// `motolii_vector::OpKind::Repeater`。
    Repeater,
    /// `motolii_vector::OpKind::RoundedCorners`。
    RoundedCorners,
    /// `motolii_vector::OpKind::PuckerBloat`。
    PuckerBloat,
    /// `motolii_vector::OpKind::ZigZag`。
    ZigZag,
    /// `motolii_vector::OpKind::OffsetPath`。
    OffsetPath,
    /// `motolii_vector::OpKind::Twist`。
    Twist,
}

/// preview-local カタログ1枚ぶんの静的カード(mock `#thumbnail-grid` の
/// `data-tab="effects"/"create"/"panels"` カードの転写)。**mock 冒頭コメント
/// の宣言どおり preview 専用データ** — filesystem/Document/Host/intent/
/// persistence のどの経路にも接続しない(`&'static` の定数リテラルのみ)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreviewCard {
    /// 静的 id(mock `data-item` そのまま — カード click の [`CardKey::Preview`]
    /// が運ぶ同定子。Glow は mock 外の実在 plugin なので同型の小文字 slug)。
    pub id: &'static str,
    /// カード名(mock `.cardCopy strong`)。
    pub name: &'static str,
    /// caption(mock `.cardCopy small`、`種別 · 分類` の形)。
    pub caption: &'static str,
    /// thumb に載せるグリフ(mock `.libraryThumb b`)。
    pub glyph: &'static str,
    /// rail/filter のカテゴリ(mock `data-tags` のうち rail が宣言するタグ —
    /// 例 Rectangle は mock `data-tags="shape builtin favorite"` → Shapes と
    /// Built-in の両方に属する。`favorite` は COLLECTIONS 予約地の語彙なので
    /// 転写しない、crate 冒頭 doc の予約地参照)。
    pub tags: &'static [PreviewTag],
    /// このカードが「作る」kind(B36 実体化)。**create タブのカードだけが
    /// `Some`** — effects/panels のカードは `None`(作るものが無い)。view は
    /// `Some` のカードにだけダブルクリック=作成
    /// ([`crate::state::Message::CreateFromCard`])を配線する(AE/Figma 慣習
    /// S0: シングル=選択・ダブル=作成)。
    pub creates: Option<CreateKind>,
    /// このカードが「選択中の単一レイヤーへ足す」意図([`SelectionAction`]、
    /// 裁定205 施工第2号)。`creates` とは排他(新規レイヤーを作るか、既存
    /// レイヤーへ足すかのどちらか一方)——**両方 `Some` のカードは無い**、
    /// `every_create_card_declares_its_create_kind`/`effects_action_cards_
    /// declare_their_selection_action` の2オラクルがそれぞれ独立に確かめる。
    /// 2026-08-22 時点では Effects タブの Glow(`ApplyEffect`)と Mask
    /// (`AddMask`)の2枚だけが `Some`。
    pub applies_to_selection: Option<SelectionAction>,
}

/// effects タブの preview カタログ。実在する Glow provider、Mask、Shape Ops だけを
/// 掲載する。mock-only の Echo Bloom/Opacity/Sine は、実装 pass が無い状態で
/// 操作可能に見せないため除外した。
const EFFECTS_PREVIEW: [PreviewCard; 9] = [
    // Glow は実在 plugin(`"motolii.glow"`、
    // `motolii-compositor::effects::EffectPass::Glow`)なので選択へ適用できる。
    PreviewCard {
        id: "glow",
        name: "Glow",
        caption: "effect · Color",
        glyph: "FX",
        tags: &[PreviewTag::Color],
        creates: None,
        applies_to_selection: Some(SelectionAction::ApplyEffect("motolii.glow")),
    },
    // 裁定205 施工第2号 §A: マスクは新規レイヤーを作らない(既存の選択レイヤーへ
    // 1枚足すだけ)ので `creates` ではなく `applies_to_selection` を使う。置き場は
    // Create タブではなく Effects タブ(`EFFECTS_PREVIEW` doc 冒頭の判断根拠参照)。
    PreviewCard {
        id: "mask",
        name: "Mask",
        caption: "mask · Layer",
        glyph: "⬚",
        tags: &[PreviewTag::Masks],
        creates: None,
        applies_to_selection: Some(SelectionAction::AddMask),
    },
    // 2026-08-24「ブラウザに8枚の札」発注 §2: `OpKind` 全7種の「選択へ適用
    // する」札。配置は `EFFECTS_PREVIEW`(= 実質「選択へ適用する語彙」の
    // タブ — `Mask` がエフェクトでないのにここに居るのが先例、発注書どおり)。
    // 並びは `motolii_vector::OpKind` の宣言順(裁定10 の移植元 `pathgeom.rs`
    // が踏襲した Lottie `shapes/*` 掲載順)をそのまま保つ。
    PreviewCard {
        id: "trim-path",
        name: "Trim Path",
        caption: "shape op · Shape ops",
        glyph: "◗",
        tags: &[PreviewTag::ShapeOps],
        creates: None,
        applies_to_selection: Some(SelectionAction::ApplyOp(ShapeOpKind::TrimPath)),
    },
    PreviewCard {
        id: "repeater",
        name: "Repeater",
        caption: "shape op · Shape ops",
        glyph: "≡",
        tags: &[PreviewTag::ShapeOps],
        creates: None,
        applies_to_selection: Some(SelectionAction::ApplyOp(ShapeOpKind::Repeater)),
    },
    PreviewCard {
        id: "rounded-corners",
        name: "Rounded Corners",
        caption: "shape op · Shape ops",
        glyph: "▢",
        tags: &[PreviewTag::ShapeOps],
        creates: None,
        applies_to_selection: Some(SelectionAction::ApplyOp(ShapeOpKind::RoundedCorners)),
    },
    PreviewCard {
        id: "pucker-bloat",
        name: "Pucker & Bloat",
        caption: "shape op · Shape ops",
        glyph: "✺",
        tags: &[PreviewTag::ShapeOps],
        creates: None,
        applies_to_selection: Some(SelectionAction::ApplyOp(ShapeOpKind::PuckerBloat)),
    },
    PreviewCard {
        id: "zig-zag",
        name: "Zig Zag",
        caption: "shape op · Shape ops",
        glyph: "⌁",
        tags: &[PreviewTag::ShapeOps],
        creates: None,
        applies_to_selection: Some(SelectionAction::ApplyOp(ShapeOpKind::ZigZag)),
    },
    PreviewCard {
        id: "offset-path",
        name: "Offset Path",
        caption: "shape op · Shape ops",
        glyph: "⧉",
        tags: &[PreviewTag::ShapeOps],
        creates: None,
        applies_to_selection: Some(SelectionAction::ApplyOp(ShapeOpKind::OffsetPath)),
    },
    PreviewCard {
        id: "twist",
        name: "Twist",
        caption: "shape op · Shape ops",
        glyph: "☯",
        tags: &[PreviewTag::ShapeOps],
        creates: None,
        applies_to_selection: Some(SelectionAction::ApplyOp(ShapeOpKind::Twist)),
    },
];

/// create タブの preview カタログ。先頭2枚は mock html:532-537 の転写
/// (`data-tags="shape builtin"` — Shapes/Built-in の両カテゴリ)。**Solid/
/// Null の2枚は mock 外 — map B36 行の消化**([`CreateKind`] doc の消化台帳:
/// 900/959/313=Solid・898/903=Null。store `LayerSource` に既にある語彙のみ)。
/// タグは mock の `builtin` 語彙(Built-in)へ載せる — シェイプではないので
/// Shapes には属さない。並びは mock 転写分を先頭に保ち、追加分を末尾へ
/// (effects の Glow 追加と同じ形)。**Text は2026-08-22 利用者裁定で末尾に
/// 追加**(`CreateKind::Text` doc 参照 — 「追加するものは Browser の中に
/// 全部入れる」、Layer メニュー等の別入口は作らない)。
const CREATE_PREVIEW: [PreviewCard; 6] = [
    PreviewCard {
        id: "rectangle",
        name: "Rectangle",
        caption: "shape · Built-in",
        glyph: "□",
        tags: &[PreviewTag::Shapes, PreviewTag::BuiltIn],
        creates: Some(CreateKind::Rectangle),
        applies_to_selection: None,
    },
    PreviewCard {
        id: "ellipse",
        name: "Ellipse",
        caption: "shape · Built-in",
        glyph: "○",
        tags: &[PreviewTag::Shapes, PreviewTag::BuiltIn],
        creates: Some(CreateKind::Ellipse),
        applies_to_selection: None,
    },
    // 2026-08-24「ブラウザに8枚の札」発注 §1: `PathSource::PolyStar` の
    // Create 札。Rectangle/Ellipse と同じ Shapes+Built-in の2カテゴリ
    // (`motolii_vector::PathSource` の3つ目のパス源 — Bezier はペン道具の
    // Stage 側入口なので対象外、`scripts/check_browser_entries.py` doc 参照)。
    PreviewCard {
        id: "poly-star",
        name: "Star",
        caption: "shape · Built-in",
        glyph: "★",
        tags: &[PreviewTag::Shapes, PreviewTag::BuiltIn],
        creates: Some(CreateKind::PolyStar),
        applies_to_selection: None,
    },
    PreviewCard {
        id: "solid",
        name: "Solid",
        caption: "layer · Built-in",
        glyph: "■",
        tags: &[PreviewTag::BuiltIn],
        creates: Some(CreateKind::Solid),
        applies_to_selection: None,
    },
    PreviewCard {
        id: "null",
        name: "Null",
        caption: "layer · Built-in",
        glyph: "◇",
        tags: &[PreviewTag::BuiltIn],
        creates: Some(CreateKind::Null),
        applies_to_selection: None,
    },
    // 2026-08-22 利用者裁定「追加するものは Browser の中に全部入れる」—
    // 歌詞動画/MV ペルソナの致命的欠落(テキストレイヤーを作る入口が
    // リポ全体に存在しない、`docs/reviews/2026-08-22-persona-lyric-mv.md`)への
    // 対処。Solid/Null と同じ「layer · Built-in」区分(シェイプではない)。
    PreviewCard {
        id: "text",
        name: "Text",
        caption: "layer · Built-in",
        glyph: "T",
        tags: &[PreviewTag::BuiltIn],
        creates: Some(CreateKind::Text),
        applies_to_selection: None,
    },
];

/// panels タブの preview カタログ(mock html:539-547)。
const PANELS_PREVIEW: [PreviewCard; 3] = [
    PreviewCard {
        id: "asset-tags",
        name: "Asset tagging",
        caption: "panel · Tags",
        glyph: "#",
        tags: &[PreviewTag::Tags],
        creates: None,
        applies_to_selection: None,
    },
    PreviewCard {
        id: "notes",
        name: "Notes",
        caption: "panel · Notes",
        glyph: "✎",
        tags: &[PreviewTag::Notes],
        creates: None,
        applies_to_selection: None,
    },
    PreviewCard {
        id: "export-notes",
        name: "Export notes",
        caption: "panel · Export",
        glyph: "↗",
        tags: &[PreviewTag::Export],
        creates: None,
        applies_to_selection: None,
    },
];

/// タブごとの preview-local 静的カタログ。**media は空**(media タブは
/// Document 台帳投影([`assets`]/[`visible`])の経路であって静的データを
/// 混ぜない — 発注の境界)。
pub fn preview_catalog(tab: LibraryTab) -> &'static [PreviewCard] {
    match tab {
        LibraryTab::Media => &[],
        LibraryTab::Effects => &EFFECTS_PREVIEW,
        LibraryTab::Create => &CREATE_PREVIEW,
        LibraryTab::Panels => &PANELS_PREVIEW,
    }
}

/// catalog grid の1枚(タブ別投影の結果)。media タブ由来か preview-local
/// 由来かを型で分ける — 2系統のデータが view の手前で混線しないための壁。
#[derive(Debug, Clone, PartialEq)]
pub enum CatalogCard {
    /// Document 台帳投影(media タブのみ)。
    Media(AssetListItem),
    /// preview-local 静的カタログ(effects/create/panels タブのみ)。
    Preview(PreviewCard),
}

/// タブ別のカタログ投影(純関数、IO なし)。
/// - media: [`visible`](rail scope + 検索)をそのまま通した台帳投影のみ。
/// - effects/create/panels: [`preview_visible`]([`PreviewScope`] + 検索)の
///   preview-local カタログのみ。media の rail scope(種別の語彙)は効かない
///   (mock `chooseTab` が非 media タブで `source='all'` へ戻すのと同じ意味)。
pub fn catalog(
    tab: LibraryTab,
    media: &[AssetListItem],
    scope: RailScope,
    preview_scope: PreviewScope,
    query: &str,
) -> Vec<CatalogCard> {
    match tab {
        LibraryTab::Media => visible(media, scope, query)
            .into_iter()
            .map(CatalogCard::Media)
            .collect(),
        tab => preview_visible(tab, preview_scope, query)
            .into_iter()
            .map(CatalogCard::Preview)
            .collect(),
    }
}

/// 非 media タブのカタログを [`PreviewScope`] + 検索文字列で絞る純関数
/// ([`visible`] の preview-local 版 — 構造の対称化)。
///
/// - scope: [`PreviewScope::matches`](rail 行/filter チップ、どちらから
///   触っても同じ絞り込み — media と同じ「2つの入口が同じ状態を書く」
///   Ableton可視性原理)。
/// - query: 名前の部分一致・大小無視・前後空白無視(mock は `data-search` を
///   全タブで照合する — preview カードに fingerprint は無いので名前のみ)。
/// - 順序は [`preview_catalog`] の宣言順(mock 掲載順)を保つ — 並べ替えない。
pub fn preview_visible(tab: LibraryTab, scope: PreviewScope, query: &str) -> Vec<PreviewCard> {
    let query = query.trim().to_lowercase();
    preview_catalog(tab)
        .iter()
        .filter(|card| scope.matches(card))
        .filter(|card| query.is_empty() || card.name.to_lowercase().contains(&query))
        .copied()
        .collect()
}

#[cfg(test)]
mod tests;
