//! shell の pane_grid 化(実装レーン、2026-08-22 利用者指示「iced にある標準
//! 機能ならリサイズ・ドッキングを早めにつけよう」の第1切片)。`Shell::view()`
//! の静的 `column![]/row![]` を `iced::widget::pane_grid` へ移行するための
//! **状態と純粋な構成ロジック**をここへ集める(`Shell::view` は組み立てだけ)。
//!
//! **Session 水準**(`Document` に乗らない — `Shell` の `checkerboard`/
//! `observation`/`resolution_cap` と同格の「意味を持たない純表示状態」、
//! 裁定157の観測視点と同じ扱い)。再起動間の永続化は本切片の NON-GOAL
//! (RETURN 「分離レーンへの引き渡しメモ」に置き場候補を記す)。
//!
//! fork rev `73e686ee05efd7d1b61cfea2647186b336d9ab9c`(iced 0.15.0-dev、
//! `next/Cargo.toml` 裁定170)のベンダソース(`~/.asdf/.../checkouts/
//! iced-*/73e686e/widget/src/pane_grid/{state,node,configuration}.rs`)を実測して
//! 設計した:
//!
//! - `pane_grid::State<T>::split(axis, pane, state)` は**指定した既存 pane の
//!   矩形だけ**を分割する(`state.rs::split_node`)。**root 全体を新しい兄弟で
//!   包む公開 API は無い**(`split_node(axis, pane: Option<Pane>, ..)` の
//!   `pane: None`(major node)分岐は `pub(crate)`——`move_to_edge`/`drop`経由
//!   でしか使えず、どちらも「既に木にある pane」が前提。木から一度
//!   `close()` した pane は `panes` からも消える(`state.rs::close`)ため、
//!   「閉じた Browser を左の全高カラムとして戻す」操作を直接表す公開 API が
//!   存在しない)。そのため Browser パネルの開閉は「木への挿抜」ではなく、
//!   **`Configuration` から `State` を作り直す**形にした([`build_state`])。
//!   開閉のたびに他 split の ratio を失わないよう、直前の ratio を
//!   [`Ratios`] へ抽出してから作り直す([`extract_ratios`])。
//! - resize(`pane_grid::ResizeEvent`)は `State::resize` へそのまま委譲——
//!   pane_grid 自身の決定論(`Node::resize`、`state.rs`)に乗る。
//!   ドッキング(`DragEvent::Dropped`)は `State::drop` へそのまま委譲——
//!   同上(`examples/pane_grid/src/main.rs` の公式サンプルと同じ扱い:
//!   `Picked`/`Canceled` は state 変更なし)。
//!
//! **既知の限界(NON-GOAL 隣接、分離レーンへの申し送り)**: Browser 開閉で
//! `Configuration` から作り直すため、開閉をまたいだ**ドラッグ並べ替えの
//! 記憶は保持しない**(Inspector/Stage/Timeline の *リサイズ比率* は
//! [`Ratios`] 経由で保持するが、ドラッグで入れ替えた *配置* そのものは
//! 開閉のたびに既定形(Inspector 左・Stage 右・Timeline 下帯)へ戻る)。

use std::collections::BTreeMap;

use iced::widget::pane_grid;

/// 4つのパネル種別。`pane_grid::State<PaneKind>` の `T`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneKind {
    Browser,
    Inspector,
    Stage,
    Timeline,
}

/// 3本の split 比率。Browser 開閉(`Configuration` からの作り直し)をまたいで
/// [`extract_ratios`]/[`build_configuration`] の往復で保持する(モジュール
/// 冒頭 doc の限界参照——保持するのは *比率* だけで、ドラッグによる *配置*
/// ではない)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ratios {
    /// Browser | (Inspector/Stage/Timeline) を分ける split(`Axis::Vertical`)。
    pub browser: f32,
    /// Inspector | Stage を分ける split(`Axis::Vertical`)。
    pub inspector: f32,
    /// (Inspector/Stage) | Timeline を分ける split(`Axis::Horizontal`)。
    pub content_timeline: f32,
}

