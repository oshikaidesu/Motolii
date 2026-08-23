//! 一覧 projection(裁定162 切片 B1)+ rail/filter(切片 B2)— `StoreView::assets()`
//! から Browser が描く行を組み、rail scope + 検索文字列で絞る純関数。**IO も
//! 評価もしない** — `StoreView` が既に読んだ台帳をそのまま並べ替える/絞るだけの
//! 読み専用の投影(`timeline_pane::rows`/`projection.rs` と同じ形)。
//!
//! 移植元(意味の正本)は旧 `crates/motolii-shell-iced/src/browser.rs` の
//! `BrowserCard` 投影だが、この切片(B1+B2)の範囲は一覧+rail/filter まで —
//! 視覚(B3、`browser-library.html` 構造のトンマナ読み替え・Shell::view への
//! 組み込み)・サムネ(B4/B5)はまだこの crate に無い(crate 冒頭 doc 参照)。
//!
//! rail scope は mock(`browser-library.html` `.librarySidebar` `LIBRARY` 節)の
//! **種別のみ**(第一波、裁定162 付随裁定: MEDIA 種別のみ)。`COLLECTIONS`
//! (色ドット bin)・`PLACES`(Starter/Project/Motion フォルダ)は
//! `browser-semantics.html` 救出台帳で「予約地」(タグ束・filesystem 走査裁定
//! 待ち)と明記済み — この切片では実装しない。

use motolii_store::{Asset, AssetId, LayerId, LayerSource, RationalTime, StoreView};

/// 一覧1行ぶんの投影。`Asset` から Browser が要る最小の面だけを切り出す
/// (`AssetListItem { id, name, kind, path, fingerprint, duration }` — EXACT
/// TARGET #1)。**`fingerprint` は B2 で追加**(`Asset::content_hash` をそのまま
/// 運ぶ) — 検索欄が「表示名/fingerprint マッチ」(OUTCOME)を満たすための面。
/// **`duration` は B3 で追加**(`Asset::duration` をそのまま運ぶ) — カード grid
/// の「尺」表示(B3 OUTCOME「種別アイコン+名前+尺のカード骨格」)のための面。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetListItem {
    pub id: AssetId,
    pub name: String,
    /// `Asset::asset_type` をそのまま運ぶ(opaque 文字列、例 `video/mp4`)。
    /// rail/filter(B2、[`category_of`])がここから種別を導出する。
    pub kind: String,
    pub path: Option<String>,
    /// `Asset::content_hash` の写し(検索欄のマッチ対象、[`visible`] 参照)。
    pub fingerprint: String,
    /// `Asset::duration` の写し。probe が尺を読めなかった素材は `None`
    /// (`Asset::duration` の doc と同じ「分かる時だけ入る」)— カード grid は
    /// [`format_duration`] で「—」へ丸める。
    pub duration: Option<RationalTime>,
}

/// rail/filter が読む粗い種別。`AssetListItem::kind` の prefix(`/` 区切りの
/// 前半)だけを見る — 意味起草タスク#14(正確な種別判定)の空席を埋めない
/// 暫定判定(`Shell::guess_asset_type` が作る疑似 MIME 文字列を前提にする、
/// この crate 冒頭 doc と同じ立場)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Video,
    Image,
    Audio,
    /// mime prefix が `video`/`image`/`audio` のいずれでもない
    /// (`application/*` 等)。`RailScope::AllMedia` には含まれるが、種別
    /// scope(Video/Images/Audio)には現れない。
    Other,
}

/// `kind` 文字列(`video/mp4` 等)→ [`Category`]。純関数、IO なし。
pub fn category_of(kind: &str) -> Category {
    match kind.split('/').next().unwrap_or("") {
        "video" => Category::Video,
        "image" => Category::Image,
        "audio" => Category::Audio,
        _ => Category::Other,
    }
}

impl Category {
    /// カード grid の caption(B3)が読む表示文言。`RailScope::label` と語彙は
    /// 揃えるが単数形(1件のカードの種別を言う文なので「Video」等そのまま —
    /// rail 側は scope の複数件を言う文脈なので同じ語で兼用できる)。
    pub fn label(self) -> &'static str {
        match self {
            Self::Video => "Video",
            Self::Image => "Image",
            Self::Audio => "Audio",
            Self::Other => "Other",
        }
    }

    /// カード grid の thumb に載せる種別グリフ(B3)。rail の種別行アイコン
    /// (`browser-semantics.html` `▣ Video`/`▧ Images`/`♪ Audio`)と同じ語彙を
    /// 再利用する — thumb とrail が別の記号語彙を持つと意味が2つに割れる。
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Video => "▣",
            Self::Image => "▧",
            Self::Audio => "♪",
            Self::Other => "▪",
        }
    }
}

