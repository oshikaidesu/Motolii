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
//! **例外(survey §3.2 exception 1)**: `ScrubTo` は本来 core 腕だが
//! `input.rs` の内部(ルーラー/空白部クリック)からも直接発行される。
//! `Select`/`ToggleMute`/`ToggleSolo`/`ToggleLock` は既存の root との結線互換
//! のために同名の腕を残している(上の doc 節の `toggle_layer_hidden` 共有の
//! 判断とセット)。pane crate から root の `Message` を参照できないため、
//! ここに同名の腕を複製した。
//! **`Shell::update` はこの5腕を[`PaneState::update`]へ渡す前に先取りする**
//! (`select_single`/`playhead`/`toggle_layer_*` へ直接委譲 — 既存の挙動その
//! まま)。[`PaneState::update`] にこの5腕が来ることは実運用では無いが、
//! 網羅性のために受理はする(no-op)。
//!
//! レール行の複数選択は上の `Select` 例外を使わない。`Select` は Shell の
//! core 先取りで `PaneState::update` へ届かないため、rail は
//! [`Message::SelectLayer`] を発行する。この腕は pane が表示順と修飾キー
//! から解決済みの操作を受け、ここで `Session` の選択へ適用する。
//!
//! ## SP-2 分割(2726行 → 800行以下、中身は無改変・純粋な移送)
//!
//! `write.rs` 単一ファイルだった頃、**clip のドラッグ/トリム・キーフレームの
//! ドラッグ・選択の解決・split** が同居していた(発注書の指摘どおり)。
//! `PaneState` の inherent impl は複数ファイルに分けても1つの型として振る
//! 舞う(メソッド呼び出しは型ベースの解決で `use` 不要 — Rust の仕様)ので、
//! 責任ごとに `impl PaneState { ... }` を分けた:
//! - [`clip_drag`] … クリップの move/trim(`start_drag`/`continue_drag`/`finish_drag`)
//! - [`key_drag`] … キーの時刻ドラッグ/リタイム/Nudge と共通書き口 `commit_key_frames`
//! - [`keys`] … キー選択の確定・補間・全選択系・Time-Reverse・追加削除・コピペ
//! - [`loop_work_area`] … ループ帯・作業範囲のドラッグと確定
//! - [`misc`] … Split・inline rename・意味点ジャンプ・Stage 重なり
//! - [`tests`] … 既存の `#[cfg(test)]` 8モジュールをそのまま移設
//!
//! この `mod.rs` には Message 定義・状態の struct 定義・**読み取り専用
//! アクセサ**・唯一の書き口 [`PaneState::update`](Message を各モジュールへ
//! 振り分けるだけ)を残した。子モジュールからは親の private field/型が
//! Rust の可視性規則どおり素通しで見える(descendant が ancestor の private
//! item を見られる)ので、逆方向(`update` から子の関数を呼ぶ)にだけ
//! `pub(crate)` を足した — crate 外から呼ばれる物は元から `pub` のままで、
//! `lib.rs` の再輸出も変えていない。
#[derive(Debug, Clone)]
pub enum Message {
    /// 例外: 既存の core 結線互換用。rail の行クリックは
    /// [`Message::SelectLayer`] を使うため、この腕は legacy の no-op 受け口。
    Select(LayerId),
    /// Timeline rail の行選択。`Select` と違い Shell の core 先取りを通さず、
    /// pane の `PaneState::update` へ到達する。`order` は現在 rail に見えて
    /// いる行順、`op` は rail が保持する修飾キー状態から選んだ操作。
    SelectLayer {
        order: Vec<LayerId>,
        op: rows::LayerSelectionOp,
    },
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
    /// Time-Reverse Keyframes(map 518)。選択キー集合を `(layer, property)`
    /// ごとに独立して、それぞれ自身の `[min, max]` の中で鏡映する
    /// ([`crate::keys2::reversed_key_group`] — 純関数はここより前から
    /// 存在していたが結線されていなかった、この腕がその結線)。値・`interp`
    /// は不変、frame の並びだけが入れ替わる。空選択・ロック層は理由つき拒否
    /// (M13、`SetKeyInterp` と同じ形)。**1操作 = 1 undo**。
    ReverseSelectedKeys,
    /// キーの追加/削除(map 472「Add Keyframe」・473/474(公式表記/ショートカット
    /// 表記違い)・476「Add or remove keyframe at current time」・477/478
    /// 「Add Static Keyframe」表記違い — 全て「今の playhead 位置にキーが
    /// あれば消す・無ければ足す」の同じ判定へ落ちる
    /// ([`crate::keys2::toggle_keyframe_at`] が対象決め、この腕が実際の
    /// 書き込み)。追加時は [`motolii_store::KeyframeTrack::eval`] で今の
    /// カーブの値を評価してそのまま焼き付ける(値そのものは動かない)。
    /// Static/Hold 等の種別選択は keys2 モジュール doc の「motolii-eval 側の
    /// 領分」注記どおりここでは区別しない(477/478 も 472 と同じ Linear で
    /// 追加する — RETURN の逸脱参照)。ロック層は理由つき拒否(M13)、comp が
    /// 無い(frame⇄時刻の変換ができない)場合も同様。
    ToggleKeyframeAtPlayhead { layer: LayerId, property: PropertyId },
    /// コピー(map に単体行は無い — 507「Reverse paste」の土台、keys2 モジュール
    /// doc の「Copy/Paste keyframes」節参照)。選択キーの位置
    /// ([`crate::keys2::copy_keys`])と実際の値([`motolii_store::Keyframe`]、
    /// Document から読み直す)を [`PaneState`] 側のクリップボードへ保存する。
    /// 空選択は理由つき拒否(M13)。
    CopySelectedKeys,
    /// 通常貼り付け(507 の土台)。playhead を新しい anchor として
    /// [`crate::keys2::paste_keys`] が返す位置へ、コピー時の値を書き戻す。
    /// クリップボードが空/貼り先がロックなら理由つき拒否(M13)。
    PasteKeys,
    /// Reverse paste copied keyframes(map 507)。同上、
    /// [`crate::keys2::paste_keys_reversed`] で集合を鏡映してから貼る。
    PasteKeysReversed,
    /// Next Clip/Edit(map 1088「Next Clip/Edit」・1089「Next Edit」〈Premiere
    /// 表記違い〉)。表示中の全 clip の start/end
    /// ([`crate::keys2::clip_edit_points`])を意味点として、playhead から先の
    /// 最も近い点へ渡る([`crate::nav::nearest_meaning_point`] Next)。渡る先が
    /// 無ければ no-op(`nav` モジュール doc「呼び出し側は playhead を動かさ
    /// ない」)。選択・キーは動かさない。
    JumpToNextClipEdit,
    /// Previous Clip/Edit(map 1108「Previous Clip/Edit」・1109「Previous
    /// Edit」)。同上、Prev 方向。
    JumpToPreviousClipEdit,

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

