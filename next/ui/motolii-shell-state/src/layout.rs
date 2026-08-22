//! ワークスペースレイアウト状態(B26 発注書、2026-08-22)。
//!
//! ## 背景 — 既知の穴
//! `next/shell/motolii-shell/src/pane_layout.rs`(モジュール冒頭 doc の
//! 「既知の限界」節)には既知の未解決バグがある: Browser パネルを開閉する
//! たびに `pane_grid::Configuration` を**既定形で作り直す**
//! (`pane_layout::build_state`)。ユーザーがドラッグで Inspector/Stage の
//! 位置を入れ替えていても、Browser の開閉でその配置が消え、既定配置
//! (Inspector 左・Stage 右)へ戻ってしまう。原因は同ファイルの doc が
//! 自認するとおり——「レイアウトが `pane_grid::Configuration` にしか無く、
//! Session 水準の状態として持っていない」こと。`Ratios` は比率だけを
//! 開閉をまたいで保持する応急措置で、*配置*(どちらの葉に何があるか)は
//! 保持されない。
//!
//! ## この crate の解法
//! [`LayoutNode`] は分割木の**形そのもの**を Session 水準の純データとして
//! 持つ。パネルを閉じても葉は木から消えない——[`LayoutNode::set_hidden`]で
//! `hidden` フラグを立てるだけ(発注書「隠す=葉にフラグ、が要点」)。
//! 表示用の縮約木([`LayoutNode::collapse_hidden`])は、この永続木から
//! **その場で導出**するだけなので、開閉のたびに何かを失うことがない——
//! ドラッグで入れ替えた配置も、複数回の開閉をまたいで生き残る。
//!
//! ## freq降順・第1切片(採用予定36行のうち、この crate で表現できる意味)
//! map の bundle 列 B26(`docs/reviews/2026-08-22-intent-bundles-draft.md`
//! 95行目、行id `301,303,1092,1112,1494,1496,1497,1498,1502,1505,1506,
//! 1507,1508,1509,1510,1512,1513,1516,1517,1529,1530,1533,1534,1535,1538,
//! 1539,1541,1542,1543,1544,1545,1546,1547,1548,1550,1551`)を
//! `next/reference/normal-map.tsv` で引くと、36行**全てが freq=1**
//! (単一製品にしか出現しない — freq 降順の同点)。同点のため、順位ではなく
//! 「pane_grid の分割木として表現できる意味かどうか」で第1切片を切った:
//!
//! **採用(この crate が表現する)**:
//! - `Reset UI Layout`(id 1534) → [`WorkspaceLayout::reset_to_default`]
//! - `New Workspace…`(1517) → [`WorkspaceBook::save_as`]
//! - `Delete Workspace…`(1505) → [`WorkspaceBook::delete`]
//! - `Switch to workspace`(1545)・`Switch to Page`(1544)・`Show Page`(1538)
//!   → [`WorkspaceBook::switch_to`](Resolve の「Page」は Edit/Color/
//!   Fairlight/Deliver 等の名前付きレイアウト切替そのもので、
//!   `Switch to workspace` と同型の意図と判断した)
//! - `Layout Presets`(1510) → [`WorkspaceBook::names`](一覧)
//! - `Standard`/`Minimal`/`All Panels`/`Undocked Panels`/`Animation`/
//!   `Effects`/`Media Pool Windows`/`Background Activity`/`Gallery`/
//!   `Keyword Dictionary`/`People`/`Timecode Window`/`Show Page Navigation`
//!   (1543/1513/1496/1548/1497/1507/1512/1498/1508/1509/1529/1546/1539)
//!   → これらは追加の操作ではなく、`WorkspaceBook` へ保存する**名前の値**
//!   (製品ごとのプリセット名の列挙)。既に上記の save/switch/delete/names
//!   機構が汎用にカバーする——専用コードは不要と判断した
//! - `Toggle between Composition and Timeline panels`(1547) →
//!   [`LayoutNode::set_hidden`] / [`WorkspaceLayout::hide`] /
//!   [`WorkspaceLayout::show`](パネル単位の表示/非表示そのもの)
//!
//! **不採用(この crate の対象外——別の層の仕事)**:
//! - `Dual Screen`(1506)・`Primary Display`(1530)・`Move application window
//!   to main monitor and resize`(1516)・`Resize application window to fit
//!   screen`(1535): OS ウィンドウ配置(モニタ間)の話であって pane_grid の
//!   分割木ではない。別の層(OS window API)の仕事
//! - `Single Viewer Mode`(1541)・`Split frame and create viewer with
//!   opposite locked/unlocked state`(1542)・`Viewer Mode`(1550)・
//!   `Activate view in multi-view layout`(1494)・`Cycle to previous or next
//!   item in active viewer`(1502): **1つの pane(Stage/Viewer)の中身**の
//!   マルチビューポート状態であって、pane 同士の分割木ではない。該当する
//!   pane 自身の状態として持つべきもの
//! - `Show Next Screen`/`Show Previous Screen`(301/303)・`Next Screen`/
//!   `Previous Screen`(1092/1112): モバイル/CapCut系の「1画面だけを順番に
//!   遷移する」パターン。`alight-motion-as-ux-north-star.md` 裁定
//!   (「モバイル画面推移前提のパターンはデスクトップ常設panelの証拠に
//!   ならない」)により、常設 pane_grid の対象外と判断した
//! - `Remote Rendering`(1533)・`Workflow Integrations`(1551): カテゴリ
//!   自動分類の誤り(normal-map-README.md の既知の限界どおり、
//!   `workspace` カテゴリへの機械分類ミスと見られる)。レイアウトとは
//!   無関係な機能で、この crate はもちろん B26 束自体からも実質的に
//!   スコープ外
//!
//! ## shell 結線手順(次レーンへの引き渡しメモ)
//! `motolii-shell` 側(このレーンの EXACT TARGET 外)は以下を書く:
//!
//! 1. **`Axis` の変換**: [`Axis::Horizontal`]/[`Axis::Vertical`] ↔
//!    `iced::widget::pane_grid::Axis::Horizontal`/`Vertical`(名前まで一致、
//!    1:1 の `From`/`match`)。
//! 2. **`PaneKind` を `K` に当てる**: `pane_layout::PaneKind` をそのまま
//!    `LayoutNode<pane_layout::PaneKind>` の型引数として使う(この crate は
//!    `K: Copy + Eq` しか要求しない——`PaneKind` は既に両方を derive 済み)。
//! 3. **描画直前に縮約木を作る**: 毎フレーム
//!    `workspace.visible_layout()`(= [`WorkspaceLayout::visible_layout`])
//!    を呼び、返ってきた `LayoutNode<PaneKind>` を再帰的に歩いて
//!    `pane_grid::Configuration` を組む(`Leaf` → `Configuration::Pane`、
//!    `Split` → `Configuration::Split` に軸変換をかけて再帰)。`None` が
//!    返る(=全パネルが隠れている)場合は現行の `Layout` と同様に何らかの
//!    fallback(例: 直近に隠した1枚だけ強制的に見せる)を用意すること——
//!    この crate は「全部隠す」を禁止しない代わりに、その場合の見た目は
//!    shell 側の判断に委ねる。
//! 4. **`resize`/`drag` イベントを永続木へ反映する**: pane_grid から来る
//!    `ResizeEvent`/`DragEvent::Dropped` は縮約木の split id を指す
//!    (永続木の split id ではない——縮約で split が消えている場合がある)。
//!    永続木の対応する split を「両側の可視 kind 集合」で同定する
//!    (`pane_layout::extract_ratios` が既に実装しているのと同じ手法——
//!    `LayoutNode::find` で両端の kind から `NodePath` を得て共通祖先を辿る、
//!    または縮約木と永続木を同時に DFS して対応づける)。同定できたら
//!    [`LayoutNode::resize`] へ書き戻す。`DragEvent::Dropped` は
//!    [`WorkspaceLayout::swap`] を呼ぶ(iced 側の swap も「位置は保ち中身を
//!    交換する」同じ意味論——`pane_layout.rs` の
//!    `drag_dropped_on_a_pane_swaps_their_screen_positions_deterministically`
//!    テストの doc 参照)。
//! 5. **開閉トグルの配線**: `Message::Browser` 等のパネル開閉メッセージは
//!    もう `Configuration` を作り直さず、[`WorkspaceLayout::hide`]/
//!    [`WorkspaceLayout::show`] を呼ぶだけにする——`pane_grid::State` 自体は
//!    3の縮約木から都度 `State::with_configuration` で作り直して構わない
//!    (iced 側の `Pane` id は開閉のたびに振り直っても構わない——4で述べた
//!    kind ベースの同定で吸収する)。
//! 6. **永続化の口**: [`WorkspaceBook`]/[`WorkspaceLayout`]/[`LayoutNode`]は
//!    全て `serde::Serialize`/`Deserialize`(K がそれらを満たす前提)。
//!    保存先は Session/Workspace 永続化機構が来たときに配線する
//!    (このレーンの NON-GOAL——`serde_json::to_string`/`from_str` を
//!    そのまま使えることだけを oracle にした、
//!    [`layout_tests::round_trips_through_json`] 参照)。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// 分割の向き。`iced::widget::pane_grid::Axis` と名前まで1:1対応する
/// (この crate 自体は iced に依存しない——変換は shell 側が書く、
/// モジュール冒頭 doc の「shell 結線手順」1参照)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// 分割木を根から降りるときの1歩——`Split` の `a`/`b` どちら側へ進むか
/// (`pane_grid::Configuration::Split` の `a`/`b` フィールドと同じ意味)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Branch {
    A,
    B,
}

