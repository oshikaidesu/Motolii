//! Browser pane の read-model(M-4a)。
//!
//! 移植元は egui 版 `motolii_ui::browser_panel`(card 投影・選択・配置要求)と
//! `motolii_ui::media_library`(走査・種別判定・path 解決 — **ここで再実装しない**)。
//! サムネイルも既存座席 `motolii_ui::browser_blitz::thumbnail` の縮小実体を
//! そのまま使い、失敗は stderr でなく status 帯経路へ返す(2026-08-18 外部診断
//! F-09 の流儀)。
//!
//! ## source rail は機能する3席だけ
//!
//! `All media / Project / Recent`。COLLECTIONS・Add folder のような
//! 「触れそうで触れない」rail は**置かない**(Q0。egui 版 mock の残骸を
//! 持ち込まない)。
//!
//! - **All media** … 登録 folder(starter media)の実走査 + open 中 Document の
//!   asset 台帳を、**同じ file を canonical path で dedupe** して1面に出す。
//!   Project 登録済みかどうかは card の meta が言う
//!   (`docs/ui-interaction-language.md` Browser 節の Media 統合 shell)
//! - **Project** … Document の asset 台帳そのもの(`Document::assets`)。
//!   二重帳簿を作らない — 正本は Document ただ1つ(Q5)
//! - **Recent** … 同じ台帳を新しい順(AssetId は単調採番なので、id 降順 =
//!   取り込みの新しい順)
//!
//! ## このパネルは Document を1バイトも書かない
//!
//! カードのダブルクリックは「この実ファイルを playhead へ置いてくれ」という
//! **要求**であり、実行は殻が `UiIntent::AdmitPaths` を `ShellGateway::dispatch`
//! へ流す(OS ドロップと同じ1本の合流点。egui 版 `BrowserRequest::PlaceFile` と
//! 同じ分担)。選択は Document 外の view 状態で、`UiIntent` にはまだ view 系の
//! 変種が無い(`blitz_shell/intent.rs` の将来枠)。**足りない intent はここで
//! 発明せず、pane の中の状態に留める。**

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use motolii_doc::{resolve_asset_path, Asset, AssetId, Document};
use motolii_ui::browser_blitz::{thumbnail, BrowserItem};
use motolii_ui::media_library::{LibraryItem, LibraryProjection, MediaLibrary};

/// source rail の3席。**機能する物だけ**(Q0)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrowserRail {
    /// 登録 folder + Document asset の統合面(dedupe あり)。
    #[default]
    AllMedia,
    /// Document の asset 台帳。
    Project,
    /// 同じ台帳を取り込みの新しい順で。
    Recent,
}

/// 結果 grid の表示形(browser-library.html:94-97 `.viewModes` の3席)。
/// **browser-revise レーンの追加**(egui 版 `browser_panel::BrowserView` と
/// 同じ意味関数の移植 — 見た目は `browser_pane.rs` 側、ここは列数に効く
/// 状態を持つだけ)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrowserViewMode {
    /// browser-library.css:269 `[data-view="thumbnails"]` — 4列、名前無し。
    Thumbnails,
    /// 既定(html:96 `aria-pressed="true"`)。2列、名前+meta 付き。
    #[default]
    Grid,
    /// browser-library.css:273 `[data-view="list"]` — 1列、横並びの行。
    List,
}

/// card 1枚ぶんの表示投影。egui 版 `BrowserCardModel` の対応物。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserCard {
    /// pane 内で card を名指しする id(`lib:…` / `asset:…`)。
    pub id: String,
    /// 表示名(file 名)。運転席はこの文字列で card を掴む(完全一致)。
    pub name: String,
    /// video / audio / image / file。
    pub kind: &'static str,
    /// 2行目(`video · mp4` の形。All media では登録状態も言う)。
    pub meta: String,
    pub selected: bool,
    /// Project の asset 台帳に載っているか(All media の dedupe 表示)。
    pub in_project: bool,
    /// 縮小実体の path(image kind で作れた物だけ)。元画像へは戻さない。
    pub thumbnail: Option<PathBuf>,
}