    // ---- Timeline マーカー(B19、S2 発注 #22「追加 UI が無い」の穴埋め) ----
    /// ルーラ帯の locator lane を右クリック(ドラッグ中でない時)= その
    /// クリック位置のフレームへマーカーを置く。キーボード M
    /// (`next/shell/motolii-shell/src/input.rs` の `resolve_navigation_key`、
    /// `Message::Marker(MarkerMessage::AddAtPlayhead)`)と併存する2入口目
    /// (S6 併存、裁定195)。**Select/ScrubTo と同じ「shell 先取りの例外」** —
    /// 意味(`Intent::SetMarkers`)は shell の `update_marker` が持つので
    /// [`PaneState::update`] では no-op。
    AddMarkerAt(i64),

    // ---- Split(レイヤー分割、B39 — `crate::split` モジュール doc「統合手順」) ----
    /// Command+B(map 267)/メニュー Split(id 163/317 ほか)。選択レイヤー
    /// (複数可)を playhead で割る。`crate::split::Message::SplitAtPlayhead`
    /// (旧・宣言のみの pane-local message)をここへ畳んだ — `split.rs` 側の
    /// 宣言は重複させないため削除済み(モジュール doc「統合手順1」参照)。
    SplitAtPlayhead,

    // ---- Timeline 音声波形(TL7 統合手順3・5、`crate::waveform_view`
    //      モジュール doc「次波の統合手順」) ----
    /// 非同期取得の完了。shell が(`PaneState::plan_waveforms` が返した要求を
    /// `iced::Task::perform(motolii_media::waveform_peaks(path, buckets), ...)`
    /// へ変換して実際に発火した後)結果をこの腕で送り返す想定 — **この
    /// レーンでは発火自体はしない**(`waveform_view` モジュール doc 参照)。
    /// `buckets` が現在の `Loading` と食い違えば(取得中に再ズームされ別要求が
    /// 有効になっている)stale として捨てる — `waveform_view::plan` と同じ
    /// 「今の要求と一致するかだけ見る」ヒステリシスの思想。
    WaveformFetched { layer: LayerId, buckets: usize, peaks: Vec<(f32, f32)> },
    /// 非同期取得の失敗(音声トラック無し・ffmpeg 不在 等)。`NotRequested` へ
    /// 戻すと `plan` が次のフレームで即再要求してしまう(ヒステリシスが
    /// 効かない無限リトライ)ので、空 peaks の `Ready` へ落とす —
    /// `waveform_segments` は空 peaks を panic なく「何も描かない」に畳む
    /// (`waveform_view.rs` の退化オラクル参照)。
    WaveformFetchFailed { layer: LayerId, buckets: usize },
}

