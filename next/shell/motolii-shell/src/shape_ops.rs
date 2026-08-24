//! Stage shape tool の意味実装。
//!
//! `motolii-stage-pane` は screen 座標を返すだけ。この module がその座標を
//! `motolii-vector` の path へ写し、layer の追加・shape の中身・選択を一つの
//! `Document::apply_all` に束ねる。既存の Browser create と同じ write route で、
//! shape tool 専用の第2 state owner は作らない。

use motolii_store::{
    Intent, LayerAttrsPatch, LayerId, LayerMeta, LayerSource, LayerTiming, ShapeNode,
};
use motolii_vector::{Contour, PathSource, Point, Shape, Stroke};

use crate::{stage, Shell};

/* motolii-component
id = "shell.shape_tool_writer"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["ShapeTool", "create_drawn_shape"]
meaning = ["Create", "CreatePen"]
evaluation = ["primitive_path", "pen_path"]
render = ["SetShapes"]
observable = ["drawn_shape_adds_selected_layer"]
*/

impl Shell {
    pub(crate) fn update_shape_tool(&mut self, message: stage::ShapeToolMessage) {
        match message {
            stage::ShapeToolMessage::Select(tool) => self.shape_tool = tool,
            stage::ShapeToolMessage::Create { tool, start, end } => {
                let Some(source) = primitive_path(tool, start, end, self.composition_size()) else {
                    return;
                };
                self.create_drawn_shape(ShapeNode::Leaf(Shape {
                    source,
                    ops: Vec::new(),
                    fill: Some(Self::default_new_object_fill()),
                    stroke: None,
                }));
            }
            stage::ShapeToolMessage::CreatePen { points } => {
                let Some(size) = self.composition_size() else {
                    return;
                };
                let Some(source) = pen_path(points, size) else {
                    return;
                };
                self.create_drawn_shape(ShapeNode::Leaf(Shape {
                    source,
                    ops: Vec::new(),
                    fill: None,
                    stroke: Some(Stroke::default()),
                }));
            }
            stage::ShapeToolMessage::Cancel => {}
        }
    }

    fn create_drawn_shape(&mut self, shape: ShapeNode) {
        let Some(composition) = self.composition() else {
            self.status = Some("コンポジションが無いため図形を置けません".to_owned());
            return;
        };
        let id = LayerId(self.next_layer_id());
        let intents = [
            Intent::AddLayer(id),
            Intent::SetMeta {
                layer: id,
                meta: LayerMeta {
                    source: LayerSource::Shape,
                    order: id.0 as i16,
                    timing: LayerTiming::place(
                        self.session.playhead,
                        None,
                        composition.duration_frames,
                    ),
                },
            },
            Intent::SetAttrs {
                layer: id,
                patch: LayerAttrsPatch {
                    label_color: Some(Some(Self::label_color_for_new_layer(id))),
                    ..Default::default()
                },
            },
            Intent::SetShapes {
                layer: id,
                shapes: vec![shape],
            },
        ];
        match self.doc.apply_all(intents) {
            Ok(()) => self.select_single(id),
            Err(error) => self.status = Some(format!("図形を作れません: {error}")),
        }
    }

    fn composition_size(&self) -> Option<[f64; 2]> {
        self.composition()
            .map(|composition| [composition.width as f64, composition.height as f64])
    }
}

fn primitive_path(
    tool: stage::ShapeTool,
    start: [f32; 2],
    end: [f32; 2],
    size: Option<[f64; 2]>,
) -> Option<PathSource> {
    let [width, height] = size?;
    let min_x = f64::from(start[0].min(end[0])).clamp(0.0, width);
    let max_x = f64::from(start[0].max(end[0])).clamp(0.0, width);
    let min_y = f64::from(start[1].min(end[1])).clamp(0.0, height);
    let max_y = f64::from(start[1].max(end[1])).clamp(0.0, height);
    if max_x - min_x < 2.0 || max_y - min_y < 2.0 {
        return None;
    }
    let center = [
        (min_x + max_x) * 0.5,
        (min_y + max_y) * 0.5,
    ];
    let radius = [(max_x - min_x) * 0.5, (max_y - min_y) * 0.5];
    let source = match tool {
        stage::ShapeTool::Rectangle => PathSource::Bezier(vec![Contour::closed([
            centered_point([width, height], [min_x as f32, min_y as f32]),
            centered_point([width, height], [max_x as f32, min_y as f32]),
            centered_point([width, height], [max_x as f32, max_y as f32]),
            centered_point([width, height], [min_x as f32, max_y as f32]),
        ])]),
        stage::ShapeTool::Ellipse => {
            let points = (0..32)
                .map(|index| {
                    let angle = std::f64::consts::TAU * index as f64 / 32.0;
                    centered_point(
                        [width, height],
                        [
                            (center[0] + radius[0] * angle.cos()) as f32,
                            (center[1] + radius[1] * angle.sin()) as f32,
                        ],
                    )
                })
                .collect::<Vec<_>>();
            PathSource::Bezier(vec![Contour::closed(points)])
        }
        _ => return None,
    };
    Some(source)
}

fn centered_point(size: [f64; 2], point: [f32; 2]) -> Point {
    Point {
        x: f64::from(point[0]) - size[0] * 0.5,
        y: f64::from(point[1]) - size[1] * 0.5,
    }
}

fn pen_path(points: Vec<[f32; 2]>, size: [f64; 2]) -> Option<PathSource> {
    let points = points
        .into_iter()
        .map(|point| centered_point(size, point))
        .collect::<Vec<_>>();
    (points.len() >= 2).then_some(PathSource::Bezier(vec![Contour::open(points)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drawn_shape_adds_selected_layer() {
        let (mut shell, _) = Shell::new_fixture();
        let before = shell.layer_count();

        shell.update_shape_tool(stage::ShapeToolMessage::Create {
            tool: stage::ShapeTool::Rectangle,
            start: [100.0, 80.0],
            end: [300.0, 200.0],
        });

        assert_eq!(shell.layer_count(), before + 1);
        let layer = shell.session.selection.expect("描いた図形が選択されていない");
        let shapes = shell.doc.view().shapes(layer).expect("図形の中身が無い");
        assert_eq!(shapes.len(), 1);
    }

    #[test]
    fn primitive_path_is_centered_in_the_composition_canvas() {
        let PathSource::Bezier(contours) = primitive_path(
            stage::ShapeTool::Rectangle,
            [100.0, 80.0],
            [300.0, 200.0],
            Some([640.0, 360.0]),
        )
        .expect("rectangle") else {
            panic!("rectangle must be a bezier contour");
        };
        assert_eq!(contours[0].vertices[0].point, Point { x: -220.0, y: -100.0 });
        assert_eq!(contours[0].vertices[2].point, Point { x: -20.0, y: 20.0 });
        assert!(contours[0].closed);
    }
}
