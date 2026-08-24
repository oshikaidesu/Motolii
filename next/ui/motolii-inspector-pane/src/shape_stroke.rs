//! Shape の線 component。
//!
//! 塗り(`crate::shape_fill`)と責任を分け、線幅・端点・接合部・破線だけを
//! projection から Inspector へ出す。全ての書き戻しは `SetShapes` へ集め、
//! Shell は Message と一時下書きの WIRE だけを持つ。

use iced::widget::{button, column, row as row_widget, text, text_input};
use iced::{Element, Length};

use motolii_settings_pane::chrome::section_header;
use motolii_store::{Document, Intent, LayerId, ShapeNode, StoreError, StoreView};
use motolii_tokens_rs::{Colors, Dimensions};
use motolii_vector::{Dash, LineCap, LineJoin, Stroke};

use crate::chrome::{bordered_row, name_input_style, value_cell_padding};
use crate::Message;

/* motolii-component
id = "inspector.shape_stroke"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["ShapeStrokeProjection", "commit_shape_stroke"]
meaning = ["ShapeStrokeInput", "ShapeStrokeCap", "ShapeStrokeJoin", "ShapeStrokeDash"]
evaluation = ["parse_stroke_width", "shape_stroke_changes"]
render = ["shape_stroke_section", "stroke_width_input"]
observable = ["shape_stroke_changes"]
*/