/// 分割木の根からある葉(または分岐点)までの経路。
pub type NodePath = Vec<Branch>;

/// pane 分割木——`pane_grid::Configuration<K>` と同じ形
/// (`Leaf`↔`Configuration::Pane`、`Split`↔`Configuration::Split`)。
/// **葉は隠れていても木から消えない**(`hidden` フラグ、モジュール冒頭 doc
/// 参照)——これが B26 発注書の「隠す=葉にフラグ、が要点」に対応する。
///
/// `K` はパネル種別(shell 側の `pane_layout::PaneKind` 等)。この crate は
/// `K: Copy + Eq` しか要求しない——iced にも `motolii-shell` にも依存しない
/// 純データにするための意図的な一般化(発注書「依存を張らずに済むよう、
/// この crate は iced に依存しない純データが望ましい」)。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LayoutNode<K> {
    Leaf { kind: K, hidden: bool },
    Split { axis: Axis, ratio: f32, a: Box<LayoutNode<K>>, b: Box<LayoutNode<K>> },
}

impl<K: Copy + Eq> LayoutNode<K> {
    /// 見えている葉1枚。
    pub fn leaf(kind: K) -> Self {
        LayoutNode::Leaf { kind, hidden: false }
    }

    /// 2つの子を持つ split。
    pub fn split(axis: Axis, ratio: f32, a: LayoutNode<K>, b: LayoutNode<K>) -> Self {
        LayoutNode::Split { axis, ratio, a: Box::new(a), b: Box::new(b) }
    }

