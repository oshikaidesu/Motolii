use crate::doc::store::*;
pub(crate) fn write_content(
    doc: &mut Document,
    layer: LayerId,
    t: RationalTime,
    content: String,
) -> Result<(), crate::doc::store::StoreError> {
    let Some(mut document) = doc.view().text_document(layer)? else {
        return Ok(());
    };
    // 文字は時間を開けていない限り 1 つ。キーが 1 つ以下なら差し替え、2 つ以上なら今の時刻に足す。
    let keys = document.content.keys();
    if keys.len() <= 1 {
        let at = keys.first().map_or(t, |k| k.t);
        let mut only = crate::doc::store::ContentTrack::new();
        only.insert(ContentKeyframe { t: at, content });
        document.content = only;
    } else {
        document.content.insert(ContentKeyframe { t, content });
    }
    doc.apply(Intent::SetTextDocument { layer, document })
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
