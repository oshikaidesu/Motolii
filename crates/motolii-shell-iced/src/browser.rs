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

use std::path::{Path, PathBuf};

use motolii_doc::Document;

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
    rail: BrowserRail,
    selected: Option<String>,
    drop_hover: bool,
}

impl BrowserPane {
    /// pane 既定(starter media を登録 folder として1本)。
    pub fn default_shell() -> Self {
        Self::with_root(default_library_root())
    }

    pub fn with_root(_root: PathBuf) -> Self {
        Self {
            rail: BrowserRail::default(),
            selected: None,
            drop_hover: false,
        }
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

    /// 登録 folder の名乗り(空状態の文言が folder を名指しするのに使う)。
    pub fn library_root_name(&self) -> String {
        String::new()
    }

    /// いまの rail の card 一覧。
    pub fn cards(
        &self,
        _document: Option<&Document>,
        _project_root: Option<&Path>,
    ) -> Vec<BrowserCard> {
        Vec::new()
    }

    /// card 1枚が指している実ファイル(配置要求の中身)。
    /// 解決できない(消えた・移動した)なら `None` — 存在しない path を
    /// intent にして流さない。
    pub fn place_path(
        &self,
        _id: &str,
        _document: Option<&Document>,
        _project_root: Option<&Path>,
    ) -> Option<PathBuf> {
        None
    }

    /// Document の image asset の縮小実体を(まだ無ければ)作る。
    /// 呼ぶのは殻の `update` — 描画中に fs を触らない。
    pub fn ensure_asset_thumbnails(
        &mut self,
        _document: &Document,
        _project_root: Option<&Path>,
        _revision: u64,
    ) {
    }

    /// 出せなかったサムネイル等の理由を引き取る(引き取ったら空)。
    /// 殻が `ShellTranscript` へ写す = status 帯経路(F-09)。
    pub fn take_notices(&mut self) -> Vec<String> {
        Vec::new()
    }
}

/// 既定の登録 folder(repo の starter media)。
fn default_library_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/mocks-ui/starter-media/media")
}
