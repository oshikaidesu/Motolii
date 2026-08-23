//! Timeline クリップの move/trim ドラッグ(SP-2 分割、`write.rs` 908-1090行を
//! 移設)。**中身は無改変**。

use super::*;
use super::misc::comp_duration;

impl PaneState {
    // ---- Timeline クリップの move/trim(第2波T2、正典 §2) ----

    /// bar を掴んだ瞬間。**ロック中は掴む前に断る**(正典 §2 拘束6・M13: 無反応
    /// ゼロ)。move(`Body`)は**掴んだ layer が既に選択集合(複数)に居れば
    /// その選択を保つ、居なければ掴んだ瞬間に単独選択へ差し替える**
    /// (`start_key_drag` の「既に選択済みのキーを掴んだ場合は選択を保つ」と
    /// 同じ形 — 軸台帳 A08「Timeline clip move/trim」単数のみの穴を閉じる、
    /// E-2 RETURN 参照)。trim(`Edge*`)は選択を変えない・掴んだ1本だけを
    /// 対象にする(複数 clip を跨いだ trim は意味が無い — 正典 §2「単独 clip
    /// の start/end のみ動かす」)。
    ///
    /// `session.selection`/`selected_layers` を直接更新する(旧
    /// `Shell::select_single` のロジックをここへ複製 — pane crate は Shell の
    /// private メソッドを呼べないため。RETURN の finding 参照)。
    pub(crate) fn start_drag(
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
        let origins: Vec<(LayerId, LayerTiming)> = if matches!(part, BarPart::Body) {
            let already_multi = session.selected_layers.len() > 1 && session.selected_layers.contains(&layer);
            if !already_multi {
                session.selection = Some(layer);
                session.selected_layers = vec![layer];
            }
            // 選択集合のうち素材があり・ロックされていない layer だけを一括
            // move の対象にする(ロック済みの他 layer は黙って対象外 —
            // 掴んだ layer 自身は上で既にロック検分済み、`toggle_selected_
            // layers_attr` 系と同じ「混じっていても動く分だけ動く」姿勢)。
            let mut targets: Vec<(LayerId, LayerTiming)> = session
                .selected_layers
                .iter()
                .filter_map(|&id| {
                    let m = doc.view().meta(id).ok().flatten()?;
                    let locked = doc.view().attrs(id).ok().flatten().unwrap_or_default().locked;
                    if locked {
                        return None;
                    }
                    Some((id, m.timing))
                })
                .collect();
            if !targets.iter().any(|(id, _)| *id == layer) {
                targets.push((layer, meta.timing));
            }
            targets
        } else {
            vec![(layer, meta.timing)]
        };
        self.drag = Some(TimelineDragState {
            layer,
            part,
            origins: origins.clone(),
            grab_at_frame: at_frame,
            preview: origins,
        });
        None
    }

    /// ドラッグ中のポインタ移動。**掴んだ瞬間の値(`origins`)を基準に絶対値で
    /// 出し直す**(delta 蓄積禁止、正典 §2)。**Document はまだ一切触らない**
    /// (`preview` は transient な一時値、release まで `apply` しない)。
    /// **複数 layer の move は同じ delta を全員へ適用して相対間隔を保つ**
    /// (`continue_key_drag`/`key_gesture::moved_key_group` と同じ「塊制約」の
    /// 形 — 個別 clamp すると相対間隔が壊れる、`key_gesture::clamp_group_delta`
    /// をそのまま流用する)。trim(`Edge*`)は `origins` が掴んだ1本だけなので
    /// 従来どおり単体の計算。
    pub(crate) fn continue_drag(
        &mut self,
        doc: &mut Document,
        session: &Session,
        at_frame: i64,
        px_per_frame: f32,
        modifiers: iced::keyboard::Modifiers,
    ) {
        let Some(drag) = self.drag.clone() else {
            return;
        };
        let Some(&(_, grabbed_origin)) = drag.origins.iter().find(|(id, _)| *id == drag.layer) else {
            return; // 起こらないはずだが安全側(grabbed は必ず origins に居る)。
        };
        let comp_duration = comp_duration(doc);
        let snap_enabled = !modifiers.command();
        let timeline_rows = rows(&doc.view(), session);
        let candidates =
            clip_gesture::snap_candidates(&timeline_rows, drag.layer, session.playhead, comp_duration);

        let preview: Vec<(LayerId, LayerTiming)> = match drag.part {
            BarPart::Body => {
                let raw_target = grabbed_origin.start + (at_frame - drag.grab_at_frame);
                let snapped_target = if snap_enabled {
                    clip_gesture::snap_frame(raw_target, &candidates, px_per_frame)
                } else {
                    raw_target
                };
                let mut delta = snapped_target - grabbed_origin.start;
                // 塊制約: 集合の誰かが comp の外へ出そうになったら、**全員が
                // そこで止まる delta へ丸め直す**(個別 clamp では相対間隔が
                // 壊れる、`key_gesture::clamp_group_delta` doc 参照)。
                let min_start = drag.origins.iter().map(|(_, t)| t.start).min().unwrap_or(grabbed_origin.start);
                let max_end = drag
                    .origins
                    .iter()
                    .map(|(_, t)| t.start + t.duration)
                    .max()
                    .unwrap_or(grabbed_origin.start + grabbed_origin.duration);
                delta = key_gesture::clamp_group_delta(min_start, max_end, delta, 0, comp_duration);
                drag.origins
                    .iter()
                    .map(|&(id, timing)| {
                        let mut next = timing;
                        next.start = timing.start + delta;
                        (id, next)
                    })
                    .collect()
            }
            BarPart::EdgeIn => {
                let end = grabbed_origin.start + grabbed_origin.duration;
                let new_start =
                    clip_gesture::trimmed_in_start(end, at_frame, &candidates, px_per_frame, snap_enabled);
                let delta = new_start - grabbed_origin.start;
                let mut timing = grabbed_origin;
                timing.start = new_start;
                timing.duration = grabbed_origin.duration - delta;
                timing.source_in = grabbed_origin.source_in + delta;
                vec![(drag.layer, timing)]
            }
            BarPart::EdgeOut => {
                let new_end = clip_gesture::trimmed_out_end(
                    grabbed_origin.start,
                    at_frame,
                    comp_duration,
                    &candidates,
                    px_per_frame,
                    snap_enabled,
                );
                let mut timing = grabbed_origin;
                timing.duration = new_end - grabbed_origin.start;
                vec![(drag.layer, timing)]
            }
        };

        if let Some(drag) = self.drag.as_mut() {
            drag.preview = preview;
        }
    }

    /// release = 確定。**掴んだだけで未移動なら no-op**。動いた layer だけ
    /// `Intent::SetTiming` を束ね、1回の `apply_all` で確定する(**1 gesture =
    /// 1 undo** — `finish_key_drag`/`commit_key_frames` と同じ形。N 本動かして
    /// も undo は1回)。
    pub(crate) fn finish_drag(&mut self, doc: &mut Document) -> Option<String> {
        let drag = self.drag.take()?;
        if drag.preview == drag.origins {
            return None;
        }
        let intents: Vec<Intent> = drag
            .origins
            .iter()
            .zip(drag.preview.iter())
            .filter(|((_, origin), (_, preview))| origin != preview)
            .map(|(_, &(layer, timing))| Intent::SetTiming { layer, timing })
            .collect();
        if intents.is_empty() {
            return None;
        }
        if let Err(error) = doc.apply_all(intents) {
            return Some(format!("timing を書けない: {error}"));
        }
        None
    }
}
