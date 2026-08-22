//! Timeline pane の書き込みロジック(裁定160 切片7)。旧
//! `motolii-shell/src/lib.rs` の private メソッド(pane split survey
//! `docs/reviews/2026-08-21-pane-split-survey.md` §1.2 Timeline 小計、
//! move/trim・キー選択・キー時刻ドラッグ/リタイム・NudgeKeyframe・
//! commit_key_frames、計584行)をここへ移設した。**ロジックは無改変** —
//! `&mut self.doc`/`&mut self.session`/`self.keyboard_modifiers` だった
//! 暗黙アクセスを、明示引数(`&mut Document`/`&mut Session`/
//! `iced::keyboard::Modifiers`)へ書き換えただけ(pane crate 化に伴う機械的な
//! シグネチャ変更、survey §2.1)。
//!
//! **`toggle_layer_hidden`/`toggle_layer_solo`/`toggle_layer_lock` は
//! ここへ移していない**(survey の 584行見積りには含まれるが、判断でここに
//! 残した) — `toggle_layer_hidden` は Inspector の `Message::InspectorToggleHidden`
//! (`toggle_inspector_hidden` 経由)とも共有される Shell 側の汎用ヘルパーで、
//! Timeline 専用ではない。Pane crate 化の目的(pane 同士が依存しない)を
//! 壊さず M/S/L 3つを割るより、3つとも Shell(assembler)側に残す方が単純
//! (RETURN の state 扱いの finding で詳述)。
//!
//! ## pane-local `Message`(survey §3.1「pane 分割を成立させるために構造上必須」)
//!
//! `input.rs`・`key_rows.rs` は widget コールバックの中で直接 `Message::Xxx`
//! を組み立てる(root crate の `Message` は pane crate から参照できない —
//! 参照すると `motolii-timeline-pane → motolii-shell → motolii-timeline-pane`
//! の循環になる)。ここで定義する `Message` がその発行先 — `motolii-shell` 側は
//! `Message::Timeline(motolii_timeline_pane::Message)` で1回だけ畳む
//! (`Shell::update` 参照)。
//!
//! **例外(survey §3.2 exception 1)**: `Select`/`ScrubTo` は本来 core 腕だが
//! `input.rs` の内部(レーンバー行クリック・ルーラー/空白部クリック)からも
//! 直接発行される。pane crate から root の `Message` を参照できないので、
//! ここに同名の腕を複製した。`ToggleMute`/`ToggleSolo`/`ToggleLock` も同じ
//! 理由で複製している(上の doc 節の `toggle_layer_hidden` 共有の判断とセット)。
//! **`Shell::update` はこの5腕を[`PaneState::update`]へ渡す前に先取りする**
//! (`select_single`/`playhead`/`toggle_layer_*` へ直接委譲 — 既存の挙動その
//! まま)。[`PaneState::update`] にこの5腕が来ることは実運用では無いが、
//! 網羅性のために受理はする(no-op)。
#[derive(Debug, Clone)]
pub enum Message {
    /// 例外: 本来は core 腕(`input.rs` のレーンバー行クリックが直接発行)。
    Select(LayerId),
    /// 例外: 本来は core 腕(`input.rs` のルーラー/空白部クリックが直接発行)。
    ScrubTo(i64),
    /// 例外: `toggle_layer_hidden` が Inspector と共有のため Shell に残る。
    ToggleMute(LayerId),
    ToggleSolo(LayerId),
    ToggleLock(LayerId),

    // ---- Timeline クリップの move/trim(第2波T2、正典 §2) ----
    BarGrabbed {
        layer: LayerId,
        part: BarPart,
        at_frame: i64,
    },
    DragMoved {
        at_frame: i64,
        px_per_frame: f32,
    },
    DragReleased,
    DragCancelled,

    // ---- Timeline property 行(キー行) ----
    KeySelect(KeySelectionOp),
    DeleteSelectedKeys,

    // ---- キーの補間・選択動詞(第3切片 B15) ----
    /// 選択キーの補間切替(map 495/512/513/514・正典 §3「イージング」+ §7-1
    /// 「全 param へ開放で確定」)。`Interp` は store の実在型(`motolii_eval` の
    /// re-export) — Hold/Linear/Bezier をこの1腕で運ぶ。Easy Ease 系プリセット
    /// (map 485〜490)は keymap/メニュー層がこの腕へ [`EASY_EASE`] 等の定数を
    /// 渡すだけ(専用の腕を増やさない — 意味は「補間を設定する」1つ)。
    /// 空選択・ロック層は理由つき拒否(M13)。**1操作 = 1 undo**。
    SetKeyInterp(Interp),
    /// 全キー・選択解除(map 484 Deselect all keyframes)。Session だけを触る
    /// (`ToggleFold` と同じ「shell 先取りなしで完結する」腕)。
    DeselectAllKeys,
    /// property の全キー選択(map 509・正典 §8.1 SelectAllKeysOfProperty
    /// 「property 名クリック」)。rail の property 行ラベル(`rail::property_row`)
    /// がこの腕を発行する。
    SelectAllKeysOfProperty { layer: LayerId, property: PropertyId },
    /// 表示中の全キー・全 property を選択(map 510)。「表示中」= 今の
    /// `projection::property_rows`(選択 layer のキー持ち property 行)に
    /// 見えているキー全部 — 見えているとおりに採れる(正典 §4 Cmd+A と同じ思想)。
    SelectAllVisibleKeys,

    // ---- Stage 重なり(第3切片 — map B44 184/292/293・正典 §8.1
    //      ReorderLayerUp/Down(+ToEnd))----
    /// 選択 layer(複数は block)の Stage 合成順(`meta.order`)を動かす。
    /// **Timeline 行の縦位置は動かない**([`crate::stacking`] モジュール doc —
    /// 行位置は LayerId 昇順で order の投影ではない)。`Intent::SetOrder` を
    /// 1回の `apply_all` で束ねる(**1操作 = 1 undo**)。空選択・ロック層は
    /// 理由つき拒否(M13)。
    RestackLayer(StackDirection),

    // ---- レイヤー名の inline rename(第3切片 — map B02 785・正典 §6
    //      「リネーム」)----
    /// rename 開始(正典 §6: Enter(単一選択時)/メニュー — 入口は shell の
    /// keymap/メニュー層がこの腕を発行する。拘束4によりダブルクリックは
    /// 使わない)。現在名を下書きへ写し、rail の名前 text を `text_input` に
    /// 差し替える([`PaneState::rename_draft`] → `TimelinePane::with_rename`)。
    /// ロック層は理由つき拒否。
    RenameBegin(LayerId),
    /// rename 下書きの毎打鍵(rail の `text_input.on_input`)。Document 不接触。
    RenameEdited(String),
    /// rename 確定(`text_input.on_submit` = Enter)。空名は拒否して**編集継続**
    /// (入力を失わない)・同名は no-op(正典 §6)。実書き込みは
    /// `Intent::SetAttrs`(`LayerAttrsPatch.name` — Inspector の改名
    /// `commit_inspector_name` と同じ書き口)1回。
    RenameCommit,
    /// rename 取消(Esc は shell の `EscapePressed` が
    /// [`PaneState::cancel_rename`] を直接呼ぶ — 裁定151「キャンセルの一般化」。
    /// この腕は将来の UI 内取消ボタン等のための Message 経路)。
    RenameCancel,

    // ---- Timeline キーの時刻編集(第2波T4) ----
    KeyGrabbed {
        key: KeySelector,
        at_frame: i64,
        retime: bool,
    },
    KeyDragMoved {
        at_frame: i64,
        px_per_frame: f32,
    },
    KeyDragReleased,
    KeyDragCancelled,
    NudgeKeyframe(i64),

    // ---- Timeline transport 帯(map 1041-1045 採用済・1138 Timecode) ----
    /// transport の Play‖Pause(map 1041)。**Select/ScrubTo と同じ「例外」の
    /// 形** — 意味の実装は shell 側の既存 `Message::TogglePlayback`
    /// (`Shell::toggle_playback`、拘束5「再生と掴みは相互排他」の判断ごと)に
    /// 既にあり、pane はボタンの顔だけを持つ。shell は
    /// `Message::Timeline(TogglePlayback)` を5例外と同様に先取りして既存腕へ
    /// 写す(実結線は supervisor 統合時)。[`PaneState::update`] では no-op。
    TogglePlayback,
    /// transport の 1コマ戻/進(map 1042/1043)。shell の既存
    /// `Message::StepPlayhead(i64)`(`nav::step_playhead` 経由)へ素直に写る —
    /// 符号と歩幅は呼び出し側が決める既存の役割分担どおり(ボタンは ±1 固定)。
    StepPlayhead(i64),
    /// transport の先頭へ(map 1045)。shell の既存 `Message::JumpPlayheadToStart`。
    JumpPlayheadToStart,
    /// transport の末尾へ(map 1044)。shell の既存 `Message::JumpPlayheadToEnd`
    /// (`nav::comp_end_frame`)。
    JumpPlayheadToEnd,

    // ---- Timeline ツリー行(裁定173 H2) ----
    /// rail の fold 三角(開閉ボタン)クリック。**Shell の5例外に含まれない**
    /// ので、`Message::Timeline(other)` の受け皿がそのまま
    /// [`PaneState::update`] へ渡す(shell/src の改修は不要 — mod doc の
    /// 「5腕だけ先取り」節参照)。`layer` が Document に既に存在しない場合も
    /// `TimelineFoldState::toggle` は黙って無視できる(fold 状態は LayerId の
    /// 存在に依存しない Session 側の集合)。
    ToggleFold(LayerId),