use std::collections::{BTreeMap, HashMap, HashSet};

use motolii_store::{
    Composition, Document, Intent, Interp, KeyframeTrack, LayerAttrsPatch, LayerId, LayerTiming,
    PropertyId, RationalTime,
};

use crate::hit::BarPart;
use crate::keys2;
use crate::nav;
use crate::shuttle::ShuttleCommand;
use crate::split;
use crate::stacking::{self, StackDirection};
use crate::state::Session;
use crate::waveform_view::{self, WaveformAction, WaveformState};
use crate::work_area::{self, LoopBandPart, WorkArea};
use crate::{
    clip_gesture, key_gesture, key_order, property_rows, rows, AudioRowProjection, KeySelectionOp,
    KeySelector,
};

use keys::{
    apply_key_selection, delete_selected_keys, reverse_selected_keys, select_all_keys_of_property,
    select_all_visible_keys, set_key_interp, toggle_keyframe_at_playhead,
};
use key_drag::nudge_keyframe;
use misc::{comp_duration, jump_to_clip_edit, restack_layers};

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
/// **E-2 で複数選択へ拡張**(軸台帳 A08「Timeline clip move/trim」単数のみの
/// 穴)。`origins`/`preview` は影響を受ける全 layer の組 — trim(`Edge*`)は
/// 掴んだ1本だけ(`vec![(layer, origin)]`)、move(`Body`)は掴んだ瞬間に
/// 選択されていた全 layer(`TimelineKeyDragState::origins` と同じ形 —
/// キー側が既に「掴んだキー全員の origin を持ち、delta を全員へ適用」して
/// いるのに倣った)。`layer`(grabbed)は delta の基準・防御的な単体参照に
/// 使う。
#[derive(Clone)]
struct TimelineDragState {
    layer: LayerId,
    part: BarPart,
    /// 掴んだ瞬間に Document から読んだそのままの値(影響を受ける全 layer 分)。
    /// **move/trim の計算は毎回これを基準に絶対値で出し直す**(delta 蓄積禁止、
    /// 正典 §2)。
    origins: Vec<(LayerId, LayerTiming)>,
    /// 掴んだ瞬間のポインタ位置(comp frame、スナップ前)。
    grab_at_frame: i64,
    /// 直近の move/trim 計算結果。release がこれを(`origins` と違う要素だけ)
    /// 1回 `apply_all` する。
    preview: Vec<(LayerId, LayerTiming)>,
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
    /// 波形取得状態(TL7 統合手順3)。key = layer。`work_area`/`loop_enabled`
    /// と同じ「フレームを跨いで生きる」pane-local write-set(このレーンの
    /// write-set は `next/ui/motolii-timeline-pane/src/**` のみ)。
    waveforms: HashMap<LayerId, WaveformState>,
    /// キーのコピー/ペースト(map 507 の土台)。`work_area`/`waveforms` と同じ
    /// 「フレームを跨いで生きる pane-local state」— Copy 後に他の操作を
    /// 挟んでも保持される。**`Session` へは足さない** —
    /// `Session`(`motolii-shell-state` crate)は他 pane とも共有される親で
    /// あり、clipboard は Timeline pane だけの概念なので、cross-crate な
    /// 昇格をせずこの crate 内(このレーンの write-set)に閉じる
    /// (`key_anchor` は選択の一部として `Session` にある既存の身分だが、
    /// clipboard は選択とは別物 — 発注書「`key_anchor` の流儀に合わせて
    /// 決める」に対する判断: 流儀は踏襲しつつ置き場は pane 側)。
    key_clipboard: KeyClipboardState,
    /// Timeline rail の Shift 範囲選択の基点。`Session` は layer 選択集合を
    /// 持つが layer 用 anchor は持たないため、キー側の `Session::key_anchor`
    /// と同じ transient 身分として pane に閉じる。rail 選択以外の既存経路を
    /// 侵食しないため、最初の Shift クリックは anchor 無しの安全側へ倒れる。
    layer_selection_anchor: Option<LayerId>,
}

