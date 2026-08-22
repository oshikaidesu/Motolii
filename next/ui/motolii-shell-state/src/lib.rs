//! owns: shell 横断の共有状態型(Session/KeySelector/KeySelectionOp)。pane crate と assembler の共通の親 — 循環回避のための leaf。
//! front だけが持つ共有状態の正本。裁定160 切片6(pane split survey
//! `docs/reviews/2026-08-21-pane-split-survey.md` §2.3)で `motolii-shell` 内の
//! `state.rs` モジュールへ移設し、切片7(timeline-pane crate 抽出)で
//! この crate へさらに抽出した — `motolii-timeline-pane`(pane crate)は
//! `motolii-shell`(root/assembler crate)へ依存できない(循環になる)ので、
//! `Session`/`KeySelector` を両者の**共通の親**として leaf crate 化する必要が
//! あった(§2.3 の Session ⇄ timeline 循環解消と同じ理由を、pane crate 分割へ
//! もう一段延長した形)。
//!
//! [`Session`] は [`KeySelector`] を持ち、`motolii_timeline_pane::rows`/
//! `property_rows` は `&Session` を要求する — この2つを同じ leaf へ同居させて
//! 依存方向を `motolii-timeline-pane`/`motolii-shell` → `motolii-shell-state`
//! (この向きだけ)に保つ。**このクレートは `motolii-shell`/`motolii-timeline-pane`
//! のどちらも import しないこと**。
//!
//! 純粋な再配置: 型の定義・フィールド・ロジックは無改変、置き場所だけを
//! `motolii-shell/src/state.rs` からここへ移した。
//!
//! [`layout`] モジュール(B26 発注書、2026-08-22)は上記2型とは独立した
//! 追加——ワークスペースレイアウト(pane 分割木)の状態モデル。詳細は
//! 同モジュールの doc 参照。

use std::collections::HashSet;

use motolii_store::{LayerId, PropertyId};

pub mod layout;

/// front だけが持つ状態。**Document の写しは1つも入れないこと**。
#[derive(Debug, Clone)]
pub struct Session {
    /// 再生位置(フレーム番号)。
    pub playhead: i64,
    pub selection: Option<LayerId>,
    /// 複数 layer 選択(普通地図 消化第1波 U1: Select All / Deselect All が
    /// 対象とする集合)。**`selection`(Inspector/Timeline が読む単一 focus)とは
    /// 別の身分** — `Message::Select`/`AddLayer`/クリップボードの貼付/複製は
    /// `select_single`(lib.rs)経由で両方を単一集合へ揃えるが、`timeline_pane`/
    /// `inspector_pane` の行 UI 自体はまだこちらを読まない(multi-select の見た目
    /// 表示は write-set 外、RETURN の finding 参照)。Document には乗らない。
    pub selected_layers: Vec<LayerId>,
    /// Timeline property 行(キー行)の選択(第2波 T3・EXACT TARGET 3)。
    /// **Document には乗らない** — layer 選択と同じ Session の身分。
    pub selected_keys: Vec<KeySelector>,
    /// Shift 範囲選択の基点(直前に単独/Cmd クリックしたキー)。`key_order`
    /// (行順→時刻順)上の範囲は毎回この基点から張り直す(正典 §3・§4 と同じ
    /// 「anchor」文法)。
    pub key_anchor: Option<KeySelector>,
    /// Timeline ツリー行の折り畳み状態(裁定173 H2)。**Document には乗らない**
    /// — 旧世界 `timeline_rows.rs` の「開閉は Undo の対象ではなく Project
    /// session に置く」思想をそのまま踏襲([`TimelineFoldState`] doc 参照)。
    pub timeline_fold: TimelineFoldState,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            playhead: 0,
            selection: None,
            selected_layers: Vec::new(),
            selected_keys: Vec::new(),
            key_anchor: None,
            timeline_fold: TimelineFoldState::default(),
        }
    }
}