    // ---- 作業範囲/ループ帯(B21+B18 第1切片、正典 §5「ループ帯」) ----
    /// ループ on/off(map 1082/1083 Loop/Unloop・transport 帯のループボタン・
    /// 既定割当 L は keymap 層)。**帯は消えない**(正典 §5 — 引き直さず戻せる)。
    /// 帯が無い時は理由つき拒否(M13: 無反応ゼロ)。`ToggleFold` と同じ
    /// 「shell 先取りなしで [`PaneState::update`] が完結する」腕。
    ToggleLoop,
    /// ループ帯(ルーラ最上段)を押した瞬間(正典 §5: 空白=新規・端=リサイズ・
    /// 中=平行移動)。どこを押したかの判定は押した瞬間の座標で済ませてある
    /// (正典 §1、`input.rs` が `classify_loop_band` を1回だけ呼ぶ)。
    LoopBandGrabbed { part: LoopBandPart, at_frame: i64 },
    /// ループ帯ドラッグ中のポインタ移動。スナップは持たない(正典 §2 の
    /// スナップ対象は clip/キーのドラッグ側 — 帯自身は対象であって主体ではない)。
    LoopDragMoved { at_frame: i64 },
    LoopDragReleased,
    LoopDragCancelled,
    /// Mark In / Set Work Area In(map 725・296): In 点を playhead へ。
    SetWorkAreaIn,
    /// Mark Out / Set Work Area Out(map 726・297): Out 点を playhead へ。
    SetWorkAreaOut,
    /// Clear In(map 719): In 点だけ解除(先頭へ開く)。帯は残る。
    ClearWorkAreaIn,
    /// Clear Out(map 721): Out 点だけ解除(終端へ開く)。帯は残る。
    ClearWorkAreaOut,
    /// Clear In and Out(map 720): 作業範囲そのものを消す。
    ClearWorkArea,
    /// Mark Clip / Mark Selection(map 724/727): 選択 layer(複数選択は
    /// その合併区間)の clip 範囲を作業範囲にする。選択が無ければ理由つき拒否。
    SetWorkAreaToSelection,

    // ---- JKL シャトル(B21、map 1097/1098・1100/1101・1125-1127・1135/1136) ----
    /// transport 4腕(`TogglePlayback` 等)と同じ**shell 先取りの例外** —
    /// 実時間再生の clock は shell(A2)が持つので、pane は意味
    /// ([`crate::shuttle::ShuttleState::apply`] の状態機械)だけを所有し、
    /// この腕は運搬役。[`PaneState::update`] では no-op。
    Shuttle(ShuttleCommand),
}

use std::collections::{BTreeMap, HashSet};

use motolii_store::{
    Composition, Document, Intent, Interp, KeyframeTrack, LayerAttrsPatch, LayerId, LayerTiming,
    PropertyId, RationalTime,
};

use crate::hit::BarPart;
use crate::shuttle::ShuttleCommand;
use crate::stacking::{self, StackDirection};
use crate::state::Session;
use crate::work_area::{self, LoopBandPart, WorkArea};
use crate::{clip_gesture, key_gesture, key_order, property_rows, rows, KeySelectionOp, KeySelector};

/// Easy Ease(map 485/488): AE の既定 influence 33% を cubic-bezier へ写した
/// プリセット。**区間モデルの注記**(拘束7(a)の構造差 — 逸脱理由): store の
/// [`Interp`] は「このキーから次のキーまで」の**出射区間**に付く(AE の
/// キー単位の入射/出射タンジェント対とは構造が違う)ので、Easy Ease 系は
/// 「選択キーの出射区間の形」として適用される。In/Out の名前は区間の
/// 入り口側/出口側どちらを緩めるかを指す。
pub const EASY_EASE: Interp = Interp::Bezier { x1: 0.333, y1: 0.0, x2: 0.667, y2: 1.0 };
/// Easy Ease In(map 486/489): 区間の入り口(加速側)だけ緩める。
pub const EASY_EASE_IN: Interp = Interp::Bezier { x1: 0.333, y1: 0.0, x2: 1.0, y2: 1.0 };
/// Easy Ease Out(map 487/490): 区間の出口(減速側)だけ緩める。
pub const EASY_EASE_OUT: Interp = Interp::Bezier { x1: 0.0, y1: 0.0, x2: 0.667, y2: 1.0 };

/// Timeline クリップの move/trim、進行中の一時状態(第2波T2)。**Document では
/// ない** — 押し口の transient(`Shell::update` の唯一の書き口の外で確定前の
/// 値を持ち回す形)。**Document は release まで一切触らない**
/// (`finish_drag` が1回だけ `Intent::SetTiming` を出す) — Esc/右クリックでの
/// 復元(`cancel_drag`)は履歴に触れていないぶん、単にこの構造体を捨てるだけで
/// 完全に無傷になる。
#[derive(Clone, Copy)]
struct TimelineDragState {
    layer: LayerId,
    part: BarPart,
    /// 掴んだ瞬間に Document から読んだそのままの値。**move/trim の計算は毎回
    /// これを基準に絶対値で出し直す**(delta 蓄積禁止、正典 §2)。
    origin: LayerTiming,
    /// 掴んだ瞬間のポインタ位置(comp frame、スナップ前)。
    grab_at_frame: i64,
    /// 直近の move/trim 計算結果。release がこれを(`origin` と違えば)1回
    /// `apply` する。
    preview: LayerTiming,
}

/// Timeline キーの時刻ドラッグ/リタイム、進行中の一時状態(第2波T4、正典
/// §3・裁定146)。**`TimelineDragState` と同じ「pane 側の transient」の形**。
#[derive(Clone)]
struct TimelineKeyDragState {
    kind: TimelineKeyDragKind,
    /// 実際に掴んだキー(`origins`/`preview` のどの添字かは毎回引き直す)。
    grabbed: KeySelector,
    /// 掴んだ瞬間のポインタ位置(comp frame、スナップ前)。
    grab_at_frame: i64,
    /// 掴んだキーが属する layer の clip 範囲(`[clip_start, clip_end]`)。
    /// EXACT TARGET 1「0秒〜clip 範囲 clamp」の出典。
    clip_start: i64,
    clip_end: i64,
    /// 掴んだ瞬間の選択キー全員(`Session::selected_keys` のクローン)。
    /// **move/retime の計算は毎回これを基準に絶対値で出し直す**(delta 蓄積
    /// 禁止、正典 §2 と同じ思想をキーへ延長)。
    origins: Vec<KeySelector>,
    /// 直近の計算結果。release がこれを(`origins` と違えば)1回書き戻す。
    preview: Vec<KeySelector>,
}

#[derive(Clone, Copy)]
enum TimelineKeyDragKind {
    /// 通常の時刻ドラッグ(正典 §3・§8.1 の複数選択の一括移動)。
    Move,
    /// RetimeSelection(裁定146)。`anchor_frame` は固定端、`edge_origin_frame`
    /// は掴んだ端の掴んだ瞬間の frame(スケール1.0の基準)。
    Retime { anchor_frame: i64, edge_origin_frame: i64 },
}

/// ループ帯ドラッグ、進行中の一時状態(B21+B18 第1切片、正典 §5)。
/// `TimelineDragState` と同じ transient の形だが、書き戻し先が Document では
/// なく [`PaneState::work_area`](同じ struct 内)なので、**live に書き換えて
/// よい**(undo の対象ではない — 正典 §5.5 の scroll/zoom/fold と同格)。
/// キャンセル復元のために掴んだ瞬間の値(`origin_*`)だけ控える。
#[derive(Clone, Copy)]
struct LoopDragState {
    kind: LoopDragKind,
    /// 掴んだ瞬間の作業範囲(キャンセル復元用)。
    origin_area: Option<WorkArea>,
    /// 掴んだ瞬間のループ on/off(新規ドラッグは即 on にするので、これも戻す)。
    origin_enabled: bool,
}

#[derive(Clone, Copy)]
enum LoopDragKind {
    /// 新規(anchor = 押した瞬間の frame)とリサイズ(anchor = 固定する反対端)
    /// の共通形 — `work_area::dragged_area` 1本で済む(正典 §5「左右どちらから
    /// 引いても同じ」「反対端は掴んだ瞬間の値で固定」)。
    Span { anchor: i64 },
    /// 平行移動(正典 §5「中=平行移動」)。
    Move { origin: WorkArea, grab_at_frame: i64 },
}

/// Shell が持つ、Timeline pane 専用の状態(旧 `Shell::timeline_drag`/
/// `timeline_key_drag` の2フィールドをまとめた形)。**Document ではない**
/// (`TimelineDragState`/`TimelineKeyDragState` の doc comment 参照)。
///
/// `work_area`/`loop_enabled` だけは transient ではなくフレームを跨いで生きる
/// (正典 §5.5 の scroll/zoom/fold と同じ Session 級の身分。本籍は `Session` が
/// 自然だが、このレーンの write-set は pane crate のみ — `work_area.rs` の
/// モジュール doc「型の置き場」参照。Session への昇格は supervisor 判断)。
#[derive(Default)]
pub struct PaneState {
    drag: Option<TimelineDragState>,
    key_drag: Option<TimelineKeyDragState>,
    /// 作業範囲(In-Out)。`None` = 帯を一度も引いていない/Clear In and Out 済み。
    work_area: Option<WorkArea>,
    /// ループ on/off(map 1082/1083)。**帯が消えても値は残る**が、帯なしでは
    /// 折り返しに効かない(`work_area::advanced_playhead` が両方を要求する)。
    loop_enabled: bool,
    loop_drag: Option<LoopDragState>,
    /// inline rename の進行中下書き(第3切片、正典 §6)。**Document ではない**
    /// — `TimelineDragState` と同じ transient の形(確定まで Document 不接触・
    /// 取消は捨てるだけで履歴無傷)。
    rename: Option<RenameDraft>,
}

/// inline rename の一時状態(正典 §6「リネーム」)。掴んだ layer と毎打鍵の
/// 下書きだけ — 確定([`Message::RenameCommit`])が `Intent::SetAttrs` を
/// 1回出すまで Document を触らない。
struct RenameDraft {
    layer: LayerId,
    draft: String,
}

