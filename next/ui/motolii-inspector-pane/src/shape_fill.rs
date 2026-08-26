//! Shape の塗り component。
//!
//! Shape の寸法を扱う [`crate::shape`] とは分け、塗りの色・gradient と
//! Inspector の入口だけを持つ。入力は下書きとして pane 側に置き、確定時の
//! `Intent::SetShapes` はこの module の自由関数へ集める。

use iced::widget::{button, column, container, row as row_widget, text, text_input};
use iced::{Element, Length};

use motolii_store::{Document, Intent, LayerId, ShapeNode, StoreError, StoreView};
use motolii_settings_pane::chrome::section_header;
use motolii_tokens_rs::{Colors, Dimensions};
use motolii_vector::{Brush, Fill, Gradient, GradientStop, GradientType, Point, Rgb};

use crate::chrome::{bordered_row, name_input_style, value_cell_padding};
use crate::projection::ShapeFillProjection;
use crate::Message;

/* motolii-component
id = "inspector.shape_fill"
kind = "semantic"
weight = "core_edit"
maps = []
entry = ["ShapeFillProjection", "commit_shape_fill"]
meaning = ["ShapeFillInput", "ShapeFillGradient"]
evaluation = ["parse_hex_color", "shape_fill_hex_changes_fill"]
render = ["shape_fill_section", "shape_fill_swatch"]
observable = ["shape_fill_hex_changes_fill"]
*/

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeFillField {
    Hex(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapeFillDraft {
    pub field: ShapeFillField,
    pub text: String,
}

/// Shape の fill を読む投影。`ShapeNode::Group` は編集対象にしない。
pub fn project_shape_fill(
    store: &StoreView<'_>,
    layer: LayerId,
) -> Result<Option<ShapeFillProjection>, StoreError> {
    let rows = store
        .shapes(layer)?
        .into_iter()
        .enumerate()
        .filter_map(|(index, node)| {
            let ShapeNode::Leaf(shape) = node else { return None };
            Some(crate::projection::ShapeFillRowProjection {
                index,
                fill: shape.fill,
            })
        })
        .collect::<Vec<_>>();
    Ok((!rows.is_empty()).then_some(ShapeFillProjection { rows }))
}

/// `#RRGGBB` または `#RRGGBBAA` を `Rgb` と opacity へ読む。
pub fn parse_hex_color(text: &str) -> Result<([f64; 3], f64), String> {
    let raw = text.trim().strip_prefix('#').unwrap_or(text.trim());
    if raw.len() != 6 && raw.len() != 8 {
        return Err("色は #RRGGBB または #RRGGBBAA で入力してください".to_owned());
    }
    let channel = |start| {
        u8::from_str_radix(&raw[start..start + 2], 16)
            .map(|value| f64::from(value) / 255.0)
            .map_err(|_| "色の16進数が不正です".to_owned())
    };
    let rgb = [channel(0)?, channel(2)?, channel(4)?];
    let alpha = if raw.len() == 8 { channel(6)? } else { 1.0 };
    Ok((rgb, alpha))
}

pub fn format_fill_hex(fill: Option<&Fill>) -> String {
    let Some(fill) = fill else { return "#00000000".to_owned() };
    match &fill.brush {
        Brush::Solid(rgb) => format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            channel_u8(rgb.r),
            channel_u8(rgb.g),
            channel_u8(rgb.b),
            channel_u8(fill.opacity),
        ),
        Brush::Gradient(_) => "gradient".to_owned(),
    }
}

pub fn shape_fill_input_id(index: usize) -> iced::widget::Id {
    iced::widget::Id::from(format!("inspector-shape-fill-{index}"))
}

fn channel_u8(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn fill_from_hex(fill: Option<Fill>, text: &str) -> Result<Fill, String> {
    let (rgb, opacity) = parse_hex_color(text)?;
    let mut next = fill.unwrap_or_default();
    next.brush = Brush::Solid(Rgb { r: rgb[0], g: rgb[1], b: rgb[2] });
    next.opacity = opacity;
    next.hidden = false;
    Ok(next)
}

/// Solid color の下書きを1回の Document 編集へ確定する。
pub fn commit_shape_fill(
    doc: &mut Document,
    layer: Option<LayerId>,
    draft: &mut Option<ShapeFillDraft>,
) -> Result<(), String> {
    let Some(draft) = draft.take() else { return Ok(()) };
    let Some(layer) = layer else { return Ok(()) };
    let index = match draft.field {
        ShapeFillField::Hex(index) => index,
    };
    let mut shapes = doc
        .view()
        .shapes(layer)
        .map_err(|error| format!("シェイプを読めない: {error}"))?;
    let Some(ShapeNode::Leaf(shape)) = shapes.get_mut(index) else { return Ok(()) };
    shape.fill = Some(fill_from_hex(shape.fill.take(), &draft.text)?);
    doc.apply(Intent::SetShapes { layer, shapes })
        .map_err(|error| format!("塗りを書けない: {error}"))
}

/// fill が無い shape にも gradient を作れるよう、既定の2停止点を1回で入れる。
pub fn apply_shape_gradient(
    doc: &mut Document,
    layer: Option<LayerId>,
    index: usize,
) -> Result<(), String> {
    let Some(layer) = layer else { return Ok(()) };
    let mut shapes = doc
        .view()
        .shapes(layer)
        .map_err(|error| format!("シェイプを読めない: {error}"))?;
    let Some(ShapeNode::Leaf(shape)) = shapes.get_mut(index) else { return Ok(()) };
    let mut fill = shape.fill.take().unwrap_or_default();
    fill.brush = Brush::Gradient(Gradient {
        kind: GradientType::Linear,
        start: Point { x: -120.0, y: 0.0 },
        end: Point { x: 120.0, y: 0.0 },
        stops: vec![
            GradientStop { offset: 0.0, color: Rgb { r: 0.1, g: 0.4, b: 1.0 } },
            GradientStop { offset: 1.0, color: Rgb { r: 1.0, g: 0.25, b: 0.1 } },
        ],
    });
    fill.opacity = 1.0;
    fill.hidden = false;
    shape.fill = Some(fill);
    doc.apply(Intent::SetShapes { layer, shapes })
        .map_err(|error| format!("グラデーションを書けない: {error}"))
}

pub fn shape_fill_section(
    projection: &ShapeFillProjection,
    draft: Option<&ShapeFillDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let mut rows = column![section_header("FILL", dims, colors)];
    for row in &projection.rows {
        let field = ShapeFillField::Hex(row.index);
        let displayed = draft
            .filter(|draft| draft.field == field)
            .map(|draft| draft.text.clone())
            .unwrap_or_else(|| format_fill_hex(row.fill.as_ref()));
        let swatch = shape_fill_swatch(row.fill.as_ref(), row.index, dims, colors);
        let input = text_input("#RRGGBB", displayed)
            .id(shape_fill_input_id(row.index))
            .on_input(move |text| Message::ShapeFillInput(field, text))
            .on_submit(Message::ShapeFillSubmit(field))
            .size(dims.theme().text.body)
            .width(Length::Fill)
            .padding(value_cell_padding(dims))
            .style(move |_theme, status| name_input_style(dims, colors, status));
        let gradient = button(text("Gradient").size(dims.theme().text.caption))
            .on_press(Message::ShapeFillGradient(row.index))
            .padding([dims.theme().space.xs, dims.theme().space.s]);
        rows = rows.push(bordered_row(
            row_widget![
                text(format!("{}", row.index + 1)).size(dims.theme().text.caption),
                swatch,
                input,
                gradient,
            ]
            .spacing(dims.theme().space.xs)
            .align_y(iced::alignment::Vertical::Center)
            .into(),
            dims,
        ));
    }
    rows.into()
}

fn shape_fill_swatch(
    fill: Option<&Fill>,
    index: usize,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let rgba = match fill {
        Some(Fill { brush: Brush::Solid(rgb), opacity, .. }) => [rgb.r, rgb.g, rgb.b, *opacity],
        Some(Fill { brush: Brush::Gradient(_), .. }) => [0.5, 0.2, 0.8, 1.0],
        None => [0.0, 0.0, 0.0, 0.0],
    };
    let chip = container(text(""))
        .width(Length::Fixed(dims.inspector_row_height * 0.46))
        .height(Length::Fixed(dims.inspector_row_height * 0.46))
        .style(move |_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(iced::Color {
                r: rgba[0] as f32,
                g: rgba[1] as f32,
                b: rgba[2] as f32,
                a: rgba[3] as f32,
            })),
            border: iced::Border { color: colors.border_default, width: dims.theme().stroke.hairline, radius: 0.0.into() },
            ..Default::default()
        });
    button(chip)
        .on_press(Message::ShapeFillFocus(index))
        .padding(0.0)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_store::{Shape, VectorPoint};

    fn document_with_shape() -> (Document, LayerId) {
        let mut doc = Document::new();
        let layer = LayerId(1);
        doc.apply(Intent::AddLayer(layer)).expect("layer");
        doc.apply(Intent::SetShapes {
            layer,
            shapes: vec![ShapeNode::Leaf(Shape::new(motolii_store::PathSource::Rectangle {
                size: VectorPoint { x: 100.0, y: 50.0 },
            }))],
        })
        .expect("shape");
        (doc, layer)
    }

    #[test]
    fn shape_fill_hex_changes_fill() {
        let (mut doc, layer) = document_with_shape();
        let mut draft = Some(ShapeFillDraft { field: ShapeFillField::Hex(0), text: "#3366CC80".into() });
        commit_shape_fill(&mut doc, Some(layer), &mut draft).expect("fill");
        let shapes = doc.view().shapes(layer).expect("read");
        let ShapeNode::Leaf(shape) = &shapes[0] else { panic!("leaf") };
        let Some(fill) = &shape.fill else { panic!("fill") };
        assert!(matches!(fill.brush, Brush::Solid(Rgb { r, g, b }) if (r - 0.2).abs() < 0.01 && (g - 0.4).abs() < 0.01 && (b - 0.8).abs() < 0.01));
        assert!((fill.opacity - 0.502).abs() < 0.01);
    }

    #[test]
    fn shape_fill_gradient_changes_fill() {
        let (mut doc, layer) = document_with_shape();
        apply_shape_gradient(&mut doc, Some(layer), 0).expect("gradient");
        let shapes = doc.view().shapes(layer).expect("read");
        let ShapeNode::Leaf(shape) = &shapes[0] else { panic!("leaf") };
        assert!(matches!(shape.fill.as_ref().map(|fill| &fill.brush), Some(Brush::Gradient(gradient)) if gradient.stops.len() == 2));
    }
}
