//! Browser の Create カードから shape layer を作る writer。
//!
//! Rectangle/Ellipse/PolyStar の shape source と既定 fill、新規 layer の
//! `apply_all` をこの component に閉じ込める。既存 shape へ modifier を足す
//! `shape_operator`、Stage で手描きする `shape_ops` とは責任を分ける。

use motolii_store::{
    Intent, LayerAttrsPatch, LayerId, LayerMeta, LayerSource, LayerTiming,
    PathSource, Shape as VectorShape, ShapeNode, VectorPoint,
};
use motolii_vector::{Brush, Fill, Rgb, StarType};

use crate::{browser_pane, inspector_pane, Shell};

/* motolii-component
id = "shell.shape_creator_writer"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["create_from_card", "default_shape_path_source"]
meaning = ["CreateFromCard", "CreateKind"]
evaluation = ["default_shape_path_source", "apply_all"]
render = ["SetShapes", "PathSource"]
observable = ["create_poly_star_adds_a_shape_layer_with_a_polystar_path_source"]
*/

impl Shell {
    /// Create タブのカード実体化。1 `apply_all` = 1 undo とし、shape は
    /// `LayerSource::Shape` だけで終わらせず `SetShapes` まで同じ操作へ積む。
    pub(crate) fn create_from_card(&mut self, kind: browser_pane::model::CreateKind) {
        use browser_pane::model::CreateKind;
        let id = LayerId(self.next_layer_id());
        let source = match kind {
            CreateKind::Rectangle | CreateKind::Ellipse | CreateKind::PolyStar => {
                LayerSource::Shape
            }
            CreateKind::Solid => LayerSource::Solid {
                rgba: [80, 160, 220, 255],
                width: 240,
                height: 135,
            },
            CreateKind::Null => LayerSource::Null,
            CreateKind::Text => LayerSource::Text,
        };
        let mut intents = vec![
            Intent::AddLayer(id),
            Intent::SetMeta {
                layer: id,
                meta: LayerMeta {
                    source,
                    order: id.0 as i16,
                    timing: LayerTiming::place(self.session.playhead, None, self.comp_duration()),
                },
            },
            Intent::SetAttrs {
                layer: id,
                patch: LayerAttrsPatch {
                    label_color: Some(Some(Self::label_color_for_new_layer(id))),
                    ..Default::default()
                },
            },
        ];
        if matches!(kind, CreateKind::Text) {
            intents.push(Intent::SetTextDocument {
                layer: id,
                document: inspector_pane::default_text_document(),
            });
        }
        if let Some(path_source) = Self::default_shape_path_source(kind) {
            intents.push(Intent::SetShapes {
                layer: id,
                shapes: vec![ShapeNode::Leaf(VectorShape {
                    source: path_source,
                    ops: Vec::new(),
                    fill: Some(Self::default_new_object_fill()),
                    stroke: None,
                })],
            });
        }
        let placed = self.doc.apply_all(intents);
        match placed {
            Ok(()) => self.select_single(id),
            Err(error) => self.status = Some(format!("layer を作れない: {error}")),
        }
    }

    /// Rectangle/Ellipse/PolyStar の shape source。PolyStar は 5 点の星を
    /// 既定値とし、Rectangle/Ellipse と同じ 240×135 の footprint に収める。
    pub(crate) fn default_shape_path_source(
        kind: browser_pane::model::CreateKind,
    ) -> Option<PathSource> {
        use browser_pane::model::CreateKind;
        let size = VectorPoint { x: 240.0, y: 135.0 };
        match kind {
            CreateKind::Rectangle => Some(PathSource::Rectangle { size }),
            CreateKind::Ellipse => Some(PathSource::Ellipse { size }),
            CreateKind::PolyStar => Some(PathSource::PolyStar {
                points: 5.0,
                outer_radius: 67.0,
                inner_radius: 33.0,
                star_type: StarType::Star,
            }),
            _ => None,
        }
    }

    /// 「新規に置いた物」の既定塗り。Solid の既定色と同じ差し色を使う。
    pub(crate) fn default_new_object_fill() -> Fill {
        Fill {
            brush: Brush::Solid(Rgb {
                r: 80.0 / 255.0,
                g: 160.0 / 255.0,
                b: 220.0 / 255.0,
            }),
            ..Fill::default()
        }
    }
}