impl PaneState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Esc / 右クリック = 進行中ジェスチャの破棄(正典 §2・裁定151「キャンセルの
    /// 一般化」)。**Document は最初から触っていない**ので、復元は state を
    /// 捨てるだけで成立する。`Shell::update` の `Message::EscapePressed` が
    /// 直接呼ぶ(Message 経由ではない — clip/key の順序に意味がある排他処理、
    /// 元の `Shell::update` の doc comment どおり)。戻り値は「何か捨てたか」。
    pub fn cancel_drag(&mut self) -> bool {
        self.drag.take().is_some()
    }

    /// 同上、キー drag/リタイム版。
    pub fn cancel_key_drag(&mut self) -> bool {
        self.key_drag.take().is_some()
    }

    /// 同上、ループ帯ドラッグ版(B21+B18 第1切片)。掴んだ瞬間の作業範囲と
    /// ループ on/off へ**復元する**(clip/key と違い live に書いているので、
    /// 捨てるだけでは戻らない — origin を書き戻す)。`Shell::update` の
    /// `Message::EscapePressed` はこれも呼ぶこと(supervisor 結線 — 裁定151
    /// 「キャンセルの一般化」の柵)。
    pub fn cancel_loop_drag(&mut self) -> bool {
        let Some(drag) = self.loop_drag.take() else {
            return false;
        };
        self.work_area = drag.origin_area;
        self.loop_enabled = drag.origin_enabled;
        true
    }

    /// 作業範囲の現在値(`TimelinePane::with_work_area` へそのまま渡す読み取り
    /// 専用)。ドラッグ中は live 更新済みの値がそのまま見える(正典 §5.5
    /// 「プレビューは毎フレーム」— こちらは Document でないので preview と
    /// 確定の区別自体が無い)。
    pub fn work_area(&self) -> Option<WorkArea> {
        self.work_area
    }

    /// ループ on/off の現在値(同上 + shell の PlaybackClock が
    /// `work_area::advanced_playhead` へ渡す)。
    pub fn loop_enabled(&self) -> bool {
        self.loop_enabled
    }

    /// `TimelinePane::with_key_drag_active` へそのまま渡す読み取り専用フラグ。
    pub fn key_drag_active(&self) -> bool {
        self.key_drag.is_some()
    }

    /// inline rename の進行中下書き(`TimelinePane::with_rename` へそのまま
    /// 渡す読み取り専用)。`None` = rename 中ではない。
    pub fn rename_draft(&self) -> Option<(LayerId, &str)> {
        self.rename.as_ref().map(|r| (r.layer, r.draft.as_str()))
    }

    /// rename の取消(Esc — `cancel_drag` と同じく shell の
    /// `Message::EscapePressed` が直接呼ぶ、裁定151「キャンセルの一般化」)。
    /// Document は最初から触っていないので、state を捨てるだけで無傷。
    /// 戻り値は「何か捨てたか」。
    pub fn cancel_rename(&mut self) -> bool {
        self.rename.take().is_some()
    }

    /// clip drag/keyドラッグ/ループ帯ドラッグのどれかが進行中か。実時間再生
    /// (A2、正典 §2 拘束5「再生と掴みは相互排他: ドラッグ中に Space は
    /// 効かない」)が `Shell::toggle_playback` から読む — ループ帯も「掴み」
    /// なので同じ排他に入る。
    pub fn is_dragging(&self) -> bool {
        self.drag.is_some() || self.key_drag.is_some() || self.loop_drag.is_some()
    }

    /// `TimelinePane::with_clip_preview` へそのまま渡す。`TimelineDragState` は
    /// `Copy` なので `&self` のまま値で読める。
    pub fn clip_preview(&self) -> Option<(LayerId, LayerTiming)> {
        self.drag.map(|drag| (drag.layer, drag.preview))
    }

    /// `TimelinePane::with_key_preview` へそのまま渡す。`origins`(掴んだ瞬間の
    /// selector・旧 frame)と `preview`(同じ並びで frame だけ更新済み)を
    /// index でゆわえ、(selector, 新frame) のペア列にする(EXACT TARGET 4)。
    pub fn key_preview(&self) -> Option<Vec<(KeySelector, i64)>> {
        self.key_drag.as_ref().map(|drag| {
            drag.origins
                .iter()
                .cloned()
                .zip(drag.preview.iter().map(|key| key.frame))
                .collect()
        })
    }

    /// **pane 側の唯一の書き口**。`Message::Select`/`ScrubTo`/`ToggleMute`/
    /// `ToggleSolo`/`ToggleLock` は `Shell::update` が先取りするので実運用では
    /// ここに来ない(来ても no-op、`Message` の doc 参照)。戻り値は拒否理由
    /// (`Shell::status` へそのまま渡す文字列) — `None` は「拒否なし」。
    pub fn update(
        &mut self,
        message: Message,
        doc: &mut Document,
        session: &mut Session,
        modifiers: iced::keyboard::Modifiers,
    ) -> Option<String> {
        match message {
            Message::BarGrabbed { layer, part, at_frame } => self.start_drag(doc, session, layer, part, at_frame),
            Message::DragMoved { at_frame, px_per_frame } => {
                self.continue_drag(doc, session, at_frame, px_per_frame, modifiers);
                None
            }
            Message::DragReleased => self.finish_drag(doc),
            Message::DragCancelled => {
                self.cancel_drag();
                None
            }
            Message::KeySelect(op) => {
                apply_key_selection(session, doc, op);
                None
            }
            Message::DeleteSelectedKeys => delete_selected_keys(doc, session),
            Message::SetKeyInterp(interp) => set_key_interp(doc, session, interp),
            Message::DeselectAllKeys => {
                session.selected_keys.clear();
                session.key_anchor = None;
                None
            }
            Message::SelectAllKeysOfProperty { layer, property } => {
                select_all_keys_of_property(doc, session, layer, property)
            }
            Message::SelectAllVisibleKeys => select_all_visible_keys(doc, session),
            Message::RestackLayer(direction) => restack_layers(doc, session, direction),
            Message::RenameBegin(layer) => self.begin_rename(doc, layer),
            Message::RenameEdited(text) => {
                if let Some(rename) = self.rename.as_mut() {
                    rename.draft = text;
                }
                None
            }
            Message::RenameCommit => self.commit_rename(doc),
            Message::RenameCancel => {
                self.cancel_rename();
                None
            }
            Message::KeyGrabbed { key, at_frame, retime } => {
                self.start_key_drag(doc, session, key, at_frame, retime)
            }
            Message::KeyDragMoved { at_frame, px_per_frame } => {
                self.continue_key_drag(doc, session, at_frame, px_per_frame, modifiers);
                None
            }
            Message::KeyDragReleased => self.finish_key_drag(doc, session),
            Message::KeyDragCancelled => {
                self.cancel_key_drag();
                None
            }
            Message::NudgeKeyframe(delta) => nudge_keyframe(doc, session, delta),
            Message::ToggleFold(layer) => {
                session.timeline_fold.toggle(layer);
                None
            }

            // ---- 作業範囲/ループ帯(B21+B18 第1切片) ----
            Message::ToggleLoop => self.toggle_loop(),
            Message::LoopBandGrabbed { part, at_frame } => {
                self.start_loop_drag(doc, part, at_frame);
                None
            }
            Message::LoopDragMoved { at_frame } => {
                self.continue_loop_drag(doc, at_frame);
                None
            }
            Message::LoopDragReleased => {
                // 確定 = drag state を捨てるだけ(`work_area` は live 更新済み。
                // 未移動 release でも最短1フレームの帯が残る — 正典 §5 の
                // 「最短1フレーム保証」どおり)。
                self.loop_drag = None;
                None
            }
            Message::LoopDragCancelled => {
                self.cancel_loop_drag();
                None
            }
            Message::SetWorkAreaIn => {
                self.work_area =
                    Some(work_area::with_in(self.work_area, session.playhead, comp_duration(doc)));
                None
            }
            Message::SetWorkAreaOut => {
                self.work_area =
                    Some(work_area::with_out(self.work_area, session.playhead, comp_duration(doc)));
                None
            }
            Message::ClearWorkAreaIn => {
                self.work_area = self.work_area.map(work_area::cleared_in);
                None
            }
            Message::ClearWorkAreaOut => {
                let duration = comp_duration(doc);
                self.work_area = self.work_area.map(|area| work_area::cleared_out(area, duration));
                None
            }
            Message::ClearWorkArea => {
                self.work_area = None;
                None
            }
            Message::SetWorkAreaToSelection => self.set_work_area_to_selection(doc, session),

            // transport 4腕+Shuttle も Select/ScrubTo と同じ「shell が先取り
            // する例外」— 実運用ではここに来ない(来ても no-op、`Message` の
            // doc 参照)。
            Message::Select(_)
            | Message::ScrubTo(_)
            | Message::ToggleMute(_)
            | Message::ToggleSolo(_)
            | Message::ToggleLock(_)
            | Message::TogglePlayback
            | Message::StepPlayhead(_)
            | Message::JumpPlayheadToStart
            | Message::JumpPlayheadToEnd
            | Message::Shuttle(_) => None,
        }
    }

    // ---- 作業範囲/ループ帯(B21+B18 第1切片、正典 §5) ----

    /// ループ on/off(map 1082/1083)。帯が無い時は理由つき拒否(M13: 無反応
    /// ゼロ — 何も起きないトグルを黙って飲み込まない)。
    fn toggle_loop(&mut self) -> Option<String> {
        if self.work_area.is_none() {
            return Some(
                "作業範囲が無いのでループできない — ルーラー上端の帯をドラッグして範囲を引く".into(),
            );
        }
        self.loop_enabled = !self.loop_enabled;
        None
    }

    /// ループ帯を掴んだ瞬間(正典 §5: 空白=新規・端=リサイズ・中=平行移動)。
    /// **新規は引いたら即 on**(正典 §5 — 別キーで有効化させない)。リサイズ/
    /// 移動は on/off を変えない。
    fn start_loop_drag(&mut self, doc: &Document, part: LoopBandPart, at_frame: i64) {
        if self.loop_drag.is_some() {
            return; // 既に別のドラッグが進行中 — 多重起動しない(clip と同型)。
        }
        let duration = comp_duration(doc);
        let origin_area = self.work_area;
        let origin_enabled = self.loop_enabled;
        let kind = match (part, self.work_area) {
            (LoopBandPart::EdgeIn, Some(area)) => LoopDragKind::Span { anchor: area.end },
            (LoopBandPart::EdgeOut, Some(area)) => LoopDragKind::Span { anchor: area.start },
            (LoopBandPart::Body, Some(area)) => {
                LoopDragKind::Move { origin: area, grab_at_frame: at_frame }
            }
            // 空白=新規(帯が無い時の Edge*/Body は classify が返さないが、
            // 万一来ても新規へ倒す — 安全側)。
            _ => {
                self.loop_enabled = true; // 引いたら即 on(正典 §5)。
                self.work_area = Some(work_area::dragged_area(at_frame, at_frame, duration));
                LoopDragKind::Span { anchor: at_frame }
            }
        };
        self.loop_drag = Some(LoopDragState { kind, origin_area, origin_enabled });
    }

    /// ループ帯ドラッグ中のポインタ移動。掴んだ瞬間の anchor/origin を基準に
    /// **絶対値で出し直す**(delta 蓄積禁止 — 正典 §2 の思想)。
    fn continue_loop_drag(&mut self, doc: &Document, at_frame: i64) {
        let Some(drag) = self.loop_drag else {
            return;
        };
        let duration = comp_duration(doc);
        self.work_area = Some(match drag.kind {
            LoopDragKind::Span { anchor } => work_area::dragged_area(anchor, at_frame, duration),
            LoopDragKind::Move { origin, grab_at_frame } => {
                work_area::moved_area(origin, grab_at_frame, at_frame, duration)
            }
        });
    }

    /// Mark Clip / Mark Selection(map 724/727): 選択 layer の clip 範囲
    /// (複数選択は合併区間)を作業範囲へ。選択が無ければ理由つき拒否(M13)。
    fn set_work_area_to_selection(&mut self, doc: &Document, session: &Session) -> Option<String> {
        let targets: Vec<LayerId> = if session.selected_layers.is_empty() {
            session.selection.into_iter().collect()
        } else {
            session.selected_layers.clone()
        };
        if targets.is_empty() {
            return Some("選択が無いので作業範囲にできない — layer を選んでから".into());
        }
        let rows = rows(&doc.view(), session);
        let spans: Vec<(i64, i64)> = rows
            .iter()
            .filter(|row| targets.contains(&row.id))
            .map(|row| (row.start, row.start + row.duration))
            .collect();
        let (Some(start), Some(end)) = (
            spans.iter().map(|(s, _)| *s).min(),
            spans.iter().map(|(_, e)| *e).max(),
        ) else {
            return Some("選択 layer が見当たらないので作業範囲にできない".into());
        };
        // clip 範囲を dragged_area(clamp・最短1フレーム)へ通して不変量を守る。
        self.work_area = Some(work_area::dragged_area(start, end, comp_duration(doc)));
        None
    }

    // ---- inline rename(第3切片、正典 §6「リネーム」) ----

    /// rename 開始。現在名を下書きへ写す(名前が空の layer は空文字から
    /// 編集を始める — rail 側の placeholder `layer {id}` が顔を担う)。
    /// 消えた layer は黙って無視(stale な入口)。ロック層は理由つき拒否(M13、
    /// `start_drag` と同じ柵)。
    fn begin_rename(&mut self, doc: &Document, layer: LayerId) -> Option<String> {
        let Ok(Some(_meta)) = doc.view().meta(layer) else {
            return None; // 素材が無い layer は改名対象にしない(安全側)。
        };
        let attrs = doc.view().attrs(layer).ok().flatten().unwrap_or_default();
        if attrs.locked {
            return Some(format!("layer {} はロックされているので改名できない", layer.0));
        }
        self.rename = Some(RenameDraft { layer, draft: attrs.name });
        None
    }

    /// rename 確定(正典 §6: **空名拒否・同名 no-op**)。空名の拒否は下書きを
    /// **捨てない**(編集継続 — 打った文字を拒否で失わない)。同名は Intent を
    /// 出さない(空 undo を作らない)。実書き込みは `Intent::SetAttrs`
    /// (`LayerAttrsPatch.name`)1回 = 1 undo。
    fn commit_rename(&mut self, doc: &mut Document) -> Option<String> {
        let Some(rename) = self.rename.take() else {
            return None;
        };
        if rename.draft.trim().is_empty() {
            let reason = Some("空の名前にはできない — 文字を入れるか Esc で取消".to_owned());
            self.rename = Some(rename); // 編集継続(入力を失わない)。
            return reason;
        }
        let current = doc.view().attrs(rename.layer).ok().flatten().unwrap_or_default().name;
        if current == rename.draft {
            return None; // 同名 no-op(正典 §6)。
        }
        let patch = LayerAttrsPatch { name: Some(rename.draft), ..Default::default() };
        if let Err(error) = doc.apply(Intent::SetAttrs { layer: rename.layer, patch }) {
            return Some(format!("名前を書けない: {error}"));
        }
        None
    }

    // ---- Timeline クリップの move/trim(第2波T2、正典 §2) ----

    /// bar を掴んだ瞬間。**ロック中は掴む前に断る**(正典 §2 拘束6・M13: 無反応
    /// ゼロ)。move(`Body`)は**未選択なら掴んだ瞬間に単独選択へ差し替える**
    /// (正典 §2) — このソフトの選択は単一(`Session::selection`)なので、
    /// 「差し替え」は常に「選ぶ」と同義。trim(`Edge*`)は選択を変えない。
    ///
    /// `session.selection`/`selected_layers` を直接更新する(旧
    /// `Shell::select_single` のロジックをここへ複製 — pane crate は Shell の
    /// private メソッドを呼べないため。RETURN の finding 参照)。
    fn start_drag(
        &mut self,
        doc: &mut Document,
        session: &mut Session,
        layer: LayerId,
        part: BarPart,
        at_frame: i64,
    ) -> Option<String> {
        if self.drag.is_some() {
            return None; // 既に別のドラッグが進行中 — 多重起動しない
        }
        let Ok(Some(meta)) = doc.view().meta(layer) else {
            return None; // 素材が無い layer は掴めない(起こらないはずだが安全側)
        };
        let locked = doc.view().attrs(layer).ok().flatten().unwrap_or_default().locked;
        if locked {
            return Some(format!("layer {} はロックされているので動かせない", layer.0));
        }
        if matches!(part, BarPart::Body) {
            session.selection = Some(layer);
            session.selected_layers = vec![layer];
        }
        self.drag = Some(TimelineDragState {
            layer,
            part,
            origin: meta.timing,
            grab_at_frame: at_frame,
            preview: meta.timing,
        });
        None
    }

    /// ドラッグ中のポインタ移動。**掴んだ瞬間の値(`origin`)を基準に絶対値で
    /// 出し直す**(delta 蓄積禁止、正典 §2)。**Document はまだ一切触らない**
    /// (`preview` は transient な一時値、release まで `apply` しない)。
    fn continue_drag(
        &mut self,
        doc: &mut Document,
        session: &Session,
        at_frame: i64,
        px_per_frame: f32,
        modifiers: iced::keyboard::Modifiers,
    ) {
        let Some(drag) = self.drag else {
            return;
        };
        let comp_duration = comp_duration(doc);
        let snap_enabled = !modifiers.command();
        let timeline_rows = rows(&doc.view(), session);
        let candidates =
            clip_gesture::snap_candidates(&timeline_rows, drag.layer, session.playhead, comp_duration);

        let mut timing = drag.origin;
        match drag.part {
            BarPart::Body => {
                timing.start = clip_gesture::moved_start(
                    drag.origin.start,
                    drag.origin.duration,
                    drag.grab_at_frame,
                    at_frame,
                    comp_duration,
                    &candidates,
                    px_per_frame,
                    snap_enabled,
                );
            }
            BarPart::EdgeIn => {
                let end = drag.origin.start + drag.origin.duration;
                let new_start =
                    clip_gesture::trimmed_in_start(end, at_frame, &candidates, px_per_frame, snap_enabled);
                let delta = new_start - drag.origin.start;
                timing.start = new_start;
                timing.duration = drag.origin.duration - delta;
                timing.source_in = drag.origin.source_in + delta;
            }
            BarPart::EdgeOut => {
                let new_end = clip_gesture::trimmed_out_end(
                    drag.origin.start,
                    at_frame,
                    comp_duration,
                    &candidates,
                    px_per_frame,
                    snap_enabled,
                );
                timing.duration = new_end - drag.origin.start;
            }
        }

        if let Some(drag) = self.drag.as_mut() {
            drag.preview = timing;
        }
    }

    /// release = 確定。**掴んだだけで未移動なら no-op**。動いていれば
    /// `Intent::SetTiming` を1回だけ出す(1 gesture = 1 undo)。
    fn finish_drag(&mut self, doc: &mut Document) -> Option<String> {
        let drag = self.drag.take()?;
        if drag.preview == drag.origin {
            return None;
        }
        if let Err(error) = doc.apply(Intent::SetTiming { layer: drag.layer, timing: drag.preview }) {
            return Some(format!("timing を書けない: {error}"));
        }
        None
    }

    // ---- Timeline キーの時刻編集(第2波T4、正典 §3・§8.1・裁定146) ----

    /// キー菱形を掴んだ瞬間。**ロック中は掴む前に断る**。move は**未選択なら
    /// 掴んだ瞬間に単独選択へ差し替え**、既に選択済みのキーを掴んだ場合は
    /// 選択(複数)を保つ(一括ドラッグを壊さない)。retime は選択2本以上・
    /// 掴んだキーがその端であることを呼び出し元が既に確認済み — ここでは
    /// 安全側にもう一度検分するだけ。
    fn start_key_drag(
        &mut self,
        doc: &mut Document,
        session: &mut Session,
        key: KeySelector,
        at_frame: i64,
        retime: bool,
    ) -> Option<String> {
        if self.key_drag.is_some() {
            return None; // 既に別のキー drag が進行中 — 多重起動しない。
        }
        let Some(row) = rows(&doc.view(), session).into_iter().find(|row| row.id == key.layer) else {
            return None; // 素材が無い layer は掴めない(起こらないはずだが安全側)。
        };
        if row.locked {
            return Some(format!("layer {} はロックされているのでキーを動かせない", key.layer.0));
        }
        let clip_start = row.start;
        // EXACT TARGET 1「0秒〜clip 範囲 clamp」: 上限は `start + duration`。
        let clip_end = (row.start + row.duration).max(row.start);

        if retime {
            let selected = session.selected_keys.clone();
            if selected.len() < 2 || !selected.contains(&key) {
                return None; // key_rows 側の判定とズレていた — 安全側で不成立にする。
            }
            let min_frame = selected.iter().map(|k| k.frame).min().unwrap_or(key.frame);
            let max_frame = selected.iter().map(|k| k.frame).max().unwrap_or(key.frame);
            if key.frame != min_frame && key.frame != max_frame {
                return None; // 端ではないキーを掴んだ — retime は不成立。
            }
            let anchor_frame = if key.frame == min_frame { max_frame } else { min_frame };
            self.key_drag = Some(TimelineKeyDragState {
                kind: TimelineKeyDragKind::Retime { anchor_frame, edge_origin_frame: key.frame },
                grabbed: key,
                grab_at_frame: at_frame,
                clip_start,
                clip_end,
                origins: selected.clone(),
                preview: selected,
            });
            return None;
        }

        if !session.selected_keys.contains(&key) {
            session.selected_keys = vec![key.clone()];
            session.key_anchor = Some(key.clone());
        }
        let origins = session.selected_keys.clone();
        self.key_drag = Some(TimelineKeyDragState {
            kind: TimelineKeyDragKind::Move,
            grabbed: key,
            grab_at_frame: at_frame,
            clip_start,
            clip_end,
            origins: origins.clone(),
            preview: origins,
        });
        None
    }

    /// ドラッグ中のポインタ移動。**掴んだ瞬間の値(`origins`)を基準に絶対値で
    /// 出し直す**(delta 蓄積禁止、正典 §2・§3)。**Document はまだ一切触らない**。
    fn continue_key_drag(
        &mut self,
        doc: &mut Document,
        session: &Session,
        at_frame: i64,
        px_per_frame: f32,
        modifiers: iced::keyboard::Modifiers,
    ) {
        let Some(drag) = self.key_drag.clone() else {
            return;
        };
        let candidates =
            key_gesture::key_snap_candidates(&rows(&doc.view(), session), session.playhead, comp_duration(doc));
        let origin_frames: Vec<i64> = drag.origins.iter().map(|k| k.frame).collect();

        let new_frames = match drag.kind {
            TimelineKeyDragKind::Move => {
                let grabbed_origin_frame = drag
                    .origins
                    .iter()
                    .find(|k| **k == drag.grabbed)
                    .map(|k| k.frame)
                    .unwrap_or(drag.grabbed.frame);
                let snap_enabled = !modifiers.command();
                key_gesture::moved_key_group(
                    &origin_frames,
                    grabbed_origin_frame,
                    drag.grab_at_frame,
                    at_frame,
                    drag.clip_start,
                    drag.clip_end,
                    &candidates,
                    px_per_frame,
                    snap_enabled,
                )
            }
            TimelineKeyDragKind::Retime { anchor_frame, edge_origin_frame } => {
                let raw_edge = edge_origin_frame + (at_frame - drag.grab_at_frame);
                let snapped_edge = clip_gesture::snap_frame(raw_edge, &candidates, px_per_frame);
                let clamped_edge = snapped_edge.clamp(drag.clip_start, drag.clip_end);
                key_gesture::retimed_key_group(&origin_frames, anchor_frame, edge_origin_frame, clamped_edge)
            }
        };

        if let Some(drag) = self.key_drag.as_mut() {
            for (selector, frame) in drag.preview.iter_mut().zip(new_frames) {
                selector.frame = frame;
            }
        }
    }

    /// release = 確定。**掴んだだけで未移動なら no-op** — ただし retime が
    /// 未移動のまま release された場合は「Cmd クリックで動かさなかった」と
    /// 同義なので、`KeySelectionOp::Toggle` へ安全側で倒す。動いていれば
    /// property ごとに `Intent::SetTrack` をまとめ、1回の `apply_all` で
    /// 確定する(**1操作 = 1 undo**)。
    fn finish_key_drag(&mut self, doc: &mut Document, session: &mut Session) -> Option<String> {
        let drag = self.key_drag.take()?;
        if drag.preview == drag.origins {
            if matches!(drag.kind, TimelineKeyDragKind::Retime { .. }) {
                apply_key_selection(session, doc, KeySelectionOp::Toggle(drag.grabbed));
            }
            return None;
        }
        let origin_frames: Vec<i64> = drag.origins.iter().map(|k| k.frame).collect();
        let new_frames: Vec<i64> = drag.preview.iter().map(|k| k.frame).collect();
        let grabbed_index = drag.origins.iter().position(|k| *k == drag.grabbed).unwrap_or(0);
        let representative_delta = new_frames.get(grabbed_index).copied().unwrap_or(0)
            - origin_frames.get(grabbed_index).copied().unwrap_or(0);
        commit_key_frames(doc, session, &drag.origins, &new_frames, representative_delta)
    }
}