#[derive(Clone, Debug, PartialEq)]
pub struct ShapeStrokeProjection {
    pub rows: Vec<ShapeStrokeRowProjection>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShapeStrokeRowProjection {
    pub index: usize,
    pub stroke: Option<Stroke>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeStrokeField {
    Width(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapeStrokeDraft {
    pub field: ShapeStrokeField,
    pub text: String,
}

/// Shape の線を読む投影。Group は flatten せず、leaf の順番をそのまま保つ。
pub fn project_shape_stroke(
    store: &StoreView<'_>,
    layer: LayerId,
) -> Result<Option<ShapeStrokeProjection>, StoreError> {
    let rows = store
        .shapes(layer)?
        .into_iter()
        .enumerate()
        .filter_map(|(index, node)| {
            let ShapeNode::Leaf(shape) = node else { return None };
            Some(ShapeStrokeRowProjection {
                index,
                stroke: shape.stroke,
            })
        })
        .collect::<Vec<_>>();
    Ok((!rows.is_empty()).then_some(ShapeStrokeProjection { rows }))
}

pub fn parse_stroke_width(text: &str) -> Result<f64, String> {
    let value = text
        .trim()
        .parse::<f64>()
        .map_err(|_| "線幅は数値で入力してください".to_owned())?;
    if !value.is_finite() || value < 0.0 {
        return Err("線幅は0以上の有限値にしてください".to_owned());
    }
    Ok(value)
}

pub fn shape_stroke_input_id(index: usize) -> iced::widget::Id {
    iced::widget::Id::from(format!("inspector-shape-stroke-{index}"))
}

fn default_dash() -> Dash {
    Dash {
        pattern: vec![8.0, 4.0],
        offset: 0.0,
    }
}

fn write_stroke(
    doc: &mut Document,
    layer: Option<LayerId>,
    index: usize,
    update: impl FnOnce(&mut Stroke),
) -> Result<(), String> {
    let Some(layer) = layer else { return Ok(()) };
    let mut shapes = doc
        .view()
        .shapes(layer)
        .map_err(|error| format!("シェイプを読めない: {error}"))?;
    let Some(ShapeNode::Leaf(shape)) = shapes.get_mut(index) else { return Ok(()) };
    update(shape.stroke.get_or_insert_with(Stroke::default));
    doc.apply(Intent::SetShapes { layer, shapes })
        .map_err(|error| format!("線を書けない: {error}"))
}

/// 線幅の下書きを1回の Document 編集へ確定する。
pub fn commit_shape_stroke(
    doc: &mut Document,
    layer: Option<LayerId>,
    draft: &mut Option<ShapeStrokeDraft>,
) -> Result<(), String> {
    let Some(draft) = draft.take() else { return Ok(()) };
    let index = match draft.field {
        ShapeStrokeField::Width(index) => index,
    };
    let value = parse_stroke_width(&draft.text)?;
    write_stroke(doc, layer, index, |stroke| stroke.width = value)
}

pub fn cycle_shape_stroke_cap(
    doc: &mut Document,
    layer: Option<LayerId>,
    index: usize,
) -> Result<(), String> {
    write_stroke(doc, layer, index, |stroke| {
        stroke.cap = match stroke.cap {
            LineCap::Butt => LineCap::Round,
            LineCap::Round => LineCap::Square,
            LineCap::Square => LineCap::Butt,
        };
    })
}

pub fn cycle_shape_stroke_join(
    doc: &mut Document,
    layer: Option<LayerId>,
    index: usize,
) -> Result<(), String> {
    write_stroke(doc, layer, index, |stroke| {
        stroke.join = match stroke.join {
            LineJoin::Miter => LineJoin::Round,
            LineJoin::Round => LineJoin::Bevel,
            LineJoin::Bevel => LineJoin::Miter,
        };
    })
}

pub fn toggle_shape_stroke_dash(
    doc: &mut Document,
    layer: Option<LayerId>,
    index: usize,
) -> Result<(), String> {
    write_stroke(doc, layer, index, |stroke| {
        stroke.dash = stroke.dash.take().or_else(|| Some(default_dash()));
    })
}

pub fn shape_stroke_section(
    projection: &ShapeStrokeProjection,
    draft: Option<&ShapeStrokeDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let mut rows = column![section_header("STROKE", dims, colors)];
    for row in &projection.rows {
        let stroke = row.stroke.as_ref();
        let field = ShapeStrokeField::Width(row.index);
        let displayed = draft
            .filter(|draft| draft.field == field)
            .map(|draft| draft.text.clone())
            .unwrap_or_else(|| {
                stroke
                    .map(|stroke| crate::transform::format_number(stroke.width, 1))
                    .unwrap_or_else(|| "1.0".to_owned())
            });
        let width = stroke_width_input(field, displayed, dims, colors);
        let cap = button(text(cap_label(stroke.map(|stroke| stroke.cap).unwrap_or_default())))
            .on_press(Message::ShapeStrokeCap(row.index))
            .padding([dims.spacing_xs, dims.spacing_s]);
        let join = button(text(join_label(stroke.map(|stroke| stroke.join).unwrap_or_default())))
            .on_press(Message::ShapeStrokeJoin(row.index))
            .padding([dims.spacing_xs, dims.spacing_s]);
        let dash = button(text(dash_label(stroke.and_then(|stroke| stroke.dash.as_ref()))))
            .on_press(Message::ShapeStrokeDash(row.index))
            .padding([dims.spacing_xs, dims.spacing_s]);
        rows = rows.push(bordered_row(
            row_widget![
                text(format!("{}", row.index + 1)).size(dims.caption_text),
                width,
                cap,
                join,
                dash,
            ]
            .spacing(dims.spacing_xs)
            .align_y(iced::alignment::Vertical::Center)
            .into(),
            dims,
        ));
    }
    rows.into()
}

fn stroke_width_input(
    field: ShapeStrokeField,
    displayed: String,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    text_input("1.0", displayed)
        .id(shape_stroke_input_id(match field {
            ShapeStrokeField::Width(index) => index,
        }))
        .on_input(move |text| Message::ShapeStrokeInput(field, text))
        .on_submit(Message::ShapeStrokeSubmit(field))
        .size(dims.body_text)
        .width(Length::Fixed(dims.inspector_value_width))
        .padding(value_cell_padding(dims))
        .style(move |_theme, status| name_input_style(dims, colors, status))
        .into()
}

fn cap_label(cap: LineCap) -> &'static str {
    match cap {
        LineCap::Butt => "Butt",
        LineCap::Round => "Round",
        LineCap::Square => "Square",
    }
}

fn join_label(join: LineJoin) -> &'static str {
    match join {
        LineJoin::Miter => "Miter",
        LineJoin::Round => "Round",
        LineJoin::Bevel => "Bevel",
    }
}

fn dash_label(dash: Option<&Dash>) -> &'static str {
    if dash.is_some() { "Dash" } else { "Solid" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_store::{Intent, Shape, VectorPoint};
    use motolii_vector::{LineCap, LineJoin};

    fn document_with_shape() -> (Document, LayerId) {
        let mut doc = Document::new();
        let layer = LayerId(1);
        doc.apply(Intent::AddLayer(layer)).expect("layer");
        doc.apply(Intent::SetShapes {
            layer,
            shapes: vec![ShapeNode::Leaf(Shape::new(
                motolii_store::PathSource::Rectangle {
                    size: VectorPoint { x: 100.0, y: 50.0 },
                },
            ))],
        })
        .expect("shape");
        (doc, layer)
    }

    #[test]
    fn shape_stroke_changes() {
        let (mut doc, layer) = document_with_shape();
        let mut draft = Some(ShapeStrokeDraft {
            field: ShapeStrokeField::Width(0),
            text: "6.5".into(),
        });
        commit_shape_stroke(&mut doc, Some(layer), &mut draft).expect("width");
        cycle_shape_stroke_cap(&mut doc, Some(layer), 0).expect("cap");
        cycle_shape_stroke_join(&mut doc, Some(layer), 0).expect("join");
        toggle_shape_stroke_dash(&mut doc, Some(layer), 0).expect("dash");
        let shapes = doc.view().shapes(layer).expect("read");
        let ShapeNode::Leaf(shape) = &shapes[0] else { panic!("leaf") };
        let Some(stroke) = &shape.stroke else { panic!("stroke") };
        assert_eq!(stroke.width, 6.5);
        assert_eq!(stroke.cap, LineCap::Round);
        assert_eq!(stroke.join, LineJoin::Round);
        assert_eq!(
            stroke.dash.as_ref().map(|dash| dash.pattern.as_slice()),
            Some(&[8.0, 4.0][..])
        );
    }
}
