//! filter rail の scope(SP-6 分割: 元 `model.rs` から移送 — 素材種別による
//! rail/filter の絞り込み語彙。`Category` は `projection` モジュール側に
//! 残った投影の基礎型なのでここから参照する)。

use super::projection::Category;

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
    pub(crate) fn matches(self, category: Category) -> bool {
        match self {
            Self::AllMedia => true,
            Self::Video => matches!(category, Category::Video),
            Self::Images => matches!(category, Category::Image),
            Self::Audio => matches!(category, Category::Audio),
        }
    }
}