// ---------------------------------------------------------------------------
// self を持たない書き口(drag state を触らない = PaneState のフィールドが要らない)。
// ---------------------------------------------------------------------------

fn composition(doc: &Document) -> Option<Composition> {
    doc.view().composition().ok().flatten()
}

fn comp_duration(doc: &Document) -> i64 {
    composition(doc).map(|c| c.duration_frames).unwrap_or(0)
}

/// キー選択の確定(第2波 T3・裁定148/151)。`key_rows::update` は「どのキーを・
/// どの操作で」までしか判定しない(canvas 側は Document/Session を直接書けない)
/// ので、`Session::selected_keys`/`key_anchor` の実際の読み書きはここで行う。
fn apply_key_selection(session: &mut Session, doc: &mut Document, op: KeySelectionOp) {
    match op {
        KeySelectionOp::Single(key) => {
            session.selected_keys = vec![key.clone()];
            session.key_anchor = Some(key);
        }
        KeySelectionOp::Toggle(key) => {
            if let Some(pos) = session.selected_keys.iter().position(|k| *k == key) {
                session.selected_keys.remove(pos);
            } else {
                session.selected_keys.push(key.clone());
            }
            session.key_anchor = Some(key);
        }
        KeySelectionOp::Range(key) => {
            let Some(anchor) = session.key_anchor.clone() else {
                // 基点が無ければ単独選択と同じ扱いへ安全側で倒す。
                session.selected_keys = vec![key.clone()];
                session.key_anchor = Some(key);
                return;
            };
            let fps = composition(doc).map(|c| c.fps);
            let property_rows = property_rows(&doc.view(), session, fps);
            let order = key_order(&property_rows);
            let anchor_pos = order.iter().position(|k| *k == anchor);
            let clicked_pos = order.iter().position(|k| *k == key);
            match (anchor_pos, clicked_pos) {
                (Some(a), Some(c)) => {
                    let (lo, hi) = if a <= c { (a, c) } else { (c, a) };
                    session.selected_keys = order[lo..=hi].to_vec();
                }
                _ => {
                    // anchor/clicked のどちらかが今の property_rows に無い —
                    // 単独選択へ安全側で倒す(M16)。
                    session.selected_keys = vec![key];
                }
            }
            // anchor は不変 — 同じ基点から Shift 連打で範囲を伸縮できる。
        }
    }
}

