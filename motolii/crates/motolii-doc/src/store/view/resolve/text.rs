//! 文字の書類 — 層に書かれた字・書体・組み方から、描く側が読む書類を作る。
//! 並ぶ文字は並べた幅で折り返す。字面の整形は vector/text.rs、ここは書類の組み立て。

use super::*;

/// 解いた文字の書類。横が Fill で並ぶ文字は、並べた幅で折り返す。
pub fn resolved_text_document(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
) -> Result<Option<TextDocument>, StoreError> {
    let Some(mut document) = authored_text_document(view, layer, t)? else { return Ok(None) };
    if let (Some(wrap), Some(comp)) = (view.laid_out(layer, t)?.and_then(|slot| slot.wrap), view.composition()?) {
        document.wrap_size = Some([wrap.max(1.0), comp.height as f32]);
    }
    Ok(Some(document))
}

/// 書類に書かれた値だけで解いた文字(並べる前。並べる計算が箱を測る時はこちら)。
pub(crate) fn authored_text_document(
    view: &StoreView<'_>,
    layer: LayerId,
    t: RationalTime,
) -> Result<Option<TextDocument>, StoreError> {
    let Some(mut document) = view.text_document(layer)? else {
        return Ok(None);
    };
    // Readout: 文字の `#` を関係の値に(`#` が無ければ全部)。1 つの書体の文字として組み直す。
    if let Some(value) = view.readout(layer, t)? {
        let written = document.content.eval(t);
        let content = if written.contains('#') { written.replace('#', &value) } else { value };
        let mut track = crate::doc::store::text::ContentTrack::new();
        track.insert(crate::doc::store::text::ContentKeyframe { t: RationalTime::ZERO, content: content.clone() });
        document.content = track;
        if let Some(style) = document.runs.first().map(|r| r.style) {
            document.runs = vec![crate::doc::store::text::TextRun { len: unicode_segmentation::UnicodeSegmentation::graphemes(content.as_str(), true).count() as u32, style }];
        }
    }

    let justify_property = PropertyId::text_justify();
    match view.value_at(layer, &justify_property, t)? {
        Some(Value::Enum(v)) => {
            document.justify = crate::doc::store::TextJustify::from_enum_value(v).ok_or_else(|| {
                StoreError::Property(format!(
                    "`text_justify` track に未知の enum 値が入っている: {v}"
                ))
            })?;
        }
        Some(other) => {
            return Err(StoreError::Property(format!(
                "`text_justify` に enum でない値が入っている(track が壊れている): {other:?}"
            )))
        }
        None => {}
    }

    // 文字組みの 3 法(CSS の語)。値は選択肢の番、無ければ CSS の初期値。
    let laws = &mut document.alignment;
    let enum_at = |property: PropertyId| -> Result<Option<i64>, StoreError> {
        match view.value_at(layer, &property, t)? {
            Some(Value::Enum(v)) => Ok(Some(v)),
            Some(other) => Err(StoreError::Property(format!("`{}` に enum でない値が入っている: {other:?}", property.name()))),
            None => Ok(None),
        }
    };
    if let Some(v) = enum_at(PropertyId::text_autospace())? { laws.autospace = crate::doc::store::TextAutospace::from_enum_value(v).unwrap_or_default(); }
    if let Some(v) = enum_at(PropertyId::text_spacing_trim())? { laws.spacing_trim = crate::doc::store::TextSpacingTrim::from_enum_value(v).unwrap_or_default(); }
    if let Some(v) = enum_at(PropertyId::hanging_punctuation())? { laws.hanging = crate::doc::store::HangingPunctuation::from_enum_value(v).unwrap_or_default(); }

    for style in &mut document.styles {
        let size_property = PropertyId::text_style_size(style.id);
        if let Some(value) = view.value_at(layer, &size_property, t)? {
            match value {
                Value::F64(v) => style.size = v as f32,
                other => {
                    return Err(StoreError::Property(format!(
                        "text_style.{}.size に数値でない値が入っている: {other:?}",
                        style.id
                    )))
                }
            }
        }

        let line_height_property = PropertyId::text_style_line_height(style.id);
        if let Some(value) = view.value_at(layer, &line_height_property, t)? {
            match value {
                Value::F64(v) => style.line_height = Some(v as f32),
                other => {
                    return Err(StoreError::Property(format!(
                        "text_style.{}.line_height に数値でない値が入っている: {other:?}",
                        style.id
                    )))
                }
            }
        }

        let tracking_property = PropertyId::text_style_tracking(style.id);
        if let Some(value) = view.value_at(layer, &tracking_property, t)? {
            match value {
                Value::F64(v) => style.tracking = v as f32,
                other => {
                    return Err(StoreError::Property(format!(
                        "text_style.{}.tracking に数値でない値が入っている: {other:?}",
                        style.id
                    )))
                }
            }
        }

        let fill_property = PropertyId::text_style_fill_color(style.id);
        if let Some(value) = view.value_at(layer, &fill_property, t)? {
            match value {
                Value::Color(c) => style.fill = c,
                other => {
                    return Err(StoreError::Property(format!(
                        "text_style.{}.fill_color に色でない値が入っている: {other:?}",
                        style.id
                    )))
                }
            }
        }
    }

    Ok(Some(document))
}
