//! Stage mask path edit の Shell 側 writer。
//!
//! pane が返す layer-local 座標を `mask.{id}.shape` の `Value::Path` track へ
//! playhead のキーとして戻す。形状の評価は store、キーの upsert は既存の
//! Inspector helper に委譲し、mask 専用の時間軸機構は作らない。

use motolii_store::{Intent, LayerId, MaskId, Path, PathVertex, PropertyId, Value};

use crate::{stage, Shell};

/* motolii-component
id = "shell.mask_path_edit_writer"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["update_mask_path_edit", "commit_mask_vertex_change"]
meaning = ["MoveVertex"]
evaluation = ["move_mask_vertex", "edited_value_track"]
render = ["SetTrack"]
observable = ["mask_vertex_drag_changes_shape"]
*/

pub(crate) fn move_mask_vertex(
    path: &mut Path,
    vertex_index: usize,
    point: [f64; 2],
) -> Result<(), String> {
    if !point.iter().all(|value| value.is_finite()) {
        return Err("マスク頂点の座標が不正です".to_owned());
    }
    let Some(vertex) = path.vertices.get_mut(vertex_index) else {
        return Err("編集対象のマスク頂点が見つかりません".to_owned());
    };
    *vertex = PathVertex {
        point,
        in_tangent: vertex.in_tangent,
        out_tangent: vertex.out_tangent,
    };
    Ok(())
}

impl Shell {
    pub(crate) fn update_mask_path_edit(&mut self, message: stage::MaskPathEditMessage) {
        match message {
            stage::MaskPathEditMessage::MoveVertex {
                layer,
                target,
                point,
            } => self.commit_mask_vertex_change(layer, target.mask, target.vertex_index, point),
            stage::MaskPathEditMessage::Cancel => {}
        }
    }

    fn commit_mask_vertex_change(
        &mut self,
        layer: LayerId,
        mask: MaskId,
        vertex_index: usize,
        point: [f64; 2],
    ) {
        let Some(composition) = self.composition() else {
            self.status = Some("comp が無いためマスクを編集できません".to_owned());
            return;
        };
        let Some(time) = self.time_at_playhead() else {
            self.status = Some("playhead を時刻へ変換できません".to_owned());
            return;
        };
        let property = PropertyId::mask_shape(mask);
        let store = self.doc.view();
        let current = match store.value_at(layer, &property, time) {
            Ok(Some(Value::Path(path))) => path,
            Ok(Some(other)) => {
                self.status = Some(format!("マスク形状が Path ではありません: {other:?}"));
                return;
            }
            Ok(None) => {
                self.status = Some("マスク形状の track がありません".to_owned());
                return;
            }
            Err(error) => {
                self.status = Some(format!("マスク形状を読めません: {error}"));
                return;
            }
        };
        let mut next = current;
        if let Err(error) = move_mask_vertex(&mut next, vertex_index, point) {
            self.status = Some(error);
            return;
        }
        let existing = match store.track(layer, &property) {
            Ok(track) => track,
            Err(error) => {
                self.status = Some(format!("マスク形状 track を読めません: {error}"));
                return;
            }
        };
        let track = match crate::inspector_pane::edited_value_track(
            existing.as_ref(),
            self.session.playhead,
            composition.fps,
            Value::Path(next),
        ) {
            Ok(track) => track,
            Err(error) => {
                self.status = Some(error);
                return;
            }
        };
        if let Err(error) = self.doc.apply(Intent::SetTrack {
            layer,
            property,
            track,
        }) {
            self.status = Some(format!("マスク形状を書けません: {error}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path() -> Path {
        Path {
            vertices: vec![
                PathVertex {
                    point: [1.0, 2.0],
                    in_tangent: [-1.0, 0.0],
                    out_tangent: [1.0, 0.0],
                },
                PathVertex {
                    point: [3.0, 4.0],
                    in_tangent: [-1.0, 0.0],
                    out_tangent: [1.0, 0.0],
                },
            ],
            closed: true,
        }
    }

    #[test]
    fn mask_vertex_drag_changes_shape() {
        let mut path = path();
        move_mask_vertex(&mut path, 1, [30.0, 40.0]).expect("vertex");
        assert_eq!(path.vertices[1].point, [30.0, 40.0]);
        assert_eq!(path.vertices[1].in_tangent, [-1.0, 0.0]);
        assert_eq!(path.vertices[1].out_tangent, [1.0, 0.0]);
    }
}