/// 選択中のキーを消す(正典 §3「Delete はキー選択が層選択より優先」)。
/// property ごとにまとめて読み直し、選択されたフレームだけを落とした
/// `KeyframeTrack` を1回の `apply_all` で書き戻す(**1操作 = 1 undo**)。
/// 選択が空なら no-op。
fn delete_selected_keys(doc: &mut Document, session: &mut Session) -> Option<String> {
    if session.selected_keys.is_empty() {
        return None;
    }
    let keys = std::mem::take(&mut session.selected_keys);
    session.key_anchor = None;
    let Some(composition) = composition(doc) else {
        return None;
    };
    let fps = composition.fps;

    let mut groups: BTreeMap<(LayerId, PropertyId), Vec<i64>> = BTreeMap::new();
    for key in keys {
        groups.entry((key.layer, key.property)).or_default().push(key.frame);
    }

    let store = doc.view();
    let mut intents = Vec::new();
    for ((layer, property), frames) in groups {
        let Ok(Some(track)) = store.track(layer, &property) else {
            continue;
        };
        let mut new_track = KeyframeTrack::new();
        for existing in track.keys() {
            let Ok(frame) = existing.t.try_to_frame_round(fps) else {
                continue;
            };
            if frames.contains(&frame) {
                continue; // 選択されたキーは書き戻さない = 削除。
            }
            new_track.insert(existing.clone());
        }
        intents.push(Intent::SetTrack { layer, property, track: new_track });
    }
    drop(store);

    if !intents.is_empty() {
        if let Err(error) = doc.apply_all(intents) {
            return Some(format!("キーを消せない: {error}"));
        }
    }
    None
}

/// 選択キーの補間切替(第3切片 B15、map 495/512/513/514 + プリセット 485〜490)。
/// property ごとに track を読み直し、**選択されたキーの `interp` だけ**を
/// 差し替えた `KeyframeTrack` を1回の `apply_all` で書き戻す(**1操作 = 1 undo**、
/// `delete_selected_keys` と同じ形)。時刻・値・spatial は不変 — 選択も動かない
/// (frame が変わらないので selector はそのまま生きている)。
///
/// 空選択は理由つき拒否(M13: メニュー/キー入口の動詞が黙って何もしないのは
/// 無反応系違反 — Delete キーの空 no-op とは入口の性格が違う)。ロック層も
/// 理由つき拒否(`start_key_drag` と同じ柵)。
fn set_key_interp(doc: &mut Document, session: &Session, interp: Interp) -> Option<String> {
    if session.selected_keys.is_empty() {
        return Some("キーが選ばれていないので補間を変えられない — 菱形を選んでから".into());
    }
    let Some(composition) = composition(doc) else {
        return None;
    };
    let fps = composition.fps;

    let mut groups: BTreeMap<(LayerId, PropertyId), Vec<i64>> = BTreeMap::new();
    for key in &session.selected_keys {
        groups
            .entry((key.layer, key.property.clone()))
            .or_default()
            .push(key.frame);
    }

    let store = doc.view();
    for &(layer, _) in groups.keys() {
        let locked = store.attrs(layer).ok().flatten().unwrap_or_default().locked;
        if locked {
            drop(store);
            return Some(format!("layer {} はロックされているので補間を変えられない", layer.0));
        }
    }
    let mut intents = Vec::new();
    for ((layer, property), frames) in groups {
        let Ok(Some(track)) = store.track(layer, &property) else {
            continue;
        };
        let mut new_track = KeyframeTrack::new();
        for existing in track.keys() {
            let mut key = existing.clone();
            if let Ok(frame) = existing.t.try_to_frame_round(fps) {
                if frames.contains(&frame) {
                    key.interp = interp;
                }
            }
            new_track.insert(key);
        }
        intents.push(Intent::SetTrack { layer, property, track: new_track });
    }
    drop(store);

    if intents.is_empty() {
        return None; // 選択が stale(track が消えている等)— 黙って無視。
    }
    if let Err(error) = doc.apply_all(intents) {
        return Some(format!("補間を書けない: {error}"));
    }
    None
}

/// property の全キー選択(map 509、正典 §8.1 SelectAllKeysOfProperty)。
/// track を読み、全キーを選択集合へ差し替える(足し引きではない — AE の
/// 「property 名クリック」と同じ全置換)。anchor は先頭キー(以後の Shift 範囲の
/// 基点)。track が無い/キーが無い(stale なクリック)は黙って無視。
fn select_all_keys_of_property(
    doc: &mut Document,
    session: &mut Session,
    layer: LayerId,
    property: PropertyId,
) -> Option<String> {
    let Some(composition) = composition(doc) else {
        return None;
    };
    let fps = composition.fps;
    let store = doc.view();
    let Ok(Some(track)) = store.track(layer, &property) else {
        return None;
    };
    let keys: Vec<KeySelector> = track
        .keys()
        .iter()
        .filter_map(|key| key.t.try_to_frame_round(fps).ok())
        .map(|frame| KeySelector { layer, property: property.clone(), frame })
        .collect();
    drop(store);
    if keys.is_empty() {
        return None;
    }
    session.key_anchor = Some(keys[0].clone());
    session.selected_keys = keys;
    None
}

