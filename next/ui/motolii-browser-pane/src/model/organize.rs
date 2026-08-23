//! 素材の整理: 並べ替え+表示形式(SP-6 分割: 元 `model.rs` から移送)。

use super::projection::AssetListItem;

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
    use motolii_store::{AssetId, AssetStatus};

    fn tagged_ledger() -> Vec<AssetListItem> {
        vec![
            AssetListItem {
                id: AssetId::from_raw(0),
                name: "Bravo".to_owned(),
                kind: "video/mp4".to_owned(),
                path: None,
                fingerprint: "sha256:0".to_owned(),
                duration: None,
                status: AssetStatus::Unchecked,
            },
            AssetListItem {
                id: AssetId::from_raw(1),
                name: "alpha".to_owned(),
                kind: "audio/wav".to_owned(),
                path: None,
                fingerprint: "sha256:1".to_owned(),
                duration: None,
                status: AssetStatus::Unchecked,
            },
            AssetListItem {
                id: AssetId::from_raw(2),
                name: "charlie".to_owned(),
                kind: "image/png".to_owned(),
                path: None,
                fingerprint: "sha256:2".to_owned(),
                duration: None,
                status: AssetStatus::Unchecked,
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
                status: AssetStatus::Unchecked,
            },
            AssetListItem {
                id: AssetId::from_raw(1),
                name: "clip".to_owned(),
                kind: "video/mov".to_owned(),
                path: None,
                fingerprint: "sha256:second".to_owned(),
                duration: None,
                status: AssetStatus::Unchecked,
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
                status: AssetStatus::Unchecked,
            },
            AssetListItem {
                id: AssetId::from_raw(6),
                name: "yankee".to_owned(),
                kind: "video/mp4".to_owned(),
                path: None,
                fingerprint: "sha256:6".to_owned(),
                duration: None,
                status: AssetStatus::Unchecked,
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
}
