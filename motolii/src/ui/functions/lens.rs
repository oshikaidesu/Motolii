use crate::doc::store::{
    Document, Intent, LayerId, PropertyId, RationalTime, StoreError, StoreView, Value,
};

pub(crate) fn edit_rejection(
    view: &StoreView<'_>,
    layer: LayerId,
) -> Result<Option<&'static str>, StoreError> {
    Ok(if !view.has_layer(layer) {
        Some("layer no longer exists")
    } else if view.attrs(layer)?.is_some_and(|attrs| attrs.locked) {
        Some("locked")
    } else if view.frozen_ancestor(layer)?.is_some() {
        Some("inside a frozen group")
    } else {
        None
    })
}

pub(crate) fn place(
    doc: &Document,
    layer: LayerId,
    property: &PropertyId,
    value: Value,
    at: RationalTime,
) -> Result<Option<Intent>, StoreError> {
    doc.place_checked(layer, property, value, at)
}

pub(crate) fn require_local_source(
    view: &StoreView<'_>,
    layer: LayerId,
    property: &PropertyId,
) -> Result<(), StoreError> {
    if let Some(reason) = view.property_write_rejection(layer, property)? {
        return Err(StoreError::Property(reason.into()));
    }
    Ok(())
}