/// 表示中の全キー選択(map 510)。「表示中」= `projection::property_rows` が
/// 今出している行(選択 layer のキー持ち property)の全キー — 見えている
/// とおりに採れる(正典 §4 Cmd+A の思想)。並びは `key_order`(行順→時刻順)
/// そのもの。何も見えていなければ黙って無視(Cmd+A の空 no-op と同格)。
fn select_all_visible_keys(doc: &mut Document, session: &mut Session) -> Option<String> {
    let fps = composition(doc).map(|c| c.fps);
    let rows = property_rows(&doc.view(), session, fps);
    let order = key_order(&rows);
    if order.is_empty() {
        return None;
    }
    session.key_anchor = Some(order[0].clone());
    session.selected_keys = order;
    None
}

/// Stage 重なりの並べ替え(第3切片、map B44 184/292/293 + 正典 §8.1
/// ReorderLayerUp/Down(+ToEnd))。意味計算は [`stacking::restacked`](純関数)、
/// ここは選択の解決・ロック検分・`Intent::SetOrder` の束ね(1回の `apply_all`
/// = **1操作 = 1 undo**)だけ。端で動けない no-op は Intent を出さない
/// (`restacked` が空を返す — 空 undo を作らない)。
fn restack_layers(doc: &mut Document, session: &Session, direction: StackDirection) -> Option<String> {
    let targets: Vec<LayerId> = if session.selected_layers.is_empty() {
        session.selection.into_iter().collect()
    } else {
        session.selected_layers.clone()
    };
    if targets.is_empty() {
        return Some("選択が無いので重なりを動かせない — layer を選んでから".into());
    }
    let store = doc.view();
    for &layer in &targets {
        let locked = store.attrs(layer).ok().flatten().unwrap_or_default().locked;
        if locked {
            drop(store);
            return Some(format!("layer {} はロックされているので重なりを動かせない", layer.0));
        }
    }
    let stack: Vec<(LayerId, i16)> = store
        .layers()
        .into_iter()
        .filter_map(|id| store.meta(id).ok().flatten().map(|meta| (id, meta.order)))
        .collect();
    drop(store);

    let changes = stacking::restacked(&stack, &targets, direction);
    if changes.is_empty() {
        return None; // 端で既に止まっている等 — 黙ってスキップ(失敗ではない、§2 split と同格)。
    }
    let intents: Vec<Intent> = changes
        .into_iter()
        .map(|(layer, order)| Intent::SetOrder { layer, order })
        .collect();
    if let Err(error) = doc.apply_all(intents) {
        return Some(format!("重なりを書けない: {error}"));
    }
    None
}

/// NudgeKeyframe(正典 §8.1): 選択キーを固定 frame 数だけ前後へ。選択が
/// 空、または全キーが同一 layer である保証が崩れていれば何もしない。
fn nudge_keyframe(doc: &mut Document, session: &mut Session, delta: i64) -> Option<String> {
    if session.selected_keys.is_empty() {
        return None;
    }
    let layer = session.selected_keys[0].layer;
    let Some(row) = rows(&doc.view(), session).into_iter().find(|row| row.id == layer) else {
        return None;
    };
    if row.locked {
        return Some(format!("layer {} はロックされているのでキーを動かせない", layer.0));
    }
    let clip_start = row.start;
    let clip_end = (row.start + row.duration).max(row.start);

    let origins = session.selected_keys.clone();
    let origin_frames: Vec<i64> = origins.iter().map(|k| k.frame).collect();
    let new_frames = key_gesture::nudge_key_group(&origin_frames, delta, clip_start, clip_end);
    commit_key_frames(doc, session, &origins, &new_frames, delta)
}

/// 選択キー群の移動結果を Document へ確定する(Move/retime/Nudge 共通の
/// 書き口)。`origins`/`news` は同じ添字で対応する(選択は単一 layer 限定)。
/// property ごとにまとめて `KeyframeTrack` を再構築し、1回の `apply_all` で
/// 書き戻す(**1操作 = 1 undo**)。**同時刻衝突は移動方向でソートしてから書く**
/// (`representative_delta` の符号、正典 §3 — `key_gesture::key_write_order`)。
fn commit_key_frames(
    doc: &mut Document,
    session: &mut Session,
    origins: &[KeySelector],
    news: &[i64],
    representative_delta: i64,
) -> Option<String> {
    if origins.is_empty() || origins.len() != news.len() {
        return None;
    }
    let Some(composition) = composition(doc) else {
        return None;
    };
    let fps = composition.fps;

    let mut groups: BTreeMap<(LayerId, PropertyId), Vec<(i64, i64)>> = BTreeMap::new();
    for (origin, &new_frame) in origins.iter().zip(news) {
        groups.entry((origin.layer, origin.property.clone())).or_default().push((origin.frame, new_frame));
    }

    let store = doc.view();
    let mut intents = Vec::new();
    for ((layer, property), moves) in &groups {
        let Ok(Some(track)) = store.track(*layer, property) else {
            continue;
        };
        let old_frames: HashSet<i64> = moves.iter().map(|(old, _)| *old).collect();
        let mut new_track = KeyframeTrack::new();
        for existing in track.keys() {
            let Ok(frame) = existing.t.try_to_frame_round(fps) else {
                continue;
            };
            if old_frames.contains(&frame) {
                continue; // 動いたキーは後段でまとめて書き直す。
            }
            new_track.insert(existing.clone());
        }
        let origin_frames: Vec<i64> = moves.iter().map(|(old, _)| *old).collect();
        let order = key_gesture::key_write_order(&origin_frames, representative_delta);
        for idx in order {
            let (old_frame, new_frame) = moves[idx];
            let Some(original_key) =
                track.keys().iter().find(|k| k.t.try_to_frame_round(fps) == Ok(old_frame))
            else {
                continue;
            };
            let mut moved_key = original_key.clone();
            let Ok(t) = RationalTime::try_from_frame(new_frame, fps) else {
                continue;
            };
            moved_key.t = t;
            new_track.insert(moved_key);
        }
        intents.push(Intent::SetTrack { layer: *layer, property: property.clone(), track: new_track });
    }
    drop(store);

    if intents.is_empty() {
        return None;
    }
    if let Err(error) = doc.apply_all(intents) {
        return Some(format!("キー時刻を書けない: {error}"));
    }

    // 選択も新しい frame へ追従させる(§5.5「選択は生きたまま」)。
    for (origin, &new_frame) in origins.iter().zip(news) {
        if let Some(selected) = session.selected_keys.iter_mut().find(|k| *k == origin) {
            selected.frame = new_frame;
        }
        if let Some(anchor) = session.key_anchor.as_mut() {
            if anchor == origin {
                anchor.frame = new_frame;
            }
        }
    }
    None
}

/// 第3切片(B15+B52)のテスト共通 fixture。**テストは書くが実行しない**
/// (裁定189 追いつきターン — supervisor が波末一括で回す)。
#[cfg(test)]
mod third_slice_fixtures {
    use super::*;
    use motolii_store::{Fps, Keyframe, LayerMeta, LayerSource, Value};

    pub(super) fn doc_with_comp() -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).expect("30/1 は正の既約 fps"),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .expect("comp 設定");
        doc
    }

    pub(super) fn place(doc: &mut Document, layer: LayerId, order: i16) {
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Solid { rgba: [255, 0, 0, 255], width: 64, height: 64 },
                    order,
                    timing: LayerTiming::place(0, None, 300),
                },
            },
        ])
        .expect("layer 配置");
    }

    pub(super) fn lock(doc: &mut Document, layer: LayerId) {
        doc.apply(Intent::SetAttrs {
            layer,
            patch: LayerAttrsPatch { locked: Some(true), ..Default::default() },
        })
        .expect("ロック設定");
    }

    pub(super) fn track_with(frames: &[i64]) -> KeyframeTrack {
        let mut track = KeyframeTrack::new();
        for &frame in frames {
            track.insert(Keyframe {
                t: RationalTime::try_new(frame, 30).expect("frame は収まる"),
                value: Value::F64(1.0),
                interp: Interp::Linear,
                spatial: None,
            });
        }
        track
    }

    pub(super) fn set_track(doc: &mut Document, layer: LayerId, property: &PropertyId, frames: &[i64]) {
        doc.apply(Intent::SetTrack { layer, property: property.clone(), track: track_with(frames) })
            .expect("track を書ける");
    }

    pub(super) fn selector(layer: LayerId, property: &PropertyId, frame: i64) -> KeySelector {
        KeySelector { layer, property: property.clone(), frame }
    }

    pub(super) fn opacity() -> PropertyId {
        PropertyId::new(motolii_store::property::OPACITY).expect("opacity は予約語ではない")
    }

    pub(super) fn position_x() -> PropertyId {
        PropertyId::new(motolii_store::property::POSITION_X).expect("position.x は予約語ではない")
    }

    /// track の keys を(時刻順のまま)`(frame, interp)` へ写す検分ヘルパー。
    pub(super) fn interps_of(doc: &Document, layer: LayerId, property: &PropertyId) -> Vec<(i64, Interp)> {
        let fps = doc.view().composition().unwrap().unwrap().fps;
        doc.view()
            .track(layer, property)
            .unwrap()
            .unwrap()
            .keys()
            .iter()
            .map(|key| (key.t.try_to_frame_round(fps).unwrap(), key.interp))
            .collect()
    }

    pub(super) fn no_mods() -> iced::keyboard::Modifiers {
        iced::keyboard::Modifiers::default()
    }
}

/// 選択キーの補間切替(第3切片 B15、map 495/512/513/514・485〜490)。
/// **未実行**(裁定189 — supervisor が波末一括)。
#[cfg(test)]
mod key_interp_tests {
    use super::third_slice_fixtures::*;
    use super::*;

    /// **オラクル(赤→緑)**: 選択された2キーだけ Hold になり、非選択キーは
    /// Linear のまま。時刻は不変。確定は1回の `apply_all` = **1 undo**
    /// (undo 1回で全キー Linear へ戻り、floor に着く)。
    #[test]
    fn set_key_interp_changes_only_selected_keys_in_one_undo() {
        let mut doc = doc_with_comp();
        let layer = LayerId(1);
        let property = opacity();
        place(&mut doc, layer, 0);
        set_track(&mut doc, layer, &property, &[0, 100, 250]);
        doc.mark_undo_floor();

        let mut session = Session::default();
        session.selection = Some(layer);
        session.selected_keys = vec![selector(layer, &property, 0), selector(layer, &property, 100)];
        let mut pane = PaneState::new();

        let reason = pane.update(Message::SetKeyInterp(Interp::Hold), &mut doc, &mut session, no_mods());
        assert!(reason.is_none(), "正常系で拒否理由が返った: {reason:?}");
        assert_eq!(
            interps_of(&doc, layer, &property),
            vec![(0, Interp::Hold), (100, Interp::Hold), (250, Interp::Linear)],
            "選択キーだけが Hold になっていない(時刻も不変のはず)"
        );
        // 選択は生きたまま(frame が動かないので selector はそのまま有効)。
        assert_eq!(session.selected_keys.len(), 2);

        assert!(doc.undo(), "1回目の undo が効かない");
        assert_eq!(
            interps_of(&doc, layer, &property),
            vec![(0, Interp::Linear), (100, Interp::Linear), (250, Interp::Linear)],
            "undo 1回で確定前へ戻っていない(1操作=1undo 違反)"
        );
        assert!(!doc.can_undo(), "余分な undo 段がある(1操作=1undo 違反)");
    }