    /// `kind` を持つ葉までの経路。隠れている葉も対象(木の形の話であって
    /// 表示の話ではない)。見つからなければ `None`。
    pub fn find(&self, kind: K) -> Option<NodePath> {
        match self {
            LayoutNode::Leaf { kind: k, .. } => (*k == kind).then(Vec::new),
            LayoutNode::Split { a, b, .. } => {
                if let Some(mut path) = a.find(kind) {
                    path.insert(0, Branch::A);
                    Some(path)
                } else if let Some(mut path) = b.find(kind) {
                    path.insert(0, Branch::B);
                    Some(path)
                } else {
                    None
                }
            }
        }
    }

    /// `path` が指す部分木への参照。
    pub fn get(&self, path: &[Branch]) -> Option<&LayoutNode<K>> {
        match path.split_first() {
            None => Some(self),
            Some((Branch::A, rest)) => match self {
                LayoutNode::Split { a, .. } => a.get(rest),
                LayoutNode::Leaf { .. } => None,
            },
            Some((Branch::B, rest)) => match self {
                LayoutNode::Split { b, .. } => b.get(rest),
                LayoutNode::Leaf { .. } => None,
            },
        }
    }

    /// `path` が指す部分木への可変参照。
    pub fn get_mut(&mut self, path: &[Branch]) -> Option<&mut LayoutNode<K>> {
        match path.split_first() {
            None => Some(self),
            Some((Branch::A, rest)) => match self {
                LayoutNode::Split { a, .. } => a.get_mut(rest),
                LayoutNode::Leaf { .. } => None,
            },
            Some((Branch::B, rest)) => match self {
                LayoutNode::Split { b, .. } => b.get_mut(rest),
                LayoutNode::Leaf { .. } => None,
            },
        }
    }

