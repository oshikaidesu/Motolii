use crate::doc::store::*;
pub(crate) fn write_content(doc: &mut Document, layer: LayerId, t: RationalTime, content: String) -> Result<(), StoreError> {
    let edit = content_intent(doc, layer, t, content)?;
    doc.apply(edit)
}
pub(crate) fn content_intent(doc: &Document, layer: LayerId, t: RationalTime, content: String) -> Result<Intent, StoreError> {
    let Some(mut document) = doc.view().without_transients().text_document(layer)? else {
        return Err(StoreError::Property("Select a Text layer".into()));
    };
    // 文字は時間を開けていない限り 1 つ。キーが 1 つ以下なら差し替え、2 つ以上なら今の時刻に足す。
    let previous = document.content.eval(t).to_owned();
    crate::doc::store::text_edit::preserve_replacement(&mut document, &previous, &content);
    let keys = document.content.keys();
    if keys.len() <= 1 {
        let at = keys.first().map_or(t, |k| k.t);
        let mut only = crate::doc::store::ContentTrack::new();
        only.insert(ContentKeyframe { t: at, content });
        document.content = only;
    } else {
        document.content.insert(ContentKeyframe { t, content });
    }
    Ok(Intent::SetTextDocument { layer, document })
}
pub(crate) fn toggle_content_key(
    doc: &mut Document,
    layer: LayerId,
    t: RationalTime,
    content: String,
) -> Result<(), crate::doc::store::StoreError> {
    let Some(mut document) = doc.view().text_document(layer)? else {
        return Ok(());
    };
    let keys = document.content.keys().to_vec();
    let here = keys.iter().position(|k| k.t == t);
    // 立てる本文は**今 store に在る**この時刻の行(render 時の写しは、欄を確定した直後は古い)。
    let content = if content.is_empty() {
        keys.iter()
            .filter(|k| k.t <= t)
            .last()
            .or(keys.first())
            .map(|k| k.content.clone())
            .unwrap_or_default()
    } else {
        keys.iter()
            .filter(|k| k.t <= t)
            .last()
            .or(keys.first())
            .map(|k| k.content.clone())
            .unwrap_or(content)
    };
    match here {
        Some(_) if keys.len() <= 1 => {
            return Err(crate::doc::store::StoreError::Property(
                "The only lyric line cannot be removed".to_owned(),
            ))
        }
        Some(i) => {
            let mut rest = crate::doc::store::ContentTrack::new();
            for (k, key) in keys.iter().enumerate() {
                if k != i {
                    rest.insert(key.clone());
                }
            }
            document.content = rest;
        }
        None => document.content.insert(ContentKeyframe { t, content }),
    }
    doc.apply(Intent::SetTextDocument { layer, document })
}

pub(crate) fn font_intent(doc: &Document, layer: LayerId, family: &str) -> Result<Intent, String> {
    if !crate::doc::vector::text::font_families().iter().any(|name| name == family) {
        return Err("Font family is not installed".into());
    }
    let view = doc.view().without_transients();
    if view.attrs(layer).map_err(|e| e.to_string())?.ok_or("Layer not found")?.locked {
        return Err("Layer is locked".into());
    }
    let mut document = view.text_document(layer).map_err(|e| e.to_string())?
        .ok_or("Select a Text layer")?;
    for style in &mut document.styles {
        style.font = FontRef { family: family.into(), ..Default::default() };
    }
    Ok(Intent::SetTextDocument { layer, document })
}

#[cfg(test)]
mod font_tests {
    use super::*;
    #[test]
    fn font_family_change_preserves_text_and_undo_restores_it() {
        let mut doc = blank_project();
        let id = LayerId(1);
        doc.apply_all(crate::editor::create::new_layer_intents(id, 0, 0, 60,
            Fps::try_new(30, 1).unwrap(), (1920.0,1080.0), crate::editor::create::NewKind::Text, None)).unwrap();
        let before = doc.view().text_document(id).unwrap().unwrap();
        let family = crate::doc::vector::text::font_families().iter()
            .find(|f| **f != before.styles[0].font.family).expect("installed font");
        doc.apply(font_intent(&doc, id, family).unwrap()).unwrap();
        let after = doc.view().text_document(id).unwrap().unwrap();
        assert_eq!(after.content, before.content);
        assert_eq!(&after.styles[0].font.family, family);
        assert!(doc.undo());
        assert_eq!(doc.view().text_document(id).unwrap().unwrap(), before);
        assert!(font_intent(&doc, id, "not-an-installed-font-82731").is_err());
        doc.apply(Intent::SetAttrs{layer:id, patch:LayerAttrsPatch{locked:Some(true), ..Default::default()}}).unwrap();
        assert!(font_intent(&doc, id, family).is_err());
    }
}
