use egui::{
    Align2, Color32, FontId, Pos2, Vec2,
    epaint::{Mesh, Vertex as EguiVertex},
};
use crate::AppStageTransformEdit;
use transform_gizmo::{
    GizmoConfig, GizmoInteraction, GizmoMode, GizmoOrientation, GizmoResult,
    math::{DMat4, DQuat, DVec3, Pos2 as GizmoPos2, Rect as GizmoRect, Transform},
};

use super::adapter::{EmbeddedSpatialStage, StageGizmoAction};

impl EmbeddedSpatialStage {
    pub(super) fn show_transform_gizmo(&mut self, ui: &egui::Ui) {
        let cursor_pos = self.gizmo_pointer_position;
        let drag_started = std::mem::take(&mut self.gizmo_pointer_pressed);
        let dragging = self.gizmo_pointer_down;
        let released = std::mem::take(&mut self.gizmo_pointer_released);
        let viewport = ui.clip_rect();
        if viewport.width() <= 0.0 || viewport.height() <= 0.0 {
            return;
        }

        let Some((selected_layer, target_transform)) = self.selected_gizmo_transform() else {
            return;
        };

        // なぜ: authoring gizmoもRerun標準cameraが実際に評価したEyeをそのまま使う。
        let Some(eye) = self.spatial_stage.last_eye() else {
            return;
        };
        let eye_translation = eye.world_from_rub_view.translation();
        let eye_rotation = eye.world_from_rub_view.rotation();
        let world_from_view = DMat4::from_rotation_translation(
            DQuat::from_xyzw(
                f64::from(eye_rotation.x),
                f64::from(eye_rotation.y),
                f64::from(eye_rotation.z),
                f64::from(eye_rotation.w),
            ),
            DVec3::new(
                f64::from(eye_translation.x),
                f64::from(eye_translation.y),
                f64::from(eye_translation.z),
            ),
        );
        let view_matrix = world_from_view.inverse();
        let fov_y = f64::from(eye.fov_y.unwrap_or(re_view_spatial::Eye::DEFAULT_FOV_Y));
        let projection_matrix = DMat4::perspective_infinite_rh(
            fov_y,
            f64::from(viewport.width() / viewport.height()),
            0.01,
        );
        self.gizmo.update_config(GizmoConfig {
            view_matrix: view_matrix.into(),
            projection_matrix: projection_matrix.into(),
            viewport: GizmoRect::from_min_max(
                GizmoPos2::new(viewport.min.x, viewport.min.y),
                GizmoPos2::new(viewport.max.x, viewport.max.y),
            ),
            // なぜ: all_* は RotateX/Y/View が None、ScaleZ が XY noop、Uniform が RotateView と排他。
            modes: GizmoMode::TranslateX
                | GizmoMode::TranslateY
                | GizmoMode::TranslateXY
                | GizmoMode::RotateZ
                | GizmoMode::ScaleX
                | GizmoMode::ScaleY
                | GizmoMode::ScaleUniform,
            orientation: GizmoOrientation::Local,
            pixels_per_point: ui.ctx().pixels_per_point(),
            ..Default::default()
        });

        let interaction = GizmoInteraction {
            cursor_pos: (cursor_pos.x, cursor_pos.y),
            hovered: viewport.contains(cursor_pos),
            drag_started,
            dragging,
        };
        if let Some((result, _transforms)) = self.gizmo.update(interaction, &[target_transform]) {
            // 原文は let chain。この crate は edition 2021 なのでネストへ開いた（意味は同じ）。
            if let Some(edit) = stage_transform_edit(result, DQuat::from(target_transform.rotation))
            {
                if !stage_transform_edit_is_noop(edit) {
                    self.gizmo_gesture = Some((selected_layer.clone(), edit));
                    self.pending_gizmo_action = Some(StageGizmoAction::Preview {
                        layer_id: selected_layer.clone(),
                        edit,
                    });
                }
            }
        }

        if self.gizmo_cancel_requested {
            self.gizmo_cancel_requested = false;
            if self.gizmo_gesture.take().is_some() {
                self.pending_gizmo_action = Some(StageGizmoAction::Cancel);
            }
        } else if released {
            // 原文は let chain。この crate は edition 2021 なのでネストへ開いた（意味は同じ）。
            if let Some((layer_id, edit)) = self.gizmo_gesture.take() {
                self.pending_gizmo_action = Some(StageGizmoAction::Commit { layer_id, edit });
            }
        }

        let draw_data = self.gizmo.draw();
        ui.painter().add(Mesh {
            indices: draw_data.indices,
            vertices: draw_data
                .vertices
                .into_iter()
                .zip(draw_data.colors)
                .map(|(position, [r, g, b, a])| EguiVertex {
                    pos: Pos2::new(position[0], position[1]),
                    uv: Pos2::ZERO,
                    color: egui::Rgba::from_rgba_premultiplied(r, g, b, a).into(),
                })
                .collect(),
            ..Default::default()
        });
    }