/// Browser pane 1面の状態。Document は**持たない**(読みは都度 snapshot を渡される)。
pub struct BrowserPane {
    /// 走査・種別判定・path 解決の owner(egui 版と同じ `media_library`)。
    library: MediaLibrary,
    /// 走査結果。構築時に1回だけ scan する(毎フレーム fs を触らない)。
    projection: LibraryProjection,
    /// 登録 folder の item id → canonical path(dedupe とサムネイル参照に使う)。
    library_paths: BTreeMap<String, PathBuf>,
    /// 実ファイル(canonical path)→ 縮小実体。既存座席
    /// (`browser_blitz::thumbnail`)が作った cache の path。
    thumbnails: BTreeMap<PathBuf, PathBuf>,
    /// 既に縮小実体を試した source。**失敗しても再試行しない**
    /// (egui 版が `None` を覚えるのと同じ方針)。
    attempted: BTreeSet<PathBuf>,
    /// 出せなかった理由の控え。殻が status 帯へ写す(F-09)。
    notices: Vec<String>,
    /// `ensure_asset_thumbnails` が最後に見た writer 世代。
    seen_revision: Option<u64>,
    rail: BrowserRail,
    selected: Option<String>,
    drop_hover: bool,
    /// browser-library.html:25 検索欄の移植(egui `BrowserPanel::query` と同じ
    /// 意味関数 — 小文字化して trim 済みの語彙で name/meta を照合する)。
    query: String,
    /// browser-library.html:103 filter chip(Video/Image/Audio)の移植。
    /// `card.kind` と完全一致する物だけを残す(egui `BrowserPanel::kind` と同じ)。
    kind_filter: Option<&'static str>,
    /// 結果の表示形(html:94-97)。列数は `browser_pane.rs` 側が読む。
    view: BrowserViewMode,
    /// filter shelf の開閉(html:26 `#filter-toggle` — 既定は開、
    /// `docs/ui-interaction-language.md`:122 の「常時同時表示しない」を
    /// 利用者が自分で開閉できる形で満たす)。
    shelf_open: bool,
}

impl BrowserPane {
    /// pane 既定(starter media を登録 folder として1本)。
    pub fn default_shell() -> Self {
        Self::with_root(default_library_root())
    }

    pub fn with_root(root: PathBuf) -> Self {
        let library = MediaLibrary::with_root(root);
        let projection = library.project();
        let library_paths = projection
            .items
            .iter()
            .filter_map(|item| {
                library
                    .resolve(&item.id)
                    .map(|resolved| (item.id.clone(), canon(&resolved.path)))
            })
            .collect();
        let mut pane = Self {
            library,
            projection,
            library_paths,
            thumbnails: BTreeMap::new(),
            attempted: BTreeSet::new(),
            notices: Vec::new(),
            seen_revision: None,
            rail: BrowserRail::default(),
            selected: None,
            drop_hover: false,
            query: String::new(),
            kind_filter: None,
            view: BrowserViewMode::default(),
            shelf_open: true,
        };
        // 登録 folder の image kind の縮小実体(既存座席)。
        let items: Vec<BrowserItem> = pane
            .projection
            .items
            .iter()
            .filter(|item| item.kind == "image")
            .filter_map(|item| pane.library.resolve(&item.id))
            .collect();
        pane.prepare_thumbnails(items);
        pane
    }

    pub fn rail(&self) -> BrowserRail {
        self.rail
    }

    pub fn set_rail(&mut self, rail: BrowserRail) {
        self.rail = rail;
    }

    pub fn select(&mut self, id: &str) {
        self.selected = Some(id.to_owned());
    }

    pub fn selected_id(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    pub fn drop_hover(&self) -> bool {
        self.drop_hover
    }

    pub fn set_drop_hover(&mut self, hovering: bool) {
        self.drop_hover = hovering;
    }

    /// いまの検索語(小文字化・trim 済み)。空 = 絞り込み無し。
    pub fn query(&self) -> &str {
        &self.query
    }

    /// 検索欄の移植(egui `BrowserPanel::set_query` と同じ正規化)。
    pub fn set_query(&mut self, query: &str) {
        self.query = query.trim().to_lowercase();
    }

    /// filter chip(Video/Image/Audio)のいまの選択。`None` = 絞り込み無し。
    pub fn kind_filter(&self) -> Option<&'static str> {
        self.kind_filter
    }

    pub fn set_kind_filter(&mut self, kind: Option<&'static str>) {
        self.kind_filter = kind;
    }

    /// 結果の表示形(html:94-97 view mode)。
    pub fn view(&self) -> BrowserViewMode {
        self.view
    }

    pub fn set_view(&mut self, view: BrowserViewMode) {
        self.view = view;
    }

    /// filter shelf が開いているか(html:26 `#filter-toggle` の押下状態)。
    pub fn shelf_open(&self) -> bool {
        self.shelf_open
    }

    pub fn set_shelf_open(&mut self, open: bool) {
        self.shelf_open = open;
    }

    /// 登録 folder の名乗り(空状態の文言が folder を名指しするのに使う)。
    pub fn library_root_name(&self) -> String {
        self.projection
            .root
            .as_ref()
            .map(|root| root.name.clone())
            .unwrap_or_else(|| "library".to_owned())
    }

