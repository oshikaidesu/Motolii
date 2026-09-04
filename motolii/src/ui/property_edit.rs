use crate::doc::store::{Document, LayerId, PropertyId, RationalTime, StoreError, Value};
use crate::ui::functions::lens;

pub(crate) type PropertyEdit = (LayerId, PropertyId, Value);

fn commit(doc: &mut Document, at: RationalTime, values: &[PropertyEdit]) -> Result<(), StoreError> {
    let mut seen = std::collections::HashSet::new();
    let mut intents = Vec::new();
    for (layer, property, value) in values {
        if !seen.insert((*layer, property.clone())) {
            return Err(StoreError::Property(
                "Overlapping property edits must be composed before committing".into(),
            ));
        }
        if let Some(intent) = lens::place(doc, *layer, property, value.clone(), at)? {
            intents.push(intent);
        }
    }
    doc.apply_all(intents)
}

pub(crate) fn preview_owned(
    doc: &mut Document,
    owner: u64,
    values: &[PropertyEdit],
) -> Result<(), StoreError> {
    let mut seen = std::collections::HashSet::new();
    let mut intents = Vec::new();
    for (layer, property, value) in values {
        if !seen.insert((*layer, property.clone())) {
            return Err(StoreError::Property(
                "Overlapping property edits must be composed before previewing".into(),
            ));
        }
        lens::require_local_source(&doc.view().without_transients(), *layer, property)?;
        intents.push(crate::doc::store::Intent::SetConstant {
            layer: *layer,
            property: property.clone(),
            value: value.clone(),
        });
    }
    doc.preview_edits(owner, &intents)
}

pub(crate) fn cancel_owned(doc: &mut Document, owner: u64) -> bool {
    doc.clear_preview_edits(owner)
}

pub(crate) fn commit_owned(
    doc: &mut Document,
    owner: u64,
    at: RationalTime,
    values: &[PropertyEdit],
) -> Result<(), StoreError> {
    if !doc.preview_is_current(owner) {
        return Err(StoreError::Property(
            "The preview was superseded by another operation".into(),
        ));
    }
    doc.clear_preview_edits(owner);
    commit(doc, at, values)
}
