//! Browser の shape-operator カードを選択中の shape へ積む writer。
//!
//! Browser は `ShapeOpKind` という軽い語彙だけを宣言し、具体的な
//! `motolii_vector::OpKind` と既定値、`Intent::SetShapes` の唯一の書き戻しは
//! この module に隔離する。create カードの layer 生成や Stage の path 編集とは
//! 責任が違うため、そこへ混ぜず「既存 shape に1段足す」component として持つ。

/* motolii-component
id = "shell.shape_operator_writer"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["apply_op_to_selected_layer", "dispatch_browser_card_intent"]
meaning = ["ApplyOpFromCard", "OpKind"]
evaluation = ["default_op_kind", "SetShapes"]
render = ["SetShapes"]
observable = ["shape_operator_cards_apply_to_selected_shape"]
*/

use motolii_store::{Intent, ShapeNode};
use motolii_vector::{
    Composite, LineJoin, OpKind, PointType, RepeaterTransform, ShapeOp, TrimMultiple,
};

use crate::{browser_pane, Shell};

impl Shell {
    /// effects タブの `OpKind` 演算子カード実体化(2026-08-24「ブラウザに8枚の札」
    /// 発注 §2)。単一選択の時だけ意味を持つ。**新しい原子 Intent は増やさない** —
    /// `Intent::SetShapes`(丸ごと差し替え)で現在の shape へ1段積んで書き戻す。
    /// `OpKind` の具体的なバリアントをここで直接組み、汎用 payload へ逃がさない。
    ///
    /// shape が無いレイヤー(Solid/Text/Null)、複数 shape、group は拒否して status
    /// へ理由を出す。黙って先頭を選ぶと利用者の意図と異なるため、今回は
    /// `create_from_card` が作る単一 leaf shape の経路に限定する。
    pub(crate) fn apply_op_to_selected_layer(&mut self, op: browser_pane::model::ShapeOpKind) {
        let target = match self.session.selected_layers.as_slice() {
            [only] => Some(*only),
            _ => None,
        };
        let Some(layer) = target else {
            self.status = Some("演算子を適用するレイヤーを1つ選んでください".to_owned());
            return;
        };
        let mut shapes = match self.doc.view().shapes(layer) {
            Ok(shapes) => shapes,
            Err(error) => {
                self.status = Some(format!("shape を読めません: {error}"));
                return;
            }
        };
        let [ShapeNode::Leaf(shape)] = shapes.as_mut_slice() else {
            self.status = Some(
                "このレイヤーには演算子を積める shape がありません(まず Rectangle/Ellipse/Star \
                 などの shape レイヤーを選んでください)"
                    .to_owned(),
            );
            return;
        };
        shape.ops.push(ShapeOp::new(Self::default_op_kind(op)));
        if let Err(error) = self.doc.apply(Intent::SetShapes { layer, shapes }) {
            self.status = Some(format!("演算子を適用できません: {error}"));
        }
    }

    /// `OpKind` の既定パラメータ。**恒等値は使わない** — 押しても見た目が変わらず
    /// 「何も起きていない」状態を作らない。Lottie の `shapes/*` スキーマにも
    /// 既定値は無いため、ここが Motolii 側の意味判断の正本になる。
    pub(crate) fn default_op_kind(op: browser_pane::model::ShapeOpKind) -> OpKind {
        use browser_pane::model::ShapeOpKind;
        match op {
            ShapeOpKind::TrimPath => OpKind::TrimPath {
                start: 0.0,
                end: 0.75,
                offset: 0.0,
                multiple: TrimMultiple::Simultaneously,
            },
            ShapeOpKind::Repeater => OpKind::Repeater {
                copies: 3.0,
                offset: 0.0,
                transform: RepeaterTransform {
                    position: motolii_vector::Point { x: 24.0, y: 0.0 },
                    ..RepeaterTransform::IDENTITY
                },
                composite: Composite::Above,
                start_opacity: 1.0,
                end_opacity: 1.0,
            },
            ShapeOpKind::RoundedCorners => OpKind::RoundedCorners { radius: 20.0 },
            ShapeOpKind::PuckerBloat => OpKind::PuckerBloat { amount: 0.3 },
            ShapeOpKind::ZigZag => OpKind::ZigZag {
                amplitude: 10.0,
                frequency: 3.0,
                point_type: PointType::Corner,
            },
            ShapeOpKind::OffsetPath => OpKind::OffsetPath {
                amount: 10.0,
                join: LineJoin::Miter,
                miter_limit: 4.0,
            },
            ShapeOpKind::Twist => OpKind::Twist {
                angle: 45.0,
                center: motolii_vector::Point { x: 0.0, y: 0.0 },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_store::{
        Intent, LayerId, LayerMeta, LayerSource, LayerTiming, PathSource, Shape as VectorShape,
        VectorPoint,
    };

    #[test]
    fn shape_operator_cards_apply_to_selected_shape() {
        let (mut shell, _) = Shell::new();
        let layer = LayerId(1);
        shell
            .doc
            .apply_all([
                Intent::AddLayer(layer),
                Intent::SetMeta {
                    layer,
                    meta: LayerMeta {
                        source: LayerSource::Shape,
                        order: 0,
                        timing: LayerTiming::place(0, None, 300),
                    },
                },
                Intent::SetShapes {
                    layer,
                    shapes: vec![ShapeNode::Leaf(VectorShape {
                        source: PathSource::Rectangle {
                            size: VectorPoint { x: 100.0, y: 80.0 },
                        },
                        ops: Vec::new(),
                        fill: None,
                        stroke: None,
                    })],
                },
            ])
            .expect("shape layer fixture");
        shell.session.selected_layers = vec![layer];

        shell.apply_op_to_selected_layer(browser_pane::model::ShapeOpKind::RoundedCorners);

        let shapes = shell.doc.view().shapes(layer).expect("shape projection");
        let [ShapeNode::Leaf(shape)] = shapes.as_slice() else {
            panic!("operator keeps the single leaf shape");
        };
        assert!(matches!(
            shape.ops.as_slice(),
            [ShapeOp {
                kind: OpKind::RoundedCorners { radius },
                ..
            }] if *radius > 0.0
        ));
    }
}