/// inline rename の一時状態(正典 §6「リネーム」)。掴んだ layer と毎打鍵の
/// 下書きだけ — 確定([`Message::RenameCommit`])が `Intent::SetAttrs` を
/// 1回出すまで Document を触らない。
struct RenameDraft {
    layer: LayerId,
    draft: String,
}

/// キーのコピー/ペースト(map 507 の土台)の実体。[`keys2::KeyClipboard`]は
/// 位置(layer/property/コピー時点の anchor からの offset)だけを運ぶ純データ
/// (`keys2` モジュール doc 参照)— 実際に書き戻す値(value/interp/spatial)は
/// `keys2` が知らないので、ここが `clip.keys` と同じ添字で対応する
/// [`motolii_store::Keyframe`] 列として持つ(コピー時に Document から読み
/// 取って保存する)。
#[derive(Clone, Default)]
struct KeyClipboardState {
    clip: keys2::KeyClipboard,
    values: Vec<motolii_store::Keyframe>,
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

    /// 波形取得状態の全体(`TimelinePane::with_waveforms` へそのまま渡す
    /// 読み取り専用、TL7 統合手順3)。空なら波形は1本も描かれない
    /// (`canvas.rs` の bar 描画ループ参照)。
    pub fn waveforms(&self) -> &HashMap<LayerId, WaveformState> {
        &self.waveforms
    }