    /// 木の形を保ったまま `kind` の葉の `hidden` を書き換える。見つかれば
    /// `true`。
    pub fn set_hidden(&mut self, kind: K, hidden: bool) -> bool {
        match self {
            LayoutNode::Leaf { kind: k, hidden: h } => {
                if *k == kind {
                    *h = hidden;
                    true
                } else {
                    false
                }
            }
            LayoutNode::Split { a, b, .. } => a.set_hidden(kind, hidden) || b.set_hidden(kind, hidden),
        }
    }

    /// `kind` の葉が今隠れているか(見つからなければ `false`)。
    pub fn is_hidden(&self, kind: K) -> bool {
        match self.find(kind) {
            Some(path) => matches!(self.get(&path), Some(LayoutNode::Leaf { hidden: true, .. })),
            None => false,
        }
    }

    /// 2つの葉が**今いる位置を保ったまま**中身(kind + hidden)を入れ替える
    /// (`pane_grid::State::swap` と同じ意味論——木の形は変えず、その位置に
    /// 何がいるかだけ交換する)。両方見つかれば `true`。
    pub fn swap(&mut self, a: K, b: K) -> bool {
        let (Some(path_a), Some(path_b)) = (self.find(a), self.find(b)) else {
            return false;
        };
        if path_a == path_b {
            return true;
        }
        let (Some(leaf_a), Some(leaf_b)) = (self.get(&path_a).cloned(), self.get(&path_b).cloned()) else {
            return false;
        };
        if let Some(slot) = self.get_mut(&path_a) {
            *slot = leaf_b;
        }
        if let Some(slot) = self.get_mut(&path_b) {
            *slot = leaf_a;
        }
        true
    }

    /// `path` が指す split の比率を書き換える。`path` が split を指して
    /// いなければ `false`。
    pub fn resize(&mut self, path: &[Branch], ratio: f32) -> bool {
        match self.get_mut(path) {
            Some(LayoutNode::Split { ratio: r, .. }) => {
                *r = ratio;
                true
            }
            _ => false,
        }
    }

    /// 見えている葉の kind を深さ優先(a→b)で集める。
    pub fn visible_kinds(&self) -> Vec<K> {
        let mut out = Vec::new();
        self.collect_kinds(false, &mut out);
        out
    }

    /// 隠れているものも含め、全ての葉の kind を深さ優先で集める。
    pub fn all_kinds(&self) -> Vec<K> {
        let mut out = Vec::new();
        self.collect_kinds(true, &mut out);
        out
    }

    fn collect_kinds(&self, include_hidden: bool, out: &mut Vec<K>) {
        match self {
            LayoutNode::Leaf { kind, hidden } => {
                if include_hidden || !hidden {
                    out.push(*kind);
                }
            }
            LayoutNode::Split { a, b, .. } => {
                a.collect_kinds(include_hidden, out);
                b.collect_kinds(include_hidden, out);
            }
        }
    }

    /// 表示用の木を**導出**する(元の木は変えない)。隠れた葉を取り除き、
    /// 片側が丸ごと隠れた split はもう片側の部分木で置き換える
    /// (`pane_grid::Configuration` は隠れ葉を表現できないため、shell は
    /// 描画直前にこの縮約形へ変換してから `Configuration` を組む——
    /// モジュール冒頭 doc「shell 結線手順」3参照)。両側とも隠れていれば
    /// `None`(=全パネルが隠れている)。
    pub fn collapse_hidden(&self) -> Option<LayoutNode<K>> {
        match self {
            LayoutNode::Leaf { kind, hidden } => {
                if *hidden {
                    None
                } else {
                    Some(LayoutNode::Leaf { kind: *kind, hidden: false })
                }
            }
            LayoutNode::Split { axis, ratio, a, b } => match (a.collapse_hidden(), b.collapse_hidden()) {
                (Some(a2), Some(b2)) => {
                    Some(LayoutNode::Split { axis: *axis, ratio: *ratio, a: Box::new(a2), b: Box::new(b2) })
                }
                (Some(a2), None) => Some(a2),
                (None, Some(b2)) => Some(b2),
                (None, None) => None,
            },
        }
    }
}

/// 1つのワークスペースが持つ状態: 現在の分割木 + 既定へ戻すための木。
/// `default` も同じ形の [`LayoutNode`] として持つ——reset は「`default` を
/// `current` へコピーするだけ」の単純な操作にする(shell が reset の都度
/// 既定木を組み直して渡し直す必要がない)。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceLayout<K> {
    current: LayoutNode<K>,
    default: LayoutNode<K>,
}

