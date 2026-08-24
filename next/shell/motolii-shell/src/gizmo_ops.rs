//! Stage設定とGizmo gestureの意味更新。
//! render pipelineとは別ownerにし、transient/commit/cancelの状態機械をここへ閉じる。

use motolii_store::{Fps, Intent, LayerId, PropertyId, Value};

use crate::{inspector_pane, stage, Shell};
/// Stage ギズモ drag、shell 側の transient(GZ 結線、第5波)。**Document では
/// ない** — Inspector の `FieldDragState` と同じ「確定まで front だけが持つ」
/// 身分。ギズモの座標解(`stage::GizmoDragState`)は canvas 内部に住み、shell は
/// 「どの layer のどの property を書いているか」と、確定のキー upsert の宛先
/// (Start 時点の playhead/fps — Inspector drag と同じ press 時点固定)だけを
/// 持つ。
pub(crate) struct GizmoShellDrag {
    pub(crate) layer: LayerId,
    /// Start が申告した property(Esc 連鎖 [`Shell::cancel_gizmo_drag`] の
    /// transient 掃除の宛先。Move/Commit は値側 [`stage::GizmoValue::property`]
    /// を読む — 契約上 1 drag = 1 property なので同じ値)。
    pub(crate) property: stage::GizmoProperty,
    /// Start 時点の playhead(frame)と fps。確定のキー upsert
    /// (`inspector_pane::edited_value_track`)の宛先 — drag の起点値は Start
    /// 時点の絵から読まれているので、確定の宛先も同じ時刻に固定する
    /// (`inspector_pane::FieldDragState::playhead_frame` と同じ判断)。
    pub(crate) playhead_frame: i64,
    pub(crate) fps: Fps,
    /// 1回でも `set_transient` を書いたか(Cancel 時に overlay を外す要否)。
    pub(crate) moved: bool,
}

/// [`stage::GizmoValue`](store の単位そのまま — `gizmo.rs` doc)→ store の
/// [`Value`]。shell 側は写すだけ(GZ 契約「shell 側は `Value::Vec2`/`Value::F64`
/// へ写すだけ」そのもの)。**`Anchor` はここへは来ない** — anchor drag は
/// anchor と position の2 property を対で書く必要があるため、
/// [`Shell::update_gizmo`] が `GizmoValue::Anchor { .. }` を専用の分岐で
/// 個別に処理する(`gizmo.rs::GizmoValue::Anchor` doc「shell は両方を同時に
/// 書く」参照)。
fn gizmo_store_value(value: stage::GizmoValue) -> Value {
    match value {
        stage::GizmoValue::Position(v) | stage::GizmoValue::Scale(v) => Value::Vec2(v),
        stage::GizmoValue::Rotation(v) => Value::F64(v),
        stage::GizmoValue::Anchor { .. } => {
            unreachable!("Anchor は update_gizmo が個別分岐で処理する — ここへは来ない")
        }
    }
}

impl Shell {
    /// pane ローカル `Message` を畳んで書き口へ渡す glue(`update_settings` と
    /// 同じ形)。**最初の2腕は元々 `self.observation` への直代入だけ**(計算を
    /// 持たない)だったので、pane crate 側には移していない。`CycleResolutionCap`/
    /// `ToggleCheckerboard`(裁定163 Stage 下縁状態帯)も同型の直代入 —
    /// `ToggleCheckerboard` は旧 `settings_pane::Message::ToggleCheckerboard`
    /// と同じ本体(`self.checkerboard` の反転)をここへ引っ越しただけ
    /// (`update_settings` 側の対応する腕は削除済み)。
    pub(crate) fn update_stage(&mut self, message: stage::Message) {
        match message {
            stage::Message::Observe(camera) => self.observation = Some(camera),
            stage::Message::ResetToRenderCamera => self.observation = None,
            stage::Message::CycleResolutionCap => {
                self.resolution_cap = self.resolution_cap.next();
            }
            stage::Message::ToggleCheckerboard => {
                self.checkerboard = !self.checkerboard;
            }
        }
    }