    /// 毎フレーム(`Shell::build_timeline_pane` が呼ぶ想定)判断(TL7 統合手順1)。
    /// `audio_rows`(`crate::audio_rows`)の `has_audio` な layer それぞれについて、
    /// `clip_width_px` で聞いた画面幅(comp フレーム→px の変換は呼び出し側 —
    /// この pane crate の `frame_to_x`/`Dimensions` を知っている canvas/shell 側の
    /// 責任)を [`waveform_view::plan`] へ渡し、[`WaveformAction::Fetch`] が
    /// 返れば該当 layer を `Loading` へ遷移させて `(layer, path, buckets)` を
    /// 要求列へ積む。
    ///
    /// **実際の非同期発火はしない**(`WaveformAction::Fetch` を pane-local な
    /// 非同期要求として表現するだけ、発注書 EXACT TARGET 1) — 返した列を
    /// 呼び出し側(shell)が `iced::Task::perform(motolii_media::
    /// waveform_peaks(path, buckets), ...)` へ変換し、完了したら
    /// `Message::WaveformFetched`/`WaveformFetchFailed` をこの pane へ送り返すのが
    /// 次波(`crate::waveform_view` モジュール doc「次波の統合手順」節・
    /// このレーンの RETURN 参照)。
    pub fn plan_waveforms(
        &mut self,
        audio_rows: &[AudioRowProjection],
        mut clip_width_px: impl FnMut(LayerId) -> f32,
    ) -> Vec<(LayerId, String, usize)> {
        let mut requests = Vec::new();
        for row in audio_rows {
            if !row.has_audio {
                continue;
            }
            let Some(path) = row.source_path.clone() else {
                continue; // has_audio だが path が無い(起こらないはずだが安全側)。
            };
            let width = clip_width_px(row.layer);
            let state = self.waveforms.entry(row.layer).or_insert(WaveformState::NotRequested);
            if let WaveformAction::Fetch(buckets) = waveform_view::plan(state, width, true) {
                *state = WaveformState::Loading { buckets };
                requests.push((row.layer, path, buckets));
            }
        }
        requests
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

    /// `TimelinePane::with_clip_preview` へ、ドラッグ中の全 layer の
    /// `(layer, preview timing)` をそのまま渡す。`origins`/`preview` は move
    /// なら選択済み全 layer、trim なら掴んだ layer だけを持つため、ここで
    /// 単一 layer へ絞らない。表示側はこの列を投影へ渡して全 bar を毎フレーム
    /// 差し替える。
    pub fn clip_preview(&self) -> Option<Vec<(LayerId, LayerTiming)>> {
        self.drag.as_ref().map(|drag| drag.preview.clone())
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
            Message::SelectLayer { order, op } => {
                let (selected, anchor) = rows::resolve_layer_selection(
                    &order,
                    self.layer_selection_anchor,
                    &session.selected_layers,
                    op,
                );
                self.layer_selection_anchor = anchor;
                session.selection = match selected.as_slice() {
                    [only] => Some(*only),
                    _ => None,
                };
                session.selected_layers = selected;
                None
            }
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
            Message::ReverseSelectedKeys => reverse_selected_keys(doc, session),
            Message::ToggleKeyframeAtPlayhead { layer, property } => {
                toggle_keyframe_at_playhead(doc, session, layer, property)
            }
            Message::CopySelectedKeys => self.copy_selected_keys(doc, session),
            Message::PasteKeys => self.paste_keys(doc, session, false),
            Message::PasteKeysReversed => self.paste_keys(doc, session, true),
            Message::JumpToNextClipEdit => jump_to_clip_edit(doc, session, nav::JumpDirection::Next),
            Message::JumpToPreviousClipEdit => jump_to_clip_edit(doc, session, nav::JumpDirection::Prev),
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

            // ---- Split(B39、`crate::split` モジュール doc「統合手順2」) ----
            Message::SplitAtPlayhead => self.split_at_playhead(doc, session),

            // ---- Timeline 音声波形(TL7 統合手順5) ----
            Message::WaveformFetched { layer, buckets, peaks } => {
                if matches!(
                    self.waveforms.get(&layer),
                    Some(WaveformState::Loading { buckets: current }) if *current == buckets
                ) {
                    self.waveforms.insert(layer, WaveformState::Ready { buckets, peaks });
                }
                // stale(取得中に別のズームで再要求済み)な結果は黙って捨てる —
                // `waveform_view::plan` と同じ「今の要求と一致するかだけ見る」思想。
                None
            }
            Message::WaveformFetchFailed { layer, buckets } => {
                if matches!(
                    self.waveforms.get(&layer),
                    Some(WaveformState::Loading { buckets: current }) if *current == buckets
                ) {
                    self.waveforms.insert(layer, WaveformState::Ready { buckets, peaks: Vec::new() });
                }
                None
            }

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
            | Message::Shuttle(_)
            | Message::AddMarkerAt(_) => None,
        }
    }
}

// ---------------------------------------------------------------------------
// 責任ごとの分割(SP-2、2726行 → 800行以下)。すべて `impl PaneState` の
// 続き(private field アクセスは子モジュールから親の private item を見える
// Rust の可視性規則そのまま — 追加の pub(crate) は「mod.rs から子を呼ぶ」
// 逆方向にのみ要る)。中身は無改変・純粋な移送。
// ---------------------------------------------------------------------------
mod clip_drag;
mod key_drag;
mod keys;
mod loop_work_area;
mod misc;
#[cfg(test)]
mod tests;