    pub(super) fn selected_gizmo_transform(&self) -> Option<(String, Transform)> {
        let layer_id = self.host_primary_layer_id.as_ref()?;
        let layer = self
            .host_geometry
            .as_ref()?
            .layers
            .iter()
            .find(|layer| &layer.layer_id == layer_id)?;
        if layer.scale[0].abs() <= f64::EPSILON || layer.scale[1].abs() <= f64::EPSILON {
            return None;
        }
        Some((
            layer_id.clone(),
            Transform::from_scale_rotation_translation(
                DVec3::new(layer.scale[0], layer.scale[1], 1.0),
                DQuat::from_rotation_z(layer.rotation),
                DVec3::new(layer.position[0], layer.position[1], 0.0),
            ),
        ))
    }

    pub(super) fn show_feedback(&self, ui: &egui::Ui) {
        let Some((message, rejected)) = &self.feedback else {
            return;
        };
        let color = if *rejected {
            Color32::from_rgb(255, 145, 145)
        } else {
            Color32::from_rgb(170, 240, 196)
        };
        ui.painter().text(
            ui.clip_rect().left_top() + Vec2::new(12.0, 12.0),
            Align2::LEFT_TOP,
            message,
            FontId::proportional(13.0),
            color,
        );
    }
}

pub(super) fn stage_transform_edit(
    result: GizmoResult,
    local_rotation: DQuat,
) -> Option<AppStageTransformEdit> {
    match result {
        GizmoResult::Translation { total, .. } => {
            // なぜ: Local の total はローカル。host の TranslateWorld は world delta を要求する。
            let world = local_rotation * DVec3::from(total);
            Some(AppStageTransformEdit::TranslateWorld([world.x, world.y]))
        }
        GizmoResult::Rotation { total, axis, .. } if axis.z.abs() >= 0.5 => {
            // なぜ: gizmo の applied は from_axis_angle(axis, -total)。Document +Z へ合わせる。
            Some(AppStageTransformEdit::RotateZ(-total * axis.z.signum()))
        }
        GizmoResult::Scale { total } => Some(AppStageTransformEdit::Scale([total.x, total.y])),
        GizmoResult::Rotation { .. } | GizmoResult::Arcball { .. } => None,
    }
}

pub(super) fn stage_transform_edit_is_noop(edit: AppStageTransformEdit) -> bool {
    match edit {
        AppStageTransformEdit::TranslateWorld(delta) => {
            delta[0].abs() <= f64::EPSILON && delta[1].abs() <= f64::EPSILON
        }
        AppStageTransformEdit::RotateZ(delta) => delta.abs() <= f64::EPSILON,
        AppStageTransformEdit::Scale(scale) => {
            (scale[0] - 1.0).abs() <= f64::EPSILON && (scale[1] - 1.0).abs() <= f64::EPSILON
        }
    }
}