/// 尺の表示整形(mm:ss)。**IO なし・純関数**。`None`(probe が尺を読めなかった
/// 素材、`Asset::duration` の doc 参照)は Inspector の「値が無い」慣用と同じ
/// 1文字「—」へ丸める。負値・NaN は現実の `Asset::duration` からは出ない想定
/// だが `max(0.0)` で防御的に 0 へ丸める(M2 系「入力起因で panic しない」流儀)。
pub fn format_duration(duration: Option<RationalTime>) -> String {
    let Some(duration) = duration else {
        return "—".to_owned();
    };
    let total_seconds = duration.as_seconds_f64().max(0.0).round() as u64;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes}:{seconds:02}")
}

/// rail の scope(mock `.librarySidebar` `LIBRARY` 節、第一波は種別のみ)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RailScope {
    /// 全件(mock `data-source="all"`、既定)。
    #[default]
    AllMedia,
    Video,
    Images,
    Audio,
}

/// rail 列の並び順(mock の `LIBRARY` 節の掲載順どおり — All media → Video →
/// Images → Audio)。view 側・試験側の両方がこの1本の並びを共有する。
pub const RAIL_SCOPES: [RailScope; 4] = [
    RailScope::AllMedia,
    RailScope::Video,
    RailScope::Images,
    RailScope::Audio,
];

/// filter shelf の種別チップ(mock `.filterGroup[data-filter-group="media"]`)。
/// `AllMedia` はここに現れない(mock にも `Clear` はあるが `All media` チップは
/// 無い — rail 側の役割、`browser-semantics.html` 「rail = 台帳の scope」)。
pub const FILTER_CHIPS: [RailScope; 3] = [RailScope::Video, RailScope::Images, RailScope::Audio];

impl RailScope {
    /// mock の表示文言そのまま(`.locationRow`/`.filterShelf button` のラベル)。
    pub fn label(self) -> &'static str {
        match self {
            Self::AllMedia => "All media",
            Self::Video => "Video",
            Self::Images => "Images",
            Self::Audio => "Audio",
        }
    }

    /// この scope が `category` を含むか。`AllMedia` は無条件で真
    /// (`Category::Other` も含む — 「全件」に取りこぼしを作らない)。
    fn matches(self, category: Category) -> bool {
        match self {
            Self::AllMedia => true,
            Self::Video => matches!(category, Category::Video),
            Self::Images => matches!(category, Category::Image),
            Self::Audio => matches!(category, Category::Audio),
        }
    }
}

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
    /// effects(mock `data-tag-filter="color"` — Echo Bloom/Glow)。
    Color,
    /// effects(mock `data-tag-filter="utility"` — Opacity)。
    Utility,
    /// effects(mock `data-tag-filter="animation"` — Sine)。
    Animation,
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
}

impl PreviewTag {
    /// mock の表示文言そのまま(rail 行とチップで共通 — html:444-446/454-455/
    /// 463-465 と html:487/490/493 は同ラベル)。`Masks` は mock に無いので
    /// 自然な英語の慣用句を採る(`SortKey::label` 等の新規 UI と同じ立場)。
    pub fn label(self) -> &'static str {
        match self {
            Self::Color => "Color",
            Self::Utility => "Utility",
            Self::Animation => "Animation",
            Self::Shapes => "Shapes",
            Self::BuiltIn => "Built-in",
            Self::Tags => "Tags",
            Self::Notes => "Notes",
            Self::Export => "Export",
            Self::Masks => "Masks",
        }
    }

    /// このタグが属するタブ(タグ語彙はタブ別 — 試験がカタログとの整合を
    /// 照合する口)。
    pub fn tab(self) -> LibraryTab {
        match self {
            Self::Color | Self::Utility | Self::Animation | Self::Masks => LibraryTab::Effects,
            Self::Shapes | Self::BuiltIn => LibraryTab::Create,
            Self::Tags | Self::Notes | Self::Export => LibraryTab::Panels,
        }
    }
}