    /// いまの rail の card 一覧。検索欄・kind chip の絞り込みを最後に通す
    /// (browser-revise レーンの追加)。**既定(空文字・`None`)は常に真** —
    /// 呼び出し側が絞り込みを一度も触らなければ、この関数の挙動は
    /// 絞り込み追加前と一字も変わらない(既存の意味を壊さない)。
    pub fn cards(
        &self,
        document: Option<&Document>,
        project_root: Option<&Path>,
    ) -> Vec<BrowserCard> {
        let cards = self.rail_cards(document, project_root);
        cards
            .into_iter()
            .filter(|card| self.passes_filters(card))
            .collect()
    }

    /// 検索欄(名前+meta の部分一致)と kind chip(完全一致)。
    /// meta には html:123 と同じ形(`kind · secondary[· in project]`)が
    /// 既に入っているので、tag 相当の語彙(拡張子・登録状態)もここで拾える。
    fn passes_filters(&self, card: &BrowserCard) -> bool {
        let kind_ok = self.kind_filter.is_none_or(|kind| card.kind == kind);
        let query_ok = self.query.is_empty() || {
            let haystack = format!("{} {}", card.name, card.meta).to_lowercase();
            haystack.contains(&self.query)
        };
        kind_ok && query_ok
    }

    fn rail_cards(
        &self,
        document: Option<&Document>,
        project_root: Option<&Path>,
    ) -> Vec<BrowserCard> {
        let assets: Vec<&Asset> = document
            .map(|document| document.assets.iter().collect())
            .unwrap_or_default();
        match self.rail {
            BrowserRail::Project => assets
                .iter()
                .map(|asset| self.asset_card(asset, project_root, false))
                .collect(),
            BrowserRail::Recent => assets
                .iter()
                .rev()
                .map(|asset| self.asset_card(asset, project_root, false))
                .collect(),
            BrowserRail::AllMedia => {
                // Document 側の canonical path(登録状態の判定台)。
                let registered: BTreeSet<PathBuf> = assets
                    .iter()
                    .filter_map(|asset| resolve_asset_path(asset, project_root))
                    .map(|path| canon(&path))
                    .collect();
                let mut covered: BTreeSet<PathBuf> = BTreeSet::new();
                let mut cards: Vec<BrowserCard> = self
                    .projection
                    .items
                    .iter()
                    .map(|item| {
                        let path = self.library_paths.get(&item.id);
                        let in_project = path.is_some_and(|path| registered.contains(path));
                        if let Some(path) = path {
                            covered.insert(path.clone());
                        }
                        self.library_card(item, in_project)
                    })
                    .collect();
                // 登録 folder の外から取り込んだ asset も All media に居る
                // (同じ file は canonical path で dedupe 済み)。
                for asset in &assets {
                    let duplicated = resolve_asset_path(asset, project_root)
                        .map(|path| canon(&path))
                        .is_some_and(|path| covered.contains(&path));
                    if !duplicated {
                        cards.push(self.asset_card(asset, project_root, true));
                    }
                }
                cards
            }
        }
    }

    /// card 1枚が指している実ファイル(配置要求の中身)。
    /// 解決できない(消えた・移動した)なら `None` — 存在しない path を
    /// intent にして流さない(egui 版 `place_request` と同じ判断)。
    pub fn place_path(
        &self,
        id: &str,
        document: Option<&Document>,
        project_root: Option<&Path>,
    ) -> Option<PathBuf> {
        if let Some(item_id) = id.strip_prefix("lib:") {
            return self.library.resolve(item_id).map(|resolved| resolved.path);
        }
        let raw = id.strip_prefix("asset:")?.parse::<u64>().ok()?;
        let asset = document?.assets.get(AssetId::from_raw(raw))?;
        resolve_asset_path(asset, project_root)
    }

    /// Document の image asset の縮小実体を(まだ無ければ)作る。
    /// 呼ぶのは殻の `update` — 描画中に fs を触らない。世代が進んでいなければ
    /// 何もしない(status 帯が毎 Message 呼んでも fs を舐めない)。
    pub fn ensure_asset_thumbnails(
        &mut self,
        document: &Document,
        project_root: Option<&Path>,
        revision: u64,
    ) {
        if self.seen_revision == Some(revision) {
            return;
        }
        self.seen_revision = Some(revision);
        let items: Vec<BrowserItem> = document
            .assets
            .iter()
            .filter(|asset| asset.asset_type.starts_with("image/"))
            .filter_map(|asset| {
                resolve_asset_path(asset, project_root).map(|path| BrowserItem {
                    id: format!("asset:{}", asset.id.get()),
                    name: asset_display_name(asset),
                    kind: "image",
                    path,
                    // `prepare` は path しか見ない。ここの型名は判定に使われない。
                    asset_type: "image",
                })
            })
            .collect();
        self.prepare_thumbnails(items);
    }

