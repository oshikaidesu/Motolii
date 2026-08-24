//! SHAPE section(P3 #15)。矩形/楕円の寸法と角丸半径を投影し、数値入力を
//! `Intent::SetShapes` へ一回で確定する。
//!
//! **持つ**: shape-layer の平坦な primitive を読む投影、入力の下書き、値の
//! 検証と shape 列の差し替え、SHAPE section の view。
//! **持たない**: Stage の座標変換・描画、任意頂点編集、fill/stroke の色。
//! そこへ責任を広げず、P3 #16 と P3 C-20〜23 の入口は別 component が持つ。
//! Group は構造を壊さないためこの section の編集対象にしない。

use motolii_settings_pane::chrome::section_header;
use motolii_store::{Document, Intent, LayerId, OpKind, PathSource, ShapeNode};
use motolii_tokens_rs::{Colors, Dimensions};

use iced::widget::{column, row as row_widget, text, text_input};
use iced::{Element, Length};

use crate::chrome::{bordered_row, name_input_style, value_cell_padding};
use crate::projection::{ShapeRowProjection, ShapeSectionProjection};
use crate::Message;

/* motolii-component
id = "inspector.shape"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["ShapeSectionProjection", "commit_shape_field"]
meaning = ["Width", "Height", "Radius"]
evaluation = ["project", "commit_shape_field"]
render = ["shape_section", "shape_numeric_input"]
observable = ["shape_inspector_changes_geometry"]
*/

/// SHAPE section の入力対象。shape の index は layer 内の `ShapeNode` 列で
/// 固定する。id を新設せず、既存の shape 列そのものを正本にする。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeField {
    Width(usize),
    Height(usize),
    Radius(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapeFieldDraft {
    pub field: ShapeField,
    pub text: String,
}

/// SHAPE の入力を `SetShapes` へ1回で確定する。
pub fn commit_shape_field(
    doc: &mut Document,
    layer: Option<LayerId>,
    draft: &mut Option<ShapeFieldDraft>,
) -> Result<(), String> {
    let Some(layer) = layer else { return Ok(()); };
    let Some(draft) = draft.take() else { return Ok(()); };
    let value = draft
        .text
        .trim()
        .parse::<f64>()
        .map_err(|_| "シェイプの値は数値で入力してください".to_owned())?;
    if !value.is_finite() || value < 0.0 {
        return Err("シェイプの値は0以上の有限値にしてください".to_owned());
    }

    let mut shapes = doc
        .view()
        .shapes(layer)
        .map_err(|error| format!("シェイプを読めない: {error}"))?;
    let index = match draft.field {
        ShapeField::Width(index) | ShapeField::Height(index) | ShapeField::Radius(index) => index,
    };
    let Some(ShapeNode::Leaf(shape)) = shapes.get_mut(index) else { return Ok(()); };

    match draft.field {
        ShapeField::Width(_) | ShapeField::Height(_) => {
            let (width, height, rectangle) = match &shape.source {
                PathSource::Rectangle { size } | PathSource::Ellipse { size } => {
                    let width = if matches!(draft.field, ShapeField::Width(_)) { value } else { size.x };
                    let height = if matches!(draft.field, ShapeField::Height(_)) { value } else { size.y };
                    (width, height, matches!(&shape.source, PathSource::Rectangle { .. }))
                }
                _ => return Ok(()),
            };
            shape.source = if rectangle {
                PathSource::Rectangle {
                    size: motolii_store::VectorPoint { x: width, y: height },
                }
            } else {
                PathSource::Ellipse {
                    size: motolii_store::VectorPoint { x: width, y: height },
                }
            };
        }
        ShapeField::Radius(_) => {
            let mut found = false;
            for op in &mut shape.ops {
                if let OpKind::RoundedCorners { radius } = &mut op.kind {
                    *radius = value;
                    found = true;
                    break;
                }
            }
            if !found {
                shape.ops.push(motolii_store::ShapeOp::new(OpKind::RoundedCorners { radius: value }));
            }
        }
    }

    doc.apply(Intent::SetShapes { layer, shapes })
        .map_err(|error| format!("シェイプを書けない: {error}"))
}

pub(crate) fn shape_section(
    projection: &ShapeSectionProjection,
    draft: Option<&ShapeFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let mut rows = column![section_header("SHAPE", dims, colors)];
    for shape in &projection.rows {
        rows = rows.push(shape_row(shape, draft, dims, colors));
    }
    rows.into()
}

fn shape_row(
    shape: &ShapeRowProjection,
    draft: Option<&ShapeFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let label = text(format!("{} {}", shape.kind, shape.index + 1))
        .size(dims.body_text)
        .color(colors.text_primary)
        .width(Length::Fill);
    let width = shape_numeric_input(ShapeField::Width(shape.index), shape.width, draft, dims, colors);
    let height = shape_numeric_input(ShapeField::Height(shape.index), shape.height, draft, dims, colors);
    let radius = shape_numeric_input(ShapeField::Radius(shape.index), shape.radius, draft, dims, colors);
    bordered_row(
        row_widget![label, width, text("×").size(dims.caption_text), height, text("r").size(dims.caption_text), radius]
            .spacing(dims.spacing_xs)
            .align_y(iced::alignment::Vertical::Center)
            .into(),
        dims,
    )
}

fn shape_numeric_input(
    field: ShapeField,
    value: f64,
    draft: Option<&ShapeFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let displayed = draft
        .filter(|draft| draft.field == field)
        .map(|draft| draft.text.clone())
        .unwrap_or_else(|| crate::transform::format_number(value, 1));
    text_input("", displayed)
        .on_input(move |text| Message::ShapeFieldInput(field, text))
        .on_submit(Message::ShapeFieldSubmit(field))
        .size(dims.body_text)
        .width(Length::Fixed(dims.inspector_value_width))
        .padding(value_cell_padding(dims))
        .align_x(iced::alignment::Horizontal::Center)
        .style(move |_theme, status| name_input_style(dims, colors, status))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_store::{Intent, Shape};

    #[test]
    fn shape_inspector_changes_geometry() {
        let mut doc = Document::new();
        let layer = LayerId(1);
        doc.apply(Intent::AddLayer(layer)).expect("layer");
        doc.apply(Intent::SetShapes {
            layer,
            shapes: vec![ShapeNode::Leaf(Shape::new(PathSource::Rectangle {
                size: motolii_store::VectorPoint { x: 100.0, y: 50.0 },
            }))],
        })
        .expect("shape");
        let mut draft = Some(ShapeFieldDraft { field: ShapeField::Radius(0), text: "12".into() });
        commit_shape_field(&mut doc, Some(layer), &mut draft).expect("radius");
        let mut width = Some(ShapeFieldDraft { field: ShapeField::Width(0), text: "140".into() });
        commit_shape_field(&mut doc, Some(layer), &mut width).expect("width");
        let shapes = doc.view().shapes(layer).expect("read");
        let ShapeNode::Leaf(shape) = &shapes[0] else { panic!("leaf") };
        assert!(matches!(&shape.source, PathSource::Rectangle { size } if size.x == 140.0 && size.y == 50.0));
        assert!(shape.ops.iter().any(|op| matches!(op.kind, OpKind::RoundedCorners { radius } if radius == 12.0)));
    }
}
