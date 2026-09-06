use crate::doc::store::{Document, Intent, LayerAttrsPatch, LayerId, LayerSource, StoreError, StoreView};

fn texture_source(view: &StoreView<'_>, layer: LayerId) -> bool {
    view.meta(layer).ok().flatten().is_some_and(|meta| match meta.source {
        LayerSource::Shape | LayerSource::Text => true,
        LayerSource::File { path, .. } => std::path::Path::new(&path).extension()
            .and_then(|ext| ext.to_str())
            .and_then(crate::render::media::asset_type_for_extension)
            .is_some_and(|kind| kind.starts_with("image/") || kind.starts_with("video/")),
        LayerSource::Group | LayerSource::Null => false,
    })
}

pub(super) fn rejection(view: &StoreView<'_>, layer: LayerId) -> Option<String> {
    if let Some(reason) = crate::ui::session::edit_rejection(view, layer) {
        return Some(reason.to_owned());
    }
    if view.attrs(layer).ok().flatten().is_some_and(|attrs| attrs.clip_to_below) {
        return None;
    }
    if view.attrs(layer).ok().flatten().is_some_and(|attrs| attrs.blend_mode == crate::doc::store::BlendMode::Add) {
        return Some("Clipping does not yet support Add on the upper layer".into());
    }
    if !texture_source(view, layer) {
        return Some("Clipping is available for text, shapes, images and video".into());
    }
    let Some(base) = view.clipping_base(layer).ok().flatten() else {
        return Some("No base layer below in this group".into());
    };
    if !texture_source(view, base) || view.attrs(base).ok().flatten().is_some_and(|attrs| attrs.matte.is_some()) {
        return Some("The base must be text, a shape, image or video without a legacy matte".into());
    }
    None
}

pub(super) fn toggle(doc: &mut Document, layer: LayerId) -> Result<(), StoreError> {
    if let Some(reason) = rejection(&doc.view(), layer) {
        return Err(StoreError::Property(reason));
    }
    let clipped = doc.view().attrs(layer)?.unwrap_or_default().clip_to_below;
    doc.apply(Intent::SetAttrs { layer, patch: LayerAttrsPatch {
        clip_to_below: Some(!clipped), ..Default::default()
    }})
}