    /// 空選択は理由つき拒否(M13)— Document は無傷。
    #[test]
    fn set_key_interp_with_no_selection_refuses_with_a_reason() {
        let mut doc = doc_with_comp();
        let layer = LayerId(1);
        place(&mut doc, layer, 0);
        doc.mark_undo_floor();
        let mut session = Session::default();
        let mut pane = PaneState::new();

        let reason = pane.update(Message::SetKeyInterp(Interp::Hold), &mut doc, &mut session, no_mods());
        assert!(reason.is_some(), "空選択が黙って飲み込まれた(M13 違反)");
        assert!(!doc.can_undo(), "拒否したのに Document が動いた");
    }

    /// ロック層は理由つき拒否 — track は不変。
    #[test]
    fn set_key_interp_refuses_a_locked_layer() {
        let mut doc = doc_with_comp();
        let layer = LayerId(1);
        let property = opacity();
        place(&mut doc, layer, 0);
        set_track(&mut doc, layer, &property, &[0, 100]);
        lock(&mut doc, layer);
        doc.mark_undo_floor();

        let mut session = Session::default();
        session.selection = Some(layer);
        session.selected_keys = vec![selector(layer, &property, 0)];
        let mut pane = PaneState::new();

        let reason = pane.update(Message::SetKeyInterp(Interp::Hold), &mut doc, &mut session, no_mods());
        assert!(reason.is_some(), "ロック層への補間変更が黙って通った");
        assert_eq!(
            interps_of(&doc, layer, &property),
            vec![(0, Interp::Linear), (100, Interp::Linear)],
            "拒否したのに track が動いた"
        );
    }

    /// Easy Ease プリセット(map 485〜490)は妥当な Bezier(x1/x2 ∈ [0,1] —
    /// `TrackError::InvalidBezier` の柵の内側)で、store を実際に往復できる。
    #[test]
    fn easy_ease_presets_are_valid_beziers_that_round_trip_through_the_store() {
        for preset in [EASY_EASE, EASY_EASE_IN, EASY_EASE_OUT] {
            let Interp::Bezier { x1, x2, .. } = preset else {
                panic!("Easy Ease プリセットが Bezier でない: {preset:?}");
            };
            assert!((0.0..=1.0).contains(&x1) && (0.0..=1.0).contains(&x2));
        }

        let mut doc = doc_with_comp();
        let layer = LayerId(1);
        let property = opacity();
        place(&mut doc, layer, 0);
        set_track(&mut doc, layer, &property, &[0, 100]);
        let mut session = Session::default();
        session.selection = Some(layer);
        session.selected_keys = vec![selector(layer, &property, 0)];
        let mut pane = PaneState::new();

        let reason = pane.update(Message::SetKeyInterp(EASY_EASE), &mut doc, &mut session, no_mods());
        assert!(reason.is_none(), "Easy Ease の適用が拒否された: {reason:?}");
        assert_eq!(interps_of(&doc, layer, &property)[0], (0, EASY_EASE));
    }
}

/// キー選択動詞(第3切片 B15、map 484/509/510)+ 既存 Delete の複数対応確認
/// (発注書2「複数キーの一括 Delete」)。**未実行**(裁定189)。
#[cfg(test)]
mod key_selection_verb_tests {
    use super::third_slice_fixtures::*;
    use super::*;

    /// map 484: 全キー選択解除 — Session だけが動き、Document は無傷。
    #[test]
    fn deselect_all_keys_clears_the_selection_and_anchor() {
        let mut doc = doc_with_comp();
        let layer = LayerId(1);
        let property = opacity();
        place(&mut doc, layer, 0);
        set_track(&mut doc, layer, &property, &[0, 100]);
        doc.mark_undo_floor();

        let mut session = Session::default();
        session.selected_keys = vec![selector(layer, &property, 0)];
        session.key_anchor = Some(selector(layer, &property, 0));
        let mut pane = PaneState::new();

        let reason = pane.update(Message::DeselectAllKeys, &mut doc, &mut session, no_mods());
        assert!(reason.is_none());
        assert!(session.selected_keys.is_empty(), "選択が残っている");
        assert!(session.key_anchor.is_none(), "anchor が残っている");
        assert!(!doc.can_undo(), "選択解除が Document を動かした");
    }

    /// map 509(§8.1 SelectAllKeysOfProperty): property の全キーが選択へ
    /// 差し替わり、anchor は先頭キー。
    #[test]
    fn select_all_keys_of_property_selects_every_key_and_anchors_the_first() {
        let mut doc = doc_with_comp();
        let layer = LayerId(1);
        let property = opacity();
        place(&mut doc, layer, 0);
        set_track(&mut doc, layer, &property, &[0, 100, 250]);

        let mut session = Session::default();
        session.selection = Some(layer);
        // 差し替え(足し引きではない)ことを見るため、別の選択を先に置く。
        session.selected_keys = vec![selector(layer, &property, 100)];
        let mut pane = PaneState::new();

        let reason = pane.update(
            Message::SelectAllKeysOfProperty { layer, property: property.clone() },
            &mut doc,
            &mut session,
            no_mods(),
        );
        assert!(reason.is_none());
        assert_eq!(
            session.selected_keys,
            vec![
                selector(layer, &property, 0),
                selector(layer, &property, 100),
                selector(layer, &property, 250),
            ],
            "property の全キーが選択されていない"
        );
        assert_eq!(session.key_anchor, Some(selector(layer, &property, 0)), "anchor が先頭キーでない");
    }

    /// map 510: 表示中(選択 layer のキー持ち property 行)の全キーが
    /// `key_order`(行順→時刻順)で選択される。
    #[test]
    fn select_all_visible_keys_spans_every_visible_property_row() {
        let mut doc = doc_with_comp();
        let layer = LayerId(1);
        let (opacity, position_x) = (opacity(), position_x());
        place(&mut doc, layer, 0);
        set_track(&mut doc, layer, &opacity, &[0, 100]);
        set_track(&mut doc, layer, &position_x, &[50]);

        let mut session = Session::default();
        session.selection = Some(layer);
        let mut pane = PaneState::new();

        let reason = pane.update(Message::SelectAllVisibleKeys, &mut doc, &mut session, no_mods());
        assert!(reason.is_none());
        assert_eq!(session.selected_keys.len(), 3, "表示中の全キー(2+1本)が選択されていない");
        for key in [
            selector(layer, &opacity, 0),
            selector(layer, &opacity, 100),
            selector(layer, &position_x, 50),
        ] {
            assert!(session.selected_keys.contains(&key), "{key:?} が選択に入っていない");
        }
        // anchor は key_order の先頭(行順→時刻順の1本目)。
        let fps = doc.view().composition().unwrap().unwrap().fps;
        let order = key_order(&property_rows(&doc.view(), &session, Some(fps)));
        assert_eq!(session.key_anchor.as_ref(), order.first(), "anchor が key_order の先頭でない");
    }

    /// **発注書2の確認**: 既存 `DeleteSelectedKeys` は複数キー(property 跨ぎ)を
    /// 1回で消し、**1 undo** で全部戻る — 複数対応は既に実装済みであることの柵。
    #[test]
    fn delete_selected_keys_removes_keys_across_properties_in_one_undo() {
        let mut doc = doc_with_comp();
        let layer = LayerId(1);
        let (opacity, position_x) = (opacity(), position_x());
        place(&mut doc, layer, 0);
        set_track(&mut doc, layer, &opacity, &[0, 100]);
        set_track(&mut doc, layer, &position_x, &[50, 60]);
        doc.mark_undo_floor();

        let mut session = Session::default();
        session.selection = Some(layer);
        session.selected_keys =
            vec![selector(layer, &opacity, 100), selector(layer, &position_x, 50)];
        let mut pane = PaneState::new();

        let reason = pane.update(Message::DeleteSelectedKeys, &mut doc, &mut session, no_mods());
        assert!(reason.is_none());
        let frames = |doc: &Document, property: &PropertyId| -> Vec<i64> {
            interps_of(doc, layer, property).into_iter().map(|(frame, _)| frame).collect()
        };
        assert_eq!(frames(&doc, &opacity), vec![0], "opacity 側の選択キーが消えていない");
        assert_eq!(frames(&doc, &position_x), vec![60], "position.x 側の選択キーが消えていない");

        assert!(doc.undo(), "undo が効かない");
        assert_eq!(frames(&doc, &opacity), vec![0, 100]);
        assert_eq!(frames(&doc, &position_x), vec![50, 60]);
        assert!(!doc.can_undo(), "property 跨ぎの削除が複数 undo 段に割れている(1操作=1undo 違反)");
    }
}

/// Stage 重なり並べ替え(第3切片、map B44 184/292/293・§8.1)。
/// **未実行**(裁定189)。
#[cfg(test)]
mod restack_message_tests {
    use super::third_slice_fixtures::*;
    use super::*;

    fn orders(doc: &Document) -> Vec<(LayerId, i16)> {
        let store = doc.view();
        let mut out: Vec<(LayerId, i16)> = store
            .layers()
            .into_iter()
            .filter_map(|id| store.meta(id).ok().flatten().map(|meta| (id, meta.order)))
            .collect();
        out.sort_by_key(|&(id, order)| (order, id));
        out
    }