impl Default for Ratios {
    /// 視覚正本 `next/reference/mocks/browser-library.html` の
    /// `.libraryBrowser { width: min(100%, 700px); }`(700px 幅の縦パネル)を
    /// 踏まえた近似——pane_grid の ratio は実行時 window 幅に対する比率で
    /// あって px 固定ではないため、厳密な 700px 一致はユーザーがドラッグで
    /// 決める(本レーンの目的そのもの)。`inspector`/`content_timeline` は
    /// 旧固定レイアウトの見た目(`dims.inspector_panel_width` = 496px /
    /// `screenshot.rs::CANVAS_WIDTH` = 1600px 系列の比)を初期値として踏襲する。
    fn default() -> Self {
        Self {
            browser: 0.3,
            inspector: 0.31,
            content_timeline: 0.62,
        }
    }
}

/// `open`/`ratios` から `pane_grid::Configuration` を組む純関数(oracle (a)
/// の直接対象)。**初期配置**(発注書 OUTCOME): Browser=左の全高カラム(`open`
/// の間だけ)・Inspector・Stage は現行の並び(Inspector 左・Stage 右)・
/// Timeline=下帯。ASCII 図(`open == true`):
///
/// ```text
/// +----------+---------------------------+
/// |          |  Inspector  |    Stage    |
/// | Browser  +---------------------------+
/// |          |          Timeline         |
/// +----------+---------------------------+
/// ```
pub fn build_configuration(open: bool, ratios: Ratios) -> pane_grid::Configuration<PaneKind> {
    let content = pane_grid::Configuration::Split {
        axis: pane_grid::Axis::Horizontal,
        ratio: ratios.content_timeline,
        a: Box::new(pane_grid::Configuration::Split {
            axis: pane_grid::Axis::Vertical,
            ratio: ratios.inspector,
            a: Box::new(pane_grid::Configuration::Pane(PaneKind::Inspector)),
            b: Box::new(pane_grid::Configuration::Pane(PaneKind::Stage)),
        }),
        b: Box::new(pane_grid::Configuration::Pane(PaneKind::Timeline)),
    };

    if open {
        pane_grid::Configuration::Split {
            axis: pane_grid::Axis::Vertical,
            ratio: ratios.browser,
            a: Box::new(pane_grid::Configuration::Pane(PaneKind::Browser)),
            b: Box::new(content),
        }
    } else {
        content
    }
}

/// [`build_configuration`] から `pane_grid::State` を作る(`State::
/// with_configuration` へそのまま渡すだけの薄い glue)。
pub fn build_state(open: bool, ratios: Ratios) -> pane_grid::State<PaneKind> {
    pane_grid::State::with_configuration(build_configuration(open, ratios))
}

/// `node` の部分木に含まれる `PaneKind` を DFS 順(a→b)で集める。小さい木
/// (最大4葉)なので `Vec` で十分。
fn kinds_in(node: &pane_grid::Node, panes: &BTreeMap<pane_grid::Pane, PaneKind>, out: &mut Vec<PaneKind>) {
    match node {
        pane_grid::Node::Split { a, b, .. } => {
            kinds_in(a, panes, out);
            kinds_in(b, panes, out);
        }
        pane_grid::Node::Pane(pane) => {
            if let Some(kind) = panes.get(pane) {
                out.push(*kind);
            }
        }
    }
}

/// `state` の木を歩いて [`Ratios`] を読み出す(resize/drag 後・Browser 開閉
/// 前に呼ぶ)。**「意味のある3本の split」を両側の `PaneKind` の組で識別**
/// する(split の `id` は開閉のたびに変わりうるため、id 直参照はしない)。
/// 木の形がドラッグで発注書スコープ外の配置へ崩れていて該当 split が
/// 見つからない場合は `previous` の値を保つ(panic しない防御的読み出し)。
pub fn extract_ratios(state: &pane_grid::State<PaneKind>, previous: Ratios) -> Ratios {
    let mut ratios = previous;
    extract_from_node(state.layout(), &state.panes, &mut ratios);
    ratios
}

fn extract_from_node(node: &pane_grid::Node, panes: &BTreeMap<pane_grid::Pane, PaneKind>, ratios: &mut Ratios) {
    if let pane_grid::Node::Split { ratio, a, b, .. } = node {
        let mut a_kinds = Vec::new();
        kinds_in(a, panes, &mut a_kinds);
        let mut b_kinds = Vec::new();
        kinds_in(b, panes, &mut b_kinds);

        match (a_kinds.as_slice(), b_kinds.as_slice()) {
            ([PaneKind::Browser], _) => ratios.browser = *ratio,
            ([PaneKind::Inspector], [PaneKind::Stage]) => ratios.inspector = *ratio,
            (_, [PaneKind::Timeline]) => ratios.content_timeline = *ratio,
            _ => {}
        }

        extract_from_node(a, panes, ratios);
        extract_from_node(b, panes, ratios);
    }
}