impl<K: Copy + Eq> WorkspaceLayout<K> {
    /// `default` を現在の木にも既定にも据えて開始する(裁定と同じ「触って
    /// いない既定状態」パターン——`Session::default()`/`TimelineFoldState`
    /// と同型)。
    pub fn new(default: LayoutNode<K>) -> Self {
        Self { current: default.clone(), default }
    }

    /// 現在の分割木(隠れた葉も含む——永続木そのもの)。
    pub fn root(&self) -> &LayoutNode<K> {
        &self.current
    }

    /// パネルを隠す(木の形は保つ、`hidden=true` にするだけ)。
    pub fn hide(&mut self, kind: K) -> bool {
        self.current.set_hidden(kind, true)
    }

    /// パネルを見せる(木の形は保つ、`hidden=false` に戻すだけ)。
    pub fn show(&mut self, kind: K) -> bool {
        self.current.set_hidden(kind, false)
    }

    pub fn is_hidden(&self, kind: K) -> bool {
        self.current.is_hidden(kind)
    }

    /// 2枚のパネルの位置を交換する(ドラッグ&ドロップ相当)。
    pub fn swap(&mut self, a: K, b: K) -> bool {
        self.current.swap(a, b)
    }

    /// `path` の split 比率を書き換える。
    pub fn resize(&mut self, path: &[Branch], ratio: f32) -> bool {
        self.current.resize(path, ratio)
    }

    pub fn visible_kinds(&self) -> Vec<K> {
        self.current.visible_kinds()
    }

    pub fn all_kinds(&self) -> Vec<K> {
        self.current.all_kinds()
    }

    /// 表示用に縮約した木([`LayoutNode::collapse_hidden`])。shell が
    /// `pane_grid::Configuration` を組む直前に呼ぶ想定。
    pub fn visible_layout(&self) -> Option<LayoutNode<K>> {
        self.current.collapse_hidden()
    }

    /// 既定配置へ戻す(`Reset UI Layout`、freq降順第1切片 §1534)。
    /// hide/show/swap/resize の履歴を丸ごと `default` の状態へ戻す。
    pub fn reset_to_default(&mut self) {
        self.current = self.default.clone();
    }

    pub fn default_layout(&self) -> &LayoutNode<K> {
        &self.default
    }
}

/// 名前付きレイアウトの束(`New Workspace…`/`Delete Workspace…`/
/// `Switch to workspace`/`Layout Presets` 系、モジュール冒頭 doc の
/// freq降順第1切片参照)。[`WorkspaceLayout`] 1本(`workspace`、現在
/// アクティブな状態)に対して、名前つきスナップショットを複数保持する。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceBook<K> {
    pub workspace: WorkspaceLayout<K>,
    saved: BTreeMap<String, LayoutNode<K>>,
}

impl<K: Copy + Eq> WorkspaceBook<K> {
    pub fn new(default: LayoutNode<K>) -> Self {
        Self { workspace: WorkspaceLayout::new(default), saved: BTreeMap::new() }
    }

    /// `New Workspace…`: 現在の木を `name` として保存する(既存の同名を
    /// 上書き)。
    pub fn save_as(&mut self, name: impl Into<String>) {
        self.saved.insert(name.into(), self.workspace.root().clone());
    }

    /// `Switch to workspace`/`Switch to Page`/`Show Page`: 保存済みの
    /// `name` を現在の木へ適用する。存在しなければ `false`(現在の木は
    /// 変えない)。
    pub fn switch_to(&mut self, name: &str) -> bool {
        match self.saved.get(name) {
            Some(layout) => {
                self.workspace.current = layout.clone();
                true
            }
            None => false,
        }
    }

    /// `Delete Workspace…`。存在すれば削除して `true`。
    pub fn delete(&mut self, name: &str) -> bool {
        self.saved.remove(name).is_some()
    }

