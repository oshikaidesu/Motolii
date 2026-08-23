//! Timeline キーフレームの時刻ドラッグ/リタイム・NudgeKeyframe・その共通書き口
//! `commit_key_frames`(SP-2 分割、`write.rs` 1092-1232行・1711-1812行を移設)。
//! **中身は無改変**。

use super::*;
use super::keys::apply_key_selection;
use super::misc::{comp_duration, composition};

impl PaneState {
    // ---- Timeline キーの時刻編集(第2波T4、正典 §3・§8.1・裁定146) ----

    /// キー菱形を掴んだ瞬間。**ロック中は掴む前に断る**。move は**未選択なら
    /// 掴んだ瞬間に単独選択へ差し替え**、既に選択済みのキーを掴んだ場合は
    /// 選択(複数)を保つ(一括ドラッグを壊さない)。retime は選択2本以上・
    /// 掴んだキーがその端であることを呼び出し元が既に確認済み — ここでは
    /// 安全側にもう一度検分するだけ。
    pub(crate) fn start_key_drag(
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
    pub(crate) fn continue_key_drag(
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
    pub(crate) fn finish_key_drag(&mut self, doc: &mut Document, session: &mut Session) -> Option<String> {
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

/// NudgeKeyframe(正典 §8.1): 選択キーを固定 frame 数だけ前後へ。選択が
/// 空、または全キーが同一 layer である保証が崩れていれば何もしない。
pub(crate) fn nudge_keyframe(doc: &mut Document, session: &mut Session, delta: i64) -> Option<String> {
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
pub(crate) fn commit_key_frames(
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