/// effects タブの rail カテゴリ並び(mock html:444-446 の掲載順 — S0 慣習順、
/// 末尾の `Masks` は裁定205 施工第2号 §A で追加)。
pub const EFFECTS_TAGS: [PreviewTag; 4] = [
    PreviewTag::Color,
    PreviewTag::Utility,
    PreviewTag::Animation,
    PreviewTag::Masks,
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

/// effects タブの preview カタログ。mock html:522-530 の3枚(Echo Bloom/
/// Opacity/Sine)+ 実在 plugin の Glow(発注: 「effects は実在 plugin 名
/// Glow を含めてよい」 — mock の並びを保ち末尾へ追加。分類は実分類=Color、
/// caption `effect · Color` と同値)+ Mask(裁定205 施工第2号 §A — マスクは
/// 新規レイヤーを作らないので Create タブの「新規レイヤー」語彙には属さず、
/// Effects タブの「選択中レイヤーへ足す」語彙(Glow と同型)へ置く。**この
/// 配置は判断であって台帳決定ではない** — 施工の RETURN に根拠を記載)。
const EFFECTS_PREVIEW: [PreviewCard; 5] = [
    PreviewCard {
        id: "echo-bloom",
        name: "Echo Bloom",
        caption: "effect · Color",
        glyph: "FX",
        tags: &[PreviewTag::Color],
        creates: None,
        applies_to_selection: None,
    },
    PreviewCard {
        id: "opacity",
        name: "Opacity",
        caption: "effect · Utility",
        glyph: "FX",
        tags: &[PreviewTag::Utility],
        creates: None,
        applies_to_selection: None,
    },
    PreviewCard {
        id: "sine",
        name: "Sine",
        caption: "effect · Animation",
        glyph: "FX",
        tags: &[PreviewTag::Animation],
        creates: None,
        applies_to_selection: None,
    },
    // 裁定205 施工第2号 §B: Glow は実在 plugin(`"motolii.glow"`、
    // `motolii-compositor::effects::EffectPass::Glow`)なので `applies_to_
    // selection` を `Some` にして shell 結線を持てる — Echo Bloom/Opacity/
    // Sine は mock 転写の見せ札(実在 plugin ではない)なので `None` のまま。
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
const CREATE_PREVIEW: [PreviewCard; 5] = [
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

/// 台帳の一覧を投影する。**順序は決定論(admit 順 = `AssetId` 昇順)** —
/// `StoreView::assets()` が既に `AssetTable`(`BTreeMap` 内部)の順で返すので、
/// この関数自身は並べ替えない(既にある決定論を壊さない、`view.rs` の
/// `assets()` doc 参照)。
///
/// `StoreView::assets()` が `Err` を返す(壊れた Document)場合は空を返す —
/// この投影は表示専用(`meta`/`markers` 読み系と同じ「表示は空へ丸める」流儀)。
/// 「読めない」ことそのものを扱いたい呼び手は `store.assets()` を直接呼ぶこと。
pub fn assets(store: &StoreView<'_>) -> Vec<AssetListItem> {
    store
        .assets()
        .unwrap_or_default()
        .into_iter()
        .map(|asset| AssetListItem {
            id: asset.id,
            name: asset.name,
            kind: asset.asset_type,
            path: asset.path_absolute,
            fingerprint: asset.content_hash,
            duration: asset.duration,
        })
        .collect()
}

/// rail scope + 検索文字列で [`assets`] の一覧をさらに絞る純関数(B2
/// EXACT TARGET)。**IO なし** — 呼び手が既に持つ投影をもう一段絞るだけ。
///
/// - scope: [`RailScope::matches`] で種別フィルタ(mock の rail 行/filter shelf
///   チップ、どちらから触っても同じ絞り込みになる — 2つの入口が同じ状態を書く
///   Ableton可視性原理どおり)。
/// - query: 表示名/fingerprint/path の部分一致・大小無視・前後空白無視
///   (OUTCOME「文字列は fingerprint/表示名マッチ」+ B08 第4切片で path を追加
///   — `AssetListItem::path` は B1 から既に投影に乗っていた実在属性
///   [`Asset::path_absolute`] の写しだが、これまで検索対象になっていなかった。
///   store に新しい属性を要求せず「検索の対象拡張」を満たせる箇所だった)。
///   空文字列は「絞らない」。
/// - 順序は [`assets`] と同じ決定論(admit 順)を保つ — この関数は並べ替えない
///   (並べ替えは [`sorted`] が別関数として担う — scope/query→sort の合成順)。
pub fn visible(items: &[AssetListItem], scope: RailScope, query: &str) -> Vec<AssetListItem> {
    let query = query.trim().to_lowercase();
    items
        .iter()
        .filter(|item| scope.matches(category_of(&item.kind)))
        .filter(|item| {
            query.is_empty()
                || item.name.to_lowercase().contains(&query)
                || item.fingerprint.to_lowercase().contains(&query)
                || item
                    .path
                    .as_deref()
                    .is_some_and(|path| path.to_lowercase().contains(&query))
        })
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------
// B08 map 616/617 消化: 素材置換(`Intent::SetSource` の UI 面 — store 側は
// 裁定112c で実装済み、UI だけが未着手だった行)。618(ドラッグ版)は
// Stage/Timeline 側の drop target が要る別 write-set のため対象外
// (crate 冒頭 doc 参照)。
// ---------------------------------------------------------------------------

/// `Asset` → `Intent::SetSource { layer, source }` へ渡す `LayerSource` を
/// 組む純関数。**IO なし** — この crate は `Intent` を発行しない
/// (supervisor が `AssetId` → `Asset` を引いた後にこれを呼び、`Some` なら
/// dispatch する、crate 冒頭 doc「shell 結線」参照)。
///
/// `path_absolute` を優先、無ければ `path_project_relative` へ落ちる —
/// 両方無い(非ファイル素材)は `None`(置換できる実体が無い)。`fingerprint`
/// は `Asset::content_hash`(admit 時から常に有る)をそのまま運ぶ —
/// `motolii-shell` の raw-file-drop 経路(実ハッシュを持たないので
/// `fingerprint: None`)とは事情が違う、ここは実ハッシュを持つので使う。
pub fn asset_to_layer_source(asset: &Asset) -> Option<LayerSource> {
    let path = asset
        .path_absolute
        .clone()
        .or_else(|| asset.path_project_relative.clone())?;
    Some(LayerSource::Media {
        path,
        fingerprint: Some(asset.content_hash.clone()),
    })
}

/// 置換先 layer のゲーティング(map 616/617「選択素材を置換」)。呼び手
/// (supervisor)は `Session::selected_layers` を `len() == 1` の時だけ
/// `Some` へ畳んだ物を渡す — 0件・2件以上の選択(曖昧な置換先)は `None`
/// のまま渡すこと(この codebase の既存慣習「曖昧な物には何もしない」)。
/// [`asset_to_layer_source`] が `None` を返す素材(パスが無い)も同様に拒む。
pub fn can_replace_source(single_selected_layer: Option<LayerId>, asset: &Asset) -> Option<LayerId> {
    let layer = single_selected_layer?;
    asset_to_layer_source(asset)?;
    Some(layer)
}

// ---------------------------------------------------------------------------
// B08 第4切片(素材の整理): 並べ替え + 表示形式。実在する `AssetListItem`/
// `Asset` の属性だけで表現できる整理系のみ(発注書「store に無い属性が要る
// 物は見送り」)。タグ付け・お気に入り・COLLECTIONS は `Asset` にその属性が
// 無いため実装しない(crate 冒頭 doc の予約地をそのまま延長 — RETURN 記載)。
// ---------------------------------------------------------------------------

/// 一覧の並べ替えキー(発注書「並べ替え(名前/追加日/種別)」)。3種とも
/// `AssetListItem` が既に運ぶ実属性だけを見る — store へ新しい列は要らない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortKey {
    /// 表示名(大小無視)。既定 — `visible`/`assets` の検索慣習
    /// (大小無視)と同じ語彙を並べ替えでも踏襲する。
    #[default]
    Name,
    /// 追加順相当。`Asset` は wall-clock timestamp を持たない
    /// (`motolii_store::asset` の `Asset` フィールド一覧参照)—
    /// `AssetTable::admit` が admission 順に単調増加させ、削除後も再利用
    /// しない `AssetId`(同 crate `AssetTable` doc 参照)を「追加順」の代理
    /// 指標として使う。新しい方が先(降順) — 一般的な NLE の
    /// 「date added」既定(新しい素材ほど見つけやすい)に倣う。
    AddedDate,
    /// 種別(`AssetListItem::kind` の生文字列、mime prefix でグルーピング
    /// される — [`category_of`] と同じ語彙の元)。
    Kind,
}

/// 並べ替えキーの並び(発注書の掲載順どおり — Name → Date added → Type)。
/// view 側・試験側の両方がこの1本の並びを共有する([`RAIL_SCOPES`] と同型)。
pub const SORT_KEYS: [SortKey; 3] = [SortKey::Name, SortKey::AddedDate, SortKey::Kind];

impl SortKey {
    /// filter shelf のチップに載る表示文言(mock に類例が無い新規 UI なので
    /// 自然な英語の慣用句を採る — `LibraryTab::label` 等と同じ短い名詞句)。
    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::AddedDate => "Date added",
            Self::Kind => "Type",
        }
    }
}

/// [`SortKey`] に従い一覧を並べ替える純関数(IO なし)。**安定ソート**
/// (`slice::sort_by` は stable — 同じキーを持つ2件は互いの相対順(元の並び、
/// 通常は admit 順)を保つ。「並べ替えても取りこぼし/無関係な入れ替わりを
/// 起こさない」という一般則、テスト `sorted_is_stable_for_equal_keys` 参照)。
/// 呼び手は [`visible`] のあとにこれを通す(scope/query→sort の順)。
pub fn sorted(items: &[AssetListItem], key: SortKey) -> Vec<AssetListItem> {
    let mut items = items.to_vec();
    match key {
        SortKey::Name => {
            items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        }
        // 新しい方が先(降順) — `AddedDate` doc 参照。
        SortKey::AddedDate => items.sort_by(|a, b| b.id.cmp(&a.id)),
        SortKey::Kind => items.sort_by(|a, b| a.kind.cmp(&b.kind)),
    }
    items
}

/// カード grid の表示形式(発注書「表示形式(グリッド/リスト —
/// `Icon::GridView`/`Icon::ViewList` 在庫あり)」)。mock 既定表示
/// `data-view="grid"` に合わせ [`Grid`](Self::Grid) を既定にする。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Grid,
    List,
}

