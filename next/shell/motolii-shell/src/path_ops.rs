//! Stage path edit の Shell 側 writer。
//!
//! `motolii-stage-pane::path_edit` が返す index と layer-local 座標を、既存の
//! `ShapeNode::Leaf(PathSource::Bezier)`へ戻し、`Document::apply(Intent::SetShapes)`
//! 一回で確定する。pane や Shell に別の shape state owner は作らない。

use motolii_store::{Intent, LayerId, PathSource, ShapeNode};
use motolii_vector::{edit, Point};

use crate::{stage, Shell};

/* motolii-component
id = "shell.path_edit_writer"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["update_path_edit", "commit_path_change"]
meaning = ["MoveVertex", "ClosePath", "OpenPath"]
evaluation = ["with_contour", "move_vertex", "close_path", "open_path"]
render = ["SetShapes"]
observable = ["path_vertex_drag_changes_shape"]
*/

#[derive(Clone, Copy)]
enum PathChange {
    Move { point: [f64; 2] },
    Close,
    Open,
}

impl Shell {
    pub(crate) fn update_path_edit(&mut self, message: stage::PathEditMessage) {
        match message {
            stage::PathEditMessage::MoveVertex {
                layer,
                target,
                point,
            } => self.commit_path_change(
                layer,
                target.shape_index,
                target.contour_index,
                target.vertex_index,
                PathChange::Move { point },
            ),
            stage::PathEditMessage::ClosePath {
                layer,
                shape_index,
                contour_index,
            } => self.commit_path_change(layer, shape_index, contour_index, 0, PathChange::Close),
            stage::PathEditMessage::OpenPath {
                layer,
                shape_index,
                contour_index,
            } => self.commit_path_change(layer, shape_index, contour_index, 0, PathChange::Open),
            stage::PathEditMessage::Cancel => {}
        }
    }

    fn commit_path_change(
        &mut self,
        layer: LayerId,
        shape_index: usize,
        contour_index: usize,
        vertex_index: usize,
        change: PathChange,
    ) {
        let Ok(mut shapes) = self.doc.view().shapes(layer) else {
            self.status = Some("パスを読み込めません".to_owned());
            return;
        };
        let Some(ShapeNode::Leaf(shape)) = shapes.get_mut(shape_index) else {
            self.status = Some("編集対象の shape が見つかりません".to_owned());
            return;
        };
        let PathSource::Bezier(path) = &shape.source else {
            self.status = Some("プリミティブ shape の頂点はまだ編集できません".to_owned());
            return;
        };
        let result = match change {
            PathChange::Move { point } => {
                if !point.iter().all(|value| value.is_finite()) {
                    self.status = Some("頂点の座標が不正です".to_owned());
                    return;
                }
                edit::with_contour(path, contour_index, |contour| {
                    edit::move_vertex(
                        contour,
                        vertex_index,
                        Point {
                            x: point[0],
                            y: point[1],
                        },
                    )
                })
            }
            PathChange::Close => edit::with_contour(path, contour_index, |contour| {
                Ok(edit::close_path(contour))
            }),
            PathChange::Open => edit::with_contour(path, contour_index, |contour| {
                Ok(edit::open_path(contour))
            }),
        };
        let Ok(updated_path) = result else {
            self.status = Some("パスの編集対象が見つかりません".to_owned());
            return;
        };
        shape.source = PathSource::Bezier(updated_path);
        if let Err(error) = self.doc.apply(Intent::SetShapes { layer, shapes }) {
            self.status = Some(format!("パスを編集できません: {error}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_store::{LayerId, Shape, VectorPoint};
    use motolii_vector::Contour;

    #[test]
    fn path_vertex_drag_changes_shape() {
        let (mut shell, _) = Shell::new_fixture();
        let layer = LayerId(shell.next_layer_id());
        shell
            .doc
            .apply_all([
                Intent::AddLayer(layer),
                Intent::SetMeta {
                    layer,
                    meta: motolii_store::LayerMeta {
                        source: motolii_store::LayerSource::Shape,
                        order: layer.0 as i16,
                        timing: motolii_store::LayerTiming::place(0, None, 300),
                    },
                },
                Intent::SetShapes {
                    layer,
                        shapes: vec![ShapeNode::Leaf(Shape::new(PathSource::Bezier(vec![
                        Contour::open([
                            VectorPoint { x: 10.0, y: 10.0 },
                            VectorPoint { x: 40.0, y: 10.0 },
                        ]),
                    ])))],
                },
            ])
            .expect("path fixture");
        shell.select_single(layer);

        shell.update_path_edit(stage::PathEditMessage::MoveVertex {
            layer,
            target: stage::VertexRef {
                shape_index: 0,
                contour_index: 0,
                vertex_index: 1,
            },
            point: [75.0, 25.0],
        });

        let shapes = shell.doc.view().shapes(layer).expect("path read");
        let ShapeNode::Leaf(shape) = &shapes[0] else {
            panic!("expected leaf")
        };
        let PathSource::Bezier(path) = &shape.source else {
            panic!("expected path")
        };
        assert_eq!(path[0].vertices[1].point.x, 75.0);
        assert_eq!(path[0].vertices[1].point.y, 25.0);
    }

    #[test]
    fn path_close_and_open_are_one_document_edit_each() {
        let (mut shell, _) = Shell::new_fixture();
        let layer = LayerId(shell.next_layer_id());
        shell
            .doc
            .apply_all([
                Intent::AddLayer(layer),
                Intent::SetMeta {
                    layer,
                    meta: motolii_store::LayerMeta {
                        source: motolii_store::LayerSource::Shape,
                        order: layer.0 as i16,
                        timing: motolii_store::LayerTiming::place(0, None, 300),
                    },
                },
                Intent::SetShapes {
                    layer,
                        shapes: vec![ShapeNode::Leaf(Shape::new(PathSource::Bezier(vec![
                        Contour::open([
                            VectorPoint { x: 0.0, y: 0.0 },
                            VectorPoint { x: 10.0, y: 0.0 },
                        ]),
                    ])))],
                },
            ])
            .expect("path fixture");
        shell.select_single(layer);
        shell.update_path_edit(stage::PathEditMessage::ClosePath {
            layer,
            shape_index: 0,
            contour_index: 0,
        });
        assert!(matches!(shell.doc.view().shapes(layer).unwrap()[0], ShapeNode::Leaf(_)));
        let ShapeNode::Leaf(shape) = &shell.doc.view().shapes(layer).unwrap()[0] else {
            unreachable!()
        };
        let PathSource::Bezier(path) = &shape.source else {
            unreachable!()
        };
        assert!(path[0].closed);
        shell.update_path_edit(stage::PathEditMessage::OpenPath {
            layer,
            shape_index: 0,
            contour_index: 0,
        });
        let ShapeNode::Leaf(shape) = &shell.doc.view().shapes(layer).unwrap()[0] else {
            unreachable!()
        };
        let PathSource::Bezier(path) = &shape.source else {
            unreachable!()
        };
        assert!(!path[0].closed);
    }
}