/// Timeline ツリー行の折り畳み状態(裁定173 H2、旧世界
/// `crates/motolii-ui/src/timeline_rows.rs::TimelineFoldState` の概念移植)。
/// **Document schema には入れない**(旧世界 module doc の踏襲: 開閉は Undo の
/// 対象ではなく、Timeline の scroll/zoom と同じ棚)。
///
/// **旧世界と方向が逆**: 旧世界は `children_open: HashSet<LayerId>`(開いている
/// ものだけを持つ、既定=全畳み)だった。新世界はツリー行の導入前、全 layer が
/// 常にフラットな行として見えていた — 導入後にその見た目を壊さないことを
/// H-survey 発注書の ORACLE が要求する(「fold 既定=全展開で現行の見た目不変」)。
/// そのため、ここでは**畳んだものだけ**を持つ(`folded: HashSet<LayerId>`) —
/// 何も触っていない `Session::default()` は自動的に「何も畳まれていない」
/// (=全展開)になり、既存の PNG/atlas 出力を無改造で保つ。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TimelineFoldState {
    folded: HashSet<LayerId>,
}

impl TimelineFoldState {
    /// `layer` の子を畳む(矢印を閉じる)。
    pub fn fold(&mut self, layer: LayerId) {
        self.folded.insert(layer);
    }

    /// `layer` の子を開く。
    pub fn unfold(&mut self, layer: LayerId) {
        self.folded.remove(&layer);
    }

    /// 開閉を反転する(rail の fold 三角クリック1回ぶん)。
    pub fn toggle(&mut self, layer: LayerId) {
        if !self.folded.insert(layer) {
            self.folded.remove(&layer);
        }
    }

    /// `layer` の子が畳まれているか。**畳まれていない(既定)なら `false`** —
    /// `children_open` は `!is_folded(layer)` として読む(`RowProjection` 参照)。
    pub fn is_folded(&self, layer: LayerId) -> bool {
        self.folded.contains(&layer)
    }
}

/// Timeline のキー選択の識別子。**Document ではなく [`Session`] が持つ**
/// (EXACT TARGET 3: 選択状態は Session、undo の対象でも Document の写しでもない)。
/// `frame` は同一 property 内で一意(`KeyframeTrack::insert` が同時刻キーを
/// 上書きする — `motolii-eval` 側の保証)なので、この3つ組で1本のキーを指せる。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeySelector {
    pub layer: LayerId,
    pub property: PropertyId,
    pub frame: i64,
}

/// property 行のキー選択操作(正典 §3・§4 と同じ文法: 単独/Cmd トグル/Shift 範囲)。
/// **選択の確定はここでは行わない** — `Session::selected_keys`/`key_anchor` の
/// 読み取りが要るので、唯一の書き口である `Shell::update` 側が確定する
/// (`timeline::key_rows` は「どのキーを・どの操作で」の判定だけを自己完結で持つ)。
#[derive(Debug, Clone, PartialEq)]
pub enum KeySelectionOp {
    /// クリック=単独。
    Single(KeySelector),
    /// Cmd=トグル(足し引き)。
    Toggle(KeySelector),
    /// Shift=`Session::key_anchor` から `key_order` 上の範囲。基点が無ければ
    /// 単独選択へ安全側で倒す(`Shell::apply_key_selection` 参照)。
    Range(KeySelector),
}

#[cfg(test)]
mod fold_tests {
    use super::*;

    /// **オラクル**: 何も触っていない既定状態は「畳まれていない」— H2 発注書
    /// の「fold 既定=全展開で現行の見た目不変」の直接の柵。
    #[test]
    fn default_state_has_nothing_folded() {
        let fold = TimelineFoldState::default();
        assert!(!fold.is_folded(LayerId(1)));
        assert!(!fold.is_folded(LayerId(999)));
    }

    #[test]
    fn fold_and_unfold_round_trip() {
        let mut fold = TimelineFoldState::default();
        let layer = LayerId(7);
        fold.fold(layer);
        assert!(fold.is_folded(layer));
        fold.unfold(layer);
        assert!(!fold.is_folded(layer));
    }

    #[test]
    fn toggle_flips_between_folded_and_unfolded() {
        let mut fold = TimelineFoldState::default();
        let layer = LayerId(3);
        fold.toggle(layer);
        assert!(fold.is_folded(layer), "1回目のtoggleで畳まれていない");
        fold.toggle(layer);
        assert!(!fold.is_folded(layer), "2回目のtoggleで開き直っていない");
    }

    #[test]
    fn folding_one_layer_does_not_affect_another() {
        let mut fold = TimelineFoldState::default();
        fold.fold(LayerId(1));
        assert!(!fold.is_folded(LayerId(2)), "無関係のlayerまで畳まれている");
    }
}