impl ViewMode {
    /// tooltip の文言(裁定187 icon+tooltip ペア — アイコン自体は
    /// `motolii-icons::Icon::GridView`/`ViewList`、この crate は Icon を
    /// 知らない純データ層のままなので tooltip 文言だけをここに持つ)。
    pub fn tooltip_label(self) -> &'static str {
        match self {
            Self::Grid => "Grid view",
            Self::List => "List view",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_store::{AssetDraft, Document, Intent};

    fn draft(content_hash: &str, name: &str) -> AssetDraft {
        draft_typed(content_hash, name, "video/mp4")
    }

    /// [`draft`] の種別指定版(B2: rail/filter の種別テストが要る)。
    fn draft_typed(content_hash: &str, name: &str, asset_type: &str) -> AssetDraft {
        AssetDraft {
            name: name.to_owned(),
            asset_type: asset_type.to_owned(),
            content_hash: content_hash.to_owned(),
            path_absolute: Some(format!("/project/media/{name}.mp4")),
            path_project_relative: None,
            file_name: Some(format!("{name}.mp4")),
            size_bytes: Some(1024),
            head_hash: None,
            tail_hash: None,
            duration: None,
        }
    }

    fn admit_all(doc: &mut Document, drafts: Vec<AssetDraft>) {
        for draft in drafts {
            doc.apply(Intent::AdmitAsset { draft }).unwrap();
        }
    }

    /// ORACLE (a): fixture 級の `StoreView`(台帳2件)→ projection 2件・
    /// 順序決定論(admit 順 = id 順)。
    #[test]
    fn projects_two_assets_in_admission_order() {
        let mut doc = Document::new();
        doc.apply(Intent::AdmitAsset {
            draft: draft("sha256:first", "first"),
        })
        .unwrap();
        doc.apply(Intent::AdmitAsset {
            draft: draft("sha256:second", "second"),
        })
        .unwrap();

        let items = assets(&doc.view());
        assert_eq!(items.len(), 2, "台帳2件が projection に2件現れない");
        assert_eq!(items[0].name, "first");
        assert_eq!(items[1].name, "second");
        assert!(items[0].id < items[1].id, "admit 順(id 昇順)を保っていない");
    }

    /// 何も admit していない Document は空の projection(`markers`/`masks` と
    /// 同じ「無い=空」)。
    #[test]
    fn empty_table_projects_to_empty_list() {
        let doc = Document::new();
        assert_eq!(assets(&doc.view()), Vec::new());
    }

    // -----------------------------------------------------------------
    // B2: category_of — `kind` の prefix だけを見る純関数。
    // -----------------------------------------------------------------

    #[test]
    fn category_of_reads_the_mime_style_prefix() {
        assert_eq!(category_of("video/mp4"), Category::Video);
        assert_eq!(category_of("image/svg+xml"), Category::Image);
        assert_eq!(category_of("audio/wav"), Category::Audio);
        assert_eq!(category_of("application/octet-stream"), Category::Other);
        assert_eq!(
            category_of(""),
            Category::Other,
            "空文字列は Other へ丸まるはず"
        );
    }

    // -----------------------------------------------------------------
    // B2 EXACT TARGET: `visible`(rail scope + 検索文字列で絞る純関数)。
    // -----------------------------------------------------------------

    fn mixed_ledger() -> Vec<AssetListItem> {
        let mut doc = Document::new();
        admit_all(
            &mut doc,
            vec![
                draft_typed("sha256:v1", "intro-clip", "video/mp4"),
                draft_typed("sha256:i1", "logo-mark", "image/png"),
                draft_typed("sha256:a1", "room-tone", "audio/wav"),
                draft_typed("sha256:v2", "cutaway", "video/mov"),
            ],
        );
        assets(&doc.view())
    }

    /// **ORACLE**: 未実装時点(`visible` が無い/常に全件を返す等)ではこの
    /// 群は red — `RailScope::Video` は video 種別だけを残す。
    #[test]
    fn rail_scope_video_narrows_to_video_kind_only() {
        let items = mixed_ledger();
        let narrowed = visible(&items, RailScope::Video, "");
        assert_eq!(narrowed.len(), 2, "video 2件のはず: {narrowed:?}");
        assert!(narrowed
            .iter()
            .all(|item| category_of(&item.kind) == Category::Video));
    }

    #[test]
    fn rail_scope_images_narrows_to_image_kind_only() {
        let items = mixed_ledger();
        let narrowed = visible(&items, RailScope::Images, "");
        assert_eq!(narrowed.len(), 1);
        assert_eq!(narrowed[0].name, "logo-mark");
    }

    #[test]
    fn rail_scope_audio_narrows_to_audio_kind_only() {
        let items = mixed_ledger();
        let narrowed = visible(&items, RailScope::Audio, "");
        assert_eq!(narrowed.len(), 1);
        assert_eq!(narrowed[0].name, "room-tone");
    }

    /// `AllMedia` は種別を問わず全件(`Category::Other` の取りこぼしも無い —
    /// mixed_ledger は Other を含まないが、境界の意味は `category_of` 試験が
    /// 別途保証する)。順序は admit 順のまま(この関数は並べ替えない)。
    #[test]
    fn rail_scope_all_media_keeps_every_item_in_admission_order() {
        let items = mixed_ledger();
        let narrowed = visible(&items, RailScope::AllMedia, "");
        assert_eq!(narrowed, items, "AllMedia は絞らない・並べ替えないはず");
    }

    #[test]
    fn query_matches_display_name_case_insensitively() {
        let items = mixed_ledger();
        let narrowed = visible(&items, RailScope::AllMedia, "INTRO");
        assert_eq!(narrowed.len(), 1);
        assert_eq!(narrowed[0].name, "intro-clip");
    }

    /// OUTCOME「文字列は fingerprint/表示名マッチ」— fingerprint
    /// (`content_hash` の写し)の部分一致でも絞れる。
    #[test]
    fn query_matches_fingerprint_substring() {
        let items = mixed_ledger();
        let narrowed = visible(&items, RailScope::AllMedia, "sha256:a1");
        assert_eq!(narrowed.len(), 1);
        assert_eq!(narrowed[0].name, "room-tone");
    }

    #[test]
    fn empty_query_after_trimming_does_not_narrow() {
        let items = mixed_ledger();
        let narrowed = visible(&items, RailScope::AllMedia, "   ");
        assert_eq!(narrowed.len(), items.len());
    }

    /// scope と query は同時に効く(AND) — mock の `state.source`/`state.tag`/
    /// `state.query` が同時に絞り込みへ効くのと同じ形。
    #[test]
    fn scope_and_query_combine_with_and_semantics() {
        let items = mixed_ledger();
        let narrowed = visible(&items, RailScope::Video, "cutaway");
        assert_eq!(narrowed.len(), 1);
        assert_eq!(narrowed[0].name, "cutaway");

        // video scope だが image の名前で検索 → 0件。
        assert!(visible(&items, RailScope::Video, "logo").is_empty());
    }

    // -----------------------------------------------------------------
    // B3 EXACT TARGET: format_duration(カード grid の「尺」表示、純関数)。
    // -----------------------------------------------------------------

    #[test]
    fn format_duration_renders_none_as_an_em_dash() {
        assert_eq!(format_duration(None), "—");
    }

    #[test]
    fn format_duration_renders_whole_seconds_as_mmss() {
        let five_seconds = RationalTime::try_new(5, 1).unwrap();
        assert_eq!(format_duration(Some(five_seconds)), "0:05");
    }

    #[test]
    fn format_duration_carries_minutes_over() {
        let two_minutes_five = RationalTime::try_new(125, 1).unwrap();
        assert_eq!(format_duration(Some(two_minutes_five)), "2:05");
    }

    #[test]
    fn format_duration_rounds_to_the_nearest_second() {
        // 4.6秒 → 丸めて5秒(切り捨てだと利用者に「短く見える」誤差になる)。
        let almost_five = RationalTime::try_new(23, 5).unwrap();
        assert_eq!(format_duration(Some(almost_five)), "0:05");
    }

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

    // -----------------------------------------------------------------
    // B08 第4切片(素材の整理): 検索の対象拡張(path)+ SortKey/sorted +
    // ViewMode。**未実行**(発注書の検収線は `cargo check --tests` まで)。
    // -----------------------------------------------------------------

    /// OUTCOME「検索の対象拡張」— path(`AssetListItem::path`、
    /// `Asset::path_absolute` の写し)の部分一致でも絞れる。名前とは無関係な
    /// 語で、path にだけ現れる語で検索する(名前一致との偶然の重なりを排除)。
    #[test]
    fn query_matches_path_substring_even_when_the_name_does_not() {
        let items = vec![AssetListItem {
            id: motolii_store::AssetId::from_raw(0),
            name: "field-recording".to_owned(),
            kind: "audio/wav".to_owned(),
            path: Some("/mnt/archive/legacy-drive/tape-07.wav".to_owned()),
            fingerprint: "sha256:tape07".to_owned(),
            duration: None,
        }];
        let narrowed = visible(&items, RailScope::AllMedia, "tape-07");
        assert_eq!(narrowed.len(), 1, "path 部分一致で絞れない: {narrowed:?}");

        assert!(
            visible(&items, RailScope::AllMedia, "legacy-drive").len() == 1,
            "path の途中セグメントでも絞れるはず"
        );
    }

    /// path が無い(`None`)素材は path 一致では絞られない(panic もしない —
    /// `Option::is_some_and` の素通し)。名前一致では引き続き絞れる。
    #[test]
    fn query_ignores_missing_path_without_panicking() {
        let items = vec![AssetListItem {
            id: motolii_store::AssetId::from_raw(0),
            name: "generated-noise".to_owned(),
            kind: "audio/wav".to_owned(),
            path: None,
            fingerprint: "sha256:noise".to_owned(),
            duration: None,
        }];
        assert!(visible(&items, RailScope::AllMedia, "mnt").is_empty());
        assert_eq!(visible(&items, RailScope::AllMedia, "generated").len(), 1);
    }

    fn tagged_ledger() -> Vec<AssetListItem> {
        vec![
            AssetListItem {
                id: AssetId::from_raw(0),
                name: "Bravo".to_owned(),
                kind: "video/mp4".to_owned(),
                path: None,
                fingerprint: "sha256:0".to_owned(),
                duration: None,
            },
            AssetListItem {
                id: AssetId::from_raw(1),
                name: "alpha".to_owned(),
                kind: "audio/wav".to_owned(),
                path: None,
                fingerprint: "sha256:1".to_owned(),
                duration: None,
            },
            AssetListItem {
                id: AssetId::from_raw(2),
                name: "charlie".to_owned(),
                kind: "image/png".to_owned(),
                path: None,
                fingerprint: "sha256:2".to_owned(),
                duration: None,
            },
        ]
    }

    /// **ORACLE**: `SortKey::Name` は大小無視のアルファベット順(admit 順の
    /// "Bravo"(id0)/"alpha"(id1)/"charlie"(id2) を並べ替えて
    /// alpha→Bravo→charlie にする)。
    #[test]
    fn sorted_by_name_orders_case_insensitively() {
        let names: Vec<String> = sorted(&tagged_ledger(), SortKey::Name)
            .into_iter()
            .map(|item| item.name)
            .collect();
        assert_eq!(names, ["alpha", "Bravo", "charlie"]);
    }

    /// **ORACLE**: `SortKey::AddedDate` は `AssetId` 降順(新しい方が先 —
    /// store に wall-clock timestamp が無いための代理指標、`SortKey::AddedDate`
    /// doc 参照)。
    #[test]
    fn sorted_by_added_date_puts_the_newest_id_first() {
        let ids: Vec<u64> = sorted(&tagged_ledger(), SortKey::AddedDate)
            .iter()
            .map(|item| item.id.get())
            .collect();
        assert_eq!(ids, [2, 1, 0]);
    }

    /// **ORACLE**: `SortKey::Kind` は `kind` 文字列の辞書順(audio < image <
    /// video)— mime prefix でグルーピングされる。
    #[test]
    fn sorted_by_kind_groups_the_mime_prefix() {
        let kinds: Vec<String> = sorted(&tagged_ledger(), SortKey::Kind)
            .into_iter()
            .map(|item| item.kind)
            .collect();
        assert_eq!(kinds, ["audio/wav", "image/png", "video/mp4"]);
    }

    /// **本命(安定性)**: 同じキーを持つ2件は元の相対順(admit 順)を保つ —
    /// 並べ替えが無関係な取りこぼし/入れ替わりを起こさないことの oracle。
    /// 同名2件(大小違いのみ)を admit 順のまま2件用意し、`Name` ソート後も
    /// 元の順(id0 が先)のままであることを確認する。
    #[test]
    fn sorted_is_stable_for_equal_keys() {
        let items = vec![
            AssetListItem {
                id: AssetId::from_raw(0),
                name: "Clip".to_owned(),
                kind: "video/mp4".to_owned(),
                path: None,
                fingerprint: "sha256:first".to_owned(),
                duration: None,
            },
            AssetListItem {
                id: AssetId::from_raw(1),
                name: "clip".to_owned(),
                kind: "video/mov".to_owned(),
                path: None,
                fingerprint: "sha256:second".to_owned(),
                duration: None,
            },
        ];
        let ids: Vec<u64> = sorted(&items, SortKey::Name)
            .iter()
            .map(|item| item.id.get())
            .collect();
        assert_eq!(
            ids,
            [0, 1],
            "同名(大小違いのみ)の2件で admit 順が入れ替わった(安定ソート違反)"
        );

        // Kind でも同じ検証(両方 video/* の2件だが文字列としては別 —
        // ここは kind 文字列そのものが同一な別ケースで安定性を見る)。
        let same_kind = vec![
            AssetListItem {
                id: AssetId::from_raw(5),
                name: "zeta".to_owned(),
                kind: "video/mp4".to_owned(),
                path: None,
                fingerprint: "sha256:5".to_owned(),
                duration: None,
            },
            AssetListItem {
                id: AssetId::from_raw(6),
                name: "yankee".to_owned(),
                kind: "video/mp4".to_owned(),
                path: None,
                fingerprint: "sha256:6".to_owned(),
                duration: None,
            },
        ];
        let ids: Vec<u64> = sorted(&same_kind, SortKey::Kind)
            .iter()
            .map(|item| item.id.get())
            .collect();
        assert_eq!(
            ids,
            [5, 6],
            "同じ kind 文字列の2件で admit 順が入れ替わった(安定ソート違反)"
        );
    }

    /// **ORACLE**: `SORT_KEYS` の並び・ラベルは発注書の掲載順(Name → Date
    /// added → Type)。
    #[test]
    fn sort_keys_follow_the_declared_order_and_labels() {
        assert_eq!(SORT_KEYS, [SortKey::Name, SortKey::AddedDate, SortKey::Kind]);
        let labels: Vec<&str> = SORT_KEYS.into_iter().map(SortKey::label).collect();
        assert_eq!(labels, ["Name", "Date added", "Type"]);
    }

    /// **ORACLE**: `ViewMode` の既定は `Grid`(mock 既定表示 `data-view="grid"`
    /// に一致)。
    #[test]
    fn view_mode_defaults_to_grid() {
        assert_eq!(ViewMode::default(), ViewMode::Grid);
        assert_eq!(ViewMode::Grid.tooltip_label(), "Grid view");
        assert_eq!(ViewMode::List.tooltip_label(), "List view");
    }

    // -----------------------------------------------------------------
    // B08 map 616/617: asset_to_layer_source / can_replace_source(純関数)。
    // -----------------------------------------------------------------

    fn asset_with_paths(path_absolute: Option<&str>, path_project_relative: Option<&str>) -> Asset {
        Asset {
            id: AssetId::from_raw(0),
            name: "clip".to_owned(),
            asset_type: "video/mp4".to_owned(),
            content_hash: "sha256:abc".to_owned(),
            path_absolute: path_absolute.map(str::to_owned),
            path_project_relative: path_project_relative.map(str::to_owned),
            // 2026-08-23 A-3: 「今そこに在るか」は環境の事実で保存されない
            // (`#[serde(skip)]`)。試験の固定値は「未確認」が正しい既定。
            status: motolii_store::AssetStatus::Unchecked,
            file_name: None,
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
            duration: None,
        }
    }

    /// **ORACLE**: `path_absolute` がある時はそちらを優先する。
    #[test]
    fn asset_to_layer_source_prefers_the_absolute_path() {
        let asset = asset_with_paths(Some("/abs/clip.mp4"), Some("media/clip.mp4"));
        let source = asset_to_layer_source(&asset).expect("path があるので Some のはず");
        assert_eq!(
            source,
            LayerSource::Media {
                path: "/abs/clip.mp4".to_owned(),
                fingerprint: Some("sha256:abc".to_owned()),
            }
        );
    }

    /// `path_absolute` が無ければ `path_project_relative` へ落ちる。
    #[test]
    fn asset_to_layer_source_falls_back_to_the_project_relative_path() {
        let asset = asset_with_paths(None, Some("media/clip.mp4"));
        let source = asset_to_layer_source(&asset).expect("relative path があるので Some のはず");
        assert_eq!(
            source,
            LayerSource::Media {
                path: "media/clip.mp4".to_owned(),
                fingerprint: Some("sha256:abc".to_owned()),
            }
        );
    }

    /// 両方無い(非ファイル素材)は `None` — 置換できる実体が無い。
    #[test]
    fn asset_to_layer_source_is_none_without_any_path() {
        let asset = asset_with_paths(None, None);
        assert_eq!(asset_to_layer_source(&asset), None);
    }

    /// 選択0件は拒む(曖昧な置換先を作らない)。
    #[test]
    fn can_replace_source_refuses_when_no_layer_is_selected() {
        let asset = asset_with_paths(Some("/abs/clip.mp4"), None);
        assert_eq!(can_replace_source(None, &asset), None);
    }

    /// **ORACLE**: 単一選択+有効な素材 → その layer を返す。
    #[test]
    fn can_replace_source_returns_the_single_selected_layer() {
        let asset = asset_with_paths(Some("/abs/clip.mp4"), None);
        let layer = LayerId(7);
        assert_eq!(can_replace_source(Some(layer), &asset), Some(layer));
    }

    /// 単一選択があってもパスの無い素材は拒む。
    #[test]
    fn can_replace_source_refuses_an_asset_without_a_usable_path() {
        let asset = asset_with_paths(None, None);
        let layer = LayerId(7);
        assert_eq!(can_replace_source(Some(layer), &asset), None);
    }
}
