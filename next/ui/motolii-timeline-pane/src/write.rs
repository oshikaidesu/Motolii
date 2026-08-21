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

    // ---- Timeline ツリー行(裁定173 H2) ----
    /// rail の fold 三角(開閉ボタン)クリック。**Shell の5例外に含まれない**
    /// ので、`Message::Timeline(other)` の受け皿がそのまま
    /// [`PaneState::update`] へ渡す(shell/src の改修は不要 — mod doc の
    /// 「5腕だけ先取り」節参照)。`layer` が Document に既に存在しない場合も
    /// `TimelineFoldState::toggle` は黙って無視できる(fold 状態は LayerId の
    /// 存在に依存しない Session 側の集合)。
    ToggleFold(LayerId),
}

use std::collections::{BTreeMap, HashSet};

use motolii_store::{Composition, Document, Intent, KeyframeTrack, LayerId, LayerTiming, PropertyId, RationalTime};

use crate::hit::BarPart;
use crate::state::Session;
use crate::{clip_gesture, key_gesture, key_order, property_rows, rows, KeySelectionOp, KeySelector};

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

/// Shell が持つ、Timeline pane 専用の transient 状態(旧 `Shell::timeline_drag`/
/// `timeline_key_drag` の2フィールドをまとめた形)。**Document ではない**
/// (`TimelineDragState`/`TimelineKeyDragState` の doc comment 参照)。
#[derive(Default)]
pub struct PaneState {
    drag: Option<TimelineDragState>,
    key_drag: Option<TimelineKeyDragState>,
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

    /// `TimelinePane::with_key_drag_active` へそのまま渡す読み取り専用フラグ。
    pub fn key_drag_active(&self) -> bool {
        self.key_drag.is_some()
    }

    /// clip drag/keyドラッグのどちらかが進行中か。実時間再生(A2、正典 §2
    /// 拘束5「再生と掴みは相互排他: ドラッグ中に Space は効かない」)が
    /// `Shell::toggle_playback` から読む。
    pub fn is_dragging(&self) -> bool {
        self.drag.is_some() || self.key_drag.is_some()
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
            Message::Select(_)
            | Message::ScrubTo(_)
            | Message::ToggleMute(_)
            | Message::ToggleSolo(_)
            | Message::ToggleLock(_) => None,
        }
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