    /// `Layout Presets`: 保存済みレイアウト名の一覧。
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.saved.keys().map(String::as_str)
    }

    pub fn get(&self, name: &str) -> Option<&LayoutNode<K>> {
        self.saved.get(name)
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    /// 実 shell の4パネル(`pane_layout::PaneKind` の写し、テスト専用の
    /// スタンドイン——このテストモジュールが依存を持ち込まないための独立
    /// 定義)。
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    enum Pane {
        Browser,
        Inspector,
        Stage,
        Timeline,
    }

    /// `pane_layout::build_configuration` と同じ既定形:
    /// Browser | ((Inspector | Stage) / Timeline)。
    fn default_tree() -> LayoutNode<Pane> {
        LayoutNode::split(
            Axis::Vertical,
            0.3,
            LayoutNode::leaf(Pane::Browser),
            LayoutNode::split(
                Axis::Horizontal,
                0.62,
                LayoutNode::split(Axis::Vertical, 0.31, LayoutNode::leaf(Pane::Inspector), LayoutNode::leaf(Pane::Stage)),
                LayoutNode::leaf(Pane::Timeline),
            ),
        )
    }

    // -----------------------------------------------------------------
    // 既知の穴の回帰テスト: Browser の開閉をまたいでも、drag で入れ替えた
    // 配置と、resize で変えた比率が両方とも保たれる。
    // -----------------------------------------------------------------

    #[test]
    fn hiding_a_pane_does_not_remove_its_leaf_from_the_tree() {
        let mut workspace = WorkspaceLayout::new(default_tree());

        assert!(workspace.hide(Pane::Browser));

        // 木の形はそのまま——Browser の葉は消えていない、hidden なだけ。
        assert_eq!(workspace.all_kinds(), vec![Pane::Browser, Pane::Inspector, Pane::Stage, Pane::Timeline]);
        assert!(workspace.is_hidden(Pane::Browser));
        // 縮約木(shell が実際に pane_grid へ渡す形)からは消える。
        assert_eq!(workspace.visible_kinds(), vec![Pane::Inspector, Pane::Stage, Pane::Timeline]);
    }

    #[test]
    fn known_hole_regression_swap_and_resize_survive_a_hide_show_round_trip() {
        let mut workspace = WorkspaceLayout::new(default_tree());

        // 1. ドラッグで Inspector と Stage を入れ替える。
        assert!(workspace.swap(Pane::Inspector, Pane::Stage));
        // 2. resize で Inspector|Stage の比率を変える(この split 自体の
        //    パスは根から数えて [B(content),A(inspector|stage split)])。
        let inspector_stage_split = vec![Branch::B, Branch::A];
        assert!(workspace.resize(&inspector_stage_split, 0.5));

        // 3. Browser を閉じる(旧実装なら Configuration が作り直され、
        //    上記の swap/resize が失われていた)。
        assert!(workspace.hide(Pane::Browser));
        // 4. Browser を開き直す。
        assert!(workspace.show(Pane::Browser));

        // オラクル: 入れ替えた配置(split の左葉に Stage)が生きている——
        // swap は位置ではなく中身を交換するため、左葉のパス自体は元のまま
        // ([B,A,A])。
        let left_of_inner_split = workspace.root().get(&[Branch::B, Branch::A, Branch::A]).expect("パスが消えている");
        assert_eq!(left_of_inner_split, &LayoutNode::Leaf { kind: Pane::Stage, hidden: false });
        let right_of_inner_split = workspace.root().get(&[Branch::B, Branch::A, Branch::B]).expect("パスが消えている");
        assert_eq!(right_of_inner_split, &LayoutNode::Leaf { kind: Pane::Inspector, hidden: false });

        // オラクル: resize した比率も生きている。
        let LayoutNode::Split { ratio, .. } = workspace.root().get(&inspector_stage_split).unwrap() else {
            panic!("split ではない");
        };
        assert_eq!(*ratio, 0.5, "Browser 開閉で resize した比率が失われた");

        // オラクル: Browser 自体も元の位置(最外郭 split の a 側)へ戻っている。
        assert!(!workspace.is_hidden(Pane::Browser));
        assert_eq!(workspace.find_path(Pane::Browser), vec![Branch::A]);
    }

    impl<K: Copy + Eq> WorkspaceLayout<K> {
        fn find_path(&self, kind: K) -> NodePath {
            self.root().find(kind).expect("見つからない")
        }
    }

    #[test]
    fn reset_to_default_undoes_hide_swap_and_resize() {
        let mut workspace = WorkspaceLayout::new(default_tree());
        workspace.hide(Pane::Browser);
        workspace.swap(Pane::Inspector, Pane::Stage);
        workspace.resize(&[Branch::B, Branch::A], 0.9);

        workspace.reset_to_default();

        assert_eq!(workspace, WorkspaceLayout::new(default_tree()));
        assert!(!workspace.is_hidden(Pane::Browser));
    }

    #[test]
    fn swap_is_position_preserving_not_tree_shape_changing() {
        let mut tree = default_tree();
        let path_before_inspector = tree.find(Pane::Inspector).unwrap();
        let path_before_stage = tree.find(Pane::Stage).unwrap();

        assert!(tree.swap(Pane::Inspector, Pane::Stage));

        // 位置(パス)そのものは変わらない——そこに座る kind が交換される。
        assert_eq!(tree.get(&path_before_inspector).unwrap(), &LayoutNode::Leaf { kind: Pane::Stage, hidden: false });
        assert_eq!(tree.get(&path_before_stage).unwrap(), &LayoutNode::Leaf { kind: Pane::Inspector, hidden: false });
    }

    #[test]
    fn swap_with_a_missing_kind_is_a_no_op_and_reports_false() {
        // Inspector を含まない木(Browser | Stage の2枚)。
        let mut tree = LayoutNode::split(Axis::Vertical, 0.3, LayoutNode::leaf(Pane::Browser), LayoutNode::leaf(Pane::Stage));
        let before = tree.clone();

        assert!(!tree.swap(Pane::Inspector, Pane::Stage), "木に無い kind との swap が true を返した");
        assert_eq!(tree, before, "見つからない swap が木を変えてしまった");
    }

    #[test]
    fn resize_on_a_leaf_path_fails_without_mutating() {
        let mut tree = default_tree();
        let browser_path = tree.find(Pane::Browser).unwrap();

        assert!(!tree.resize(&browser_path, 0.5), "葉に resize が通ってしまった");
    }

    #[test]
    fn collapse_hidden_promotes_the_visible_sibling_when_one_side_is_fully_hidden() {
        let mut tree = default_tree();
        assert!(tree.set_hidden(Pane::Browser, true));

        let collapsed = tree.collapse_hidden().expect("全部隠れていないのに None");
        // Browser | content の split が消え、content がそのまま根になる。
        assert_eq!(collapsed.visible_kinds(), vec![Pane::Inspector, Pane::Stage, Pane::Timeline]);
        let LayoutNode::Split { axis, .. } = &collapsed else {
            panic!("content 側が split ではない");
        };
        assert_eq!(*axis, Axis::Horizontal, "content/timeline split の軸が保たれていない");
    }

    #[test]
    fn collapse_hidden_returns_none_when_everything_is_hidden() {
        let mut tree = LayoutNode::split(Axis::Vertical, 0.5, LayoutNode::leaf(Pane::Browser), LayoutNode::leaf(Pane::Stage));
        tree.set_hidden(Pane::Browser, true);
        tree.set_hidden(Pane::Stage, true);

        assert_eq!(tree.collapse_hidden(), None);
    }

    #[test]
    fn round_trips_through_json() {
        let mut book = WorkspaceBook::new(default_tree());
        book.workspace.swap(Pane::Inspector, Pane::Stage);
        book.workspace.hide(Pane::Timeline);
        book.save_as("Effects");
        book.save_as("Minimal");

        let json = serde_json::to_string(&book).expect("serialize");
        let restored: WorkspaceBook<Pane> = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored, book);
        assert_eq!(restored.workspace.is_hidden(Pane::Timeline), true);
        assert_eq!(restored.names().collect::<std::collections::BTreeSet<_>>(), ["Effects", "Minimal"].into_iter().collect());
    }

    #[test]
    fn named_layout_save_switch_delete_round_trip() {
        let mut book = WorkspaceBook::new(default_tree());

        book.workspace.hide(Pane::Browser);
        book.save_as("Minimal");
        book.workspace.reset_to_default();
        assert!(!book.workspace.is_hidden(Pane::Browser));

        assert!(book.switch_to("Minimal"));
        assert!(book.workspace.is_hidden(Pane::Browser), "保存した Minimal が適用されていない");

        assert!(!book.switch_to("Does Not Exist"));
        assert!(book.delete("Minimal"));
        assert!(!book.delete("Minimal"), "2回目の delete は false のはず");
        assert_eq!(book.names().count(), 0);
    }

    #[test]
    fn find_locates_hidden_leaves_too() {
        let mut tree = default_tree();
        tree.set_hidden(Pane::Browser, true);

        assert_eq!(tree.find(Pane::Browser), Some(vec![Branch::A]));
    }
}