/// `Shell` が持つ pane_grid の layout 状態。**Session 水準**(モジュール冒頭
/// doc 参照)。
#[derive(Debug)]
pub struct Layout {
    pub state: pane_grid::State<PaneKind>,
    ratios: Ratios,
    open: bool,
    /// クリックで最後に触れた pane(`PaneGrid::on_click` 配線)。**Q0 対応の
    /// 副産物として生まれた最小の実機能** — fork rev 73e686e の pane_grid
    /// (`widget/src/pane_grid.rs::update`)は `on_resize`/`on_click` の設定に
    /// 関わらず、境界のドラッグ検出のために自分の bounds 内の
    /// `ButtonPressed` を無条件に `shell.capture_event()` する
    /// (`update()` 実測)。`on_click` を配線しないままだと「pane 本体の
    /// どこを押しても capture はされるのに Message が一切出ない」状態になり、
    /// `tests/suite/q0_fence.rs` が横断的に検出する Q0 違反(触れそうで
    /// 触れない)を pane_grid 内の全域で起こす(実測: 155件)。フォーカス
    /// 追跡は「クリックされた pane を覚える」という素朴で正直な意味を持つ
    /// ため、Q0 を満たすための場当たり配線ではなく実際の最小機能として採用した
    /// (将来のショートカット対象 pane 決定・最大化対象決定の土台にもなる —
    /// どちらも本レーンの NON-GOAL だが、この状態自体はそのまま使える)。
    focused: Option<pane_grid::Pane>,
}

impl Layout {
    /// 初期状態(Browser 閉 — `browser_pane::PaneState` の既定と揃える、Q0
    /// 「閉じていれば木に無い」)。
    pub fn new() -> Self {
        Self::with_ratios(false, Ratios::default())
    }

    fn with_ratios(open: bool, ratios: Ratios) -> Self {
        Self { state: build_state(open, ratios), ratios, open, focused: None }
    }

    /// 直近にクリックされた pane(`PaneGrid::on_click` 配線、`focused`
    /// フィールド doc 参照)。
    pub fn focused(&self) -> Option<pane_grid::Pane> {
        self.focused
    }

    /// `Message::PaneClicked` → ここへそのまま記録する。
    pub fn set_focused(&mut self, pane: pane_grid::Pane) {
        self.focused = Some(pane);
    }

    /// 現在の Browser パネル開閉(pane_grid 側の写し)。
    pub fn browser_open(&self) -> bool {
        self.open
    }

    /// 現在の比率(試験・screenshot 器具向けの読み口)。
    pub fn ratios(&self) -> Ratios {
        self.ratios
    }

    /// Browser パネルの開閉トグルを pane_grid 側へ反映する(`Shell::update`
    /// の `Message::Browser` 腕から、`browser_pane::PaneState::is_open()` の
    /// 変化に追随して呼ぶ)。**同じ値なら作り直さない**(no-op — 毎フレーム
    /// 呼んでも他 split の ratio・ドラッグ配置を無駄に潰さない)。
    pub fn set_browser_open(&mut self, open: bool) {
        if open == self.open {
            return;
        }
        self.ratios = extract_ratios(&self.state, self.ratios);
        self.open = open;
        self.state = build_state(open, self.ratios);
        // 作り直すと pane id が振り直されるため、古い id を指したままにしない
        // (存在しない pane を指す状態は防御的に避ける——`focused` フィールド
        // doc の「素朴で正直な意味」を保つ)。
        self.focused = None;
    }

    /// `pane_grid::ResizeEvent` → `State::resize`(pane_grid 自身の決定論)+
    /// [`Ratios`] へ追随記録(open トグルをまたいで保つため)。
    pub fn apply_resize(&mut self, event: pane_grid::ResizeEvent) {
        self.state.resize(event.split, event.ratio);
        self.ratios = extract_ratios(&self.state, self.ratios);
    }

