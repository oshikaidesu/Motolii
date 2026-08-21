//! front だけが持つ共有状態の正本(裁定160 切片6、pane split survey
//! `docs/reviews/2026-08-21-pane-split-survey.md` §2.3/§6)。
//!
//! [`Session`] は `timeline::KeySelector` を持ち、`timeline::projection::rows` は
//! `&Session` を要求する — 循環(`Session ⇄ timeline`)を解くため、Session と
//! それが持つ timeline 由来の型([`KeySelector`]/[`KeySelectionOp`])をこの1枚の
//! leaf モジュールへ同居させた。**このファイルは `timeline`(または他の pane
//! モジュール)を import しないこと** — 依存方向は常に
//! `timeline`/`inspector_pane`/... → `state`(この向きだけ)。
//!
//! 純粋な再配置(裁定160 切片6): 型の定義・フィールド・ロジックは無改変、
//! 置き場所だけを `lib.rs`/`timeline/projection.rs` からここへ移した。

use motolii_store::{LayerId, PropertyId};

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
}

impl Default for Session {
    fn default() -> Self {
        Self {
            playhead: 0,
            selection: None,
            selected_layers: Vec::new(),
            selected_keys: Vec::new(),
            key_anchor: None,
        }
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