    /// 出せなかったサムネイル等の理由を引き取る(引き取ったら空)。
    /// 殻が `ShellTranscript` へ写す = status 帯経路(F-09)。
    pub fn take_notices(&mut self) -> Vec<String> {
        std::mem::take(&mut self.notices)
    }

    // ---- 内部 ----

    /// 縮小実体を既存座席で作る。**decode できない型(svg 等)は試さない** —
    /// 元から glyph card で描く設計なので、毎起動おなじ理由を帯へ積まない。
    fn prepare_thumbnails(&mut self, items: Vec<BrowserItem>) {
        let fresh: Vec<BrowserItem> = items
            .into_iter()
            .filter(|item| decodable_image(&item.path))
            .map(|mut item| {
                item.path = canon(&item.path);
                item
            })
            .filter(|item| !self.attempted.contains(&item.path))
            .collect();
        if fresh.is_empty() {
            return;
        }
        for (prepared, item) in thumbnail::prepare(&fresh).into_iter().zip(&fresh) {
            self.attempted.insert(item.path.clone());
            match prepared {
                Ok(thumb) => {
                    self.thumbnails.insert(item.path.clone(), thumb);
                }
                Err(reason) => self.notices.push(format!(
                    "browser: {} の縮小実体を作れないので glyph card で描く: {reason}",
                    item.path.display()
                )),
            }
        }
    }

    fn library_card(&self, item: &LibraryItem, in_project: bool) -> BrowserCard {
        let id = format!("lib:{}", item.id);
        // card 2行目(egui 版 `card_meta` と同じ形): kind の次の派生 tag = 拡張子。
        let secondary = item
            .tags
            .iter()
            .find(|tag| tag.as_str() != item.kind)
            .cloned()
            .unwrap_or_else(|| item.kind.to_owned());
        let mut meta = format!("{} · {}", item.kind, secondary);
        if in_project {
            // Project 登録状態を結果上へ示す(ui-interaction-language の dedupe 表示)。
            meta.push_str(" · in project");
        }
        let thumbnail = self
            .library_paths
            .get(&item.id)
            .and_then(|path| self.thumbnails.get(path).cloned());
        BrowserCard {
            selected: self.selected.as_deref() == Some(id.as_str()),
            id,
            name: item.name.clone(),
            kind: item.kind,
            meta,
            in_project,
            thumbnail,
        }
    }

    fn asset_card(&self, asset: &Asset, project_root: Option<&Path>, badge: bool) -> BrowserCard {
        let id = format!("asset:{}", asset.id.get());
        let name = asset_display_name(asset);
        let kind = asset_kind(&asset.asset_type);
        let secondary = Path::new(&name)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .unwrap_or_else(|| asset.asset_type.clone());
        let mut meta = format!("{kind} · {secondary}");
        if badge {
            meta.push_str(" · in project");
        }
        let thumbnail = resolve_asset_path(asset, project_root)
            .map(|path| canon(&path))
            .and_then(|path| self.thumbnails.get(&path).cloned());
        BrowserCard {
            selected: self.selected.as_deref() == Some(id.as_str()),
            id,
            name,
            kind,
            meta,
            in_project: true,
            thumbnail,
        }
    }
}

/// asset の表示名。拡張子つきの file 名を優先する(登録 folder の card と
/// 同じ file が同じ名前で出る = dedupe 表示が読める)。
fn asset_display_name(asset: &Asset) -> String {
    asset
        .file_name
        .clone()
        .unwrap_or_else(|| asset.name.clone())
}

/// asset_type(opaque 文字列)→ card の kind。
fn asset_kind(asset_type: &str) -> &'static str {
    if asset_type.starts_with("video/") {
        "video"
    } else if asset_type.starts_with("audio/") {
        "audio"
    } else if asset_type.starts_with("image/") {
        "image"
    } else {
        "file"
    }
}

/// 縮小実体の座席(`image` crate)が decode できる型か。svg はできない —
/// egui 版も svg は glyph card で描く設計なので、試して毎回失敗を積まない。
fn decodable_image(path: &Path) -> bool {
    !path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
}

/// symlink・`./` の違いで同じ file が2枚にならないよう、取れるなら canonical。
fn canon(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// 既定の登録 folder(repo の starter media)。
fn default_library_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/mocks-ui/starter-media/media")
}