    /// **オラクル(赤→緑)**: Forward で選択 layer が直上の layer を1枚跨ぎ、
    /// 確定は1回の `apply_all` = **1 undo**。
    #[test]
    fn restack_forward_moves_the_selection_in_front_in_one_undo() {
        let mut doc = doc_with_comp();
        let (back, front) = (LayerId(1), LayerId(2));
        place(&mut doc, back, 0);
        place(&mut doc, front, 1);
        doc.mark_undo_floor();

        let mut session = Session::default();
        session.selection = Some(back);
        let mut pane = PaneState::new();

        let reason = pane.update(Message::RestackLayer(StackDirection::Forward), &mut doc, &mut session, no_mods());
        assert!(reason.is_none(), "正常系で拒否理由が返った: {reason:?}");
        let sequence: Vec<LayerId> = orders(&doc).into_iter().map(|(id, _)| id).collect();
        assert_eq!(sequence, vec![front, back], "背面の layer が前面へ出ていない");

        assert!(doc.undo(), "undo が効かない");
        let sequence: Vec<LayerId> = orders(&doc).into_iter().map(|(id, _)| id).collect();
        assert_eq!(sequence, vec![back, front], "undo 1回で元の重なりへ戻らない");
        assert!(!doc.can_undo(), "並べ替えが複数 undo 段に割れている(1操作=1undo 違反)");
    }

    /// 空選択は理由つき拒否(M13)。
    #[test]
    fn restack_with_no_selection_refuses_with_a_reason() {
        let mut doc = doc_with_comp();
        place(&mut doc, LayerId(1), 0);
        doc.mark_undo_floor();
        let mut session = Session::default();
        let mut pane = PaneState::new();

        let reason = pane.update(Message::RestackLayer(StackDirection::ToFront), &mut doc, &mut session, no_mods());
        assert!(reason.is_some(), "空選択が黙って飲み込まれた(M13 違反)");
        assert!(!doc.can_undo());
    }

    /// ロック層は理由つき拒否。
    #[test]
    fn restack_refuses_a_locked_layer() {
        let mut doc = doc_with_comp();
        let (a, b) = (LayerId(1), LayerId(2));
        place(&mut doc, a, 0);
        place(&mut doc, b, 1);
        lock(&mut doc, a);
        doc.mark_undo_floor();

        let mut session = Session::default();
        session.selection = Some(a);
        let mut pane = PaneState::new();

        let reason = pane.update(Message::RestackLayer(StackDirection::Forward), &mut doc, &mut session, no_mods());
        assert!(reason.is_some(), "ロック層の並べ替えが黙って通った");
        assert!(!doc.can_undo(), "拒否したのに Document が動いた");
    }

    /// 端で動けない移動は**黙ってスキップ**(失敗ではない — §2 split の
    /// 「端ちょうどはスキップ」と同格)で、undo エントリも作らない。
    #[test]
    fn restack_at_the_edge_is_a_silent_noop_without_an_undo_entry() {
        let mut doc = doc_with_comp();
        let (back, front) = (LayerId(1), LayerId(2));
        place(&mut doc, back, 0);
        place(&mut doc, front, 1);
        doc.mark_undo_floor();

        let mut session = Session::default();
        session.selection = Some(front);
        let mut pane = PaneState::new();

        let reason = pane.update(Message::RestackLayer(StackDirection::Forward), &mut doc, &mut session, no_mods());
        assert!(reason.is_none(), "端の no-op が拒否扱いになっている");
        assert!(!doc.can_undo(), "no-op なのに空の undo エントリができた");
    }
}

/// inline rename(第3切片、map B02 785・正典 §6)。**未実行**(裁定189)。
#[cfg(test)]
mod rename_message_tests {
    use super::third_slice_fixtures::*;
    use super::*;

    fn name_of(doc: &Document, layer: LayerId) -> String {
        doc.view().attrs(layer).unwrap().unwrap_or_default().name
    }

    fn set_name(doc: &mut Document, layer: LayerId, name: &str) {
        doc.apply(Intent::SetAttrs {
            layer,
            patch: LayerAttrsPatch { name: Some(name.to_owned()), ..Default::default() },
        })
        .expect("名前設定");
    }

    /// **オラクル(赤→緑)**: begin は現在名を下書きへ写し、commit が
    /// `SetAttrs` を1回出す(**1 undo**)。
    #[test]
    fn rename_begin_seeds_the_draft_and_commit_writes_the_name_once() {
        let mut doc = doc_with_comp();
        let layer = LayerId(1);
        place(&mut doc, layer, 0);
        set_name(&mut doc, layer, "before");
        doc.mark_undo_floor();

        let mut session = Session::default();
        session.selection = Some(layer);
        let mut pane = PaneState::new();

        assert!(pane.update(Message::RenameBegin(layer), &mut doc, &mut session, no_mods()).is_none());
        assert_eq!(pane.rename_draft(), Some((layer, "before")), "下書きが現在名から始まっていない");

        pane.update(Message::RenameEdited("after".into()), &mut doc, &mut session, no_mods());
        let reason = pane.update(Message::RenameCommit, &mut doc, &mut session, no_mods());
        assert!(reason.is_none(), "正常系の確定が拒否された: {reason:?}");
        assert!(pane.rename_draft().is_none(), "確定後も rename 状態が残っている");
        assert_eq!(name_of(&doc, layer), "after");

        assert!(doc.undo(), "undo が効かない");
        assert_eq!(name_of(&doc, layer), "before", "undo 1回で旧名へ戻らない");
        assert!(!doc.can_undo(), "改名が複数 undo 段に割れている(1操作=1undo 違反)");
    }

    /// 正典 §6「空名拒否」: 拒否は理由つきで、**下書きは捨てない**(編集継続 —
    /// 入力を失わない)。Document は無傷。
    #[test]
    fn rename_commit_rejects_an_empty_name_and_keeps_editing() {
        let mut doc = doc_with_comp();
        let layer = LayerId(1);
        place(&mut doc, layer, 0);
        set_name(&mut doc, layer, "before");
        doc.mark_undo_floor();

        let mut session = Session::default();
        let mut pane = PaneState::new();
        pane.update(Message::RenameBegin(layer), &mut doc, &mut session, no_mods());
        pane.update(Message::RenameEdited("   ".into()), &mut doc, &mut session, no_mods());

        let reason = pane.update(Message::RenameCommit, &mut doc, &mut session, no_mods());
        assert!(reason.is_some(), "空名が黙って通った/黙って捨てられた(M13 違反)");
        assert_eq!(pane.rename_draft(), Some((layer, "   ")), "拒否で下書きが失われた(編集継続でない)");
        assert_eq!(name_of(&doc, layer), "before");
        assert!(!doc.can_undo(), "拒否したのに Document が動いた");
    }

    /// 正典 §6「同名 no-op」: Intent を出さない(空 undo を作らない)。
    #[test]
    fn rename_commit_with_the_unchanged_name_is_a_noop() {
        let mut doc = doc_with_comp();
        let layer = LayerId(1);
        place(&mut doc, layer, 0);
        set_name(&mut doc, layer, "same");
        doc.mark_undo_floor();

        let mut session = Session::default();
        let mut pane = PaneState::new();
        pane.update(Message::RenameBegin(layer), &mut doc, &mut session, no_mods());
        let reason = pane.update(Message::RenameCommit, &mut doc, &mut session, no_mods());
        assert!(reason.is_none());
        assert!(pane.rename_draft().is_none());
        assert!(!doc.can_undo(), "同名確定が空の undo エントリを作った");
    }

    /// ロック層は begin の時点で理由つき拒否(`start_drag` と同じ柵)。
    #[test]
    fn rename_begin_refuses_a_locked_layer() {
        let mut doc = doc_with_comp();
        let layer = LayerId(1);
        place(&mut doc, layer, 0);
        lock(&mut doc, layer);

        let mut session = Session::default();
        let mut pane = PaneState::new();
        let reason = pane.update(Message::RenameBegin(layer), &mut doc, &mut session, no_mods());
        assert!(reason.is_some(), "ロック層の改名開始が黙って通った");
        assert!(pane.rename_draft().is_none());
    }

    /// Esc(裁定151「キャンセルの一般化」): `cancel_rename` は state を捨てる
    /// だけで Document 無傷。2回目は「何も捨てなかった」。
    #[test]
    fn cancel_rename_drops_the_draft_without_touching_the_document() {
        let mut doc = doc_with_comp();
        let layer = LayerId(1);
        place(&mut doc, layer, 0);
        set_name(&mut doc, layer, "before");
        doc.mark_undo_floor();

        let mut session = Session::default();
        let mut pane = PaneState::new();
        pane.update(Message::RenameBegin(layer), &mut doc, &mut session, no_mods());
        pane.update(Message::RenameEdited("half-typed".into()), &mut doc, &mut session, no_mods());

        assert!(pane.cancel_rename(), "捨てる物があるのに false");
        assert!(pane.rename_draft().is_none());
        assert!(!pane.cancel_rename(), "2回目のキャンセルが true を返した");
        assert_eq!(name_of(&doc, layer), "before");
        assert!(!doc.can_undo());
    }
}

#[cfg(test)]
mod fold_message_tests {
    use super::*;
    use motolii_store::Composition;

    fn doc_with_comp() -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: motolii_store::Fps::try_new(30, 1).expect("30/1 は正の既約 fps"),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .expect("comp 設定");
        doc
    }

    /// **オラクル(赤→緑)**: `Message::ToggleFold` は `PaneState::update` だけで
    /// 完結する — Shell 側(5例外の先取り)の改修は不要(mod doc の
    /// 「shell/src は改修不要」節の柵)。
    #[test]
    fn toggle_fold_flips_session_state_without_touching_the_document() {
        let mut doc = doc_with_comp();
        let mut session = Session::default();
        let mut pane = PaneState::new();
        let layer = LayerId(1);

        assert!(!session.timeline_fold.is_folded(layer));
        let reason = pane.update(
            Message::ToggleFold(layer),
            &mut doc,
            &mut session,
            iced::keyboard::Modifiers::default(),
        );
        assert!(reason.is_none(), "ToggleFold が拒否理由を返している");
        assert!(session.timeline_fold.is_folded(layer), "1回目のToggleFoldで畳まれていない");

        pane.update(Message::ToggleFold(layer), &mut doc, &mut session, iced::keyboard::Modifiers::default());
        assert!(!session.timeline_fold.is_folded(layer), "2回目のToggleFoldで開き直っていない");
    }

    /// 存在しない LayerId への ToggleFold も panic しない(fold 状態は
    /// LayerId の存在に依存しない Session 側の集合、`Message::ToggleFold` doc 参照)。
    #[test]
    fn toggle_fold_on_a_missing_layer_does_not_panic() {
        let mut doc = doc_with_comp();
        let mut session = Session::default();
        let mut pane = PaneState::new();
        let ghost = LayerId(999_999);

        let reason =
            pane.update(Message::ToggleFold(ghost), &mut doc, &mut session, iced::keyboard::Modifiers::default());
        assert!(reason.is_none());
        assert!(session.timeline_fold.is_folded(ghost));
    }
}
