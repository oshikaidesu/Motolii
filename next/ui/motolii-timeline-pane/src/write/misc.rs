//! Split・inline rename・playhead の意味点ジャンプ・Stage 重なりの並べ替え
//! (SP-2 分割、`write.rs` 836-906行・1657-1707行を移設)。**中身は無改変**。

use super::*;

// ---------------------------------------------------------------------------
// self を持たない書き口(drag state を触らない = PaneState のフィールドが要らない)。
// ---------------------------------------------------------------------------

pub(crate) fn composition(doc: &Document) -> Option<Composition> {
    doc.view().composition().ok().flatten()
}

pub(crate) fn comp_duration(doc: &Document) -> i64 {
    composition(doc).map(|c| c.duration_frames).unwrap_or(0)
}

impl PaneState {
    // ---- Split(レイヤー分割、B39 — `crate::split` モジュール doc「統合手順2」) ----

    /// `Message::SplitAtPlayhead` の実処理。`session.selected_layers`(複数選択)が
    /// 非空ならそれを、空なら `session.selection`(単一選択)を対象に、
    /// [`split::split_selected_plan`] で playhead(`session.playhead`)で割れる
    /// 分だけ割る Intent 列を組み、1回の `doc.apply_all(...)` で確定する
    /// (**1操作 = 1 undo**、`split::split_selected_plan` の doc「設計判断」参照 —
    /// 割れない layer は黙って skip、1つも割れなければ理由つき `Err`)。
    /// 選択そのものが空なら、`split_selected_plan` を呼ぶ前に理由つき拒否
    /// (M13 — `restack_layers`/`set_work_area_to_selection` と同じ柵)。
    pub(crate) fn split_at_playhead(&mut self, doc: &mut Document, session: &Session) -> Option<String> {
        let layers: Vec<LayerId> = if session.selected_layers.is_empty() {
            session.selection.into_iter().collect()
        } else {
            session.selected_layers.clone()
        };
        if layers.is_empty() {
            return Some("選択が無いので分割できない — layer を選んでから".into());
        }
        match split::split_selected_plan(&doc.view(), &layers, session.playhead) {
            Ok(intents) => {
                if let Err(error) = doc.apply_all(intents) {
                    return Some(format!("分割を書けない: {error}"));
                }
                None
            }
            Err(reason) => Some(reason),
        }
    }

    // ---- inline rename(第3切片、正典 §6「リネーム」) ----

    /// rename 開始。現在名を下書きへ写す(名前が空の layer は空文字から
    /// 編集を始める — rail 側の placeholder `layer {id}` が顔を担う)。
    /// 消えた layer は黙って無視(stale な入口)。ロック層は理由つき拒否(M13、
    /// `start_drag` と同じ柵)。
    pub(crate) fn begin_rename(&mut self, doc: &Document, layer: LayerId) -> Option<String> {
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
    pub(crate) fn commit_rename(&mut self, doc: &mut Document) -> Option<String> {
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
}

/// Next/Previous Clip/Edit(map 1088/1089・1108/1109、
/// [`keys2::clip_edit_points`]+[`nav::nearest_meaning_point`] の結線)。表示中の
/// 全 clip の start/end を意味点として playhead を渡す — 渡る先が無ければ
/// no-op(`nav` モジュール doc「呼び出し側は playhead を動かさない」)。選択・
/// キーは動かさない(playhead だけを動かす、AE/Premiere 同型)。
pub(crate) fn jump_to_clip_edit(doc: &Document, session: &mut Session, direction: nav::JumpDirection) -> Option<String> {
    let timeline_rows = rows(&doc.view(), session);
    let points = keys2::clip_edit_points(&timeline_rows);
    if let Some(frame) = nav::nearest_meaning_point(&points, session.playhead, direction) {
        session.playhead = frame;
    }
    None
}

/// Stage 重なりの並べ替え(第3切片、map B44 184/292/293 + 正典 §8.1
/// ReorderLayerUp/Down(+ToEnd))。意味計算は [`stacking::restacked`](純関数)、
/// ここは選択の解決・ロック検分・`Intent::SetOrder` の束ね(1回の `apply_all`
/// = **1操作 = 1 undo**)だけ。端で動けない no-op は Intent を出さない
/// (`restacked` が空を返す — 空 undo を作らない)。
pub(crate) fn restack_layers(doc: &mut Document, session: &Session, direction: StackDirection) -> Option<String> {
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