    /// ギズモ drag の契約(`stage::GizmoDrag` doc: 1 drag = Start → Move* →
    /// Commit|Cancel)を Inspector の drag-to-scrub と同じ経路へ写す:
    /// - Start: shell 側 transient([`GizmoShellDrag`])を立てるだけ(Document
    ///   は触らない)。宛先時刻(playhead/fps)はこの時点で凍結。
    /// - Move: `Document::set_transient`(edit timeline に触れない overlay —
    ///   undo/redo の意味論は drag 中ずっと不変)。
    /// - Commit: transient を外し、`Intent::SetTrack` を**1回**だけ出す
    ///   (1 drag = 1 undo)。track の意味は値セル編集と同じ
    ///   [`inspector_pane::edited_value_track`](キー無し=静的書き換え・
    ///   キー持ち= playhead へのキー upsert、AE 作法)。
    /// - Cancel: transient を外すだけ(Esc・空クリック)。
    pub(crate) fn update_gizmo(&mut self, event: stage::GizmoDrag) {
        match event.phase {
            stage::GizmoPhase::Start { property } => {
                let Ok(Some(composition)) = self.doc.view().composition() else {
                    return;
                };
                self.gizmo_drag = Some(GizmoShellDrag {
                    layer: event.layer,
                    property,
                    playhead_frame: self.session.playhead,
                    fps: composition.fps,
                    moved: false,
                });
            }
            stage::GizmoPhase::Move { value } => {
                let Some(drag) = self.gizmo_drag.as_mut() else {
                    return;
                };
                let layer = drag.layer;
                drag.moved = true;
                match value {
                    // 第6波(anchor drag pairing): anchor と補償済み position を
                    // 対で transient へ書く(`GizmoValue::Anchor` doc「shell は
                    // 両方を同時に書く」— 片方だけ書くと絵が跳ぶ)。
                    stage::GizmoValue::Anchor { anchor, position } => {
                        if let Ok(anchor_property) =
                            PropertyId::new(stage::GizmoProperty::Anchor.property_name())
                        {
                            self.doc
                                .set_transient(layer, anchor_property, Value::Vec2(anchor));
                        }
                        if let Ok(position_property) =
                            PropertyId::new(stage::GizmoProperty::Position.property_name())
                        {
                            self.doc
                                .set_transient(layer, position_property, Value::Vec2(position));
                        }
                    }
                    other => {
                        let Ok(property) = PropertyId::new(other.property().property_name()) else {
                            return;
                        };
                        self.doc
                            .set_transient(layer, property, gizmo_store_value(other));
                    }
                }
            }
            stage::GizmoPhase::Commit { value } => {
                let Some(drag) = self.gizmo_drag.take() else {
                    return;
                };
                match value {
                    // anchor drag の確定: 2 property(anchor/position)を
                    // `Document::apply_all` で**1 undo**へ束ねる(1 gesture =
                    // 1 commit の契約は変わらない — `GizmoValue::Anchor` doc)。
                    stage::GizmoValue::Anchor { anchor, position } => {
                        let (Ok(anchor_property), Ok(position_property)) = (
                            PropertyId::new(stage::GizmoProperty::Anchor.property_name()),
                            PropertyId::new(stage::GizmoProperty::Position.property_name()),
                        ) else {
                            return;
                        };
                        let store = self.doc.view();
                        let anchor_base = store.track(drag.layer, &anchor_property).ok().flatten();
                        let position_base =
                            store.track(drag.layer, &position_property).ok().flatten();
                        let mut write_error = None;
                        match (
                            inspector_pane::edited_value_track(
                                anchor_base.as_ref(),
                                drag.playhead_frame,
                                drag.fps,
                                Value::Vec2(anchor),
                            ),
                            inspector_pane::edited_value_track(
                                position_base.as_ref(),
                                drag.playhead_frame,
                                drag.fps,
                                Value::Vec2(position),
                            ),
                        ) {
                            (Ok(anchor_track), Ok(position_track)) => {
                                let intents = [
                                    Intent::SetTrack {
                                        layer: drag.layer,
                                        property: anchor_property.clone(),
                                        track: anchor_track,
                                    },
                                    Intent::SetTrack {
                                        layer: drag.layer,
                                        property: position_property.clone(),
                                        track: position_track,
                                    },
                                ];
                                if let Err(error) = self.doc.apply_all(intents) {
                                    write_error = Some(format!("値を書けない: {error}"));
                                }
                            }
                            (Err(error), _) | (_, Err(error)) => write_error = Some(error),
                        }
                        self.doc.clear_transient(drag.layer, &anchor_property);
                        self.doc.clear_transient(drag.layer, &position_property);
                        if let Some(error) = write_error {
                            self.status = Some(error);
                        }
                    }
                    other => {
                        let Ok(property) = PropertyId::new(other.property().property_name()) else {
                            return;
                        };
                        // transient は `track()` に映らないので、ここで読むのは drag
                        // 開始前の本 track そのもの(`finish_field_drag` と同じ注記)。
                        let base_track =
                            self.doc.view().track(drag.layer, &property).ok().flatten();
                        let mut write_error = None;
                        match inspector_pane::edited_value_track(
                            base_track.as_ref(),
                            drag.playhead_frame,
                            drag.fps,
                            gizmo_store_value(other),
                        ) {
                            Ok(track) => {
                                if let Err(error) = self.doc.apply(Intent::SetTrack {
                                    layer: drag.layer,
                                    property: property.clone(),
                                    track,
                                }) {
                                    write_error = Some(format!("値を書けない: {error}"));
                                }
                            }
                            Err(error) => write_error = Some(error),
                        }
                        // 書き込み失敗時も overlay は必ず外す(`finish_field_drag` と
                        // 同じ — overlay を残さない)。
                        self.doc.clear_transient(drag.layer, &property);
                        if let Some(error) = write_error {
                            self.status = Some(error);
                        }
                    }
                }
            }
            stage::GizmoPhase::Cancel => {
                self.cancel_gizmo_drag();
            }
        }
    }

    /// Esc 連鎖用(clip/key/loop の並び — `Message::EscapePressed` 腕)。
    /// transient overlay は edit timeline に触れていないので、外すだけで復元が
    /// 成立する(`inspector_pane::cancel_field_interaction` と同型)。冪等 —
    /// canvas 側の Esc(`GizmoPhase::Cancel`)と二重に届いても2回目は `false`。
    pub(crate) fn cancel_gizmo_drag(&mut self) -> bool {
        let Some(drag) = self.gizmo_drag.take() else {
            return false;
        };
        if drag.moved {
            // anchor drag は2 property を対で transient へ書いている
            // (`update_gizmo` の `Move` 分岐)ので、cancel も両方外す —
            // 片方だけ残すと絵が跳んだまま止まる。
            if matches!(drag.property, stage::GizmoProperty::Anchor) {
                if let Ok(property) = PropertyId::new(stage::GizmoProperty::Anchor.property_name())
                {
                    self.doc.clear_transient(drag.layer, &property);
                }
                if let Ok(property) =
                    PropertyId::new(stage::GizmoProperty::Position.property_name())
                {
                    self.doc.clear_transient(drag.layer, &property);
                }
            } else if let Ok(property) = PropertyId::new(drag.property.property_name()) {
                self.doc.clear_transient(drag.layer, &property);
            }
        }
        true
    }
}