    /// `pane_grid::DragEvent` → `Dropped` だけ `State::drop` へ委譲(iced
    /// 公式 example `examples/pane_grid/src/main.rs` と同じ扱い —
    /// `Picked`/`Canceled` は state 変更が要らない)。
    pub fn apply_drag(&mut self, event: pane_grid::DragEvent) {
        if let pane_grid::DragEvent::Dropped { pane, target } = event {
            self.state.drop(pane, target);
            self.ratios = extract_ratios(&self.state, self.ratios);
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind_of(state: &pane_grid::State<PaneKind>, kind: PaneKind) -> pane_grid::Pane {
        state
            .iter()
            .find(|(_, k)| **k == kind)
            .map(|(pane, _)| *pane)
            .unwrap_or_else(|| panic!("{kind:?} が pane_grid State に無い"))
    }

    // -----------------------------------------------------------------
    // oracle (a): 初期レイアウトの構造(Browser が左・Timeline が下)。
    // -----------------------------------------------------------------

    /// Browser 開: 最外郭は `Axis::Vertical` split で、`a` 側(左)が丸ごと
    /// Browser 1枚だけ——「左の全高カラム」の直接の証拠(Inspector/Stage/
    /// Timeline のどれも `a` 側に混ざらない)。
    #[test]
    fn browser_open_puts_browser_alone_on_the_left_of_the_outermost_split() {
        let state = build_state(true, Ratios::default());
        let pane_grid::Node::Split { axis, a, b, .. } = state.layout() else {
            panic!("最外郭が split ではない");
        };
        assert_eq!(*axis, pane_grid::Axis::Vertical, "Browser|残り の分割は縦線(左右)のはず");

        let mut a_kinds = Vec::new();
        kinds_in(a, &state.panes, &mut a_kinds);
        assert_eq!(a_kinds, vec![PaneKind::Browser], "左側が Browser 単独ではない: {a_kinds:?}");

        let mut b_kinds = Vec::new();
        kinds_in(b, &state.panes, &mut b_kinds);
        assert_eq!(
            b_kinds,
            vec![PaneKind::Inspector, PaneKind::Stage, PaneKind::Timeline],
            "右側の並びが Inspector/Stage/Timeline ではない: {b_kinds:?}"
        );
    }

    /// Timeline は常に一番下(content/timeline split の `b` 側)——Browser の
    /// 開閉に関わらず不変。
    #[test]
    fn timeline_is_always_the_bottom_band_regardless_of_browser() {
        for open in [true, false] {
            let state = build_state(open, Ratios::default());

            fn find_content_timeline_split(node: &pane_grid::Node) -> Option<(&pane_grid::Node, &pane_grid::Node)> {
                match node {
                    pane_grid::Node::Split { axis, a, b, .. } if *axis == pane_grid::Axis::Horizontal => {
                        Some((a, b))
                    }
                    pane_grid::Node::Split { a, b, .. } => {
                        find_content_timeline_split(a).or_else(|| find_content_timeline_split(b))
                    }
                    pane_grid::Node::Pane(_) => None,
                }
            }

            let (_, b) = find_content_timeline_split(state.layout())
                .unwrap_or_else(|| panic!("content/timeline split(Horizontal)が無い(open={open})"));
            let mut b_kinds = Vec::new();
            kinds_in(b, &state.panes, &mut b_kinds);
            assert_eq!(b_kinds, vec![PaneKind::Timeline], "下帯が Timeline 単独ではない(open={open}): {b_kinds:?}");
        }
    }

    /// Inspector が左・Stage が右(「Inspector・Stage は現行の並び」)。
    #[test]
    fn inspector_is_left_of_stage() {
        for open in [true, false] {
            let state = build_state(open, Ratios::default());

            fn find_inspector_stage_split(node: &pane_grid::Node) -> Option<(&pane_grid::Node, &pane_grid::Node)> {
                match node {
                    pane_grid::Node::Split { a, b, .. } => {
                        if matches!(**a, pane_grid::Node::Pane(_)) && matches!(**b, pane_grid::Node::Pane(_)) {
                            Some((a, b))
                        } else {
                            find_inspector_stage_split(a).or_else(|| find_inspector_stage_split(b))
                        }
                    }
                    pane_grid::Node::Pane(_) => None,
                }
            }

            let (a, b) = find_inspector_stage_split(state.layout())
                .unwrap_or_else(|| panic!("Inspector|Stage の葉split が無い(open={open})"));
            let mut a_kinds = Vec::new();
            kinds_in(a, &state.panes, &mut a_kinds);
            let mut b_kinds = Vec::new();
            kinds_in(b, &state.panes, &mut b_kinds);
            assert_eq!(a_kinds, vec![PaneKind::Inspector], "左が Inspector ではない(open={open})");
            assert_eq!(b_kinds, vec![PaneKind::Stage], "右が Stage ではない(open={open})");
        }
    }

    /// Q0「閉じていれば木に無い」: Browser 閉の間は `state.panes` に Browser
    /// の値が一切現れない(表示分岐ではなく構造として無い)。
    #[test]
    fn browser_closed_means_absent_from_the_tree_not_just_hidden() {
        let state = build_state(false, Ratios::default());
        assert!(
            state.iter().all(|(_, kind)| *kind != PaneKind::Browser),
            "Browser 閉のはずが state.panes に Browser が残っている"
        );
        assert_eq!(state.len(), 3, "Browser 閉は Inspector/Stage/Timeline の3枚のはず");
    }

    /// Browser 開は4枚ちょうど。
    #[test]
    fn browser_open_has_exactly_four_panes() {
        let state = build_state(true, Ratios::default());
        assert_eq!(state.len(), 4);
    }

    // -----------------------------------------------------------------
    // oracle (b): resize/drag メッセージ → State 遷移の決定論。
    // -----------------------------------------------------------------

    #[test]
    fn resize_event_updates_exactly_the_targeted_split_ratio() {
        let mut layout = Layout::new();
        let split = *layout.state.layout().splits().next().expect("split が無い");

        layout.apply_resize(pane_grid::ResizeEvent { split, ratio: 0.2 });

        let pane_grid::Node::Split { ratio, .. } = layout.state.layout() else {
            panic!("root が split ではない");
        };
        assert_eq!(*ratio, 0.2, "resize が反映されていない");
    }

    /// resize は「その split だけ」を動かす——他 split の ratio は不変
    /// (pane_grid 自身の決定論、`Node::resize` の再帰は一致 id だけ書き換える)。
    #[test]
    fn resize_is_deterministic_and_isolated_to_the_targeted_split() {
        let mut layout = Layout::new();
        let before = layout.ratios();
        let inspector_stage_split = {
            let pane_grid::Node::Split { id, a, .. } = layout.state.layout() else {
                panic!("root が split ではない");
            };
            let pane_grid::Node::Split { id: inner_id, .. } = a.as_ref() else {
                panic!("a 側が split ではない");
            };
            let _ = id;
            *inner_id
        };

        layout.apply_resize(pane_grid::ResizeEvent { split: inspector_stage_split, ratio: 0.5 });

        let after = layout.ratios();
        assert_eq!(after.inspector, 0.5, "対象 split の ratio が反映されていない");
        assert_eq!(
            after.content_timeline, before.content_timeline,
            "無関係な split の ratio が動いてしまった"
        );
    }

    /// Browser 開閉をまたいでも、resize で変えた Inspector|Stage の比率は
    /// 保持される(モジュール冒頭 doc の「比率は保持・配置は保持しない」の
    /// うち「比率は保持」の直接証拠)。
    #[test]
    fn browser_toggle_preserves_the_resized_inspector_ratio() {
        let mut layout = Layout::new();
        let inspector_stage_split = {
            let pane_grid::Node::Split { a, .. } = layout.state.layout() else {
                panic!("root が split ではない");
            };
            let pane_grid::Node::Split { id, .. } = a.as_ref() else {
                panic!("a 側が split ではない");
            };
            *id
        };
        layout.apply_resize(pane_grid::ResizeEvent { split: inspector_stage_split, ratio: 0.45 });

        layout.set_browser_open(true);
        layout.set_browser_open(false);

        assert_eq!(layout.ratios().inspector, 0.45, "Browser 開閉で resize した比率が失われた");
    }

    /// ドラッグで Inspector と Stage を入れ替える(`DragEvent::Dropped` +
    /// `Target::Pane(_, Region::Center)` は `State::swap` 相当、`state.rs::
    /// drop`/`split_with` 実測)。**`State::swap` は `panes`(id→`PaneKind`)
    /// ではなく `layout`(id→幾何位置の木)を書き換える**(`state.rs::swap`
    /// 実測 — 引っかかった箇所: 最初 `state.get(pane_id)` で判定する試験を
    /// 書いたが、id→種別の対応そのものは不変で red のまま落ちた。正しい
    /// 判定は「木の *左* 位置に今どの id がいて、その id の種別は何か」)。
    /// pane id は不変、**木のどちらの位置にいるか**だけが入れ替わる —
    /// レンダリング(`PaneGrid::layout`)はこの木を辿って各 id の Content を
    /// 置くので、見た目としては Inspector/Stage の位置が正しく入れ替わる。
    #[test]
    fn drag_dropped_on_a_pane_swaps_their_screen_positions_deterministically() {
        let mut layout = Layout::new();
        let inspector = kind_of(&layout.state, PaneKind::Inspector);
        let stage = kind_of(&layout.state, PaneKind::Stage);

        layout.apply_drag(pane_grid::DragEvent::Dropped {
            pane: inspector,
            target: pane_grid::Target::Pane(stage, pane_grid::Region::Center),
        });

        // pane id→種別の対応は不変(swap は id を配り直すだけで panes 自体は
        // 触らない)。
        assert_eq!(layout.state.get(inspector), Some(&PaneKind::Inspector));
        assert_eq!(layout.state.get(stage), Some(&PaneKind::Stage));

        // 幾何位置(木の左右)は入れ替わっている——左の葉に今いる id は元
        // Stage だった id、種別で見ると Stage が左へ来た。
        fn leaf_positions(node: &pane_grid::Node) -> Option<(pane_grid::Pane, pane_grid::Pane)> {
            match node {
                pane_grid::Node::Split { a, b, .. } => match (a.as_ref(), b.as_ref()) {
                    (pane_grid::Node::Pane(left), pane_grid::Node::Pane(right)) => {
                        Some((*left, *right))
                    }
                    _ => leaf_positions(a).or_else(|| leaf_positions(b)),
                },
                pane_grid::Node::Pane(_) => None,
            }
        }
        let (left_id, right_id) =
            leaf_positions(layout.state.layout()).expect("Inspector|Stage の葉split が無い");
        assert_eq!(
            layout.state.get(left_id),
            Some(&PaneKind::Stage),
            "ドラッグ後、左位置には Stage が来ているはず"
        );
        assert_eq!(
            layout.state.get(right_id),
            Some(&PaneKind::Inspector),
            "ドラッグ後、右位置には Inspector が来ているはず"
        );
    }

    /// `Picked`/`Canceled` は state を一切変えない(iced 公式 example と同じ
    /// 扱いの確認——このレーンが独自解釈で余計な状態変更をしていないことの
    /// 直接の証拠)。
    #[test]
    fn drag_picked_and_canceled_do_not_mutate_state() {
        let mut layout = Layout::new();
        let inspector = kind_of(&layout.state, PaneKind::Inspector);
        let before = layout.ratios();

        layout.apply_drag(pane_grid::DragEvent::Picked { pane: inspector });
        layout.apply_drag(pane_grid::DragEvent::Canceled { pane: inspector });

        assert_eq!(layout.state.get(inspector), Some(&PaneKind::Inspector));
        assert_eq!(layout.ratios(), before);
    }

    #[test]
    fn set_focused_records_the_clicked_pane() {
        let mut layout = Layout::new();
        let stage = kind_of(&layout.state, PaneKind::Stage);
        assert_eq!(layout.focused(), None);

        layout.set_focused(stage);

        assert_eq!(layout.focused(), Some(stage));
    }

    /// Browser 開閉の作り直しをまたぐと、古い pane id は意味を失うので
    /// `focused` はクリアする(`set_browser_open` doc 参照)。
    #[test]
    fn browser_toggle_clears_a_stale_focused_pane() {
        let mut layout = Layout::new();
        let stage = kind_of(&layout.state, PaneKind::Stage);
        layout.set_focused(stage);

        layout.set_browser_open(true);

        assert_eq!(layout.focused(), None, "作り直し後も古い id を指したままだった");
    }

    #[test]
    fn set_browser_open_is_a_no_op_when_the_value_does_not_change() {
        let mut layout = Layout::new();
        let inspector_stage_split = {
            let pane_grid::Node::Split { a, .. } = layout.state.layout() else {
                panic!("root が split ではない");
            };
            let pane_grid::Node::Split { id, .. } = a.as_ref() else {
                panic!("a 側が split ではない");
            };
            *id
        };
        layout.apply_resize(pane_grid::ResizeEvent { split: inspector_stage_split, ratio: 0.4 });

        layout.set_browser_open(false); // 既に閉じている——no-op のはず

        assert_eq!(layout.ratios().inspector, 0.4, "no-op のはずの set_browser_open が状態を潰した");
    }
}
