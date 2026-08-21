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

use motolii_store::{AssetId, RationalTime, StoreView};

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
pub const RAIL_SCOPES: [RailScope; 4] =
    [RailScope::AllMedia, RailScope::Video, RailScope::Images, RailScope::Audio];

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
/// - query: 表示名/fingerprint の部分一致・大小無視・前後空白無視(OUTCOME
///   「文字列は fingerprint/表示名マッチ」)。空文字列は「絞らない」。
/// - 順序は [`assets`] と同じ決定論(admit 順)を保つ — この関数は並べ替えない。
pub fn visible(items: &[AssetListItem], scope: RailScope, query: &str) -> Vec<AssetListItem> {
    let query = query.trim().to_lowercase();
    items
        .iter()
        .filter(|item| scope.matches(category_of(&item.kind)))
        .filter(|item| {
            query.is_empty()
                || item.name.to_lowercase().contains(&query)
                || item.fingerprint.to_lowercase().contains(&query)
        })
        .cloned()
        .collect()
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
        assert!(
            items[0].id < items[1].id,
            "admit 順(id 昇順)を保っていない"
        );
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
        assert_eq!(category_of(""), Category::Other, "空文字列は Other へ丸まるはず");
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
        assert!(narrowed.iter().all(|item| category_of(&item.kind) == Category::Video));
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
}
